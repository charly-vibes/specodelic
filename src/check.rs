//! check — the single judge verb (add-check-judge-verb).
//!
//! Purpose: give the pipeline one judging verb by composing the evidence
//! producers that already exist — conform's artifact gate and trace walk,
//! verify's both-gates evaluation, and audit's row assembly,
//! classification, prior diff and ladder-rung derivation.
//! Responsibilities: run the gate and the verify stage in one invocation
//! (the bounded properties runner plus the persisted model-check report);
//! map every producer outcome onto the existing `Verdict` × `AuditClass`
//! vocabulary per the capability spec's mapping requirements; fold the
//! verify-stage rows into the audit report shape (`verify:*` keys); supply
//! this run's verify verdict to the rung derivation; refuse with exit 2
//! whenever no judgement can be emitted (unavailable, stale, uncompilable,
//! terminated or scope-mismatched evidence, or zero rows).
//! Rationale: the row layer, the report shape and the exit predicate are
//! audit's, unchanged — a second report family would fork the diff and the
//! vocabulary. The only execution performed is verify's own bounded
//! property runner; no contract runner, adapter registry, sidecar binding
//! or corpus demotion enters this verb (scope B stays demand-gated). A
//! killed runner is a termination, never a verdict, so `inconclusive`
//! stays reserved and is never spelled.

use std::io::Write;

use genesis::guide::{Output, OutputFormat, Verbosity};
use serde::{Deserialize, Serialize};

use crate::audit::{self, AuditClass, AuditRecord, AuditReport};
use crate::claim_coverage::{self, ClaimCoverage};
use crate::conform::{self, ConformError, Verdict};
use crate::model_check::Outcome;
use crate::spec::Spec;
use crate::verify::{self, FileVerdict, ModelGateState, PropsGateState};

/// The evidence-scope statement for a run with no oracle corpus: only the
/// spec's own compiled evidence was consumed — never weaker than audit's
/// fixed sentence about what an oracle run does and does not establish.
pub const NO_ORACLE_SCOPE: &str = "no oracle corpus supplied — the report covers the spec's own compiled evidence only; agreement on an external corpus and behavioral equality are not claimed";

/// The check invocation parameters beyond the spec path.
pub struct CheckTarget {
    pub oracle: Option<String>,
    pub before: Option<String>,
    pub closed_world: bool,
    pub structural_only: bool,
    pub declare_expectation: bool,
    pub out_dir: String,
    pub timeout_secs: u64,
}

/// The verify stage's evidence rows: one per compiled property block and
/// one per model-gate unit, keyed `verify:prop:*` / `verify:model*`.
pub fn verify_rows(fv: &FileVerdict) -> Vec<AuditRecord> {
    let mut rows: Vec<AuditRecord> = fv.properties.blocks.iter().map(property_row).collect();
    rows.extend(model_rows(&fv.model));
    rows
}

/// One compiled property block: a pass is `permitted` (no contradiction
/// found), a failure is `forbidden` (positive contradiction with the
/// declared executable claim), carrying the runner's shrunk-input detail.
fn property_row(block: &verify::BlockResult) -> AuditRecord {
    let key = match &block.case {
        Some(case) => format!("verify:prop:{}:{case}", block.id),
        None => format!("verify:prop:{}", block.id),
    };
    let claims = vec![block.id.clone()];
    if block.passed {
        audit::row(
            key,
            Verdict::Permitted,
            format!(
                "property block `{}` ran to completion and passed against the current compiled artifact",
                block.id
            ),
            claims,
            false,
        )
    } else {
        audit::row(
            key,
            Verdict::Forbidden,
            format!(
                "property block `{}` failed — {}",
                block.id,
                block
                    .detail
                    .as_deref()
                    .unwrap_or("the runner captured no failure detail")
            ),
            claims,
            false,
        )
    }
}

/// The model gate as rows: a clean run is `permitted`; a contradicted
/// claim is `forbidden`; a run that could not decide (exploration-only
/// or timed out) is `unknown` — knowledge absence, never a pass.
fn model_rows(gate: &ModelGateState) -> Vec<AuditRecord> {
    match gate {
        ModelGateState::Clean => vec![model_row(
            "verify:model".to_string(),
            Verdict::Permitted,
            "the current model-check run reported no_counterexample and every required claim is verified"
                .to_string(),
        )],
        ModelGateState::NotClean {
            outcome,
            blocking_claims,
            ..
        } => {
            let (verdict, detail) = outcome_mapping(*outcome);
            if blocking_claims.is_empty() {
                vec![model_row(
                    "verify:model".to_string(),
                    verdict,
                    format!("the current model-check run {detail}"),
                )]
            } else {
                blocking_claims
                    .iter()
                    .map(|id| {
                        model_row(
                            format!("verify:model:{id}"),
                            verdict,
                            format!(
                                "required claim `{id}` is not verified; the current model-check run {detail}"
                            ),
                        )
                    })
                    .collect()
            }
        }
        // Stale/Missing never reach here — the run refuses before rows.
        ModelGateState::Stale { .. } | ModelGateState::Missing { .. } => Vec::new(),
    }
}

fn model_row(key: String, verdict: Verdict, reason: String) -> AuditRecord {
    audit::row(key, verdict, reason, Vec::new(), false)
}

/// The outcome → verdict mapping (design D3): only a counterexample is a
/// contradiction; a run that finished without deciding is `unknown`.
fn outcome_mapping(outcome: Outcome) -> (Verdict, &'static str) {
    match outcome {
        Outcome::CounterexampleFound => (
            Verdict::Forbidden,
            "reported counterexample_found — a declared executable claim is contradicted",
        ),
        Outcome::TimedOut => (
            Verdict::Unknown,
            "reported timed_out — the bounded exploration did not finish, so nothing is proven",
        ),
        Outcome::ExplorationOnly => (
            Verdict::Unknown,
            "reported exploration_only — zero invariant predicates were executed",
        ),
        Outcome::NoCounterexample => (
            Verdict::Unknown,
            "reported no_counterexample without a clean gate",
        ),
    }
}

fn refuse(stage: &str, message: String) -> ConformError {
    ConformError {
        stage: stage.to_string(),
        message,
    }
}

/// The properties stage's refusals (no verdict exists for any of these).
fn properties_refusal(state: &PropsGateState) -> Option<ConformError> {
    match state {
        PropsGateState::Pass | PropsGateState::Failed(_) => None,
        PropsGateState::Stale => Some(refuse(
            "verify_properties",
            "the compiled properties artifact predates the current spec — nothing was executed; run: spk compile <file>".to_string(),
        )),
        PropsGateState::MissingArtifact => Some(refuse(
            "verify_properties",
            "no <stem>_props.rs artifact in the out-dir — nothing was executed; run: spk compile <file>"
                .to_string(),
        )),
        PropsGateState::Uncompilable(detail) => Some(refuse(
            "verify_properties",
            format!("the compiled properties artifact did not compile ({detail}) — nothing was executed; run: spk compile <file>"),
        )),
        PropsGateState::RunnerUnavailable(detail) => Some(refuse(
            "verify_properties",
            format!("the properties runner could not start ({detail}) — nothing was executed"),
        )),
        PropsGateState::TimedOut { secs, detail } => Some(refuse(
            "verify_properties",
            format!("the properties runner was killed at its {secs}s wall-clock bound — a terminated run is not a verdict ({detail})"),
        )),
    }
}

/// The model stage's refusals: old or absent evidence is a rerun, never a
/// pass and never rewritten.
fn model_refusal(gate: &ModelGateState) -> Option<ConformError> {
    match gate {
        ModelGateState::Clean | ModelGateState::NotClean { .. } => None,
        ModelGateState::Stale { detail } => Some(refuse(
            "verify_model",
            format!(
                "the stored model-check report is not current evidence ({detail}) — run: spk model-check <file>"
            ),
        )),
        ModelGateState::Missing { detail } => Some(refuse(
            "verify_model",
            format!("no readable model-check report ({detail}) — run: spk model-check <file>"),
        )),
    }
}

/// A judged report plus its exit code: the code is decided on the
/// PRE-overlay evidence classes, so a diff run fails on live contradictory
/// evidence rather than on a currency label. `audit::diff_with_prior`
/// preserves `contradicted` too (it never retires a failure), so the two
/// readings agree — the pre-overlay count is what the exit predicate rests
/// on.
pub struct Judgement {
    pub report: AuditReport,
    pub exit: i32,
    /// The contradicted rows classified BEFORE the prior-report overlay:
    /// the exit predicate and its message both rest on this evidence count,
    /// never on the overlay's currency label.
    pub contradicted: usize,
}

/// The check envelope's payload: the audit report UNCHANGED — flattened, so
/// the report's field set stays exactly audit's and a check report still
/// parses as audit's `--before` prior — with the claim-coverage section
/// beside it as the one sibling key.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CheckPayload {
    /// The audit report, flattened into the payload beside the section.
    #[serde(flatten)]
    pub report: AuditReport,
    /// The claim-coverage section: every declared invariant claim of the
    /// judged file with its class.
    pub claim_coverage: ClaimCoverage,
}

/// The check envelope's payload for one judged file: the report plus the
/// claim-coverage section derived from it.
pub fn payload(spec: &Spec, report: &AuditReport) -> CheckPayload {
    CheckPayload {
        report: report.clone(),
        claim_coverage: claim_coverage::coverage(spec, report),
    }
}

/// The check verb's human view: the audit report's view, with the
/// claim-coverage counts on the verdict line and the section's own lines
/// before the record detail.
pub fn human_view(payload: &CheckPayload) -> String {
    audit::human_view_labelled_extended(
        "check",
        &payload.report,
        &claim_coverage::verdict_summary(&payload.claim_coverage),
        &claim_coverage::section_lines(&payload.claim_coverage),
    )
}

/// The judging pass: gate, verify stage, oracle walk (optional), row fold,
/// rung derivation, prior diff. Every refusal returns Err with zero rows.
pub fn run(
    spec: &Spec,
    target: &CheckTarget,
    oracle: Option<&[u8]>,
    prior: Option<&AuditReport>,
) -> Result<Judgement, ConformError> {
    let out_dir = std::path::Path::new(&target.out_dir);
    conform::gate(spec, out_dir)?;
    let runner = verify::CargoRunner {
        timeout_secs: (target.timeout_secs > 0).then_some(target.timeout_secs),
    };
    let fv = verify::evaluate_file_verdicts(std::slice::from_ref(spec), out_dir, &runner)
        .into_iter()
        .next()
        .ok_or_else(|| refuse("verify", "the verify stage produced no verdict".to_string()))?;
    if let Some(e) = properties_refusal(&fv.properties.state) {
        return Err(e);
    }
    if let Some(e) = model_refusal(&fv.model) {
        return Err(e);
    }
    let mut rows = verify_rows(&fv);
    rows.extend(audit::bind_rows(spec));
    let report = audit::fold_rows(base_report(spec, oracle, target)?, rows);
    if report.records.is_empty() {
        return Err(refuse(
            "zero_records",
            "the judged file produced zero evidence rows — an evidence-less pass is impossible; add an oracle corpus (--oracle) or compiled property evidence".to_string(),
        ));
    }
    let report =
        audit::with_derived_rung_and(report, spec, out_dir, fv.verdict.status == "verified");
    let contradicted = class_count(&report, AuditClass::Contradicted);
    let report = match prior {
        Some(prior) => audit::diff_with_prior(report, prior)?,
        None => report,
    };
    let declared = prior.is_some_and(|p| p.expectation_declared);
    let exit = if contradicted > 0 {
        1
    } else {
        audit::exit_code(&report, declared)
    };
    Ok(Judgement {
        report,
        exit,
        contradicted,
    })
}

/// The base report: conform's oracle rows when a corpus is supplied, else
/// the empty-corpus report carrying the no-oracle scope statement.
fn base_report(
    spec: &Spec,
    oracle: Option<&[u8]>,
    target: &CheckTarget,
) -> Result<AuditReport, ConformError> {
    match oracle {
        Some(bytes) => {
            let text = std::str::from_utf8(bytes).map_err(|e| {
                refuse(
                    "corpus_parse",
                    format!("oracle corpus is not valid UTF-8: {e} — one JSON object per line: {{id, setup?, trace}}"),
                )
            })?;
            let corpus = conform::parse_corpus(text)?;
            if corpus.is_empty() {
                return Err(refuse(
                    "zero_records",
                    "the oracle corpus contributed zero records — a run over no requested evidence is never a pass; add recorded scenarios (one {id, setup?, trace} object per line, --oracle)"
                        .to_string(),
                ));
            }
            Ok(audit::build_report_mode(
                spec,
                bytes,
                &corpus,
                target.closed_world,
                target.structural_only,
                target.declare_expectation,
            ))
        }
        None => {
            let mut report = audit::build_report_mode(
                spec,
                b"",
                &[],
                target.closed_world,
                target.structural_only,
                target.declare_expectation,
            );
            report.evidence_scope = NO_ORACLE_SCOPE.to_string();
            Ok(report)
        }
    }
}

/// The two input files (oracle bytes, prior report) or a refusal pair
/// (message, hint) — named so the signature stays readable.
type Inputs = (Option<Vec<u8>>, Option<AuditReport>);
type Refusal = (String, String);

/// The two input files, read before any judging: a missing oracle or an
/// unreadable prior report is an invocation refusal with its own hint.
fn read_inputs(target: &CheckTarget) -> Result<Inputs, Refusal> {
    let hint = "pass a report an audit or check run wrote (--before)";
    let oracle = match &target.oracle {
        Some(path) => Some(std::fs::read(path).map_err(|e| {
            (
                format!("cannot read the oracle corpus {path}: {e}"),
                "pass an existing JSONL oracle corpus (--oracle)".to_string(),
            )
        })?),
        None => None,
    };
    let prior = match &target.before {
        Some(path) => {
            let bytes = std::fs::read(path).map_err(|e| {
                (
                    format!("cannot read the prior report {path}: {e}"),
                    hint.to_string(),
                )
            })?;
            Some(
                audit::parse_prior_report(&bytes)
                    .map_err(|e| (format!("{}: {}", e.stage, e.message), hint.to_string()))?,
            )
        }
        None => None,
    };
    Ok((oracle, prior))
}

fn hint_for(e: &ConformError) -> &'static str {
    if e.message.contains("spk lint") {
        "run: spk lint <file>"
    } else if e.stage == "verify_model" {
        "run: spk model-check <file> (check never compiles or rewrites the report)"
    } else if e.stage == "verify_properties" {
        "run: spk compile <file> and re-run check (artifacts must be current)"
    } else if e.stage == "corpus_parse" {
        "fix the scenario record — one JSON object per line: {id, setup?, trace}"
    } else if e.stage == "before_diff" {
        "re-run the baseline over the same corpus and retry (--before)"
    } else if e.stage == "before_parse" {
        "pass a report an audit or check run wrote (--before)"
    } else {
        "address the labeled refusal above"
    }
}

fn class_count(report: &AuditReport, class: AuditClass) -> usize {
    report.records.iter().filter(|r| r.class == class).count()
}

/// Render a judged payload: exit 1 rides the findings envelope with the
/// payload attached; exit 0 carries the binds-nothing advisory. Both views
/// render the same payload values through the check verb's one rendering
/// path.
fn emit_judged(
    payload: &CheckPayload,
    exit: i32,
    contradicted: usize,
    format: OutputFormat,
    verbosity: Verbosity,
    stdout: &mut impl Write,
    stderr: &mut impl Write,
) -> i32 {
    if exit == 2 {
        audit::emit_refusal(
            "zero_records: the judged file produced zero evidence rows — an evidence-less pass is impossible".to_string(),
            "add an oracle corpus (--oracle) or compiled property evidence",
            format,
            verbosity,
            stdout,
            stderr,
        );
        return 2;
    }
    if exit == 1 {
        let stale = class_count(&payload.report, AuditClass::Stale);
        let message = if contradicted > 0 {
            format!(
                "{contradicted} contradicted record(s) — the current evidence contradicts a declared claim"
            )
        } else {
            format!(
                "{stale} stale record(s) under a declared expectation — the prior baseline no longer holds against this revision"
            )
        };
        let out = audit::findings_failure(Output::success(payload.clone()), message).with_next_step(
            "fix the model or re-record the affected evidence; re-declare the baseline with --declare-expectation",
        );
        emit_payload(out, format, verbosity, stdout, stderr).ok();
        return 1;
    }
    let binds = class_count(&payload.report, AuditClass::BindsNothing);
    let warnings = if binds > 0 {
        vec![format!(
            "{binds} binds_nothing row(s) — evidence citing no declared model element (vacuous binding); warnings-channel advisory, never a failure"
        )]
    } else {
        Vec::new()
    };
    let mut out = Output::success(payload.clone());
    out.warnings.extend(warnings);
    emit_payload(out, format, verbosity, stdout, stderr).ok();
    0
}

/// The check verb's rendering path: the human view above the suppressed
/// data dump for `--human`, the JSON envelope otherwise — the convention
/// the audit verb follows, over the check payload.
fn emit_payload(
    out: Output<CheckPayload>,
    format: OutputFormat,
    verbosity: Verbosity,
    stdout: &mut impl Write,
    stderr: &mut impl Write,
) -> std::io::Result<()> {
    if format == OutputFormat::Human {
        writeln!(stdout, "{}", human_view(&out.data))?;
        out.with_verbosity(Verbosity::MAX + 1).emit(
            env!("CARGO_PKG_VERSION"),
            format,
            verbosity,
            stdout,
            stderr,
        )
    } else {
        out.emit(env!("CARGO_PKG_VERSION"), format, verbosity, stdout, stderr)
    }
}

/// `spk check <spec-file> [--oracle <corpus.jsonl>] [...]`: the single
/// judge verb. Exit 0/1 mean a judgement was emitted; exit 2 means none
/// was — every refusal path emits zero records and a remediation hint.
pub fn cmd_check(
    spec_path: &str,
    target: CheckTarget,
    format: OutputFormat,
    verbosity: Verbosity,
    stdout: &mut impl Write,
    stderr: &mut impl Write,
) -> i32 {
    let (oracle, prior) = match read_inputs(&target) {
        Ok(inputs) => inputs,
        Err((message, hint)) => {
            audit::emit_refusal(message, &hint, format, verbosity, stdout, stderr);
            return 2;
        }
    };
    let spec = match crate::parse::parse_file(std::path::Path::new(spec_path)) {
        Ok(spec) => spec,
        Err(e) => {
            audit::emit_refusal(
                format!(
                    "the spec file failed to parse: {e} — check verdicts are never derived from an unparsed file"
                ),
                "fix the frontmatter/tables named in the parse error",
                format,
                verbosity,
                stdout,
                stderr,
            );
            return 2;
        }
    };
    match run(&spec, &target, oracle.as_deref(), prior.as_ref()) {
        Ok(judgement) => emit_judged(
            &payload(&spec, &judgement.report),
            judgement.exit,
            judgement.contradicted,
            format,
            verbosity,
            stdout,
            stderr,
        ),
        Err(e) => {
            let hint = hint_for(&e);
            audit::emit_refusal(
                format!("{}: {}", e.stage, e.message),
                hint,
                format,
                verbosity,
                stdout,
                stderr,
            );
            2
        }
    }
}
