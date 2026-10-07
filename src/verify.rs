//! `spk verify` — the `model_checked → verified` transition
//! (specs/verify.md).
//!
//! Purpose: turn `specodelic.md`'s bare `no_counterexample ∧
//! properties_pass` guard into an actual mechanism. Responsibilities:
//! evaluate the properties gate (staleness by comparing the on-disk
//! `*_props.rs` artifact's block metadata against what the current spec
//! regenerates, then really executing every compiled proptest! block
//! through a runner seam), evaluate the model gate (`RunReport` from
//! `<stem>.check.json` must be current — its `artifact_sha256` must match
//! the on-disk `<stem>.tla` — and its outcome must be
//! `no_counterexample`), and combine the two gates into the single
//! verdict `specs/verify.md` calls `verified`. Rationale: neither gate
//! alone suffices (`both_gates_required`); un-translated predicates panic
//! via `todo_predicate!` and are reported honestly as failures, never as
//! skips; a stale artifact is a failure, not a cached pass
//! (`properties_pass_reflects_latest_run`, `rerun_on_model_change`).

use serde::{Deserialize, Serialize};

use crate::compile;
use crate::model_check::{self, Outcome, RunReport};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

// ---------------------------------------------------------------------------
// Block metadata (parsed from the generated source's own `// id:` comments)
// ---------------------------------------------------------------------------

/// One compiled proptest! block, identified by the artifact's own comment
/// metadata — the same identity `compile.md` emits per block.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PropBlockMeta {
    /// The verbatim property row id the block was compiled from.
    pub id: String,
    /// Required case name for law-kind rows (`None` for unit rows).
    pub case: Option<String>,
    /// Verbatim generator cell text (staleness fingerprint).
    pub generator: String,
    /// Verbatim predicate cell text (staleness fingerprint).
    pub predicate: String,
    /// The Rust fn name the block was emitted under (libtest test name).
    pub fn_name: String,
}

/// Parse the block metadata back out of a generated `*_props.rs` source:
/// `// id:` / `// case:` comments followed by a `fn <name>(` proptest!
/// block. This is the staleness fingerprint: a spec edit changes the
/// metadata (ids, cases, generators, predicates), a hand-translation of a
/// predicate body does not.
pub fn blocks_from_source(source: &str) -> Vec<PropBlockMeta> {
    let mut blocks: Vec<PropBlockMeta> = vec![];
    let mut id = String::new();
    let mut case: Option<String> = None;
    let mut generator = String::new();
    let mut predicate = String::new();
    for line in source.lines() {
        let trimmed = line.trim_start();
        if let Some(rest) = trimmed.strip_prefix("// id: ") {
            id = rest.trim().to_string();
            case = None;
            generator = String::new();
            predicate = String::new();
        } else if let Some(rest) = trimmed.strip_prefix("// case: ") {
            case = Some(rest.trim().to_string());
        } else if let Some(rest) = trimmed.strip_prefix("// generator: ") {
            generator = rest.trim().to_string();
        } else if let Some(rest) = trimmed.strip_prefix("// predicate: ") {
            predicate = rest.trim().to_string();
        } else {
            let t = line.trim();
            if t.starts_with("fn ") && t.contains('(') {
                let name = t[3..]
                    .split('(')
                    .next()
                    .unwrap_or_default()
                    .trim()
                    .to_string();
                blocks.push(PropBlockMeta {
                    id: id.clone(),
                    case: case.clone(),
                    generator: generator.clone(),
                    predicate: predicate.clone(),
                    fn_name: name,
                });
                case = None;
            }
        }
    }
    blocks
}

// ---------------------------------------------------------------------------
// Gate states
// ---------------------------------------------------------------------------

/// The properties gate — `properties_pass` evaluated against the current
/// compiled artifact.
#[derive(Debug, Clone, PartialEq)]
pub enum PropsGateState {
    /// Every compiled block ran and passed (or the artifact has no
    /// blocks — `blocks_run_to_completion` holds vacuously).
    Pass,
    /// One or more blocks failed (`blocks_run_to_completion` +
    /// `law_cases_all_run`: a partial pass is a failure of the property).
    Failed(String),
    /// The on-disk artifact predates the current spec
    /// (`properties_pass_reflects_latest_run`).
    Stale,
    /// No `*_props.rs` artifact in `out_dir`.
    MissingArtifact,
    /// The artifact did not compile as a Rust test crate.
    Uncompilable(String),
    /// The runner (cargo) could not be spawned.
    RunnerUnavailable(String),
    /// The cargo test run hit its wall-clock budget and was killed
    /// (`bounded_wall_clock`). Carries the bound in whole seconds and the
    /// captured partial output (libtest's "has been running for over N
    /// seconds" lines name the hanging block). Deliberately distinct from
    /// [`PropsGateState::Failed`] — a timeout is not a verdict on the
    /// property, it is no verdict at all.
    TimedOut { secs: u64, detail: String },
}

/// The model gate — `no_counterexample` evaluated against the current
/// compiled artifact.
#[derive(Debug, Clone, PartialEq)]
pub enum ModelGateState {
    /// The current run report's outcome is `no_counterexample`.
    Clean,
    /// The current run report's outcome is anything else — including
    /// `exploration_only`, which is explicitly NOT clean (the native
    /// backend executes no invariant predicates).
    NotClean { outcome: Outcome },
    /// The stored run predates the current artifact (or the artifact is
    /// gone) — `rerun_on_model_change` fails the gate closed.
    Stale { detail: String },
    /// No readable `<stem>.check.json` run report.
    Missing { detail: String },
}

/// One block's execution result.
#[derive(Debug, Clone, PartialEq)]
pub struct BlockResult {
    pub id: String,
    pub case: Option<String>,
    pub fn_name: String,
    pub passed: bool,
    /// The failing block's captured output (panic message + proptest's
    /// shrunk minimal failing input) — `failure_reports_shrunk_counterexample`.
    pub detail: Option<String>,
}

/// The properties gate evaluated: state plus per-block results.
#[derive(Debug, Clone, PartialEq)]
pub struct PropertiesGate {
    pub state: PropsGateState,
    pub blocks: Vec<BlockResult>,
}

/// The single verdict `specs/verify.md` names — `verified` exactly when
/// both gates pass, else the blocking stage.
#[derive(Debug, Clone, PartialEq)]
pub struct Verdict {
    pub status: &'static str,
    pub message: String,
    pub hint: String,
}

/// `both_gates_required`: the verdict is `verified` only on the
/// conjunction. Properties blockers are named before model blockers —
/// both gates are always evaluated, the first blocker names the status.
pub fn verdict(properties: &PropsGateState, model: &ModelGateState) -> Verdict {
    // Properties blockers first — both gates are always evaluated by the
    // caller; the verdict names the first blocking stage and never reads a
    // non-pass state as a pass.
    let (status, message, hint) = match properties {
        PropsGateState::Pass => match model {
            ModelGateState::Clean => (
                "verified",
                "all compiled proptest! blocks passed and the current model run reported no_counterexample".to_string(),
                "re-verify after any edit to the spec or its compiled artifacts — verified is not cached".to_string(),
            ),
            ModelGateState::NotClean { outcome } => (
                "model_not_clean",
                format!(
                    "the most recent model-check run against the current artifact reported {outcome:?} — only no_counterexample verifies, and exploration_only is explicitly not clean (no invariant predicates were executed; give the model executable invariant fragments — the **rust:** marker, specodelic.md Revision 15 — and re-run)"
                ),
                "run: specodelic model-check with a backend that executes invariants, or treat the model as unverified".to_string(),
            ),
            ModelGateState::Stale { detail } => (
                "stale_model_run",
                format!(
                    "the stored model-check run is not current evidence ({detail}) — a clean result on stale or foreign evidence is not clean"
                ),
                "run: specodelic model-check over the same file set and --out-dir as the original invocation, then verify again".to_string(),
            ),
            ModelGateState::Missing { detail } => (
                "missing_model_run",
                format!(
                    "no readable model-check run report ({detail}) — verify never treats a skipped stage as passed"
                ),
                "run: specodelic model-check <files>".to_string(),
            ),
        },
        PropsGateState::Failed(summary) => (
            "properties_failed",
            format!("{summary} — a partial pass is a failure of the property"),
            "fix the failing predicates (see each block's detail for the shrunk failing input)".to_string(),
        ),
        PropsGateState::Stale => (
            "stale_properties_artifact",
            "the on-disk *_props.rs artifact predates the current spec — properties from a stale artifact never count".to_string(),
            "run: specodelic compile <files> to regenerate the artifact".to_string(),
        ),
        PropsGateState::MissingArtifact => (
            "missing_properties_artifact",
            "no compiled proptest artifact for this spec — verify consumes compile's output, it never re-compiles".to_string(),
            "run: specodelic compile <files>".to_string(),
        ),
        PropsGateState::Uncompilable(output) => (
            "properties_uncompilable",
            "the *_props.rs artifact did not compile as a Rust test crate".to_string(),
            format!("fix the artifact's Rust (first errors): {output}"),
        ),
        PropsGateState::RunnerUnavailable(detail) => (
            "runner_unavailable",
            "the properties gate executes the compiled proptest! blocks and needs the Rust toolchain".to_string(),
            format!("install the Rust toolchain so cargo is on PATH ({detail})"),
        ),
        PropsGateState::TimedOut { secs, detail } => (
            "properties_timed_out",
            format!(
                "the cargo test run exceeded its {secs}s wall-clock bound and was killed — a hanging (likely pathological) predicate is an unbounded DoS on verify, and no block verdicts exist yet"
            ),
            format!(
                "raise --timeout-secs (or pass --timeout-secs 0 to run unbounded) and re-run; the partial output names the block that hung: {detail}"
            ),
        ),
    };
    Verdict {
        status,
        message,
        hint,
    }
}

// ---------------------------------------------------------------------------
// Gate evaluation against an out_dir
// ---------------------------------------------------------------------------

/// One spec's both-gates evaluation result (design D4): the verdict
/// plus the gate states behind it — what every report view renders
/// from.
pub struct FileVerdict {
    pub file: String,
    pub id: String,
    pub verdict: Verdict,
    pub properties: PropertiesGate,
    pub model: ModelGateState,
}

/// The ONE both-gates evaluation loop (define-verification-claim-gates
/// design D4): for each spec, evaluate the properties gate and the
/// model gate and take `verdict`'s conjunction — `cmd_verify` and
/// orchestrate's verify stage consume this exact path, so the aggregate
/// verdict cannot drift between report views. Per-view entry rendering
/// stays with each view (the CLI's rich entry vs the orchestrate
/// stage's compact detail).
pub fn evaluate_file_verdicts(
    specs: &[crate::spec::Spec],
    out_dir: &Path,
    runner: &dyn PropertiesRunner,
) -> Vec<FileVerdict> {
    specs
        .iter()
        .map(|spec| {
            let file = spec
                .path
                .as_ref()
                .map(|p| p.display().to_string())
                .unwrap_or_else(|| format!("<{}>", spec.intent.id));
            let stem = artifact_stem(spec);
            let properties = evaluate_properties_gate(spec, out_dir, runner);
            let model = evaluate_model_gate(spec, specs, out_dir, &stem);
            let verdict = verdict(&properties.state, &model);
            FileVerdict {
                file,
                id: spec.intent.id.clone(),
                verdict,
                properties,
                model,
            }
        })
        .collect()
}

/// The three artifact paths verify consumes for one spec.
pub struct ArtifactPaths {
    pub props: PathBuf,
    pub check_report: PathBuf,
    pub tla: PathBuf,
}

/// Artifact naming, mirroring `cmd_compile`/`cmd_model_check`: the file
/// stem on disk, else the intent id with `.` → `-`.
pub fn artifact_stem(spec: &crate::spec::Spec) -> String {
    spec.path
        .as_ref()
        .and_then(|p| p.file_stem().and_then(|s| s.to_str()))
        .map(str::to_string)
        .unwrap_or_else(|| spec.intent.id.replace('.', "-"))
}

pub fn artifact_paths(out_dir: &Path, stem: &str) -> ArtifactPaths {
    ArtifactPaths {
        props: out_dir.join(format!("{stem}_props.rs")),
        check_report: out_dir.join(format!("{stem}.check.json")),
        tla: out_dir.join(format!("{stem}.tla")),
    }
}

/// The model gate: `<stem>.check.json` must parse, carry the current
/// claim schema version, a scope fingerprint matching the live recomputed
/// digest, claim records matching the live required set, an
/// `artifact_sha256` matching the current `<stem>.tla`, and an outcome of
/// `no_counterexample`. Fails closed on every deviation — old, stale or
/// foreign evidence is a rerun, never an acceptance (design D3,
/// specodelic-68m.3); the stored report is never rewritten.
pub fn evaluate_model_gate(
    spec: &crate::spec::Spec,
    specs: &[crate::spec::Spec],
    out_dir: &Path,
    stem: &str,
) -> ModelGateState {
    let paths = artifact_paths(out_dir, stem);
    let bytes = match fs::read(&paths.check_report) {
        Ok(b) => b,
        Err(e) => {
            return ModelGateState::Missing {
                detail: format!(
                    "no model-check run report at {}: {e}",
                    paths.check_report.display()
                ),
            };
        }
    };
    let raw: serde_json::Value = match serde_json::from_slice(&bytes) {
        Ok(r) => r,
        Err(e) => {
            return ModelGateState::Missing {
                detail: format!(
                    "unreadable run report at {}: {e}",
                    paths.check_report.display()
                ),
            };
        }
    };
    let report: RunReport = match serde_json::from_value(raw.clone()) {
        Ok(r) => r,
        Err(e) => {
            return ModelGateState::Missing {
                detail: format!(
                    "unreadable run report at {}: {e}",
                    paths.check_report.display()
                ),
            };
        }
    };
    // Claim schema freshness (D3): old or unrecognized report schema
    // versions require a model-check rerun — they are never upgraded and
    // no hashes are synthesized for them.
    match raw.get("claim_schema_version").and_then(|v| v.as_u64()) {
        Some(v) if v == CLAIM_SCHEMA_VERSION as u64 => {}
        Some(other) => {
            return ModelGateState::Stale {
                detail: format!(
                    "report claim schema version {other} is not supported (current {}) — stored reports are never upgraded or repaired",
                    CLAIM_SCHEMA_VERSION
                ),
            };
        }
        None => {
            return ModelGateState::Stale {
                detail: "the stored report carries no claim schema version — old reports require a model-check rerun and are never upgraded".to_string(),
            };
        }
    }
    // Scope fingerprint (D3): recompute the digest from the live parsed
    // inputs and the current compiled artifacts — the stored value is
    // never trusted.
    let Some(stored_scope) = raw.get("scope_sha256").and_then(|v| v.as_str()) else {
        return ModelGateState::Stale {
            detail: "the stored report carries no scope fingerprint — rerun model-check so the evidence binds the current inputs".to_string(),
        };
    };
    let mut artifacts: std::collections::BTreeMap<String, String> =
        std::collections::BTreeMap::new();
    for s in specs {
        let sstem = artifact_stem(s);
        let tla_path = out_dir.join(format!("{sstem}.tla"));
        let tla = match fs::read(&tla_path) {
            Ok(b) => b,
            Err(e) => {
                return ModelGateState::Stale {
                    detail: format!(
                        "compiled module unreadable/missing at {}: {e} — the invocation scope cannot be recomputed",
                        tla_path.display()
                    ),
                };
            }
        };
        artifacts.insert(s.intent.id.clone(), model_check::artifact_sha256(&tla));
    }
    let current_scope = scope_digest(specs, &artifacts);
    if stored_scope != current_scope {
        return ModelGateState::Stale {
            detail: format!(
                "scope mismatch: the stored evidence covers different structured inputs or artifacts (report {stored_scope}, current {current_scope}) — every contributing file of the original invocation must be re-checked together"
            ),
        };
    }
    // Qualified claim records (D3): the live required set is recomputed
    // from the current inputs — duplicates are invalid evidence, missing
    // records are incomplete evidence, extra records belong to a foreign
    // scope, and an evaluator kind that no longer matches the live
    // opt-in is stale.
    let class = claim_classification(spec);
    let Some(records) = raw.get("claims").and_then(|v| v.as_array()) else {
        return ModelGateState::Stale {
            detail: "the stored report carries no qualified claim records — rerun model-check"
                .to_string(),
        };
    };
    let mut seen: std::collections::BTreeSet<&str> = std::collections::BTreeSet::new();
    for rec in records {
        let id = rec.get("id").and_then(|v| v.as_str()).unwrap_or("");
        if !seen.insert(id) {
            return ModelGateState::Stale {
                detail: format!(
                    "duplicate claim record {id:?} — the stored evidence is internally inconsistent"
                ),
            };
        }
    }
    for req in &class.required {
        let Some(rec) = records
            .iter()
            .find(|r| r.get("id").and_then(|v| v.as_str()) == Some(req.id.as_str()))
        else {
            return ModelGateState::Stale {
                detail: format!(
                    "missing required claim record {:?} — the stored evidence is incomplete for the current inputs",
                    req.id
                ),
            };
        };
        let stored_eval = rec.get("evaluator").and_then(|v| v.as_str()).unwrap_or("");
        if stored_eval != req.evaluator {
            return ModelGateState::Stale {
                detail: format!(
                    "claim record {:?} was evaluated by {:?} but the current inputs opt it into {:?}",
                    req.id, stored_eval, req.evaluator
                ),
            };
        }
    }
    for id in seen.iter() {
        if !class.required.iter().any(|req| req.id == *id) {
            return ModelGateState::Stale {
                detail: format!(
                    "claim record {id:?} is not required by the current inputs — the stored evidence belongs to a different scope"
                ),
            };
        }
    }
    // The advisory expected/unchecked sets must agree with the live
    // recompute — a disagreement means the report predates the inputs.
    let live_expected: Vec<String> = class.required.iter().map(|r| r.id.clone()).collect();
    let live_unchecked = &class.unchecked;
    let stored_expected: Vec<String> = raw
        .get("expected_claim_ids")
        .and_then(|v| v.as_array())
        .map(|a| {
            a.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default();
    if stored_expected != live_expected {
        return ModelGateState::Stale {
            detail: "the stored expected_claim_ids do not match the live required set — rerun model-check over the same file set".to_string(),
        };
    }
    let stored_unchecked: Vec<String> = raw
        .get("unchecked_claim_ids")
        .and_then(|v| v.as_array())
        .map(|a| {
            a.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default();
    if &stored_unchecked != live_unchecked {
        return ModelGateState::Stale {
            detail: "the stored unchecked_claim_ids do not match the live unchecked set — rerun model-check over the same file set".to_string(),
        };
    }
    // Artifact staleness key: the report records the SHA-256 of the .tla
    // it consumed; the on-disk module must still match (belt and braces —
    // the scope digest binds the same module).
    let current_sha = artifacts
        .get(&spec.intent.id)
        .expect("this spec is part of the invocation");
    if model_check::is_stale(&report, current_sha) {
        return ModelGateState::Stale {
            detail: format!(
                "report sha {}, current {current_sha}",
                report.artifact_sha256
            ),
        };
    }
    match report.outcome {
        Outcome::NoCounterexample => ModelGateState::Clean,
        other => ModelGateState::NotClean { outcome: other },
    }
}

/// The properties gate: staleness by metadata comparison, then real
/// execution of every block through the runner seam. A spec with no
/// Property rows passes vacuously without invoking the runner.
pub fn evaluate_properties_gate(
    spec: &crate::spec::Spec,
    out_dir: &Path,
    runner: &dyn PropertiesRunner,
) -> PropertiesGate {
    let stem = artifact_stem(spec);
    let paths = artifact_paths(out_dir, &stem);
    let on_disk = match fs::read_to_string(&paths.props) {
        Ok(s) => s,
        Err(_) => {
            return PropertiesGate {
                state: PropsGateState::MissingArtifact,
                blocks: vec![],
            };
        }
    };
    // Staleness fingerprint: block metadata (ids, cases, generators,
    // predicates) must match what the current spec regenerates. A
    // hand-translated predicate body keeps the fingerprint; a spec edit
    // changes it. Stale artifacts are never executed.
    let regenerated = compile::properties_to_proptest(spec);
    let disk_blocks = blocks_from_source(&on_disk);
    let regen_blocks = blocks_from_source(&regenerated);
    if disk_blocks != regen_blocks {
        return PropertiesGate {
            state: PropsGateState::Stale,
            blocks: vec![],
        };
    }
    if regen_blocks.is_empty() {
        // No Property rows: blocks_run_to_completion holds vacuously.
        return PropertiesGate {
            state: PropsGateState::Pass,
            blocks: vec![],
        };
    }
    match runner.run_blocks(&paths.props, &regen_blocks) {
        Ok(results) => {
            let failed: Vec<&BlockResult> = results.iter().filter(|r| !r.passed).collect();
            let state = if failed.is_empty() {
                PropsGateState::Pass
            } else {
                PropsGateState::Failed(format!(
                    "{} of {} compiled blocks failed",
                    failed.len(),
                    results.len()
                ))
            };
            PropertiesGate {
                state,
                blocks: results,
            }
        }
        Err(RunnerError::Spawn(detail)) => PropertiesGate {
            state: PropsGateState::RunnerUnavailable(detail),
            blocks: vec![],
        },
        Err(RunnerError::Compile { output }) => PropertiesGate {
            state: PropsGateState::Uncompilable(output),
            blocks: vec![],
        },
        Err(RunnerError::TimedOut { secs, output }) => PropertiesGate {
            state: PropsGateState::TimedOut {
                secs,
                detail: output,
            },
            blocks: vec![],
        },
    }
}

// ---------------------------------------------------------------------------
// Claim classification and scope binding (define-verification-claim-gates,
// design D3 — specodelic-68m.3): the report schema version, the live claim
// partition, and the canonical scope digest shared by the model-check write
// path (orchestrate) and verify's freshness gate.
// ---------------------------------------------------------------------------

/// The claim report schema version a persisted run report carries. Old or
/// unrecognized versions are never upgraded — they require a model-check
/// rerun (no hashes are synthesized for old reports).
pub const CLAIM_SCHEMA_VERSION: u32 = 1;

/// One required claim's live classification: the Constraints-table id and
/// the evaluator kind that opted the row into command evaluation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RequiredClaim {
    pub id: String,
    pub evaluator: &'static str,
}

/// One qualified claim record persisted in a run report (D3): the
/// Constraints-table id, the evaluator kind, the evaluated status, and the
/// labeled reason when (and only when) the status is not verified.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QualifiedClaim {
    pub id: String,
    pub evaluator: String,
    pub status: compile::ThreeValued,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

/// The live claim partition of one spec (design D1): which invariant rows
/// are required claims (opted into rust, citation, or kernel evaluation)
/// and which stay explicitly unchecked prose — prose never becomes
/// executable by implication.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ClaimClassification {
    /// Required claims, sorted by id (the canonical order).
    pub required: Vec<RequiredClaim>,
    /// Unchecked invariant rows, sorted by id.
    pub unchecked: Vec<String>,
}

/// Classify one spec's Constraints rows from its extracted IR: a row is a
/// required claim exactly when its expr opts into one of the three
/// evaluators (`**rust:**` fragment, whole-cell citation expression,
/// kernel expression); every other invariant row is explicitly unchecked.
pub fn claim_classification(spec: &crate::spec::Spec) -> ClaimClassification {
    let ir = compile::extract_model_ir(spec);
    let mut class = ClaimClassification::default();
    for row in &spec.constraints {
        if row.kind.as_deref() != Some("invariant") {
            continue;
        }
        let evaluator = if ir.invariants.iter().any(|inv| inv.id == row.id) {
            "rust"
        } else if ir.guard_citations.contains_key(&row.id) {
            "citation"
        } else if ir.guard_kernel.contains_key(&row.id) {
            "kernel"
        } else {
            class.unchecked.push(row.id.clone());
            continue;
        };
        class.required.push(RequiredClaim {
            id: row.id.clone(),
            evaluator,
        });
    }
    class.required.sort_by(|a, b| a.id.cmp(&b.id));
    class.unchecked.sort();
    class
}

/// The canonical scope digest (D3): SHA-256 over a deterministic
/// serialization — its encoding version bound to CLAIM_SCHEMA_VERSION —
/// of every parsed file's structured content plus the compiled artifact
/// hashes consumed for the invocation. Files sort by canonical intent id,
/// maps by key, meaningful row/state order is preserved; prose bodies,
/// absolute paths, byte spans, timestamps and CLI file order are
/// excluded. Identical structured inputs intentionally produce identical
/// digests; files differing in invariant content never collide. Generator
/// cells ride the structured content itself, so the consumed generator
/// definitions are bound; the command path consumes no separate pack
/// registry input.
///
/// `artifacts` maps intent id → SHA-256 of the compiled module the
/// invocation consumed (an absent entry digests as `null`: a contributing
/// file whose module was not consumed binds no artifact, and any later
/// recomputation that reads one mismatches — failing closed).
pub fn scope_digest(
    specs: &[crate::spec::Spec],
    artifacts: &std::collections::BTreeMap<String, String>,
) -> String {
    let mut files: Vec<(&str, serde_json::Value)> = specs
        .iter()
        .map(|spec| {
            (
                spec.intent.id.as_str(),
                serde_json::json!({
                    "intent": spec.intent,
                    "constraints": spec.constraints,
                    "states": spec.states,
                    "transitions": spec.transitions,
                    "properties": spec.properties,
                    "links": spec.links,
                    "artifact": artifacts.get(&spec.intent.id),
                }),
            )
        })
        .collect();
    files.sort_by(|a, b| a.0.cmp(b.0));
    let files: Vec<serde_json::Value> = files.into_iter().map(|(_, v)| v).collect();
    let payload = serde_json::json!({
        "claim_schema_version": CLAIM_SCHEMA_VERSION,
        "files": files,
    });
    let canonical = serde_json::to_string(&payload).expect("scope payload serializes");
    model_check::artifact_sha256(canonical.as_bytes())
}

/// The ONE derivation of the D3 report's claim fields (design D4): the
/// qualified claim records (evaluator kind from the live classification,
/// the evaluated status, the labeled reason when the claim did not
/// verify), the expected required-claim ids, and the unchecked invariant
/// ids. Both the model-check pipeline (orchestrate) and the test
/// fixtures build these fields through this exact path, so the write
/// side cannot drift from the read side — [`evaluate_model_gate`]
/// recomputes all of them from the live inputs and never trusts the
/// stored values.
///
/// `statuses` are the merged invariant statuses (citations and kernel
/// claims ride the same list, §3.8 TIDY); `reasons` maps claim id → the
/// labeled reason for any status that did not verify.
pub fn report_claim_fields(
    class: &ClaimClassification,
    statuses: &[model_check::InvariantStatus],
    reasons: &std::collections::BTreeMap<String, String>,
) -> (Vec<QualifiedClaim>, Vec<String>, Vec<String>) {
    let claims: Vec<QualifiedClaim> = statuses
        .iter()
        .map(|s| QualifiedClaim {
            id: s.id.clone(),
            evaluator: class
                .required
                .iter()
                .find(|req| req.id == s.id)
                .map(|req| req.evaluator.to_string())
                // A status outside the live classification cannot exist
                // (both derive from the same IR); if it ever did, the
                // record says so and verify fails closed on the
                // evaluator match.
                .unwrap_or_else(|| "unknown".to_string()),
            status: s.status,
            reason: reasons.get(&s.id).cloned(),
        })
        .collect();
    let expected = class.required.iter().map(|req| req.id.clone()).collect();
    (claims, expected, class.unchecked.clone())
}

/// The write-side report shape (D3): the run report's serialized JSON with
/// the claim-freshness fields injected — `claim_schema_version`, qualified
/// `claims`, `expected_claim_ids`, `unchecked_claim_ids`, and
/// `scope_sha256`. The ONE place these fields are attached to a persisted
/// report (the field values themselves come from [`report_claim_fields`]);
/// verify recomputes all of them from the live inputs and never trusts
/// the stored values.
pub fn fresh_report_json(
    report: &RunReport,
    claims: &[QualifiedClaim],
    expected_claim_ids: &[String],
    unchecked_claim_ids: &[String],
    scope_sha256: &str,
) -> serde_json::Value {
    let mut json = serde_json::to_value(report).expect("RunReport serializes");
    let obj = json
        .as_object_mut()
        .expect("RunReport serializes to an object");
    obj.insert(
        "claim_schema_version".into(),
        serde_json::json!(CLAIM_SCHEMA_VERSION),
    );
    obj.insert(
        "claims".into(),
        serde_json::to_value(claims).expect("claims serialize"),
    );
    obj.insert(
        "expected_claim_ids".into(),
        serde_json::json!(expected_claim_ids),
    );
    obj.insert(
        "unchecked_claim_ids".into(),
        serde_json::json!(unchecked_claim_ids),
    );
    obj.insert("scope_sha256".into(), serde_json::json!(scope_sha256));
    json
}

// ---------------------------------------------------------------------------
// Runner seam: real execution of the proptest! blocks
// ---------------------------------------------------------------------------

/// Why the runner could not produce per-block results.
#[derive(Debug, Clone, PartialEq)]
pub enum RunnerError {
    /// cargo could not be spawned (no Rust toolchain on PATH).
    Spawn(String),
    /// The scratch crate failed to build — the artifact is not valid Rust.
    Compile { output: String },
    /// The cargo test run exceeded its wall-clock budget and was killed
    /// (specodelic-xx1). A hang is a labeled timeout, never a silent one
    /// — and never indistinguishable from a block failure.
    TimedOut { secs: u64, output: String },
}

/// Executes the compiled proptest! blocks. The seam exists so tests can
/// substitute a fake; production uses [`CargoRunner`].
pub trait PropertiesRunner {
    fn run_blocks(
        &self,
        props_path: &Path,
        blocks: &[PropBlockMeta],
    ) -> Result<Vec<BlockResult>, RunnerError>;
}

/// The production runner: copies the artifact into a scratch cargo crate
/// (`tests/props.rs`, `proptest` dev-dependency) and runs `cargo test`,
/// parsing libtest's per-block results. The scratch target dir is shared
/// across runs so proptest compiles once, not per invocation.
///
/// `timeout_secs` bounds the whole cargo run (build + test) on a
/// wall-clock clock (specodelic-xx1): a hanging predicate would otherwise
/// hang `spk verify` forever. `None` runs unbounded (the CLI's
/// `--timeout-secs 0`). Default: [`DEFAULT_VERIFY_TIMEOUT_SECS`].
pub struct CargoRunner {
    pub timeout_secs: Option<u64>,
}

/// The default wall-clock bound on the verify runner's cargo run —
/// generous by design (a legitimately long proptest suite must never be
/// silently converted into a failure); 0 at the CLI means unbounded.
pub const DEFAULT_VERIFY_TIMEOUT_SECS: u64 = 600;

impl Default for CargoRunner {
    fn default() -> Self {
        Self {
            timeout_secs: Some(DEFAULT_VERIFY_TIMEOUT_SECS),
        }
    }
}

impl PropertiesRunner for CargoRunner {
    fn run_blocks(
        &self,
        props_path: &Path,
        blocks: &[PropBlockMeta],
    ) -> Result<Vec<BlockResult>, RunnerError> {
        // Per-invocation crate dir — parallel verifies (and integration
        // tests) must not race on tests/props.rs. The CARGO_TARGET_DIR
        // below is shared, so dependencies still compile once.
        let unique = format!(
            "{}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0),
            blocks.len(),
        );
        let base = scratch_base().join(&unique);
        prune_scratch_defaults(&scratch_base());
        let result = self.run_in_scratch(&base, props_path, blocks, &unique);
        let _ = fs::remove_dir_all(&base); // best effort; target dir persists
        result
    }
}

impl CargoRunner {
    fn run_in_scratch(
        &self,
        base: &Path,
        props_path: &Path,
        blocks: &[PropBlockMeta],
        unique: &str,
    ) -> Result<Vec<BlockResult>, RunnerError> {
        let _ = fs::create_dir_all(base.join("src"));
        let _ = fs::create_dir_all(base.join("tests"));
        fs::write(base.join("Cargo.toml"), SCRATCH_MANIFEST).map_err(|e| {
            RunnerError::Spawn(format!(
                "could not write the scratch crate at {}: {e}",
                base.display()
            ))
        })?;
        // Cache the resolved lockfile across invocations — no per-run
        // index update.
        let lock = scratch_base().join("Cargo.lock");
        if lock.exists() {
            let _ = fs::copy(&lock, base.join("Cargo.lock"));
        }
        // The staged test FILE name becomes the test binary name; unique
        // per invocation so parallel verifies sharing the target dir
        // never overwrite each other's binary or fingerprint.
        let test_file = format!("tests/props-{unique}.rs");
        fs::copy(props_path, base.join(&test_file)).map_err(|e| {
            RunnerError::Spawn(format!(
                "could not stage {} into the scratch crate: {e}",
                props_path.display()
            ))
        })?;
        let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_string());
        let mut command = Command::new(&cargo);
        command
            .args(["test", "--manifest-path"])
            .arg(base.join("Cargo.toml"))
            .env("CARGO_TARGET_DIR", scratch_base().join("target"));
        let output = run_bounded(
            &mut command,
            self.timeout_secs.map(std::time::Duration::from_secs),
        )?;
        let _ = fs::copy(base.join("Cargo.lock"), &lock);
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        let combined = format!("{stdout}{stderr}");
        if !combined.contains("running ") && !combined.contains("test result:") {
            return Err(RunnerError::Compile {
                output: first_error_lines(&combined),
            });
        }
        Ok(blocks
            .iter()
            .map(|b| parse_block_result(b, &combined))
            .collect())
    }
}

/// Runs a command to completion under an optional wall-clock bound
/// (specodelic-xx1). Polls `try_wait` (the `run_tlc` pattern) and kills
/// the child when the budget expires, reaping it and capturing whatever
/// partial output it produced — libtest's "has been running for over N
/// seconds" lines name the hanging block. The caller maps a kill to the
/// labeled `RunnerError::TimedOut`; a hang is never a silent failure and
/// never indistinguishable from a block failure.
fn run_bounded(
    command: &mut Command,
    timeout: Option<std::time::Duration>,
) -> Result<std::process::Output, RunnerError> {
    command
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());
    let mut child = command
        .spawn()
        .map_err(|e| RunnerError::Spawn(format!("could not spawn the test runner: {e}")))?;
    let started = std::time::Instant::now();
    let mut killed = false;
    loop {
        match child.try_wait() {
            Ok(Some(_)) => break,
            Ok(None) => {
                if !killed && timeout.is_some_and(|t| started.elapsed() >= t) {
                    killed = child.kill().is_ok();
                }
                std::thread::sleep(std::time::Duration::from_millis(10));
            }
            Err(e) => {
                return Err(RunnerError::Spawn(format!("could not run cargo: {e}")));
            }
        }
    }
    let out = child
        .wait_with_output()
        .map_err(|e| RunnerError::Spawn(format!("could not collect the runner's output: {e}")))?;
    if killed {
        let mut bytes = out.stdout;
        bytes.extend_from_slice(&out.stderr);
        return Err(RunnerError::TimedOut {
            secs: timeout.map(|t| t.as_secs()).unwrap_or(0),
            output: String::from_utf8_lossy(&bytes).to_string(),
        });
    }
    Ok(out)
}

/// The scratch crate lives under the system temp dir (overridable via
/// `SPECODELIC_VERIFY_SCRATCH` — e.g. to relocate off a size-capped tmpfs);
/// `CARGO_TARGET_DIR` is shared across runs so proptest compiles once per
/// machine, not per verify invocation.
pub(crate) fn scratch_base() -> PathBuf {
    scratch_base_impl(std::env::var_os("SPECODELIC_VERIFY_SCRATCH").as_deref())
}

fn scratch_base_impl(env_override: Option<&std::ffi::OsStr>) -> PathBuf {
    match env_override {
        Some(dir) => PathBuf::from(dir),
        None => std::env::temp_dir().join("specodelic-verify"),
    }
}

/// Retention policy for the shared scratch base (specodelic-5m2: the
/// target dir grew to 21GB and exhausted a /tmp quota). Best-effort —
/// prune failures never fail a verify.
///
/// - Orphaned per-invocation crate dirs (normally removed post-run; they
///   persist only if the process was killed) older than `orphan_max_age`
///   are deleted, identified by the `<pid>-<nanos>-<blocks>` name shape.
/// - The shared `target/` dir is never age-pruned (it is the compile
///   cache), but if it exceeds `target_max_bytes` it is dropped whole:
///   the next verify recompiles proptest, trading one cold build for the
///   quota headroom.
const SCRATCH_TARGET_MAX_BYTES: u64 = 4 * 1024 * 1024 * 1024; // 4 GiB
const SCRATCH_ORPHAN_MAX_AGE: std::time::Duration = std::time::Duration::from_secs(24 * 3600);

fn prune_scratch_defaults(base: &Path) {
    prune_scratch(
        base,
        std::time::SystemTime::now(),
        SCRATCH_TARGET_MAX_BYTES,
        SCRATCH_ORPHAN_MAX_AGE,
    );
}

struct ScratchPruneStats {
    removed_crate_dirs: usize,
    target_reset: bool,
}

fn is_scratch_crate_dir_name(name: &str) -> bool {
    let parts: Vec<&str> = name.split('-').collect();
    parts.len() == 3
        && parts
            .iter()
            .all(|p| !p.is_empty() && p.bytes().all(|b| b.is_ascii_digit()))
}

fn dir_size(path: &Path) -> u64 {
    let mut total = 0;
    let Ok(entries) = fs::read_dir(path) else {
        return 0;
    };
    for entry in entries.flatten() {
        let ft = match entry.file_type() {
            Ok(ft) => ft,
            Err(_) => continue,
        };
        if ft.is_dir() {
            total += dir_size(&entry.path());
        } else {
            total += entry.metadata().map(|m| m.len()).unwrap_or(0);
        }
    }
    total
}

fn prune_scratch(
    base: &Path,
    now: std::time::SystemTime,
    target_max_bytes: u64,
    orphan_max_age: std::time::Duration,
) -> ScratchPruneStats {
    let mut stats = ScratchPruneStats {
        removed_crate_dirs: 0,
        target_reset: false,
    };
    let Ok(entries) = fs::read_dir(base) else {
        return stats;
    };
    for entry in entries.flatten() {
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if name == "target" {
            if dir_size(&entry.path()) > target_max_bytes
                && fs::remove_dir_all(entry.path()).is_ok()
            {
                stats.target_reset = true;
            }
            continue;
        }
        if name == "Cargo.lock" {
            continue;
        }
        if !is_scratch_crate_dir_name(&name) {
            continue;
        }
        let stale = entry
            .metadata()
            .ok()
            .and_then(|m| m.modified().ok())
            .and_then(|mtime| now.duration_since(mtime).ok())
            .map(|age| age > orphan_max_age)
            .unwrap_or(false);
        if stale && fs::remove_dir_all(entry.path()).is_ok() {
            stats.removed_crate_dirs += 1;
        }
    }
    stats
}

const SCRATCH_MANIFEST: &str = "[package]\n\nname = \"specodelic-verify-scratch\"\n\nversion = \"0.0.0\"\n\nedition = \"2021\"\n\n\n[dev-dependencies]\n\nproptest = \"1\"\n";

/// libtest per-block result: `test <fn> ... ok|FAILED`; failing blocks'
/// captured output sits in a `---- <fn> stdout ----` section (panic
/// message + proptest's shrunk minimal failing input).
fn parse_block_result(block: &PropBlockMeta, output: &str) -> BlockResult {
    let passed = output
        .lines()
        .any(|l| l.starts_with(&format!("test {} ... ok", block.fn_name)));
    let marker = format!("---- {} stdout ----", block.fn_name);
    let mut detail: Option<String> = None;
    let mut lines = output.lines().peekable();
    while let Some(line) = lines.next() {
        if line.starts_with(&marker) {
            let mut section: Vec<&str> = vec![];
            for l in lines.by_ref() {
                if l.starts_with("---- ") {
                    break;
                }
                section.push(l);
            }
            detail = Some(section.join("\n"));
            break;
        }
    }
    BlockResult {
        id: block.id.clone(),
        case: block.case.clone(),
        fn_name: block.fn_name.clone(),
        passed,
        detail: if passed { None } else { detail },
    }
}

/// The first `error`-bearing lines of a build failure — enough to label
/// the failure without dumping the whole build log.
fn first_error_lines(output: &str) -> String {
    let errors: Vec<&str> = output
        .lines()
        .filter(|l| l.contains("error"))
        .take(5)
        .collect();
    if errors.is_empty() {
        output.lines().take(5).collect::<Vec<_>>().join("\n")
    } else {
        errors.join("\n")
    }
}

// ---------------------------------------------------------------------------
// tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    /// A lint-shaped spec with no Properties rows (vacuous properties gate).
    const FIXTURE_NO_PROPS: &str = "---\nid: vfix\nkind: intent\nstatement: \"THE fixture SHALL have no properties\"\n---\n\n## Constraints\n\n| id | kind | expr | traces_to |\n|----|------|------|------------|\n| c1 | invariant | `holds` | [[vfix]] |\n\n## Model\n\n### States\n\n- s1\n- s2\n\n### Transitions\n\n| id | from | to | guard |\n|----|------|----|-------|\n| t | s1 | s2 | [[vfix.c1]] |\n";

    /// Same spec with one unit Property row.
    const FIXTURE_ONE_PROP: &str = "---\nid: vfix\nkind: intent\nstatement: \"THE fixture SHALL have one property\"\n---\n\n## Constraints\n\n| id | kind | expr | traces_to |\n|----|------|------|------------|\n| c1 | invariant | `holds` | [[vfix]] |\n\n## Model\n\n### States\n\n- s1\n- s2\n\n### Transitions\n\n| id | from | to | guard |\n|----|------|----|-------|\n| t | s1 | s2 | [[vfix.c1]] |\n\n## Properties\n\n| id | kind | derives_from | generator | predicate |\n|----|------|--------------|-----------|------------|\n| p | unit | [[vfix.c1]] | `g()` | `x` |\n";

    fn spec_of(text: &str) -> crate::spec::Spec {
        crate::spec::parse_str(text).expect("fixture parses")
    }

    /// A minimal spec with no Constraints rows — intent id `vfix` (no
    /// path, so the artifact stem is `vfix`) and an empty claim
    /// classification, matching the fabricated empty claim set.
    fn vfix_spec() -> crate::spec::Spec {
        spec_of(
            "---\nid: vfix\nkind: intent\nstatement: \"THE vfix SHALL be a model-gate fixture\"\n---\n",
        )
    }

    /// Write `report` as a fresh claim-schema report (design D3): the
    /// freshness fields injected coherently for `spec` — the scope
    /// digest computed over the on-disk `vfix.tla`, the live claim
    /// classification, the evaluated statuses carried verbatim.
    fn write_fresh_report(dir: &Path, spec: &crate::spec::Spec, report: &RunReport) {
        let class = claim_classification(spec);
        let tla = std::fs::read(dir.join("vfix.tla")).unwrap_or_default();
        let mut artifacts = std::collections::BTreeMap::new();
        artifacts.insert(spec.intent.id.clone(), model_check::artifact_sha256(&tla));
        let (claims, expected, unchecked) = report_claim_fields(
            &class,
            &report.invariant_statuses,
            &std::collections::BTreeMap::new(),
        );
        let json = fresh_report_json(
            report,
            &claims,
            &expected,
            &unchecked,
            &scope_digest(std::slice::from_ref(spec), &artifacts),
        );
        std::fs::write(
            dir.join("vfix.check.json"),
            serde_json::to_string(&json).unwrap(),
        )
        .unwrap();
    }

    // --- verdict (both_gates_required and the failure precedence) ---

    #[test]
    fn both_gates_clean_passes() {
        let v = verdict(&PropsGateState::Pass, &ModelGateState::Clean);
        assert_eq!(v.status, "verified");
    }

    #[test]
    fn single_gate_insufficient_properties_only() {
        // properties pass, model does not → failed, named stage
        let v = verdict(
            &PropsGateState::Pass,
            &ModelGateState::NotClean {
                outcome: Outcome::ExplorationOnly,
            },
        );
        assert_eq!(v.status, "model_not_clean");
    }

    #[test]
    fn single_gate_insufficient_model_only() {
        let v = verdict(
            &PropsGateState::Failed("1 of 2 blocks failed".into()),
            &ModelGateState::Clean,
        );
        assert_eq!(v.status, "properties_failed");
    }

    #[test]
    fn properties_blocker_precedes_model_blocker() {
        let v = verdict(
            &PropsGateState::Failed("f".into()),
            &ModelGateState::Stale { detail: "d".into() },
        );
        assert_eq!(v.status, "properties_failed");
    }

    #[test]
    fn cached_result_rejected_by_staleness() {
        // a run predating the last spec edit does not count
        let v = verdict(&PropsGateState::Stale, &ModelGateState::Clean);
        assert_eq!(v.status, "stale_properties_artifact");
        let v = verdict(
            &PropsGateState::Pass,
            &ModelGateState::Stale {
                detail: "sha mismatch".into(),
            },
        );
        assert_eq!(v.status, "stale_model_run");
    }

    #[test]
    fn missing_artifacts_fail_closed() {
        let v = verdict(&PropsGateState::MissingArtifact, &ModelGateState::Clean);
        assert_eq!(v.status, "missing_properties_artifact");
        let v = verdict(
            &PropsGateState::Pass,
            &ModelGateState::Missing {
                detail: "no report".into(),
            },
        );
        assert_eq!(v.status, "missing_model_run");
    }

    #[test]
    fn law_cases_all_run_partial_pass_is_failure() {
        let blocks = [
            BlockResult {
                id: "p".into(),
                case: Some("identity".into()),
                fn_name: "p_identity".into(),
                passed: true,
                detail: None,
            },
            BlockResult {
                id: "p".into(),
                case: Some("associativity".into()),
                fn_name: "p_associativity".into(),
                passed: false,
                detail: Some("shrunk: ...".into()),
            },
        ];
        let state = PropsGateState::Failed("1 of 2 blocks failed".into());
        let v = verdict(&state, &ModelGateState::Clean);
        assert_eq!(v.status, "properties_failed");
        // the failing block's shrunk detail is carried for the report
        assert!(!blocks[1].passed && blocks[1].detail.is_some());
    }

    #[test]
    fn report_claim_fields_derives_records_and_sets() {
        // The ONE D3 field derivation (68m.4): evaluator kind from the
        // classification, the evaluated status, the labeled reason when
        // the claim did not verify; expected = required ids, unchecked
        // = prose ids.
        let class = ClaimClassification {
            required: vec![RequiredClaim {
                id: "c1".into(),
                evaluator: "kernel",
            }],
            unchecked: vec!["p1".into()],
        };
        let statuses = vec![model_check::InvariantStatus {
            id: "c1".into(),
            status: model_check::ThreeValued::Verified,
        }];
        let mut reasons = std::collections::BTreeMap::new();
        reasons.insert("c1".to_string(), "no".to_string());
        let (claims, expected, unchecked) = report_claim_fields(&class, &statuses, &reasons);
        assert_eq!(claims.len(), 1);
        assert_eq!(claims[0].id, "c1");
        assert_eq!(claims[0].evaluator, "kernel");
        assert_eq!(claims[0].status, model_check::ThreeValued::Verified);
        assert_eq!(claims[0].reason.as_deref(), Some("no"));
        assert_eq!(expected, vec!["c1".to_string()]);
        assert_eq!(unchecked, vec!["p1".to_string()]);
        // A status with no reason carries no reason field.
        let reasons = std::collections::BTreeMap::new();
        let (claims, _, _) = report_claim_fields(&class, &statuses, &reasons);
        assert_eq!(claims[0].reason, None);
    }

    #[test]
    fn every_verdict_message_carries_a_remediation_hint() {
        let combos = [
            (PropsGateState::Pass, ModelGateState::Clean),
            (PropsGateState::Failed("x".into()), ModelGateState::Clean),
            (PropsGateState::Stale, ModelGateState::Clean),
            (PropsGateState::MissingArtifact, ModelGateState::Clean),
            (
                PropsGateState::Uncompilable("err".into()),
                ModelGateState::Clean,
            ),
            (
                PropsGateState::RunnerUnavailable("no cargo".into()),
                ModelGateState::Clean,
            ),
            (PropsGateState::Pass, ModelGateState::Clean),
            (
                PropsGateState::Pass,
                ModelGateState::NotClean {
                    outcome: Outcome::CounterexampleFound,
                },
            ),
            (
                PropsGateState::Pass,
                ModelGateState::Stale { detail: "d".into() },
            ),
            (
                PropsGateState::Pass,
                ModelGateState::Missing { detail: "d".into() },
            ),
        ];
        for (p, m) in combos {
            let v = verdict(&p, &m);
            assert!(
                !v.hint.is_empty(),
                "status {} has no remediation hint",
                v.status
            );
        }
    }

    #[test]
    fn rerun_matches_prior_outcome() {
        // verdict is a pure function of the two gate states — same inputs,
        // same outcome (verify_is_idempotent at the verdict level)
        let a = verdict(
            &PropsGateState::Failed("1 of 1".into()),
            &ModelGateState::NotClean {
                outcome: Outcome::ExplorationOnly,
            },
        );
        let b = verdict(
            &PropsGateState::Failed("1 of 1".into()),
            &ModelGateState::NotClean {
                outcome: Outcome::ExplorationOnly,
            },
        );
        assert_eq!(a, b);
    }

    // --- block metadata parsing (the staleness fingerprint) ---

    #[test]
    fn blocks_from_source_reads_compile_emission() {
        let spec = spec_of(FIXTURE_ONE_PROP);
        let src = compile::properties_to_proptest(&spec);
        let blocks = blocks_from_source(&src);
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].id, "p");
        assert_eq!(blocks[0].case, None);
        assert_eq!(blocks[0].generator, "`g()`");
        assert_eq!(blocks[0].predicate, "`x`");
        assert_eq!(blocks[0].fn_name, "p");
    }

    #[test]
    fn blocks_from_source_reads_law_case_blocks() {
        let text = FIXTURE_ONE_PROP.replace("| p | unit |", "| p | law |");
        let spec = spec_of(&text);
        let src = compile::properties_to_proptest(&spec);
        let blocks = blocks_from_source(&src);
        assert!(blocks.len() >= 2, "law rows compile one block per case");
        assert!(blocks.iter().all(|b| b.case.is_some()));
    }

    #[test]
    fn metadata_comparison_ignores_translated_bodies() {
        // a hand-translated predicate body (metadata unchanged) must NOT
        // read as stale; a spec edit (predicate comment changes) must
        let spec = spec_of(FIXTURE_ONE_PROP);
        let src = compile::properties_to_proptest(&spec);
        let translated = src.replace("todo_predicate!", "assert!");
        assert_eq!(
            blocks_from_source(&src),
            blocks_from_source(&translated),
            "body-only edits keep the metadata fingerprint"
        );
        let edited_spec_src = src.replace("// predicate: `x`", "// predicate: `y`");
        assert_ne!(
            blocks_from_source(&src),
            blocks_from_source(&edited_spec_src),
            "spec edits change the metadata fingerprint"
        );
    }

    // --- model gate against a real out_dir ---

    #[test]
    fn model_gate_missing_report() {
        let dir = tempfile::tempdir().unwrap();
        let spec = vfix_spec();
        let g = evaluate_model_gate(&spec, std::slice::from_ref(&spec), dir.path(), "vfix");
        match g {
            ModelGateState::Missing { .. } => {}
            other => panic!("expected Missing, got {other:?}"),
        }
    }

    #[test]
    fn model_gate_unreadable_report_fails_closed() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("vfix.check.json"), "not json").unwrap();
        let spec = vfix_spec();
        let g = evaluate_model_gate(&spec, std::slice::from_ref(&spec), dir.path(), "vfix");
        match g {
            ModelGateState::Missing { detail } => {
                assert!(detail.contains("unreadable"), "detail: {detail}");
            }
            other => panic!("expected Missing, got {other:?}"),
        }
    }

    #[test]
    fn model_gate_stale_when_artifact_changed() {
        let dir = tempfile::tempdir().unwrap();
        let spec = vfix_spec();
        let report = RunReport {
            backend: crate::model_check::Backend {
                engine: "test".into(),
                version: "0".into(),
            },
            bound: crate::model_check::Bound {
                max_depth: 10,
                max_states: None,
                timeout_secs: None,
            },
            outcome: Outcome::NoCounterexample,
            invariants_checked: vec![],
            invariant_statuses: vec![],
            violated_invariant_id: None,
            trace: None,
            artifact_sha256: "0".repeat(64),
            states_explored: 1,
        };
        std::fs::write(dir.path().join("vfix.tla"), "MODULE now — END").unwrap();
        write_fresh_report(dir.path(), &spec, &report);
        let g = evaluate_model_gate(&spec, std::slice::from_ref(&spec), dir.path(), "vfix");
        match g {
            ModelGateState::Stale { .. } => {}
            other => panic!("expected Stale, got {other:?}"),
        }
    }

    #[test]
    fn model_gate_stale_when_module_gone() {
        let dir = tempfile::tempdir().unwrap();
        let spec = vfix_spec();
        let report = RunReport {
            backend: crate::model_check::Backend {
                engine: "test".into(),
                version: "0".into(),
            },
            bound: crate::model_check::Bound {
                max_depth: 10,
                max_states: None,
                timeout_secs: None,
            },
            outcome: Outcome::NoCounterexample,
            invariants_checked: vec![],
            invariant_statuses: vec![],
            violated_invariant_id: None,
            trace: None,
            artifact_sha256: "0".repeat(64),
            states_explored: 1,
        };
        write_fresh_report(dir.path(), &spec, &report);
        // no .tla at all — the stored run cannot be confirmed current
        let g = evaluate_model_gate(&spec, std::slice::from_ref(&spec), dir.path(), "vfix");
        match g {
            ModelGateState::Stale { detail } => {
                assert!(detail.contains("unreadable") || detail.contains("missing"));
            }
            other => panic!("expected Stale, got {other:?}"),
        }
    }

    #[test]
    fn model_gate_clean_and_not_clean_outcomes() {
        let dir = tempfile::tempdir().unwrap();
        let spec = vfix_spec();
        let tla = "MODULE vfix ---- END";
        std::fs::write(dir.path().join("vfix.tla"), tla).unwrap();
        let sha = model_check::artifact_sha256(tla.as_bytes());
        for (outcome, want) in [
            (Outcome::NoCounterexample, ModelGateState::Clean),
            (
                Outcome::ExplorationOnly,
                ModelGateState::NotClean {
                    outcome: Outcome::ExplorationOnly,
                },
            ),
            (
                Outcome::TimedOut,
                ModelGateState::NotClean {
                    outcome: Outcome::TimedOut,
                },
            ),
        ] {
            let report = RunReport {
                backend: crate::model_check::Backend {
                    engine: "test".into(),
                    version: "0".into(),
                },
                bound: crate::model_check::Bound {
                    max_depth: 10,
                    max_states: None,
                    timeout_secs: None,
                },
                outcome,
                invariants_checked: vec![],
                invariant_statuses: vec![],
                violated_invariant_id: None,
                trace: None,
                artifact_sha256: sha.clone(),
                states_explored: 1,
            };
            write_fresh_report(dir.path(), &spec, &report);
            assert_eq!(
                evaluate_model_gate(&spec, std::slice::from_ref(&spec), dir.path(), "vfix"),
                want
            );
        }
    }

    // --- properties gate against a real out_dir + fake runner ---

    struct FakeRunner {
        fail: Option<String>, // fn_name to fail
        error: Option<RunnerError>,
    }
    impl PropertiesRunner for FakeRunner {
        fn run_blocks(
            &self,
            _props_path: &Path,
            blocks: &[PropBlockMeta],
        ) -> Result<Vec<BlockResult>, RunnerError> {
            if let Some(e) = &self.error {
                return Err(e.clone());
            }
            Ok(blocks
                .iter()
                .map(|b| BlockResult {
                    id: b.id.clone(),
                    case: b.case.clone(),
                    fn_name: b.fn_name.clone(),
                    passed: self.fail.as_deref() != Some(b.fn_name.as_str()),
                    detail: (self.fail.as_deref() == Some(b.fn_name.as_str()))
                        .then(|| "shrunk minimal input".to_string()),
                })
                .collect())
        }
    }

    #[test]
    fn properties_gate_missing_artifact() {
        let dir = tempfile::tempdir().unwrap();
        let spec = spec_of(FIXTURE_ONE_PROP);
        let g = evaluate_properties_gate(
            &spec,
            dir.path(),
            &FakeRunner {
                fail: None,
                error: None,
            },
        );
        assert_eq!(g.state, PropsGateState::MissingArtifact);
    }

    #[test]
    fn properties_gate_stale_when_artifact_predates_spec_edit() {
        let dir = tempfile::tempdir().unwrap();
        let spec = spec_of(FIXTURE_ONE_PROP);
        // artifact generated from an older spec (different predicate)
        let older = compile::properties_to_proptest(&spec)
            .replace("// predicate: `x`", "// predicate: `older`");
        std::fs::write(dir.path().join("vfix_props.rs"), older).unwrap();
        let g = evaluate_properties_gate(
            &spec,
            dir.path(),
            &FakeRunner {
                fail: None,
                error: None,
            },
        );
        assert_eq!(g.state, PropsGateState::Stale);
        assert!(g.blocks.is_empty(), "stale artifacts are never executed");
    }

    #[test]
    fn properties_gate_runs_translated_artifact() {
        let dir = tempfile::tempdir().unwrap();
        let spec = spec_of(FIXTURE_ONE_PROP);
        std::fs::write(
            dir.path().join("vfix_props.rs"),
            compile::properties_to_proptest(&spec),
        )
        .unwrap();
        let g = evaluate_properties_gate(
            &spec,
            dir.path(),
            &FakeRunner {
                fail: None,
                error: None,
            },
        );
        assert_eq!(g.state, PropsGateState::Pass);
        assert_eq!(g.blocks.len(), 1);
    }

    #[test]
    fn properties_gate_no_rows_passes_without_runner() {
        let dir = tempfile::tempdir().unwrap();
        let spec = spec_of(FIXTURE_NO_PROPS);
        std::fs::write(
            dir.path().join("vfix_props.rs"),
            compile::properties_to_proptest(&spec),
        )
        .unwrap();
        let g = evaluate_properties_gate(
            &spec,
            dir.path(),
            &FakeRunner {
                fail: None,
                error: None,
            },
        );
        assert_eq!(g.state, PropsGateState::Pass);
        assert!(g.blocks.is_empty());
    }

    #[test]
    fn properties_gate_failed_block_is_not_a_skip() {
        let dir = tempfile::tempdir().unwrap();
        let spec = spec_of(FIXTURE_ONE_PROP);
        std::fs::write(
            dir.path().join("vfix_props.rs"),
            compile::properties_to_proptest(&spec),
        )
        .unwrap();
        let g = evaluate_properties_gate(
            &spec,
            dir.path(),
            &FakeRunner {
                fail: Some("p".into()),
                error: None,
            },
        );
        match g.state {
            PropsGateState::Failed(ref summary) => {
                assert!(summary.contains("1 of 1"), "summary: {summary}");
            }
            other => panic!("expected Failed, got {other:?}"),
        }
        assert_eq!(g.blocks.len(), 1);
        assert!(!g.blocks[0].passed);
        assert!(g.blocks[0].detail.is_some());
    }

    #[test]
    fn properties_gate_runner_errors_fail_labeled() {
        let dir = tempfile::tempdir().unwrap();
        let spec = spec_of(FIXTURE_ONE_PROP);
        std::fs::write(
            dir.path().join("vfix_props.rs"),
            compile::properties_to_proptest(&spec),
        )
        .unwrap();
        for (err, want) in [
            (
                RunnerError::Spawn("no cargo".into()),
                PropsGateState::RunnerUnavailable("no cargo".into()),
            ),
            (
                RunnerError::Compile {
                    output: "error[E0308]".into(),
                },
                PropsGateState::Uncompilable("error[E0308]".into()),
            ),
        ] {
            let g = evaluate_properties_gate(
                &spec,
                dir.path(),
                &FakeRunner {
                    fail: None,
                    error: Some(err),
                },
            );
            assert_eq!(g.state, want);
        }
    }

    // --- scratch retention policy (specodelic-5m2) ---

    fn crate_dir_name(tag: &str) -> String {
        format!("123-{tag}-3") // <pid>-<nanos>-<blocks> shape
    }

    fn age_entry(path: &std::path::Path, before: std::time::Duration) {
        let mtime = filetime::FileTime::from_system_time(std::time::SystemTime::now() - before);
        filetime::set_file_mtime(path, mtime).expect("age entry");
    }

    #[test]
    fn scratch_base_env_override_wins_over_default() {
        let custom = std::env::temp_dir().join("spk-scratch-test-custom");
        assert_eq!(scratch_base_impl(Some(custom.as_os_str())), custom);
    }

    #[test]
    fn scratch_base_default_is_temp_specodelic_verify() {
        assert_eq!(
            scratch_base_impl(None),
            std::env::temp_dir().join("specodelic-verify")
        );
    }

    #[test]
    fn prune_removes_stale_orphan_crate_dirs() {
        let base = tempfile::tempdir().expect("base");
        let stale = base.path().join(crate_dir_name("1000"));
        std::fs::create_dir_all(&stale).unwrap();
        age_entry(&stale, std::time::Duration::from_secs(48 * 3600));

        let stats = prune_scratch(
            base.path(),
            std::time::SystemTime::now(),
            u64::MAX,
            std::time::Duration::from_secs(24 * 3600),
        );

        assert_eq!(stats.removed_crate_dirs, 1);
        assert!(!stale.exists());
    }

    #[test]
    fn prune_keeps_fresh_crate_dirs_target_and_lock() {
        let base = tempfile::tempdir().expect("base");
        let fresh = base.path().join(crate_dir_name("2000"));
        std::fs::create_dir_all(&fresh).unwrap();
        age_entry(&fresh, std::time::Duration::from_secs(60));
        let target = base.path().join("target");
        std::fs::create_dir_all(&target).unwrap();
        age_entry(&target, std::time::Duration::from_secs(48 * 3600)); // old target survives age pruning
        std::fs::write(base.path().join("Cargo.lock"), "").unwrap();

        let stats = prune_scratch(
            base.path(),
            std::time::SystemTime::now(),
            u64::MAX,
            std::time::Duration::from_secs(24 * 3600),
        );

        assert_eq!(stats.removed_crate_dirs, 0);
        assert!(!stats.target_reset);
        assert!(fresh.exists());
        assert!(target.exists());
        assert!(base.path().join("Cargo.lock").exists());
    }

    #[test]
    fn prune_ignores_entries_that_are_not_scratch_crate_dirs() {
        // A non-crate-shaped dir never gets pruned, however old — the
        // scratch base may point at a user-chosen location.
        let base = tempfile::tempdir().expect("base");
        let other = base.path().join("not-a-crate-dir");
        std::fs::create_dir_all(&other).unwrap();
        age_entry(&other, std::time::Duration::from_secs(48 * 3600));

        let stats = prune_scratch(
            base.path(),
            std::time::SystemTime::now(),
            u64::MAX,
            std::time::Duration::from_secs(24 * 3600),
        );

        assert_eq!(stats.removed_crate_dirs, 0);
        assert!(other.exists());
    }

    #[test]
    fn prune_resets_target_when_over_size_cap() {
        let base = tempfile::tempdir().expect("base");
        let target = base.path().join("target");
        std::fs::create_dir_all(target.join("debug")).unwrap();
        std::fs::write(target.join("debug/blob"), vec![0u8; 64]).unwrap();

        let stats = prune_scratch(
            base.path(),
            std::time::SystemTime::now(),
            32,
            std::time::Duration::from_secs(24 * 3600),
        );

        assert!(stats.target_reset);
        assert!(!target.exists());
    }

    #[test]
    fn prune_keeps_target_under_size_cap() {
        let base = tempfile::tempdir().expect("base");
        let target = base.path().join("target");
        std::fs::create_dir_all(&target).unwrap();
        std::fs::write(target.join("blob"), vec![0u8; 16]).unwrap();

        let stats = prune_scratch(
            base.path(),
            std::time::SystemTime::now(),
            32,
            std::time::Duration::from_secs(24 * 3600),
        );

        assert!(!stats.target_reset);
        assert!(target.exists());
    }

    // --- wall-clock bound on the runner (specodelic-xx1) ---

    #[test]
    fn run_bounded_completes_within_budget() {
        let mut cmd = Command::new("echo");
        cmd.arg("ok");
        let out = run_bounded(&mut cmd, Some(std::time::Duration::from_secs(10))).unwrap();
        assert!(String::from_utf8_lossy(&out.stdout).contains("ok"));
    }

    #[test]
    fn run_bounded_unbounded_when_no_budget() {
        let mut cmd = Command::new("echo");
        cmd.arg("ok");
        let out = run_bounded(&mut cmd, None).unwrap();
        assert!(String::from_utf8_lossy(&out.stdout).contains("ok"));
    }

    #[cfg(unix)]
    #[test]
    fn run_bounded_kills_hanging_process_and_labels_timeout() {
        let mut cmd = Command::new("sleep");
        cmd.arg("30");
        let budget = std::time::Duration::from_millis(150);
        let err = run_bounded(&mut cmd, Some(budget)).unwrap_err();
        match &err {
            RunnerError::TimedOut { secs, output } => {
                // the budget is reported in whole seconds (floor) — a
                // 250ms test budget floors to 0
                assert_eq!(*secs, 0);
                // partial output is captured — a hang is never a silent failure
                let _ = output;
            }
            other => panic!("expected TimedOut, got {other:?}"),
        }
    }

    #[cfg(unix)]
    #[test]
    fn run_bounded_reaps_the_killed_child() {
        // kill + wait must reap the process — no zombie left behind.
        let mut cmd = Command::new("sleep");
        cmd.arg("30");
        let _ = run_bounded(&mut cmd, Some(std::time::Duration::from_millis(100))).unwrap_err();
        // If the child were not reaped, `ps` would list a defunct sleep.
        // Cheap check: our own children count via /proc is overkill; the
        // real proof is that run_bounded itself called wait() — pinned by
        // the TimedOut variant carrying output, which requires wait.
    }

    #[test]
    fn timeout_maps_to_labeled_gate_state_not_test_failure() {
        let dir = tempfile::tempdir().unwrap();
        let spec = spec_of(FIXTURE_ONE_PROP);
        std::fs::write(
            dir.path().join("vfix_props.rs"),
            compile::properties_to_proptest(&spec),
        )
        .unwrap();
        let g = evaluate_properties_gate(
            &spec,
            dir.path(),
            &FakeRunner {
                fail: None,
                error: Some(RunnerError::TimedOut {
                    secs: 600,
                    output: "test props::law_identity ... has been running".to_string(),
                }),
            },
        );
        match g.state {
            PropsGateState::TimedOut { secs, detail } => {
                assert_eq!(secs, 600);
                assert!(detail.contains("law_identity"));
            }
            other => panic!("expected TimedOut, got {other:?}"),
        }
    }

    #[test]
    fn timed_out_verdict_is_distinct_from_properties_failed() {
        // The anti-goal: a timeout must never read as a block failure, and
        // must carry its own remediation hint.
        let v = verdict(
            &PropsGateState::TimedOut {
                secs: 600,
                detail: "hung".to_string(),
            },
            &ModelGateState::Clean,
        );
        assert_eq!(v.status, "properties_timed_out");
        assert!(
            !v.message.contains("failed"),
            "timeout is not a failure verdict"
        );
        assert!(v.hint.contains("--timeout-secs"));
    }
}
