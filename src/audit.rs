//! audit — the graded-evidence audit verb (add-evidence-audit, slice 1).
//!
//! Purpose: re-validate recorded evidence (the example/oracle corpus)
//! against the current model instead of trusting a prior run's good
//! verdicts — the graded-evidence failures the studies found were all
//! one concept: evidence whose scope silently widened.
//! Responsibilities: classify every oracle row into the audit-local
//! closed evidence-lifecycle vocabulary {current, stale, unchecked,
//! contradicted, binds_nothing} — slice 1 implements the subset its
//! tests exercise (`binds_nothing` is slice 3) — with every record
//! carrying BOTH layers (the audit class and the underlying conform
//! verdict it was derived from); the persisted report with the fixed
//! evidence_scope statement verbatim and a scope_sha256 binding the
//! exact oracle corpus bytes consumed (deliberately spec-independent,
//! so revision diffing works: staleness detects SPEC drift, the digest
//! binds the EVIDENCE set); the `--before` prior-report diff (same
//! class ⇒ current, worsened ⇒ stale, no prior row ⇒ new; a
//! scope_sha256 mismatch between reports is a labeled refusal — never
//! silently diff across corpus versions); the exit contract (1 iff
//! contradicted, or stale under the prior report's declared
//! expectation; 2 on gate refusal or zero records — an audit over no
//! evidence is never a pass; never nonzero on unchecked/new); and the
//! genesis envelope emit plumbing + CLI handler.
//! Reuses conform's walk machinery unchanged (classify_traces, the
//! artifact gate, corpus parsing) — no second walk implementation, no
//! new checking semantics: nothing here checks what conform could not.
//! Read-only like conform: it evaluates, it never compiles, never
//! writes artifacts, never advances a lifecycle transition.
//! Rationale: the walk-reuse boundary keeps the audit a thin
//! evidence-lifecycle layer over conform's verdicts; the CLI handler
//! lives here rather than in commands/ because slice 1's gate runs in
//! the isolated-file mode conform documents — corpus-context discovery
//! is a later refinement if in-repo usage needs it.

use std::collections::{BTreeMap, BTreeSet};
use std::io::Write;

use genesis::guide::{Output, OutputFormat, Verbosity};
use serde::{Deserialize, Serialize};

use crate::conform::{self, ConformError, Verdict};
use crate::spec::Spec;

// ---------------------------------------------------------------------------
// The audit-local closed evidence-lifecycle vocabulary (design D4) —
// layered on conform's untouched verdict taxonomy, never replacing it
// ---------------------------------------------------------------------------

/// One oracle row's evidence-lifecycle class. Closed set: `current`,
/// `stale`, `unchecked`, `contradicted`, `new` (slice 1's subset of the
/// vocabulary; `binds_nothing` lands in slice 3). An audit record
/// carries BOTH this class and the underlying conform verdict.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuditClass {
    /// The row's evidence agrees with the current model at least as
    /// strongly as in the prior report (or there was no prior report
    /// and the walk permitted the trace).
    Current,
    /// The row's class dropped/worsened since the prior report — the
    /// prior good verdict no longer holds against this revision (the
    /// stale-example and revision-preservation failure modes).
    Stale,
    /// The row was never truly checkable (underspecified / unknown /
    /// unsupported / agree): absence of a passing verdict, never a pass.
    Unchecked,
    /// The current model positively contradicts the recorded example.
    Contradicted,
    /// The current run graded a row the prior report never saw.
    New,
}

impl AuditClass {
    /// The class's snake_case spelling (records, counts, and reasons
    /// all use this one form).
    pub fn as_str(self) -> &'static str {
        match self {
            AuditClass::Current => "current",
            AuditClass::Stale => "stale",
            AuditClass::Unchecked => "unchecked",
            AuditClass::Contradicted => "contradicted",
            AuditClass::New => "new",
        }
    }

    /// The class derived from the conform verdict it carries: the
    /// audit layer rides conform's verdicts, never re-derives them.
    fn of(verdict: Verdict) -> AuditClass {
        match verdict {
            Verdict::Permitted => AuditClass::Current,
            Verdict::Forbidden => AuditClass::Contradicted,
            Verdict::Underspecified | Verdict::Unknown | Verdict::Unsupported | Verdict::Agree => {
                AuditClass::Unchecked
            }
        }
    }

    /// Worsening order for the `--before` overlay: current(0) <
    /// unchecked(1) < contradicted(2). `Stale`/`New` are diff overlays
    /// a fresh build never produces; their ranks only matter if an
    /// already-diffed report were re-diffed, which the CLI never does.
    fn rank(self) -> u8 {
        match self {
            AuditClass::Current | AuditClass::New => 0,
            AuditClass::Unchecked | AuditClass::Stale => 1,
            AuditClass::Contradicted => 2,
        }
    }
}

// ---------------------------------------------------------------------------
// The audit report — schema version, verbatim evidence scope, corpus
// digest, the class layer AND the verdict layer, per-row records
// ---------------------------------------------------------------------------

/// The audit report schema version — audit-LOCAL, distinct from
/// conform's and verify's: the three report families version
/// independently and never claim to be each other.
pub const AUDIT_SCHEMA_VERSION: u32 = 1;

/// One audit record: the scenario it classifies, the audit class, and
/// the underlying conform verdict with its labeled reason — the audit
/// layer rides conform's verdicts, so a record can always answer "why"
/// in conform's own terms.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuditRecord {
    pub scenario_id: String,
    pub class: AuditClass,
    pub conform_verdict: Verdict,
    pub reason: String,
    pub evaluated_claim_ids: Vec<String>,
    pub closed_world: bool,
}

/// The persisted audit report: audit-local schema version, the fixed
/// evidence_scope statement verbatim, the scope digest over the exact
/// oracle corpus bytes consumed, the recorded expectation declaration,
/// per-verdict and per-class counts, and the per-row records sorted by
/// scenario id. No timestamps, no iteration-order leakage — two runs
/// over identical inputs serialize byte-identically.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuditReport {
    pub report_schema_version: u32,
    pub evidence_scope: String,
    pub scope_sha256: String,
    /// Whether this report was recorded as a declared expectation
    /// baseline (`--declare-expectation`): when used later as a
    /// `--before` prior, stale rows then fail the run.
    pub expectation_declared: bool,
    /// Underlying conform verdict counts (snake_case keyed) — the
    /// verdict layer rides through untouched.
    pub verdict_counts: BTreeMap<String, usize>,
    /// Audit-class counts (snake_case keyed).
    pub class_counts: BTreeMap<String, usize>,
    /// Per-row records, sorted by scenario id.
    pub records: Vec<AuditRecord>,
}

/// The scope digest (audit-local discipline): SHA-256 over the exact
/// oracle corpus bytes the run consumed — the EVIDENCE set identity.
/// Deliberately independent of the spec's structured content: the
/// `--before` diff exists to detect SPEC drift between reports, so the
/// spec must be free to change; the corpus must not (any corpus change
/// flips the digest and the diff refuses — never silently diff across
/// corpus versions).
pub fn scope_sha256(corpus_bytes: &[u8]) -> String {
    crate::model_check::artifact_sha256(corpus_bytes)
}

/// Build the audit report for one classification pass: run conform's
/// walk UNCHANGED over the corpus, derive each record's class from the
/// verdict it carries, sort records by scenario id, count both layers,
/// and bind the digest over the exact corpus bytes consumed.
pub fn build_report(
    spec: &Spec,
    corpus_bytes: &[u8],
    corpus: &[conform::ScenarioTrace],
    closed_world: bool,
    expectation_declared: bool,
) -> AuditReport {
    let mut records: Vec<AuditRecord> = conform::classify_traces(spec, corpus, closed_world)
        .into_iter()
        .map(|v| AuditRecord {
            class: AuditClass::of(v.verdict),
            scenario_id: v.scenario_id,
            conform_verdict: v.verdict,
            reason: v.reason,
            evaluated_claim_ids: v.evaluated_claim_ids,
            closed_world: v.closed_world,
        })
        .collect();
    records.sort_by(|a, b| a.scenario_id.cmp(&b.scenario_id));
    let mut verdict_counts: BTreeMap<String, usize> = BTreeMap::new();
    let mut class_counts: BTreeMap<String, usize> = BTreeMap::new();
    for record in &records {
        *verdict_counts
            .entry(record.conform_verdict.as_str().to_string())
            .or_insert(0) += 1;
        *class_counts
            .entry(record.class.as_str().to_string())
            .or_insert(0) += 1;
    }
    AuditReport {
        report_schema_version: AUDIT_SCHEMA_VERSION,
        evidence_scope: conform::EVIDENCE_SCOPE.to_string(),
        scope_sha256: scope_sha256(corpus_bytes),
        expectation_declared,
        verdict_counts,
        class_counts,
        records,
    }
}

// ---------------------------------------------------------------------------
// The --before prior-report diff (design D1 — diff reports, not files)
// ---------------------------------------------------------------------------

/// Parse a prior report file (`--before`): the JSON envelope a previous
/// `spk audit --json` run wrote (the report rides the envelope's
/// `data`), or the bare report object. Anything else is a labeled
/// refusal with a remediation hint — never a silent fallback.
pub fn parse_prior_report(bytes: &[u8]) -> Result<AuditReport, ConformError> {
    let refuse = |detail: String| ConformError {
        stage: "before_parse".into(),
        message: format!(
            "{detail} — pass the JSON envelope a prior `spk audit --json` run wrote (--before)"
        ),
    };
    let text = std::str::from_utf8(bytes)
        .map_err(|e| refuse(format!("the prior report is not valid UTF-8: {e}")))?;
    let value: serde_json::Value = serde_json::from_str(text)
        .map_err(|e| refuse(format!("the prior report is not valid JSON: {e}")))?;
    let data = match value.get("data") {
        Some(d) if d.is_object() => d,
        _ => &value,
    };
    serde_json::from_value(data.clone())
        .map_err(|e| refuse(format!("the prior report is not an audit report: {e}")))
        .and_then(|report: AuditReport| {
            // The diff keys rows by scenario id — a duplicate would
            // silently let the row lookup pick whichever came first.
            let mut seen = BTreeSet::new();
            if report
                .records
                .iter()
                .any(|r| !seen.insert(r.scenario_id.clone()))
            {
                return Err(refuse(
                    "the prior report carries duplicate scenario id(s)".to_string(),
                ));
            }
            Ok(report)
        })
}

/// The prior-report diff (design D1): compare per-row classes with the
/// prior report — same class ⇒ `current`, class dropped/worsened ⇒
/// `stale`, improved ⇒ `current` (the evidence got better; it is
/// current), no prior row ⇒ `new`. A `scope_sha256` mismatch between
/// the reports is a labeled refusal: per-row comparison would silently
/// diff different evidence across corpus versions (the spec may differ
/// — that is the diff's purpose; the corpus may not).
pub fn diff_with_prior(
    mut current: AuditReport,
    prior: &AuditReport,
) -> Result<AuditReport, ConformError> {
    // Reports from different audit schema versions are never silently
    // diffed — the version field exists to guard exactly this drift.
    if prior.report_schema_version != current.report_schema_version {
        return Err(ConformError {
            stage: "before_diff".into(),
            message: format!(
                "the prior report's schema version ({}) does not match this audit's ({}) — reports from different audit schema versions are never silently diffed (re-run the baseline: spk audit <file> --oracle <corpus.jsonl> --declare-expectation)",
                prior.report_schema_version, current.report_schema_version
            ),
        });
    }
    if current.scope_sha256 != prior.scope_sha256 {
        return Err(ConformError {
            stage: "before_diff".into(),
            message: format!(
                "scope_sha256 mismatch between the reports (prior {}, current {}) — the oracle corpus changed between the runs and per-row comparison would silently diff different evidence; never silently diff across corpus versions (re-run the baseline against the same corpus: spk audit <file> --oracle <corpus.jsonl> --declare-expectation)",
                prior.scope_sha256, current.scope_sha256
            ),
        });
    }
    for record in &mut current.records {
        record.class = match prior
            .records
            .iter()
            .find(|r| r.scenario_id == record.scenario_id)
        {
            None => AuditClass::New,
            Some(prior_row)
                if AuditClass::rank(record.class) > AuditClass::rank(prior_row.class) =>
            {
                AuditClass::Stale
            }
            Some(_) => AuditClass::Current,
        };
    }
    let mut class_counts: BTreeMap<String, usize> = BTreeMap::new();
    for record in &current.records {
        *class_counts
            .entry(record.class.as_str().to_string())
            .or_insert(0) += 1;
    }
    current.class_counts = class_counts;
    Ok(current)
}

/// The exit contract (proposal): 1 iff a `contradicted` record exists —
/// or a `stale` record when the prior report declared an expectation
/// (revision preservation); 2 on zero records (an audit over no
/// evidence is never a pass — the empty-checked-set failure); never
/// nonzero on `unchecked`/`new`, and `stale` without a declared
/// expectation is informational.
pub fn exit_code(report: &AuditReport, prior_declared_expectation: bool) -> i32 {
    if report.records.is_empty() {
        return 2;
    }
    if report
        .records
        .iter()
        .any(|r| r.class == AuditClass::Contradicted)
    {
        return 1;
    }
    if prior_declared_expectation && report.records.iter().any(|r| r.class == AuditClass::Stale) {
        return 1;
    }
    0
}

// ---------------------------------------------------------------------------
// The command path — the reused gate, corpus validation, the
// zero-records refusal, build, optional diff
// ---------------------------------------------------------------------------

/// The command-path run: the reused artifact gate (conform's, unchanged
/// — lint-clean file, current compiled artifacts), corpus
/// parse/validation (conform's, unchanged), the zero-records refusal
/// (an audit over no evidence is never a pass), the report build, and
/// the optional prior diff. Every refusal path returns Err — zero
/// records are emitted for a gated-out invocation.
pub fn run(
    spec: &Spec,
    out_dir: &std::path::Path,
    corpus_bytes: &[u8],
    closed_world: bool,
    expectation_declared: bool,
    prior: Option<&AuditReport>,
) -> Result<AuditReport, ConformError> {
    conform::gate(spec, out_dir)?;
    let corpus = conform::parse_corpus(std::str::from_utf8(corpus_bytes).map_err(|e| {
        ConformError {
            stage: "corpus_parse".into(),
            message: format!(
                "oracle corpus is not valid UTF-8: {e} — fix the scenario record (one JSON object per line: {{id, setup?, trace}})"
            ),
        }
    })?)?;
    if corpus.is_empty() {
        return Err(ConformError {
            stage: "zero_records".into(),
            message: "the oracle corpus contributed zero records — an audit over no evidence is never a pass (the empty-checked-set failure this change exists to prevent); add recorded scenarios: one {id, setup?, trace} object per line (--oracle)".into(),
        });
    }
    let report = build_report(
        spec,
        corpus_bytes,
        &corpus,
        closed_world,
        expectation_declared,
    );
    match prior {
        Some(prior) => diff_with_prior(report, prior),
        None => Ok(report),
    }
}

// ---------------------------------------------------------------------------
// Emit plumbing — the single rendering path (the conform convention)
// ---------------------------------------------------------------------------

/// The `--human` view of the report (one text formatter rendered from
/// the same report values the JSON envelope carries, so the two views
/// cannot drift). Always carries the fixed `evidence_scope` statement.
pub fn human_view(report: &AuditReport) -> String {
    let mut classes: Vec<String> = Vec::new();
    for (class, count) in &report.class_counts {
        classes.push(format!("{class} {count}"));
    }
    let mut out = format!(
        "audit: {} record(s) — {}",
        report.records.len(),
        classes.join(", ")
    );
    out.push_str(&format!("\n  evidence_scope: {}", report.evidence_scope));
    out.push_str(&format!("\n  scope_sha256: {}", report.scope_sha256));
    out.push_str(&format!(
        "\n  expectation_declared: {}",
        report.expectation_declared
    ));
    for record in &report.records {
        out.push_str(&format!(
            "\n  {}: {} ({}) — {}",
            record.scenario_id,
            record.class.as_str(),
            record.conform_verdict,
            record.reason
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

/// THE single rendering path for audit report outputs: both the exit-0
/// success envelope and the exit-1 findings envelope render here — the
/// human view printed above the suppressed data dump for `--human`, the
/// JSON envelope via [`Output::emit`] otherwise. One path means the two
/// views and the two exit classes cannot drift (the conform convention).
pub fn emit_output(
    out: Output<AuditReport>,
    format: OutputFormat,
    verbosity: Verbosity,
    stdout: &mut impl Write,
    stderr: &mut impl Write,
) -> std::io::Result<()> {
    if format == OutputFormat::Human {
        writeln!(stdout, "{}", human_view(&out.data))?;
        // Suppress the Debug data line so the human view is not followed
        // by a Rust dump — the same convention as conform::emit_output.
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

/// Emit a success report through [`emit_output`].
pub fn emit_report(
    report: &AuditReport,
    format: OutputFormat,
    verbosity: Verbosity,
    stdout: &mut impl Write,
    stderr: &mut impl Write,
) -> std::io::Result<()> {
    emit_output(
        Output::success(report.clone()),
        format,
        verbosity,
        stdout,
        stderr,
    )
}

/// Emit a refusal (invocation error, gate refusal, zero records, diff
/// refusal): an error-kind envelope carrying NO report — refusal paths
/// emit zero records.
fn emit_refusal(
    stage_and_message: String,
    hint: &str,
    format: OutputFormat,
    verbosity: Verbosity,
    stdout: &mut impl Write,
    stderr: &mut impl Write,
) {
    let out: Output<serde_json::Value> = Output::failure(stage_and_message).with_next_step(hint);
    if format == OutputFormat::Human {
        out.with_verbosity(Verbosity::MAX + 1)
            .emit(env!("CARGO_PKG_VERSION"), format, verbosity, stdout, stderr)
            .ok();
    } else {
        out.emit(env!("CARGO_PKG_VERSION"), format, verbosity, stdout, stderr)
            .ok();
    }
}

/// The audit-local copy of commands::as_failure (the library cannot
/// reach the binary's commands module): flip the envelope kind to
/// error while keeping the payload, message leading the warnings
/// channel (specodelic-4v1 / envelope_error_kind) — a failed audit run
/// never rides a success-shaped envelope.
fn findings_failure<T: std::fmt::Debug>(mut out: Output<T>, message: String) -> Output<T> {
    out.is_error = true;
    out.warnings.insert(0, message);
    out.verbosity = 0;
    out
}

// ---------------------------------------------------------------------------
// The CLI handler — `spk audit <file> --oracle <corpus.jsonl> [--before
// report.json] [--declare-expectation] [--closed-world]`
// ---------------------------------------------------------------------------

/// The audit invocation parameters beyond the spec path, grouped to
/// keep the shared emit plumbing within the arg lint (the conform
/// ConformTarget precedent).
pub struct AuditTarget {
    pub oracle: String,
    pub before: Option<String>,
    pub closed_world: bool,
    pub declare_expectation: bool,
    pub out_dir: String,
}

/// `spk audit <spec-path> --oracle <corpus.jsonl>` (add-evidence-audit
/// slice 1): the READ-ONLY graded-evidence audit over [`run`] — the
/// reused artifact gate, mechanical classification (conform's walk,
/// unchanged), the persisted report, and the optional `--before` diff.
/// Exit codes: 0 when no record is contradicted (and no stale record
/// under a declared expectation), 1 on contradicted / stale-under-
/// expectation (an error-kind envelope — records ride along), 2 on
/// invocation errors and gate/diff/zero-records refusals — with ZERO
/// records emitted from any refusal path.
pub fn cmd_audit(
    spec_path: &str,
    target: AuditTarget,
    format: OutputFormat,
    verbosity: Verbosity,
    stdout: &mut impl Write,
    stderr: &mut impl Write,
) -> i32 {
    let AuditTarget {
        oracle,
        before,
        closed_world,
        declare_expectation,
        out_dir,
    } = target;
    // Invocation errors precede everything: zero records emitted.
    let corpus_bytes = match std::fs::read(&oracle) {
        Ok(bytes) => bytes,
        Err(e) => {
            emit_refusal(
                format!("cannot read the oracle corpus {oracle}: {e}"),
                "pass an existing JSONL oracle corpus file — one {id, setup?, trace} object per line (--oracle)",
                format,
                verbosity,
                stdout,
                stderr,
            );
            return 2;
        }
    };
    let prior = match before.as_deref() {
        Some(path) => match std::fs::read(path)
            .map_err(|e| ConformError {
                stage: "before_parse".into(),
                message: format!("cannot read the prior report {path}: {e}"),
            })
            .and_then(|bytes| parse_prior_report(&bytes))
        {
            Ok(prior) => Some(prior),
            Err(e) => {
                emit_refusal(
                    format!("{}: {}", e.stage, e.message),
                    "pass a report a prior `spk audit --json` run wrote (--before)",
                    format,
                    verbosity,
                    stdout,
                    stderr,
                );
                return 2;
            }
        },
        None => None,
    };
    let prior_declared_expectation = prior.as_ref().is_some_and(|p| p.expectation_declared);
    let spec = match crate::parse::parse_file(std::path::Path::new(spec_path)) {
        Ok(spec) => spec,
        Err(e) => {
            emit_refusal(
                format!(
                    "the spec file failed to parse: {e} — audit verdicts are never derived from an unparsed file"
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
    match run(
        &spec,
        std::path::Path::new(&out_dir),
        &corpus_bytes,
        closed_world,
        declare_expectation,
        prior.as_ref(),
    ) {
        Ok(report) => {
            let exit = exit_code(&report, prior_declared_expectation);
            if exit == 1 {
                // Findings-or-failure (envelope_error_kind): the
                // contradicted/stale-under-expectation run never rides a
                // success-shaped envelope — the report rides along.
                let contradicted = report
                    .records
                    .iter()
                    .filter(|r| r.class == AuditClass::Contradicted)
                    .count();
                let stale = report
                    .records
                    .iter()
                    .filter(|r| r.class == AuditClass::Stale)
                    .count();
                let message = if contradicted > 0 {
                    format!(
                        "{contradicted} contradicted record(s) — the current model contradicts recorded evidence"
                    )
                } else {
                    format!(
                        "{stale} stale record(s) under a declared expectation — the prior baseline no longer holds against this revision"
                    )
                };
                let out = findings_failure(Output::success(report), message).with_next_step(
                    "fix the model or re-record the affected oracle rows; re-declare the baseline with --declare-expectation",
                );
                emit_output(out, format, verbosity, stdout, stderr).ok();
                1
            } else {
                emit_report(&report, format, verbosity, stdout, stderr).ok();
                0
            }
        }
        Err(e) => {
            // Gate refusals, corpus refusals, zero records, and diff
            // refusals stay exit 2 with zero records emitted; the
            // message already carries the failing stage and the
            // remediation command where one exists.
            let hint = if e.message.contains("spk lint") {
                "run: spk lint <file>"
            } else if e.message.contains("spk compile") {
                "run: spk compile <file>"
            } else if e.stage == "corpus_parse" {
                "fix the scenario record — one JSON object per line: {id, setup?, trace}"
            } else if e.stage == "zero_records" {
                "add recorded scenarios to the oracle corpus — one {id, setup?, trace} object per line (--oracle)"
            } else if e.stage == "before_diff" {
                "re-run the baseline over the same corpus and retry (--before)"
            } else if e.stage == "before_parse" {
                "pass a report a prior `spk audit --json` run wrote (--before)"
            } else {
                "address the labeled refusal above"
            };
            emit_refusal(
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
