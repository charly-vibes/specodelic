//! audit — the graded-evidence audit verb (add-evidence-audit, slices
//! 1–2).
//!
//! Purpose: re-validate recorded evidence (the example/oracle corpus)
//! against the current model instead of trusting a prior run's good
//! verdicts — the graded-evidence failures the studies found were all
//! one concept: evidence whose scope silently widened.
//! Responsibilities: classify every evidence row — oracle records AND
//! pack-check results — into the audit-local closed
//! evidence-lifecycle vocabulary {current, stale, unchecked,
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
//! Slice 2 adds the pack-check row surface: a checker whose
//! checked-set is empty yields the structured `unchecked` verdict —
//! countable in the totals, never a silent pass (the va3q failure) —
//! and non-empty checked-set results carry through as their real
//! verdicts.
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

/// One evidence row's evidence-lifecycle class — an oracle record or
/// a pack-check result. Closed set: `current`, `stale`, `unchecked`,
/// `contradicted`, `new` (slice 1's subset of the vocabulary;
/// `binds_nothing` lands in slice 3). An audit record carries BOTH
/// this class and the underlying conform verdict.
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
// Pack-check rows (slice 2 — tasks 2.1–2.2, supersedes va3q): pack
// checker outcomes as structured, countable audit rows
// ---------------------------------------------------------------------------

/// One violation in a pack checker's checked-set — the checker's
/// labeled finding: the pack rule it violates and the finding message.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackViolation {
    pub rule_id: String,
    pub message: String,
}

/// One pack-checker outcome as the audit receives it (slice 2): the
/// pack that ran, the spec it was activated for, the rules it
/// declares, and the checked-set it produced. The checked-set is the
/// honesty subject: an EMPTY one means the checker ran over activated
/// vocabulary but produced nothing — the row is `unchecked`, never a
/// pass (the va3q failure: an empty-checked-set warning string read
/// as a pass even when the checked file had real problems).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackCheck {
    /// The pack that ran (its manifest id).
    pub pack_id: String,
    /// The spec the pack was activated for (its intent id) — the row
    /// key's subject half.
    pub subject_id: String,
    /// The rules the pack declares (its vocabulary): context for the
    /// honesty reason — the reason names how many rules the pack
    /// declares, so a checker over zero rules with an empty
    /// checked-set reads as doubly vacuous, not as doubly verified.
    pub declared_rules: Vec<String>,
    /// The checked-set: the violations the checker produced.
    pub violations: Vec<PackViolation>,
}

/// One pack-check row (tasks 2.2): the audit record classifying one
/// checker run's outcome, keyed `pack:<pack_id>:<subject_id>` —
/// deterministic and diffable across reports. An empty checked-set
/// yields the structured `unchecked` verdict: the verdict carrier is
/// conform's nothing-was-interpreted `agree`, whose audit layer IS
/// [`AuditClass::Unchecked`], so the record carries BOTH layers
/// without ever claiming a check happened — `evaluated_claim_ids`
/// stays empty (nothing was evaluated). A non-empty checked-set
/// carries through as its real verdicts: each violation is a positive
/// contradiction with the pack's declared rules — `forbidden` riding
/// `contradicted` — with the violated rules naming what the verdict
/// rests on.
pub fn pack_check_row(check: &PackCheck) -> AuditRecord {
    let scenario_id = format!("pack:{}:{}", check.pack_id, check.subject_id);
    if check.violations.is_empty() {
        AuditRecord {
            scenario_id,
            class: AuditClass::of(Verdict::Agree),
            conform_verdict: Verdict::Agree,
            reason: format!(
                "pack `{}` activated for `{}` over {} declared rule(s) — empty checked-set (no violations to report): nothing was checked; absence of violations is absence of evidence, never a pass",
                check.pack_id,
                check.subject_id,
                check.declared_rules.len(),
            ),
            evaluated_claim_ids: Vec::new(),
            closed_world: false,
        }
    } else {
        // Deterministic listing: each violation surfaces as
        // `rule (message)` — the checker's labeled finding carries
        // through verbatim, not just its rule id — sorted and deduped;
        // the ids the row rests on stay the plain deduped rule ids.
        let mut shown: Vec<String> = check
            .violations
            .iter()
            .map(|v| {
                if v.message.is_empty() {
                    v.rule_id.clone()
                } else {
                    format!("{} ({})", v.rule_id, v.message)
                }
            })
            .collect();
        shown.sort();
        shown.dedup();
        let mut violated: Vec<String> =
            check.violations.iter().map(|v| v.rule_id.clone()).collect();
        violated.sort();
        violated.dedup();
        AuditRecord {
            scenario_id,
            class: AuditClass::of(Verdict::Forbidden),
            conform_verdict: Verdict::Forbidden,
            reason: format!(
                "pack `{}` found {} checked-set violation(s) for `{}`: {}",
                check.pack_id,
                check.violations.len(),
                check.subject_id,
                shown.join(", "),
            ),
            evaluated_claim_ids: violated,
            closed_world: false,
        }
    }
}

/// The pack-check rows for a run's checker outcomes (tasks 2.2), in
/// the report's canonical order — sorted by row key, the same
/// discipline as [`build_report`], so serialization stays
/// byte-identical across runs over identical inputs. One check per
/// `(pack_id, subject_id)` is the caller's discipline: duplicate row
/// keys are refused labeled if the merged report later feeds
/// `--before` (the slice-1 prior-parse guard).
pub fn pack_check_rows(checks: &[PackCheck]) -> Vec<AuditRecord> {
    let mut rows: Vec<AuditRecord> = checks.iter().map(pack_check_row).collect();
    rows.sort_by(|a, b| a.scenario_id.cmp(&b.scenario_id));
    rows
}

/// Fold pack-check rows into a report (tasks 2.2): the rows join the
/// records and BOTH count maps are recomputed over the merged set —
/// pack evidence counts toward the totals like every other row, never
/// silently dropped from the summary. The exit contract composes
/// unchanged: all-unchecked pack rows stay exit 0 with counts
/// surfaced; a violation row is contradicted evidence and fails the
/// run. Duplicate row keys in `rows` pass through as-is here — the
/// `--before` prior-parse guard refuses them labeled downstream.
pub fn with_pack_rows(mut report: AuditReport, rows: Vec<AuditRecord>) -> AuditReport {
    report.records.extend(rows);
    report
        .records
        .sort_by(|a, b| a.scenario_id.cmp(&b.scenario_id));
    let mut verdict_counts: BTreeMap<String, usize> = BTreeMap::new();
    let mut class_counts: BTreeMap<String, usize> = BTreeMap::new();
    for record in &report.records {
        *verdict_counts
            .entry(record.conform_verdict.as_str().to_string())
            .or_insert(0) += 1;
        *class_counts
            .entry(record.class.as_str().to_string())
            .or_insert(0) += 1;
    }
    report.verdict_counts = verdict_counts;
    report.class_counts = class_counts;
    report
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

/// The diff's diffability preconditions (design D1 + risks), checked
/// before any row is compared: reports from different audit schema
/// versions are never silently diffed — the version field exists to
/// guard exactly this drift — and a `scope_sha256` mismatch is a
/// labeled refusal: per-row comparison would silently diff different
/// evidence across corpus versions (the spec may differ — that is the
/// diff's purpose; the corpus may not).
pub(crate) fn ensure_diffable(
    current: &AuditReport,
    prior: &AuditReport,
) -> Result<(), ConformError> {
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
    Ok(())
}

/// The pairwise overlay decision (design D1): the current class held
/// or improved ⇒ `current` (improved evidence is current); it worsened
/// by the [`AuditClass::rank`] order ⇒ `stale`. The `new` overlay is
/// not a pairwise decision — a row the prior report never saw has no
/// prior class to compare — so it stays in [`diff_with_prior`].
pub(crate) fn overlay_class(current: AuditClass, prior: AuditClass) -> AuditClass {
    if AuditClass::rank(current) > AuditClass::rank(prior) {
        AuditClass::Stale
    } else {
        AuditClass::Current
    }
}

/// The prior-report diff (design D1): compare per-row classes with the
/// prior report — same class ⇒ `current`, class dropped/worsened ⇒
/// `stale`, improved ⇒ `current` (the evidence got better; it is
/// current), no prior row ⇒ `new`; [`ensure_diffable`] refusals come
/// first. Recomputes `class_counts` over the diffed classes.
pub fn diff_with_prior(
    mut current: AuditReport,
    prior: &AuditReport,
) -> Result<AuditReport, ConformError> {
    ensure_diffable(&current, prior)?;
    for record in &mut current.records {
        record.class = match prior
            .records
            .iter()
            .find(|r| r.scenario_id == record.scenario_id)
        {
            None => AuditClass::New,
            Some(prior_row) => overlay_class(record.class, prior_row.class),
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

// ---------------------------------------------------------------------------
// Unit tests — the `--before` diff unit pinned in isolation (tasks.md
// 1.5, slice-1 refactor): constructed reports only, so the pins test
// the UNIT — no spec parse, no conform walk, no CLI. The counts and
// scope fields a constructed report carries are inert here: the diff
// recomputes class_counts and only reads the schema version, the
// digest, and the per-row records.
// ---------------------------------------------------------------------------

#[cfg(test)]
mod diff_tests {
    use super::*;

    /// A minimal report: only the diff's load-bearing fields matter
    /// (schema version, digest, records).
    fn report(digest: &str, records: Vec<AuditRecord>) -> AuditReport {
        AuditReport {
            report_schema_version: AUDIT_SCHEMA_VERSION,
            evidence_scope: String::new(),
            scope_sha256: digest.to_string(),
            expectation_declared: false,
            verdict_counts: BTreeMap::new(),
            class_counts: BTreeMap::new(),
            records,
        }
    }

    fn record(id: &str, class: AuditClass, verdict: Verdict) -> AuditRecord {
        AuditRecord {
            scenario_id: id.to_string(),
            class,
            conform_verdict: verdict,
            reason: "fixture reason".to_string(),
            evaluated_claim_ids: Vec::new(),
            closed_world: false,
        }
    }

    fn class_of(diffed: &AuditReport, id: &str) -> AuditClass {
        diffed
            .records
            .iter()
            .find(|r| r.scenario_id == id)
            .unwrap_or_else(|| panic!("diffed row for {id} present"))
            .class
    }

    /// Same class ⇒ `current`; improved ⇒ `current` too (the evidence
    /// got better; it is current).
    #[test]
    fn same_or_improved_class_is_current() {
        let prior = report(
            "digest",
            vec![
                record("held", AuditClass::Current, Verdict::Permitted),
                record("gap", AuditClass::Unchecked, Verdict::Underspecified),
            ],
        );
        let current = report(
            "digest",
            vec![
                record("held", AuditClass::Current, Verdict::Permitted),
                record("gap", AuditClass::Current, Verdict::Permitted),
            ],
        );
        let diffed = diff_with_prior(current, &prior).expect("same digest diffs");
        assert_eq!(class_of(&diffed, "held"), AuditClass::Current);
        assert_eq!(class_of(&diffed, "gap"), AuditClass::Current);
        assert_eq!(diffed.class_counts.get("current"), Some(&2));
    }

    /// Dropped/worsened class ⇒ `stale`, and the stale record carries
    /// the CURRENT run's conform verdict — the audit rides the fresh
    /// walk, never the prior report's verdicts.
    #[test]
    fn worsened_class_is_stale_carrying_the_current_verdict() {
        let prior = report(
            "digest",
            vec![
                record("dropped", AuditClass::Current, Verdict::Permitted),
                record("worsened", AuditClass::Current, Verdict::Permitted),
            ],
        );
        let current = report(
            "digest",
            vec![
                record("dropped", AuditClass::Unchecked, Verdict::Underspecified),
                record("worsened", AuditClass::Contradicted, Verdict::Forbidden),
            ],
        );
        let diffed = diff_with_prior(current, &prior).expect("same digest diffs");
        let dropped = diffed
            .records
            .iter()
            .find(|r| r.scenario_id == "dropped")
            .expect("diffed row for dropped present");
        assert_eq!(dropped.class, AuditClass::Stale);
        assert_eq!(dropped.conform_verdict, Verdict::Underspecified);
        let worsened = diffed
            .records
            .iter()
            .find(|r| r.scenario_id == "worsened")
            .expect("diffed row for worsened present");
        assert_eq!(worsened.class, AuditClass::Stale);
        assert_eq!(worsened.conform_verdict, Verdict::Forbidden);
        assert_eq!(diffed.class_counts.get("stale"), Some(&2));
    }

    /// A row the prior report never saw ⇒ `new` — fresh evidence is
    /// never graded against a baseline that did not record it.
    #[test]
    fn row_absent_from_prior_is_new() {
        let prior = report("digest", vec![]);
        let current = report(
            "digest",
            vec![record("fresh", AuditClass::Current, Verdict::Permitted)],
        );
        let diffed = diff_with_prior(current, &prior).expect("same digest diffs");
        assert_eq!(class_of(&diffed, "fresh"), AuditClass::New);
        assert_eq!(diffed.class_counts.get("new"), Some(&1));
    }

    /// An empty prior report (a baseline that shared no rows with this
    /// run) ⇒ every current row is `new`.
    #[test]
    fn empty_prior_report_makes_every_row_new() {
        let prior = report("digest", vec![]);
        let current = report(
            "digest",
            vec![
                record("a", AuditClass::Current, Verdict::Permitted),
                record("b", AuditClass::Unchecked, Verdict::Unknown),
            ],
        );
        let diffed = diff_with_prior(current, &prior).expect("same digest diffs");
        assert_eq!(class_of(&diffed, "a"), AuditClass::New);
        assert_eq!(class_of(&diffed, "b"), AuditClass::New);
        assert_eq!(diffed.class_counts.get("new"), Some(&2));
    }

    /// A `scope_sha256` mismatch between the reports is a labeled
    /// refusal — rows are never compared across corpus versions.
    #[test]
    fn digest_mismatch_is_a_labeled_refusal() {
        let prior = report("prior-digest", vec![]);
        let current = report(
            "current-digest",
            vec![record("a", AuditClass::Current, Verdict::Permitted)],
        );
        let err = diff_with_prior(current, &prior).expect_err("digest mismatch refuses");
        assert_eq!(err.stage, "before_diff");
        assert!(
            err.message.contains("scope_sha256"),
            "the refusal names the digest field: {err}"
        );
        assert!(
            err.message.contains("spk audit"),
            "the refusal carries a remediation hint: {err}"
        );
    }

    /// A schema-version mismatch is a labeled refusal, checked before
    /// the digest — the version field exists to guard exactly this
    /// drift between report families' generations.
    #[test]
    fn schema_version_mismatch_is_a_labeled_refusal() {
        let mut prior = report("digest", vec![]);
        prior.report_schema_version = AUDIT_SCHEMA_VERSION + 1;
        let current = report(
            "digest",
            vec![record("a", AuditClass::Current, Verdict::Permitted)],
        );
        let err = diff_with_prior(current, &prior).expect_err("schema mismatch refuses");
        assert_eq!(err.stage, "before_diff");
        assert!(
            err.message.contains("schema version"),
            "the refusal names the schema drift: {err}"
        );
    }

    /// The extracted pairwise unit's contract at its own granularity:
    /// held/improved ⇒ `current`, worsened (current→unchecked,
    /// unchecked→contradicted) ⇒ `stale` — the [`AuditClass::rank`]
    /// order is the whole decision.
    #[test]
    fn overlay_class_ranks_worsening_only() {
        assert_eq!(
            overlay_class(AuditClass::Current, AuditClass::Current),
            AuditClass::Current
        );
        assert_eq!(
            overlay_class(AuditClass::Unchecked, AuditClass::Current),
            AuditClass::Stale
        );
        assert_eq!(
            overlay_class(AuditClass::Contradicted, AuditClass::Unchecked),
            AuditClass::Stale
        );
        assert_eq!(
            overlay_class(AuditClass::Current, AuditClass::Unchecked),
            AuditClass::Current
        );
    }
}

#[cfg(test)]
mod pack_row_tests {
    use super::*;

    /// The pack-check row unit pinned in isolation (tasks.md 2.1–2.2,
    /// slice 2): constructed [`PackCheck`] outcomes only — no spec
    /// parse, no conform walk, no CLI. The pins cover the delta-spec
    /// scenarios "Empty checked-set is unchecked" and "Unchecked never
    /// fails the run": the vacuous checked-set is a structured,
    /// countable `unchecked` row — never a pass — and a non-empty
    /// checked-set carries its violations through as their real
    /// verdicts (forbidden ⇒ contradicted).

    fn check(
        pack: &str,
        subject: &str,
        rules: &[&str],
        violations: Vec<PackViolation>,
    ) -> PackCheck {
        PackCheck {
            pack_id: pack.to_string(),
            subject_id: subject.to_string(),
            declared_rules: rules.iter().map(|s| s.to_string()).collect(),
            violations,
        }
    }

    fn violation(rule: &str, message: &str) -> PackViolation {
        PackViolation {
            rule_id: rule.to_string(),
            message: message.to_string(),
        }
    }

    /// An empty report to fold pack rows into: only the fold's
    /// load-bearing fields matter (records + the two count maps).
    fn empty_report() -> AuditReport {
        AuditReport {
            report_schema_version: AUDIT_SCHEMA_VERSION,
            evidence_scope: String::new(),
            scope_sha256: "digest".to_string(),
            expectation_declared: false,
            verdict_counts: BTreeMap::new(),
            class_counts: BTreeMap::new(),
            records: vec![],
        }
    }

    /// The delta scenario "Empty checked-set is unchecked": a checker
    /// that ran over vocabulary it activates but produced an empty
    /// checked-set is the structured `unchecked` class — countable,
    /// never a pass. The row carries BOTH layers: the verdict carrier
    /// is conform's nothing-was-interpreted verdict (`agree`), and the
    /// audit class is exactly its [`AuditClass::of`] layer, so the
    /// record never claims a check happened.
    #[test]
    fn empty_checked_set_is_unchecked_never_a_pass() {
        let rows = pack_check_rows(&[check(
            "packs.py",
            "some_spec",
            &["packs.py.kind_a", "packs.py.kind_b"],
            vec![],
        )]);
        assert_eq!(rows.len(), 1, "one row per checker run");
        let row = &rows[0];
        assert_eq!(row.class, AuditClass::Unchecked);
        assert_eq!(row.conform_verdict, Verdict::Agree);
        assert_eq!(
            AuditClass::of(row.conform_verdict),
            row.class,
            "the both-layer invariant: the class is the verdict's audit layer"
        );
        assert_eq!(
            row.scenario_id, "pack:packs.py:some_spec",
            "the row key names the pack and the subject"
        );
        assert!(
            row.reason.contains("empty checked-set"),
            "the reason names the vacuity: {}",
            row.reason
        );
        assert!(
            row.reason.contains("never a pass"),
            "the reason states the honesty rule: {}",
            row.reason
        );
        assert!(
            row.reason.contains("packs.py") && row.reason.contains("some_spec"),
            "the reason names the pack and the subject: {}",
            row.reason
        );
        assert!(
            row.reason.contains("2 declared rule(s)"),
            "the reason names how many rules the pack declares: {}",
            row.reason
        );
        assert!(
            row.evaluated_claim_ids.is_empty(),
            "nothing was evaluated — the row must not pretend otherwise"
        );
        assert!(!row.closed_world, "pack checks are not closed-world walks");
    }

    /// Non-empty checked-set results carry through as their real
    /// verdicts: a violation positively contradicts the pack's
    /// declared rules — `forbidden` riding its `contradicted` layer.
    #[test]
    fn violation_rows_carry_the_real_verdict() {
        let rows = pack_check_rows(&[check(
            "packs.py",
            "other_spec",
            &["packs.py.kind_a"],
            vec![
                violation("packs.py.kind_b", "kind not declared by the pack"),
                violation("packs.py.kind_c", "kind not declared by the pack"),
            ],
        )]);
        assert_eq!(rows.len(), 1, "one row per checker run");
        let row = &rows[0];
        assert_eq!(row.class, AuditClass::Contradicted);
        assert_eq!(row.conform_verdict, Verdict::Forbidden);
        assert_eq!(
            AuditClass::of(row.conform_verdict),
            row.class,
            "the both-layer invariant: the class is the verdict's audit layer"
        );
        assert_eq!(
            row.evaluated_claim_ids,
            vec!["packs.py.kind_b", "packs.py.kind_c"],
            "the violated rules carry through as the ids the row rests on"
        );
        assert!(
            row.reason.contains("2 checked-set violation(s)"),
            "the reason names the violation count: {}",
            row.reason
        );
        assert!(
            row.reason
                .contains("packs.py.kind_b (kind not declared by the pack)"),
            "the reason surfaces the violation's message verbatim: {}",
            row.reason
        );
    }

    /// Rows count toward the report's totals: the fold appends the
    /// rows, re-sorts by scenario id, and recomputes BOTH count maps —
    /// pack evidence is never silently dropped from the summary.
    #[test]
    fn pack_rows_count_toward_report_totals() {
        let rows = pack_check_rows(&[
            // fed out of id order to pin the fold's re-sort
            check(
                "packs.b",
                "spec_y",
                &["packs.b.t"],
                vec![violation("packs.b.v", "boom")],
            ),
            check("packs.a", "spec_x", &["packs.a.t"], vec![]),
        ]);
        let merged = with_pack_rows(empty_report(), rows);
        assert_eq!(merged.records.len(), 2);
        assert_eq!(
            merged.records[0].scenario_id, "pack:packs.a:spec_x",
            "records stay sorted by scenario id"
        );
        assert_eq!(merged.class_counts.get("unchecked"), Some(&1));
        assert_eq!(merged.class_counts.get("contradicted"), Some(&1));
        assert_eq!(merged.verdict_counts.get("agree"), Some(&1));
        assert_eq!(merged.verdict_counts.get("forbidden"), Some(&1));
    }

    /// The realistic fold (the slice-4 shape): pack rows joining
    /// EXISTING oracle rows. Counts are recomputed FROM SCRATCH over
    /// the merged records — seeded with a deliberately stale map, the
    /// fold must rebuild it, never increment onto it — and records
    /// stay sorted by row key across kinds.
    #[test]
    fn pack_rows_fold_alongside_oracle_rows() {
        let mut report = empty_report();
        report.records.push(AuditRecord {
            scenario_id: "oracle_setup_holds".to_string(),
            class: AuditClass::Current,
            conform_verdict: Verdict::Permitted,
            reason: "fixture reason".to_string(),
            evaluated_claim_ids: Vec::new(),
            closed_world: false,
        });
        // deliberately stale maps: a prior diff's leftovers that must
        // not survive a from-scratch recompute
        report.class_counts.insert("stale".to_string(), 5);
        report.verdict_counts.insert("unknown".to_string(), 5);
        let rows = pack_check_rows(&[check("packs.a", "spec_x", &["packs.a.t"], vec![])]);
        let merged = with_pack_rows(report, rows);
        assert_eq!(merged.records.len(), 2);
        assert_eq!(
            merged.records[0].scenario_id, "oracle_setup_holds",
            "row-key order puts the oracle row first"
        );
        assert_eq!(
            merged.class_counts.get("stale"),
            None,
            "stale counts never survive the recompute"
        );
        assert_eq!(merged.verdict_counts.get("unknown"), None);
        assert_eq!(merged.class_counts.get("current"), Some(&1));
        assert_eq!(merged.class_counts.get("unchecked"), Some(&1));
        assert_eq!(merged.verdict_counts.get("permitted"), Some(&1));
        assert_eq!(merged.verdict_counts.get("agree"), Some(&1));
    }

    /// The delta scenario "Unchecked never fails the run": every row
    /// unchecked ⇒ exit 0 (under or without a declared expectation),
    /// with the counts countable in the summary — the vacuous
    /// checked-set is visible, but it never fails the run.
    #[test]
    fn all_unchecked_pack_rows_exit_zero_with_counts() {
        let rows = pack_check_rows(&[
            check("packs.a", "spec_x", &["packs.a.t"], vec![]),
            check("packs.b", "spec_y", &["packs.b.t"], vec![]),
        ]);
        let merged = with_pack_rows(empty_report(), rows);
        assert_eq!(exit_code(&merged, false), 0);
        assert_eq!(exit_code(&merged, true), 0);
        assert!(
            merged.class_counts.get("unchecked").copied().unwrap_or(0) >= 2,
            "the unchecked rows are countable in the summary"
        );
    }

    /// A pack violation is contradicted evidence: it fails the run
    /// (exit 1) — never a silent pass in either direction.
    #[test]
    fn pack_violation_rows_fail_the_run() {
        let rows = pack_check_rows(&[check(
            "packs.b",
            "spec_y",
            &["packs.b.t"],
            vec![violation("packs.b.v", "boom")],
        )]);
        let merged = with_pack_rows(empty_report(), rows);
        assert_eq!(exit_code(&merged, false), 1);
    }
}

// ---------------------------------------------------------------------------
// Emission-surface pins (tasks.md 2.3, slice-2 refactor): the unified
// row-emission surface's contract, pinned BEFORE the unification and
// written against the current internal API locations — oracle rows via
// [`build_report`]'s walk-and-construct, pack rows via
// [`pack_check_row`]'s branches, counts via both compute sites — so
// the extraction into one shared surface must keep these green (zero
// behavior change IS the ticket's pin). Constructed fixtures only: a
// parsed spec + corpus for the oracle side, constructed [`PackCheck`]s
// for the pack side — no CLI, no gate, no compiled artifacts.
// ---------------------------------------------------------------------------

#[cfg(test)]
mod emission_tests {
    use super::*;

    /// The oracle-side fixture spec: declares `confirm` AND `reopen`
    /// (the integration fixture's shape at tests/audit_oracle_rows.rs;
    /// no gate runs here, so no Properties table is needed).
    const SPEC: &str = r#"---
id: demo.audit
kind: intent
statement: "THE system SHALL keep the ledger consistent"
---

## Constraints

| id | kind | expr | traces_to |
|----|------|------|-----------|
| locked | invariant | `**rust:** status != "open"` | [[demo.audit]] |

## Model

### States

- held
- confirmed

### Transitions

| id | from | to | guard |
|----|------|----|-------|
| confirm | held | confirmed | [[demo.audit.locked]] |
| reopen | confirmed | held | [[demo.audit.locked]] |
"#;

    /// The three oracle corpus lines spread the full class range:
    /// permitted, uncovered (underspecified open-world), and a positive
    /// contradiction.
    const CORPUS_LINES: &[&str] = &[
        r#"{"id": "confirm-happy", "setup": {"state": "held"}, "trace": [{"action": "confirm", "observations": {"status": "confirmed"}}]}"#,
        r#"{"id": "undo-undeclared", "setup": {"state": "held"}, "trace": [{"action": "undo"}]}"#,
        r#"{"id": "confirm-from-wrong-state", "setup": {"state": "confirmed"}, "trace": [{"action": "confirm"}]}"#,
    ];

    /// The oracle report built over the fixture corpus.
    fn oracle_report(corpus_lines: &[&str]) -> AuditReport {
        let spec = crate::spec::parse_str(SPEC).expect("fixture spec parses");
        let corpus_bytes = format!("{}\n", corpus_lines.join("\n"));
        let corpus = conform::parse_corpus(&corpus_bytes).expect("fixture corpus parses");
        build_report(&spec, corpus_bytes.as_bytes(), &corpus, false, false)
    }

    /// A shell report carrying only the fold's load-bearing fields.
    fn shell() -> AuditReport {
        AuditReport {
            report_schema_version: AUDIT_SCHEMA_VERSION,
            evidence_scope: String::new(),
            scope_sha256: "digest".to_string(),
            expectation_declared: false,
            verdict_counts: BTreeMap::new(),
            class_counts: BTreeMap::new(),
            records: vec![],
        }
    }

    fn pack_check(pack: &str, subject: &str) -> PackCheck {
        PackCheck {
            pack_id: pack.to_string(),
            subject_id: subject.to_string(),
            declared_rules: vec![format!("{pack}.t")],
            violations: vec![],
        }
    }

    /// The unified emission surface's core contract across BOTH row
    /// kinds: every emitted row carries the BOTH-layer payload — the
    /// conform verdict AND the class that is exactly its
    /// [`AuditClass::of`] layer — with the row key verbatim (oracle ids
    /// straight from the corpus, pack keys `pack:<pack_id>:<subject_id>`).
    #[test]
    fn emission_is_both_layered_across_row_kinds() {
        let report = oracle_report(CORPUS_LINES);
        let expected: &[(&str, Verdict, AuditClass)] = &[
            (
                "confirm-from-wrong-state",
                Verdict::Forbidden,
                AuditClass::Contradicted,
            ),
            ("confirm-happy", Verdict::Permitted, AuditClass::Current),
            ("undo-undeclared", Verdict::Underspecified, AuditClass::Unchecked),
        ];
        assert_eq!(report.records.len(), expected.len(), "one row per corpus line");
        for (row, (id, verdict, class)) in report.records.iter().zip(expected) {
            assert_eq!(&row.scenario_id, id, "the oracle row key is the corpus id");
            assert_eq!(&row.conform_verdict, verdict, "the verdict rides through");
            assert_eq!(&row.class, class, "the class is the verdict's audit layer");
            assert_eq!(
                AuditClass::of(row.conform_verdict),
                row.class,
                "the both-layer invariant holds for oracle row {id}"
            );
        }
        // The pack branches emit through the same contract: the empty
        // checked-set rides `agree` (its layer IS unchecked), the
        // violation rides `forbidden` (its layer IS contradicted).
        let empty = pack_check_row(&pack_check("packs.a", "spec_x"));
        assert_eq!(empty.scenario_id, "pack:packs.a:spec_x");
        assert_eq!(empty.conform_verdict, Verdict::Agree);
        assert_eq!(AuditClass::of(empty.conform_verdict), empty.class);
        assert_eq!(empty.class, AuditClass::Unchecked);
        let mut violated = pack_check("packs.a", "spec_x");
        violated.violations = vec![PackViolation {
            rule_id: "packs.a.v".to_string(),
            message: "boom".to_string(),
        }];
        let violated = pack_check_row(&violated);
        assert_eq!(violated.scenario_id, "pack:packs.a:spec_x");
        assert_eq!(violated.conform_verdict, Verdict::Forbidden);
        assert_eq!(AuditClass::of(violated.conform_verdict), violated.class);
        assert_eq!(violated.class, AuditClass::Contradicted);
    }

    /// The two count-map compute sites agree: recomputing
    /// [`with_pack_rows`]' maps from scratch over [`build_report`]'s own
    /// records reproduces exactly the maps [`build_report`] produced —
    /// seeded garbage counts never survive — and pack rows fold onto
    /// those counts additively.
    #[test]
    fn count_maps_agree_between_the_compute_sites() {
        let built = oracle_report(CORPUS_LINES);
        let mut reseeded = shell();
        reseeded.records = built.records.clone();
        // deliberately stale maps: prior-diff leftovers that must not
        // survive a from-scratch recompute
        reseeded.class_counts.insert("stale".to_string(), 99);
        reseeded.verdict_counts.insert("unknown".to_string(), 99);
        let recomputed = with_pack_rows(reseeded, vec![]);
        assert_eq!(recomputed.class_counts, built.class_counts);
        assert_eq!(recomputed.verdict_counts, built.verdict_counts);
        // pack rows fold onto the recomputed counts additively: the
        // pack's unchecked row joins the oracle's unchecked row
        let merged = with_pack_rows(recomputed, pack_check_rows(&[
            pack_check("packs.a", "spec_x"),
        ]));
        assert_eq!(merged.class_counts.get("unchecked"), Some(&2));
        assert_eq!(merged.class_counts.get("contradicted"), Some(&1));
        assert_eq!(merged.verdict_counts.get("agree"), Some(&1));
        assert_eq!(merged.verdict_counts.get("underspecified"), Some(&1));
    }

    /// Row order is key-sorted across BOTH paths: [`build_report`]
    /// sorts its walk output even when the corpus is fed out of id
    /// order, and [`with_pack_rows`] interleaves oracle and pack rows
    /// into one key-sorted sequence across kinds.
    #[test]
    fn row_order_is_key_sorted_across_kinds_in_both_paths() {
        let shuffled: Vec<&str> = CORPUS_LINES.iter().rev().copied().collect();
        let built = oracle_report(&shuffled);
        let oracle_ids: Vec<&str> = built
            .records
            .iter()
            .map(|r| r.scenario_id.as_str())
            .collect();
        let mut sorted = oracle_ids.clone();
        sorted.sort();
        assert_eq!(oracle_ids, sorted, "oracle rows come out key-sorted");
        let merged = with_pack_rows(
            built,
            pack_check_rows(&[
                pack_check("packs.m", "spec_z"),
                pack_check("packs.a", "spec_b"),
            ]),
        );
        let ids: Vec<&str> = merged
            .records
            .iter()
            .map(|r| r.scenario_id.as_str())
            .collect();
        let mut expected = ids.clone();
        expected.sort();
        assert_eq!(ids, expected, "the merged rows interleave in key order");
        assert_eq!(
            ids.iter().filter(|id| id.starts_with("pack:")).count(),
            2,
            "both pack rows ride in the merged order"
        );
    }
}
