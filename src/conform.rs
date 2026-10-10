//! conform — external-oracle trace conformance verdict engine.
//!
//! Purpose: check a spec against reality instead of against itself —
//! classify traces recorded from an external oracle against what the
//! spec declares and can execute (openspec/changes/add-conform).
//! Responsibilities: the closed five-valued verdict taxonomy (design D1),
//! claim classification reusing model_check's machinery (D4), mechanical
//! trace classification against the compiled Model's transition relation
//! (D3 — string identity after trimming), and the closed-world evidence
//! gate (D2), plus the persisted report: conform-local schema version,
//! fixed evidence_scope (D6), scope digest binding structured content and
//! the consumed scenario corpus bytes, and genesis envelope emit
//! plumbing, plus the command-path input gate (D5): lint-clean file,
//! current compiled artifacts, and a fully validated scenario corpus.
//! Read-only: it evaluates, it never generates.
//! Rationale: no CLI wiring yet — the `spk conform` command lands in
//! phase 4; the gate and emit layers here are the testable API surface
//! it will call.

use std::collections::{BTreeMap, BTreeSet};
use std::io::Write;

use genesis::guide::{Output, OutputFormat, Verbosity};
use serde::{Deserialize, Serialize};

use crate::compile::{self, ModelIr};
use crate::spec::Spec;

// ---------------------------------------------------------------------------
// The closed verdict taxonomy (design D1 — five distinct values, never
// collapsed to pass/fail)
// ---------------------------------------------------------------------------

/// One oracle trace's verdict. The set is closed: `permitted`,
/// `forbidden`, `underspecified`, `unknown`, `unsupported` — distinct
/// values with distinct remediations, never coerced into a binary
/// pass/fail (report §13: never collapse the ladder into one PASS label).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Verdict {
    /// The trace is explainable by the declared Model and executable
    /// claims over it.
    Permitted,
    /// Positive contradiction with a declared executable claim (any
    /// invocation mode), or absence of coverage under an explicitly
    /// declared closed-world mode (design D2).
    Forbidden,
    /// No declared Model element or claim covers the trace — author a
    /// claim or declare the trace's vocabulary.
    Underspecified,
    /// A claim covers the trace but its content is not interpretable by
    /// this run (prose-only expr) — make the claim interpretable.
    Unknown,
    /// The covering claim's evaluator kind exists in the format but is
    /// not executable in this run (e.g. `**py:**` — no emitter yet).
    Unsupported,
}

impl Verdict {
    /// The verdict's snake_case spelling (report records, exit-code
    /// mapping, and reason text all use this one form).
    pub fn as_str(self) -> &'static str {
        match self {
            Verdict::Permitted => "permitted",
            Verdict::Forbidden => "forbidden",
            Verdict::Underspecified => "underspecified",
            Verdict::Unknown => "unknown",
            Verdict::Unsupported => "unsupported",
        }
    }
}

impl std::fmt::Display for Verdict {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

// ---------------------------------------------------------------------------
// Claim classification (design D4 — reuse model_check's machinery, no new
// status vocabulary: the required/unchecked split of
// `required_claims_classified`, extended only by the existing
// no_emitter_labeled_failure evaluator kinds)
// ---------------------------------------------------------------------------

/// The evaluator kind a claim opts into — the format's own closed set
/// (rust/py/ts fragments, kernel, citation); `prose` is the default that
/// opts into nothing (`required_claims_classified`: prose never implies
/// evaluation).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Evaluator {
    /// `**rust:**` fragment — executable in this run.
    RustFragment,
    /// `**kernel:**` expression — executable in this run.
    Kernel,
    /// Citation expression (`[[a]] ∧ [[b]]`) — executable via the
    /// citation algebra.
    Citation,
    /// Prose — not interpretable by this run; never implies a judgment.
    Prose,
    /// `**py:**` fragment — grammar-valid, no emitter in this run
    /// (`no_emitter_labeled_failure`).
    Py,
    /// `**ts:**` fragment — grammar-valid, no emitter in this run.
    Ts,
}

impl Evaluator {
    /// The kind's label as written in the format (`rust`, `kernel`,
    /// `citation`, `prose`, `py`, `ts`).
    pub fn label(self) -> &'static str {
        match self {
            Evaluator::RustFragment => "rust",
            Evaluator::Kernel => "kernel",
            Evaluator::Citation => "citation",
            Evaluator::Prose => "prose",
            Evaluator::Py => "py",
            Evaluator::Ts => "ts",
        }
    }

    /// True when the kind is executable in this run — the required set of
    /// `required_claims_classified`.
    pub fn is_executable(self) -> bool {
        matches!(
            self,
            Evaluator::RustFragment | Evaluator::Kernel | Evaluator::Citation
        )
    }
}

/// One invariant-kind Constraint row and the evaluator kind its expr
/// cell opts into.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClassifiedClaim {
    pub id: String,
    pub evaluator: Evaluator,
}

impl ClassifiedClaim {
    /// The kind's label (`unsupported_kind_named` names the kind).
    pub fn evaluator_label(&self) -> &'static str {
        self.evaluator.label()
    }
}

/// The `required_claims_classified` partition for one file's invariant
/// claims: `required` (executable in this run), `unchecked` (prose-only —
/// never implied verified or judged), `unsupported` (kind exists in the
/// format but has no emitter in this run).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ClaimClassification {
    pub required: Vec<ClassifiedClaim>,
    pub unchecked: Vec<ClassifiedClaim>,
    pub unsupported: Vec<ClassifiedClaim>,
}

impl ClaimClassification {
    /// The claim with this id, if the file declares it as an invariant.
    pub fn claim(&self, id: &str) -> Option<&ClassifiedClaim> {
        self.required
            .iter()
            .chain(self.unchecked.iter())
            .chain(self.unsupported.iter())
            .find(|c| c.id == id)
    }
}

/// Partition the file's invariant-kind Constraints into
/// required/unchecked/unsupported (design D4 — the same classification
/// model_check's `required_claims_classified` fixes: executable =
/// rust fragment, kernel, or citation; prose-only = unchecked; py/ts =
/// grammar-valid but emitter-less in this run). Advisory/effect/
/// extension-point rows are never required, so they never appear.
pub fn classify_claims(spec: &Spec) -> ClaimClassification {
    let ir = extract_ir(spec);
    let mut out = ClaimClassification::default();
    for row in &spec.constraints {
        if row.cells.get("kind").map(String::as_str) != Some("invariant") {
            continue;
        }
        let expr = row.cells.get("expr").map(String::as_str).unwrap_or("");
        let evaluator = match compile::fragment_of_strict(expr) {
            // A marker-shaped cell — its language decides executability.
            Ok(Some(fragment)) => match fragment.lang {
                compile::FragmentLang::Rust => Evaluator::RustFragment,
                compile::FragmentLang::Py => Evaluator::Py,
                compile::FragmentLang::Ts => Evaluator::Ts,
            },
            // No marker: kernel opt-in, else citation expr, else prose.
            _ => {
                if ir.guard_kernel.contains_key(row.id.as_str()) {
                    Evaluator::Kernel
                } else if compile::parse_citation_expr(expr).is_some() {
                    Evaluator::Citation
                } else {
                    Evaluator::Prose
                }
            }
        };
        let claim = ClassifiedClaim {
            id: row.id.clone(),
            evaluator,
        };
        match evaluator {
            e if e.is_executable() => out.required.push(claim),
            Evaluator::Prose => out.unchecked.push(claim),
            _ => out.unsupported.push(claim),
        }
    }
    out
}

/// Extract the compiled Model IR once per classification pass (the same
/// extraction model_check consumes — design D4). Memoized behind a cell
/// is unnecessary: the IR extraction is cheap and pure.
fn extract_ir(spec: &Spec) -> ModelIr {
    compile::extract_model_ir(spec)
}

// ---------------------------------------------------------------------------
// Scenario corpus (design D3 — one JSON object per line, mechanical name
// identity after trimming)
// ---------------------------------------------------------------------------

/// One recorded step of an oracle trace: the action taken (a transition
/// id/name) and optional observations (state/effect references). The
/// field is named `observations` — never `observes` — to avoid colliding
/// with the format's typed reference field of the same name (design D3).
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TraceStep {
    pub action: String,
    /// Observed state/effect references: the string values of the JSON
    /// object (keys are free-variable names, opaque to conform).
    #[serde(default)]
    pub observations: BTreeMap<String, serde_json::Value>,
}

/// One scenario: a unique id, the setup (opaque except for the initial
/// state it names), and the ordered trace.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ScenarioTrace {
    pub id: String,
    #[serde(default)]
    pub setup: BTreeMap<String, serde_json::Value>,
    pub trace: Vec<TraceStep>,
}

/// Rule 1's verdict depends on the recorded declaration (design D2):
/// absence of coverage is `underspecified` in the default open-world mode
/// (`uncovered_trace_never_forbidden`) and `forbidden` only under the
/// explicitly declared closed-world mode (`closed_world_forbidden_recorded`).
/// The detail names the uncovered element; the declaration is named in
/// the forbidden reason.
fn uncovered_record(
    scenario_id: &str,
    detail: String,
    evaluated: Vec<String>,
    closed_world: bool,
) -> VerdictRecord {
    let (verdict, reason) = if closed_world {
        (Verdict::Forbidden, reason::uncovered_forbidden(&detail))
    } else {
        (
            Verdict::Underspecified,
            reason::uncovered_open_world(&detail),
        )
    };
    VerdictRecord {
        scenario_id: scenario_id.to_string(),
        verdict,
        reason,
        evaluated_claim_ids: evaluated,
        closed_world,
    }
}

/// A labeled failure — never a silent partial result. Carries a
/// remediation hint per the error contract (design D3: unknown fields
/// are refused with a hint, never silently ignored).
#[derive(Debug, Clone, PartialEq)]
pub struct ConformError {
    pub stage: String,
    pub message: String,
}

impl std::fmt::Display for ConformError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.stage, self.message)
    }
}

impl std::error::Error for ConformError {}

/// Parse a JSONL scenario corpus: one object per line, each
/// `{id, setup?, trace: [{action, observations?}]}`. Unknown fields and
/// malformed lines are refused with a remediation hint, never silently
/// ignored (design D3): missing `id` and duplicate ids are refused too —
/// the report is keyed by scenario id, so a duplicate would silently
/// overwrite evidence.
pub fn parse_corpus(jsonl: &str) -> Result<Vec<ScenarioTrace>, ConformError> {
    let mut out = Vec::new();
    let mut seen: BTreeSet<String> = BTreeSet::new();
    for (n, line) in jsonl.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let scenario: ScenarioTrace = serde_json::from_str(line).map_err(|e| ConformError {
            stage: "corpus_parse".into(),
            message: format!(
                "line {}: {e} — fix the scenario record (one JSON object per line: {{id, setup?, trace}}; unknown fields are refused, never ignored)",
                n + 1
            ),
        })?;
        if !seen.insert(scenario.id.clone()) {
            return Err(ConformError {
                stage: "corpus_parse".into(),
                message: format!(
                    "line {}: duplicate scenario id `{}` — scenario ids must be unique across the corpus (one JSON object per line: {{id, setup?, trace}}); rename or drop the duplicate record",
                    n + 1,
                    scenario.id
                ),
            });
        }
        out.push(scenario);
    }
    Ok(out)
}

// ---------------------------------------------------------------------------
// The input gate (design D5 — lint-clean file, current compiled artifacts)
// ---------------------------------------------------------------------------

/// The command-path input gate (design D5, delta `artifact_gate`): the
/// target file must lint clean and its compiled artifacts must be
/// current before any classification runs. The checks reuse the exact
/// machinery the pipeline stages run — the orchestrate lint stage (the
/// seven gate checkers, via `orchestrate::run_lint_stage`) and compile's
/// own byte-stable emission (`compile_spec`, whose `.tla` artifact is
/// the staleness key `model_check::is_stale`/verify compare against).
/// A spec that fails to parse never reaches this gate: the parse stage
/// refuses it upstream (phase-4 CLI wiring), the same discipline
/// orchestrate applies before lint.
///
/// Refusals carry a remediation hint naming the failing stage's command
/// (`spk lint` / `spk compile`) per the error contract; the caller emits
/// zero verdict records on an Err.
pub fn gate(spec: &Spec, out_dir: &std::path::Path) -> Result<(), ConformError> {
    // Lint stage first — a lint-dirty file must be refused with the lint
    // remediation even if its artifacts are also missing/stale (the same
    // stage order orchestrate applies: parse → lint → compile).
    let stage = crate::orchestrate::run_lint_stage(std::slice::from_ref(spec), &[]);
    if stage.status != "passed" {
        let findings = lint_findings_of(&stage);
        return Err(ConformError {
            stage: "artifact_gate".into(),
            message: format!(
                "the file has lint findings — conform requires a lint-clean spec, verdicts are never derived from invalid artifacts ({}; run: spk lint)",
                findings
            ),
        });
    }
    // Artifact currency: the on-disk `<stem>.tla` must equal the current
    // spec's compile emission — the same regenerate-and-compare discipline
    // the properties gate applies to `*_props.rs` (verify.rs), with
    // compile's byte-stable contract making the comparison exact.
    let stem = compile::artifact_stem(spec);
    let tla_path = out_dir.join(format!("{stem}.tla"));
    let on_disk = std::fs::read(&tla_path).map_err(|e| {
        ConformError {
            stage: "artifact_gate".into(),
            message: format!(
                "no compiled module at {}: {e} — conform consumes compile's output, it never re-compiles (run: spk compile)",
                tla_path.display()
            ),
        }
    })?;
    let compiled = compile::compile_spec(spec).map_err(|e| ConformError {
        stage: "artifact_gate".into(),
        message: format!(
            "the spec no longer compiles ({}: {}) — the on-disk artifacts cannot be current (run: spk compile)",
            e.stage, e.message
        ),
    })?;
    if crate::model_check::artifact_sha256(&on_disk)
        != crate::model_check::artifact_sha256(compiled.tla.as_bytes())
    {
        return Err(ConformError {
            stage: "artifact_gate".into(),
            message: format!(
                "the compiled module at {} is stale: the spec's structured content changed after compile — verdicts are never derived from stale artifacts (run: spk compile to refresh the artifacts)",
                tla_path.display()
            ),
        });
    }
    Ok(())
}

/// The failed lint stage's finding summary: the stage detail nests
/// issues per checker (`checkers[].issues[].message`); the refusal names
/// the count and the first finding so the author can go straight to it.
fn lint_findings_of(stage: &crate::orchestrate::Stage) -> String {
    let issues: Vec<String> = stage
        .detail
        .as_ref()
        .and_then(|d| d.get("checkers"))
        .and_then(|v| v.as_array())
        .map(|checkers| {
            checkers
                .iter()
                .filter_map(|c| c.get("issues"))
                .filter_map(|v| v.as_array())
                .flatten()
                .filter_map(|i| i.get("message"))
                .filter_map(|m| m.as_str())
                .map(String::from)
                .collect()
        })
        .unwrap_or_default();
    let first = issues.first().map(String::as_str).unwrap_or("");
    if first.is_empty() {
        format!("{} checker issue(s)", issues.len())
    } else {
        format!("{} checker issue(s): {first}", issues.len())
    }
}

/// The command-path run the phase-4 CLI calls: gate (D5), then corpus
/// parse/validation (D3), then the report. Every refusal path returns
/// Err — zero verdict records are emitted for a gated-out invocation —
/// and a zero-line corpus is a valid Ok report with zero records,
/// `evidence_scope` intact, never an error.
pub fn run(
    spec: &Spec,
    out_dir: &std::path::Path,
    corpus_bytes: &[u8],
    closed_world: bool,
) -> Result<ConformReport, ConformError> {
    gate(spec, out_dir)?;
    let corpus = std::str::from_utf8(corpus_bytes).map_err(|e| ConformError {
        stage: "corpus_parse".into(),
        message: format!(
            "scenario corpus is not valid UTF-8: {e} — fix the scenario record (one JSON object per line: {{id, setup?, trace}})"
        ),
    })?;
    let scenarios = parse_corpus(corpus)?;
    Ok(build_report(spec, corpus_bytes, &scenarios, closed_world))
}

// ---------------------------------------------------------------------------
// Trace classification — the verdict engine
// ---------------------------------------------------------------------------

/// One trace's verdict record: the scenario it classifies, the verdict,
/// the labeled reason (naming the rule and the claims/elements that
/// produced it), the ids of the claims the run evaluated, and whether
/// the invocation declared closed-world mode (recorded in EVERY record,
/// never inferred — design D2).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VerdictRecord {
    pub scenario_id: String,
    pub verdict: Verdict,
    pub reason: String,
    pub evaluated_claim_ids: Vec<String>,
    pub closed_world: bool,
}

/// Classify every corpus trace against the spec's compiled Model
/// transition relation and its invariant claims. Every trace receives
/// exactly one verdict from the closed taxonomy (totality); classification
/// is mechanical — string identity after trimming, never semantic (D3).
///
/// The rules, in evaluation order per step:
///
/// 1. **Uncovered** — the trace references a name (initial state from
///    `setup`, action, or observation value) that no declared Model
///    element declares, or `setup` names no declared state: no Model
///    element or claim covers the trace. Open-world → `underspecified`
///    (`uncovered_trace_never_forbidden`); with the closed-world
///    declaration → `forbidden` (`closed_world_forbidden_recorded`).
///    Undeclared names alone never produce `forbidden` without the
///    declaration (design D2/D3).
/// 2. **Positive contradiction** — a declared transition exercised from
///    a state it is not declared from, or an observation naming a state
///    other than the transition's declared target: the trace asserts a
///    step/outcome the compiled Model forbids — the same fact
///    model_check reports as `counterexample_found`. `forbidden` in ANY
///    invocation mode (`contradiction_forbidden_any_mode`), naming the
///    executable claim(s) covering the violated transition.
/// 3. **Unsupported** — a covering claim's evaluator kind has no emitter
///    in this run (`**py:**`/`**ts:**`): `unsupported`, naming the kind.
/// 4. **Unknown** — a covering claim is prose-only: `unknown`, naming
///    the claim (prose never implies a permitted/forbidden judgment).
/// 5. **Permitted** — the walk explains the trace through declared
///    transitions and any executable covering claims are consistent
///    (their guards enabled each step).
pub fn classify_traces(
    spec: &Spec,
    corpus: &[ScenarioTrace],
    closed_world: bool,
) -> Vec<VerdictRecord> {
    let ir = extract_ir(spec);
    let claims = classify_claims(spec);
    corpus
        .iter()
        .map(|scenario| classify_trace(spec, &ir, &claims, scenario, closed_world))
        .collect()
}

/// A declared transition, trimmed (D3 — mechanical name identity).
struct DeclaredTransition {
    id: String,
    from: String,
    to: String,
}

/// The classified verdict for one scenario, by the rules documented on
/// [`classify_traces`].
fn classify_trace(
    spec: &Spec,
    ir: &ModelIr,
    claims: &ClaimClassification,
    scenario: &ScenarioTrace,
    closed_world: bool,
) -> VerdictRecord {
    // Declared vocabulary, trimmed — string identity is the only
    // normalization (design D3, mechanical never semantic).
    let transitions: BTreeMap<&str, DeclaredTransition> = ir
        .transitions
        .iter()
        .map(|t| {
            (
                t.id.trim(),
                DeclaredTransition {
                    id: t.id.clone(),
                    from: t.from.trim().to_string(),
                    to: t.to.trim().to_string(),
                },
            )
        })
        .collect();
    let states: BTreeSet<&str> = ir.states.iter().map(|s| s.id.trim()).collect();
    // Effect ids: the constraints the Model's `emits` references.
    let effects: BTreeSet<&str> = ir.emits.values().map(|v| v.trim()).collect();

    let record = |verdict: Verdict, reason: String, evaluated: Vec<String>| VerdictRecord {
        scenario_id: scenario.id.clone(),
        verdict,
        reason,
        evaluated_claim_ids: evaluated,
        closed_world,
    };

    // Rule 1 (anchor): setup names the initial state; an absent or
    // undeclared state leaves no Model element anchoring the trace.
    let initial = scenario
        .setup
        .get("state")
        .and_then(|v| v.as_str())
        .map(str::trim)
        .map(str::to_string);
    let Some(initial) = initial else {
        return uncovered_record(
            &scenario.id,
            reason::no_initial_state(&scenario.id),
            Vec::new(),
            closed_world,
        );
    };
    if !states.contains(initial.as_str()) {
        return uncovered_record(
            &scenario.id,
            reason::undeclared_state(&initial),
            Vec::new(),
            closed_world,
        );
    }

    let mut current = initial;
    let mut covering: Vec<ClassifiedClaim> = Vec::new();
    let mut exercised: Vec<String> = Vec::new();

    for (i, step) in scenario.trace.iter().enumerate() {
        let action = step.action.trim();
        // Rule 1: the action must name a declared transition.
        let Some(t) = transitions.get(action) else {
            return uncovered_record(
                &scenario.id,
                reason::undeclared_action(i + 1, action),
                covering_ids(&covering),
                closed_world,
            );
        };
        // Rule 2: positive contradiction — the step is not enabled from
        // the current state; the compiled Model forbids this move.
        if t.from != current {
            let guards = covering_claims_of(spec, claims, &t.id);
            let evaluated = covering_ids(&guards);
            return record(
                Verdict::Forbidden,
                reason::contradiction_step(&t.id, &current, &guards),
                evaluated,
            );
        }
        // Rule 1/2 on observations: an observation value must name a
        // declared state or effect (else uncovered); a state observation
        // must equal the transition's declared target (else the trace
        // asserts an outcome the Model forbids).
        for value in observed_names(step) {
            let value = value.trim();
            if states.contains(value) {
                if value != t.to.as_str() {
                    let guards = covering_claims_of(spec, claims, &t.id);
                    return record(
                        Verdict::Forbidden,
                        reason::contradiction_observation(i + 1, value, &t.id, &t.to, &guards),
                        covering_ids(&guards),
                    );
                }
            } else if !effects.contains(value) {
                return uncovered_record(
                    &scenario.id,
                    reason::undeclared_observation(i + 1, value),
                    covering_ids(&covering),
                    closed_world,
                );
            }
        }
        // The step is explained; accumulate its covering claims.
        let guards = covering_claims_of(spec, claims, &t.id);
        for claim in guards {
            if !covering.iter().any(|c| c.id == claim.id) {
                covering.push(claim);
            }
        }
        exercised.push(t.id.clone());
        current = t.to.clone();
    }

    // Rule 3: an unsupported covering claim names its kind — the kind
    // exists in the format but is not executable in this run.
    if let Some(claim) = covering
        .iter()
        .find(|c| !c.evaluator.is_executable() && c.evaluator != Evaluator::Prose)
    {
        return record(
            Verdict::Unsupported,
            reason::unsupported_kind(claim),
            covering_ids(&covering),
        );
    }
    // Rule 4: a prose-only covering claim is not interpretable — unknown,
    // naming the claim; prose never implies a judgment.
    if let Some(claim) = covering.iter().find(|c| c.evaluator == Evaluator::Prose) {
        return record(
            Verdict::Unknown,
            reason::prose_not_interpretable(claim),
            covering_ids(&covering),
        );
    }
    // Rule 5: the walk explains the trace; executable covering claims'
    // guards enabled each step — permitted.
    record(
        Verdict::Permitted,
        reason::permitted(&scenario.id, &exercised, &current, &covering),
        covering_ids(&covering),
    )
}

/// The string values of a step's observations object — the observed
/// state/effect references (keys are free-variable names, opaque).
fn observed_names(step: &TraceStep) -> Vec<String> {
    let mut out = Vec::new();
    for value in step.observations.values() {
        match value {
            serde_json::Value::String(s) => out.push(s.clone()),
            serde_json::Value::Array(items) => {
                out.extend(items.iter().filter_map(|i| i.as_str().map(str::to_string)));
            }
            _ => {}
        }
    }
    out
}

/// The invariant claims covering a transition: the local constraint ids
/// its guard cell cites (`[[...]]` targets, resolved to local row ids).
/// Mechanical extraction — a guard's cited claim ids are exactly the
/// links the format declares between the Model and the Constraints.
fn covering_claims_of(
    spec: &Spec,
    claims: &ClaimClassification,
    transition_id: &str,
) -> Vec<ClassifiedClaim> {
    let Some(t) = spec
        .transitions
        .iter()
        .find(|t| t.id.trim() == transition_id)
    else {
        return Vec::new();
    };
    let Some(guard) = t.guard.as_deref() else {
        return Vec::new();
    };
    cited_claim_ids(spec, guard)
        .into_iter()
        .filter_map(|id| claims.claim(&id).cloned())
        .collect()
}

/// Extract `[[...]]` citation targets from a guard cell and resolve them
/// to LOCAL invariant claim ids: a bare `locked` target is already local;
/// `demo.conform.locked` → `locked` only when the dotted prefix IS the
/// file's own intent id. A foreign citation (`other.spec.audited`) is an
/// outbound typed edge of record, never a local claim — resolving it by
/// suffix alone would misattribute a claim this run does not classify.
/// Mechanical string surgery, never semantic interpretation of the
/// guard's meaning.
fn cited_claim_ids(spec: &Spec, guard: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut rest = guard;
    while let Some(start) = rest.find("[[") {
        let after = &rest[start + 2..];
        let Some(end) = after.find("]]") else { break };
        let target = after[..end].trim();
        let own_prefix = format!("{}.", spec.intent.id);
        let local = if let Some(row) = target.strip_prefix(&own_prefix) {
            // `file.row` → local row id when the prefix is the file's own
            // intent id; anything else is a foreign citation (outbound
            // leaf — this run does not classify another file's claims).
            Some(row.to_string())
        } else if !target.contains('.') {
            // A bare `row` target is already local.
            Some(target.to_string())
        } else {
            None
        };
        if let Some(local) = local {
            out.push(local);
        }
        rest = &after[end + 2..];
    }
    out
}

// ---------------------------------------------------------------------------
// Verdict-reason formatting — the ONE place reason strings are built, as
// pure functions of their labeled inputs so the reason text is directly
// testable (tasks.md 1.5). Each function names the rule marker (the same
// tokens the delta's Properties table pins) and the claims/elements that
// produced the verdict.
// ---------------------------------------------------------------------------

/// The pure verdict-reason builders — testable without a spec, a corpus,
/// or a walk.
pub mod reason {
    use super::ClassifiedClaim;

    /// The executable-claim suffix a contradiction reason carries: names
    /// the contradicted executable claim id(s) covering the violated
    /// transition.
    fn executable_claims_suffix(guards: &[ClassifiedClaim]) -> String {
        let executable: Vec<&ClassifiedClaim> = guards
            .iter()
            .filter(|c| c.evaluator.is_executable())
            .collect();
        if executable.is_empty() {
            String::new()
        } else {
            format!(
                " — contradicts executable claim(s) {}",
                executable
                    .iter()
                    .map(|c| format!("`{}`", c.id))
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        }
    }

    /// `uncovered_trace_never_forbidden` — absence of coverage in the
    /// default open-world mode is underspecification, never prohibition.
    pub fn uncovered_open_world(detail: &str) -> String {
        format!("uncovered_trace_never_forbidden: {detail}")
    }

    /// `closed_world_forbidden_recorded` — the exhaustiveness declaration
    /// is named in the reason (design D2: the flag is the recorded
    /// declaration the failure criterion requires). Only called when the
    /// declaration was made, so the reason carries it unconditionally.
    pub fn uncovered_forbidden(detail: &str) -> String {
        format!(
            "closed_world_forbidden_recorded: {detail} — forbidden under the declared closed-world mode (--closed-world)"
        )
    }

    /// Uncovered detail: the scenario's setup names no initial state.
    pub fn no_initial_state(scenario_id: &str) -> String {
        format!(
            "scenario `{scenario_id}` declares no initial state in setup — no Model element anchors the trace"
        )
    }

    /// Uncovered detail: the setup's state names no declared Model state.
    pub fn undeclared_state(state: &str) -> String {
        format!(
            "setup state `{state}` is not a declared Model state — no Model element anchors the trace"
        )
    }

    /// Uncovered detail: a step's action names no declared transition.
    pub fn undeclared_action(step: usize, action: &str) -> String {
        format!(
            "step {step} action `{action}` is not a declared Model transition — no declared Model element or claim covers the trace"
        )
    }

    /// Uncovered detail: an observation names no declared state/effect.
    pub fn undeclared_observation(step: usize, value: &str) -> String {
        format!(
            "step {step} observation `{value}` names no declared Model state or effect — no declared Model element or claim covers the trace"
        )
    }

    /// `contradiction_forbidden_any_mode` — a declared transition
    /// exercised from a state it is not declared from.
    pub fn contradiction_step(
        transition: &str,
        from_state: &str,
        guards: &[ClassifiedClaim],
    ) -> String {
        format!(
            "contradiction_forbidden_any_mode: transition `{transition}` is declared from another state — exercised from `{from_state}`, the compiled Model's transition relation forbids this step{}",
            executable_claims_suffix(guards)
        )
    }

    /// `contradiction_forbidden_any_mode` — an observation contradicting
    /// the transition's declared target state.
    pub fn contradiction_observation(
        step: usize,
        observed: &str,
        transition: &str,
        target: &str,
        guards: &[ClassifiedClaim],
    ) -> String {
        format!(
            "contradiction_forbidden_any_mode: step {step} observed state `{observed}` but transition `{transition}` declares target `{target}` — the compiled Model forbids this outcome{}",
            executable_claims_suffix(guards)
        )
    }

    /// `unsupported_evaluator_kind` — the kind exists in the format but
    /// has no emitter in this run; the reason names the kind.
    pub fn unsupported_kind(claim: &ClassifiedClaim) -> String {
        format!(
            "unsupported_evaluator_kind: claim `{}` covers this trace but its evaluator kind `{}` has no emitter in this run — the claim exists in the format, is not executable here",
            claim.id,
            claim.evaluator.label()
        )
    }

    /// `prose_claim_not_interpretable` — the reason names the prose
    /// claim; prose never implies a permitted/forbidden judgment.
    pub fn prose_not_interpretable(claim: &ClassifiedClaim) -> String {
        format!(
            "prose_claim_not_interpretable: claim `{}` covers this trace but its expr is prose — not interpretable by this run, never implied permitted or forbidden",
            claim.id
        )
    }

    /// `permitted` — the walk explains the trace through the declared
    /// transition relation, with the executable covering claims named.
    pub fn permitted(
        scenario_id: &str,
        exercised: &[String],
        current_state: &str,
        covering: &[ClassifiedClaim],
    ) -> String {
        if exercised.is_empty() {
            return format!(
                "permitted: scenario `{scenario_id}` declares an empty trace from state `{current_state}` — explainable by the declared Model"
            );
        }
        let steps = exercised
            .iter()
            .map(|s| format!("`{s}`"))
            .collect::<Vec<_>>()
            .join(", ");
        let claims = if covering.is_empty() {
            String::new()
        } else {
            let ids: Vec<String> = covering.iter().map(|c| c.id.clone()).collect();
            format!(" — executable claims evaluated: {}", ids.join(", "))
        };
        format!(
            "permitted: trace through {steps} explainable by the declared Model's transition relation{claims}"
        )
    }
}

/// Evaluated claim ids, deduplicated in first-seen order.
fn covering_ids(covering: &[ClassifiedClaim]) -> Vec<String> {
    let mut seen = BTreeSet::new();
    covering
        .iter()
        .filter(|c| seen.insert(c.id.clone()))
        .map(|c| c.id.clone())
        .collect()
}

// ---------------------------------------------------------------------------
// Persisted report (tasks 2.1–2.3; design D4/D6) — conform-local schema
// version, fixed evidence scope, scope digest, emit plumbing.
// ---------------------------------------------------------------------------

/// The conform report schema version — conform-LOCAL, deliberately
/// distinct from model_check's `claim_schema_version`
/// ([`crate::verify::CLAIM_SCHEMA_VERSION`]): the two report families
/// version independently, and a conform report never claims to be a
/// claim report.
pub const REPORT_SCHEMA_VERSION: u32 = 1;

/// The fixed evidence-scope statement (design D6): present in BOTH the
/// JSON envelope and the `--human` view — data, not documentation, so no
/// consumer can strip it by reformatting.
pub const EVIDENCE_SCOPE: &str =
    "agreement on the supplied corpus; not a proof of behavioral equality";

/// The persisted conform run report (delta `verdict_report_schema`):
/// carries its own schema version, the fixed evidence_scope statement,
/// the scope digest, the recorded closed-world declaration (report
/// header, D2), per-verdict counts, and the per-trace records sorted by
/// scenario id. No timestamps, no iteration-order leakage — two runs
/// over identical inputs serialize byte-identically.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConformReport {
    pub report_schema_version: u32,
    pub evidence_scope: String,
    pub scope_sha256: String,
    /// The recorded invocation-mode declaration (report header, D2 —
    /// every record also carries it).
    pub closed_world: bool,
    /// Verdict → count, keyed by the verdict's snake_case spelling;
    /// surfaced in both views so unknown/underspecified coverage gaps are
    /// visible without gating (delta `exit_code_verdict_mapping`).
    pub verdict_counts: BTreeMap<String, usize>,
    /// Per-trace verdict records, sorted by scenario id.
    pub records: Vec<VerdictRecord>,
}

/// The scope digest (design D4/D6, delta `verdict_report_schema`):
/// SHA-256 over a canonical JSON payload binding (a) the parsed
/// structured content and (b) the consumed scenario corpus bytes.
///
/// The structured-content half runs through the model_check computation
/// path — [`crate::verify::scope_digest`]'s canonical serialization
/// (prose, paths, byte spans excluded; intent id keyed) — reused, never
/// forked: that function's own contract is untouched, so model_check's
/// digests stay byte-identical. The corpus half binds the exact bytes
/// the run consumed, so a one-byte corpus difference flips the digest
/// while identical inputs agree across runs and path reordering.
pub fn scope_digest(spec: &Spec, corpus_bytes: &[u8]) -> String {
    let structured =
        crate::verify::scope_digest(std::slice::from_ref(spec), &BTreeMap::new());
    let payload = serde_json::json!({
        "report_schema_version": REPORT_SCHEMA_VERSION,
        "structured_content_sha256": structured,
        "scenario_corpus_sha256": crate::model_check::artifact_sha256(corpus_bytes),
    });
    let canonical = serde_json::to_string(&payload).expect("conform scope payload serializes");
    crate::model_check::artifact_sha256(canonical.as_bytes())
}

/// Build the persisted report for one classification pass: classify every
/// corpus trace (phase-1 engine), sort the records by scenario id,
/// count verdicts, and bind the digest over the spec's structured
/// content plus the exact corpus bytes consumed.
pub fn build_report(
    spec: &Spec,
    corpus_bytes: &[u8],
    corpus: &[ScenarioTrace],
    closed_world: bool,
) -> ConformReport {
    let mut records = classify_traces(spec, corpus, closed_world);
    records.sort_by(|a, b| a.scenario_id.cmp(&b.scenario_id));
    let mut verdict_counts: BTreeMap<String, usize> = BTreeMap::new();
    for record in &records {
        *verdict_counts.entry(record.verdict.as_str().to_string()).or_insert(0) += 1;
    }
    ConformReport {
        report_schema_version: REPORT_SCHEMA_VERSION,
        evidence_scope: EVIDENCE_SCOPE.to_string(),
        scope_sha256: scope_digest(spec, corpus_bytes),
        closed_world,
        verdict_counts,
        records,
    }
}

/// The `--human` view of the report (the same convention as the other
/// verbs: one text formatter rendered from the same report values the
/// JSON envelope carries, so the two views cannot drift). Always carries
/// the fixed `evidence_scope` statement (D6 — no view can omit it) and
/// the unknown/underspecified counts.
pub fn human_view(report: &ConformReport) -> String {
    let mut summary: Vec<String> = Vec::new();
    for (verdict, count) in &report.verdict_counts {
        summary.push(format!("{verdict} {count}"));
    }
    let mut out = format!(
        "conform: {} trace(s) — {}",
        report.records.len(),
        summary.join(", ")
    );
    out.push_str(&format!("\n  evidence_scope: {}", report.evidence_scope));
    out.push_str(&format!("\n  scope_sha256: {}", report.scope_sha256));
    out.push_str(&format!("\n  closed_world: {}", report.closed_world));
    for record in &report.records {
        out.push_str(&format!(
            "\n  {}: {} — {}",
            record.scenario_id, record.verdict, record.reason
        ));
        if !record.evaluated_claim_ids.is_empty() {
            out.push_str(&format!(
                "\n    claims evaluated: {}",
                record.evaluated_claim_ids.join(", ")
            ));
        }
    }
    out
}

/// Emit the report through the genesis envelope conventions (delta
/// `verdict_report_schema`): JSON envelope default for pipes, the human
/// view for `--human` TTYs. The human branch prints [`human_view`]
/// above the suppressed data dump — the human text IS the report; the
/// JSON branch rides [`Output::emit`]'s envelope. Phase-4 CLI wiring
/// calls this; the library keeps it testable over in-memory streams.
pub fn emit_report(
    report: &ConformReport,
    format: OutputFormat,
    verbosity: Verbosity,
    stdout: &mut impl Write,
    stderr: &mut impl Write,
) -> std::io::Result<()> {
    if format == OutputFormat::Human {
        writeln!(stdout, "{}", human_view(report))?;
        // Suppress the Debug data line (verbosity above the ceiling) so the
        // human view is not followed by a Rust dump — the same convention
        // as main.rs's emit_report.
        Output::success(report.clone())
            .with_verbosity(Verbosity::MAX + 1)
            .emit(env!("CARGO_PKG_VERSION"), format, verbosity, stdout, stderr)
    } else {
        Output::success(report.clone())
            .emit(env!("CARGO_PKG_VERSION"), format, verbosity, stdout, stderr)
    }
}
