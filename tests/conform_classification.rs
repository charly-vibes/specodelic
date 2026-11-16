//! Phase-1 verdict-engine tests for `conform` (openspec/changes/add-conform,
//! tasks 1.1–1.4): the closed five-valued taxonomy over a fixture spec +
//! fixture JSONL corpus, evidence-class separation (design D2), and
//! taxonomy totality. Pure library — no CLI.

use specodelic::conform::{
    Verdict, build_report, classify_claims, classify_traces, human_view, parse_corpus,
};
use specodelic::spec;

// ---------------------------------------------------------------------------
// Fixture: a spec whose Model exercises every verdict class
// ---------------------------------------------------------------------------

/// Fixture spec (parses; not compiled — phase 1 consumes the parsed spec
/// through the same IR extraction model_check uses):
///
/// - `locked`  — executable invariant (`**rust:**` fragment)
/// - `audited` — prose-only invariant (never implies a judgment)
/// - `portable`— `**py:**` fragment: evaluator kind exists in the format,
///   has no emitter in this run → `unsupported`
const SPEC_TEXT: &str = r#"---
id: demo.conform
kind: intent
statement: "THE system SHALL conform"
---

## Constraints

| id | kind | expr | traces_to |
|----|------|------|-----------|
| locked | invariant | `**rust:** status != "open"` | [[demo.conform]] |
| audited | invariant | `the ledger is audited before any confirmation` | [[demo.conform]] |
| portable | invariant | `**py:** status != "open"` | [[demo.conform]] |

## Model

### States

- held
- confirmed

### Transitions

| id | from | to | guard |
|----|------|----|-------|
| confirm | held | confirmed | [[demo.conform.locked]] ∧ [[demo.conform.audited]] |
| audit | held | confirmed | [[demo.conform.audited]] |
| reopen | confirmed | held | [[demo.conform.locked]] |
| escalate | held | held | [[demo.conform.portable]] |
| crosscheck | held | held | [[other.spec.audited]] |
"#;

fn fixture_spec() -> spec::Spec {
    spec::parse_str(SPEC_TEXT).expect("fixture spec parses")
}

fn corpus(lines: &[&str]) -> Vec<specodelic::conform::ScenarioTrace> {
    parse_corpus(&lines.join("\n")).expect("fixture corpus parses")
}

// ---------------------------------------------------------------------------
// 1.1 — the four classification cases over the fixture corpus
// ---------------------------------------------------------------------------

/// Executable-claim contradiction → `forbidden`, reason names the claim.
/// `reopen` is declared only from `confirmed`; exercising it from `held`
/// asserts a step the compiled Model's transition relation forbids — the
/// same fact model_check reports as `counterexample_found`.
#[test]
fn executable_claim_contradiction_is_forbidden_naming_the_claim() {
    let spec = fixture_spec();
    let corpus = corpus(&[
        r#"{"id": "wrong-state-reopen", "setup": {"state": "held"}, "trace": [{"action": "reopen"}]}"#,
    ]);
    let records = classify_traces(&spec, &corpus, false);
    assert_eq!(records.len(), 1);
    let r = &records[0];
    assert_eq!(r.scenario_id, "wrong-state-reopen");
    assert_eq!(r.verdict, Verdict::Forbidden);
    assert!(
        r.reason.contains("contradiction_forbidden_any_mode"),
        "reason must name the contradiction rule: {}",
        r.reason
    );
    assert!(
        r.reason.contains("locked"),
        "reason must name the contradicted executable claim id: {}",
        r.reason
    );
    assert!(
        r.evaluated_claim_ids.iter().any(|id| id == "locked"),
        "evaluated claim ids must include the contradicted claim: {:?}",
        r.evaluated_claim_ids
    );
}

/// Prose-only covering claim → `unknown` naming the claim — prose never
/// implies a permitted/forbidden judgment (`prose_never_checked`).
#[test]
fn prose_only_covering_claim_is_unknown_naming_the_claim() {
    let spec = fixture_spec();
    let corpus = corpus(&[
        r#"{"id": "confirm-happy", "setup": {"state": "held"}, "trace": [{"action": "confirm", "observations": {"status": "confirmed"}}]}"#,
    ]);
    let records = classify_traces(&spec, &corpus, false);
    assert_eq!(records.len(), 1);
    let r = &records[0];
    assert_eq!(r.verdict, Verdict::Unknown);
    assert!(
        r.reason.contains("audited"),
        "reason must name the prose claim that covers the trace: {}",
        r.reason
    );
    assert_ne!(r.verdict, Verdict::Permitted);
    assert_ne!(r.verdict, Verdict::Forbidden);
}

/// No declared Model element or claim covers the trace → `underspecified`
/// in the default open-world mode — a missing transition is
/// underspecification, never prohibition.
#[test]
fn undeclared_action_is_underspecified() {
    let spec = fixture_spec();
    let corpus = corpus(&[
        r#"{"id": "missing-expire", "setup": {"state": "held"}, "trace": [{"action": "expire"}]}"#,
    ]);
    let records = classify_traces(&spec, &corpus, false);
    assert_eq!(records.len(), 1);
    let r = &records[0];
    assert_eq!(r.verdict, Verdict::Underspecified);
    assert!(
        r.reason.contains("uncovered_trace_never_forbidden"),
        "reason must name the uncovered rule: {}",
        r.reason
    );
    // The declaration gate and the naming law are independent: the
    // scenario id must still travel with the record.
    assert_eq!(r.scenario_id, "missing-expire");
}

/// Unsupported evaluator kind → `unsupported` naming the kind — a
/// `**py:**` covering claim exists in the format but has no emitter in
/// this run (`unsupported_kind_named`).
#[test]
fn unsupported_evaluator_kind_names_the_kind() {
    let spec = fixture_spec();
    let corpus = corpus(&[
        r#"{"id": "escalate-happy", "setup": {"state": "held"}, "trace": [{"action": "escalate"}]}"#,
    ]);
    let records = classify_traces(&spec, &corpus, false);
    assert_eq!(records.len(), 1);
    let r = &records[0];
    assert_eq!(r.verdict, Verdict::Unsupported);
    assert!(
        r.reason.contains("py"),
        "reason must name the evaluator kind: {}",
        r.reason
    );
    // Naming the kind must leave exactly one labeled evaluation — the
    // unsupported claim itself, not a silent pass.
    let evaluated = r.evaluated_claim_ids.len();
    assert_eq!(evaluated, 1, "only the unsupported claim is evaluated");
}

/// A fully declared, Model-explainable trace with no prose/unsupported
/// covering claim is `permitted`.
#[test]
fn model_explainable_trace_is_permitted() {
    let spec = fixture_spec();
    let corpus = corpus(&[
        r#"{"id": "reopen-happy", "setup": {"state": "confirmed"}, "trace": [{"action": "reopen"}]}"#,
    ]);
    let records = classify_traces(&spec, &corpus, false);
    assert_eq!(records.len(), 1);
    assert_eq!(records[0].verdict, Verdict::Permitted);
    // A permitted explainable trace evaluates the corpus claims it
    // cites — the record must name at least one evaluated claim.
    assert!(
        !records[0].evaluated_claim_ids.is_empty(),
        "a permitted trace must record its evaluated claims"
    );
}

// ---------------------------------------------------------------------------
// 1.2 — claim classification reuses model_check's required/unchecked split
// ---------------------------------------------------------------------------

#[test]
fn claim_classification_partitions_required_unchecked_unsupported() {
    let spec = fixture_spec();
    let c = classify_claims(&spec);
    assert!(
        c.required.iter().any(|cl| cl.id == "locked"),
        "the rust-fragment invariant is required (executable): {:?}",
        c.required
    );
    assert!(
        c.unchecked.iter().any(|cl| cl.id == "audited"),
        "the prose-only invariant stays unchecked: {:?}",
        c.unchecked
    );
    assert!(
        c.unsupported
            .iter()
            .any(|cl| cl.id == "portable" && cl.evaluator_label() == "py"),
        "the py fragment is unsupported, naming the kind: {:?}",
        c.unsupported
    );
}

// ---------------------------------------------------------------------------
// 1.3 — evidence-class separation (design D2)
// ---------------------------------------------------------------------------

/// A contradicting trace is `forbidden` in ANY invocation mode: the
/// record carries `closed_world: false` and the reason names the claim.
#[test]
fn contradiction_forbidden_any_mode() {
    let spec = fixture_spec();
    let corpus = corpus(&[
        r#"{"id": "wrong-state-reopen", "setup": {"state": "held"}, "trace": [{"action": "reopen"}]}"#,
    ]);
    let records = classify_traces(&spec, &corpus, false);
    assert_eq!(records[0].verdict, Verdict::Forbidden);
    assert!(
        !records[0].closed_world,
        "no closed-world declaration was made"
    );
    assert!(
        records[0]
            .reason
            .contains("contradiction_forbidden_any_mode")
    );
    assert!(records[0].reason.contains("locked"));
}

/// An uncovered trace WITH the closed-world declaration → `forbidden`,
/// `closed_world: true`, the exhaustiveness declaration named in the reason.
#[test]
fn closed_world_forbidden_recorded() {
    let spec = fixture_spec();
    let corpus = corpus(&[
        r#"{"id": "missing-expire", "setup": {"state": "held"}, "trace": [{"action": "expire"}]}"#,
    ]);
    let records = classify_traces(&spec, &corpus, true);
    assert_eq!(records[0].verdict, Verdict::Forbidden);
    assert!(records[0].closed_world);
    assert!(
        records[0]
            .reason
            .contains("closed_world_forbidden_recorded"),
        "reason must name the closed-world rule: {}",
        records[0].reason
    );
}

/// The same uncovered trace WITHOUT the declaration → `underspecified`,
/// never `forbidden` (`uncovered_trace_never_forbidden`).
#[test]
fn uncovered_trace_never_forbidden_without_declaration() {
    let spec = fixture_spec();
    let corpus = corpus(&[
        r#"{"id": "missing-expire", "setup": {"state": "held"}, "trace": [{"action": "expire"}]}"#,
    ]);
    let records = classify_traces(&spec, &corpus, false);
    assert_eq!(records[0].verdict, Verdict::Underspecified);
    assert!(!records[0].closed_world);
    // Underspecified means "no gate fired" — no claims were evaluated
    // against the trace.
    let evaluated = records[0].evaluated_claim_ids.len();
    assert_eq!(evaluated, 0, "an underspecified trace evaluates nothing");
}

// ---------------------------------------------------------------------------
// 1.4 — taxonomy totality (verdict_taxonomy_distinct)
// ---------------------------------------------------------------------------

/// Every corpus trace receives exactly one verdict; the fixture corpus
/// reaches all five values (`taxonomy_is_total_and_distinct`).
#[test]
fn taxonomy_is_total_and_distinct() {
    let spec = fixture_spec();
    let corpus = corpus(&[
        r#"{"id": "reopen-happy", "setup": {"state": "confirmed"}, "trace": [{"action": "reopen"}]}"#,
        r#"{"id": "wrong-state-reopen", "setup": {"state": "held"}, "trace": [{"action": "reopen"}]}"#,
        r#"{"id": "missing-expire", "setup": {"state": "held"}, "trace": [{"action": "expire"}]}"#,
        r#"{"id": "confirm-happy", "setup": {"state": "held"}, "trace": [{"action": "confirm", "observations": {"status": "confirmed"}}]}"#,
        r#"{"id": "escalate-happy", "setup": {"state": "held"}, "trace": [{"action": "escalate"}]}"#,
    ]);
    let records = classify_traces(&spec, &corpus, false);
    assert_eq!(records.len(), 5, "one record per trace — total");
    let mut ids: Vec<&str> = records.iter().map(|r| r.scenario_id.as_str()).collect();
    ids.sort();
    ids.dedup();
    assert_eq!(ids.len(), 5, "each trace maps to exactly one verdict");
    let mut seen: Vec<Verdict> = records.iter().map(|r| r.verdict).collect();
    seen.sort();
    seen.dedup();
    assert_eq!(
        seen.len(),
        5,
        "all five verdict values are reachable and distinct: {:?}",
        seen
    );
    for v in [
        Verdict::Permitted,
        Verdict::Forbidden,
        Verdict::Underspecified,
        Verdict::Unknown,
        Verdict::Unsupported,
    ] {
        assert!(seen.contains(&v), "missing verdict {:?} in {:?}", v, seen);
    }
}

/// Mechanical name identity after trimming (design D3): a scenario whose
/// action/setup carry stray whitespace classifies identically to the
/// trimmed spelling — never semantically, only string identity.
#[test]
fn name_identity_is_mechanical_after_trimming() {
    let spec = fixture_spec();
    let corpus = corpus(&[
        r#"{"id": "trim-happy", "setup": {"state": "  confirmed  "}, "trace": [{"action": "  reopen  "}]}"#,
    ]);
    let records = classify_traces(&spec, &corpus, false);
    assert_eq!(records[0].verdict, Verdict::Permitted);
}

// ---------------------------------------------------------------------------
// RO5U review pins — corpus refusal at step level, foreign citations
// ---------------------------------------------------------------------------

/// Unknown fields are refused with a remediation hint, never silently
/// ignored — at step level too, not just the scenario envelope (design
/// D3: envelope discipline).
#[test]
fn unknown_step_field_is_refused_with_hint() {
    let err = parse_corpus(
        r#"{"id": "x", "setup": {"state": "held"}, "trace": [{"action": "reopen", "expects": "y"}]}"#,
    )
    .expect_err("an unknown step field must be refused, never ignored");
    assert!(
        err.message.contains("unknown field"),
        "names the offense: {}",
        err.message
    );
    assert!(
        err.message.contains("expects"),
        "names the offending field: {}",
        err.message
    );
    assert!(
        err.message.contains("one JSON object per line"),
        "carries the remediation hint: {}",
        err.message
    );
}

/// A foreign citation in a guard (`other.spec.audited`) is an outbound
/// typed edge of record — it must NOT resolve to a local claim by suffix
/// identity. conform classifies THIS file's claims only (OQ1: one file
/// per invocation), so the trace is Model-explainable → permitted.
#[test]
fn foreign_citation_never_resolves_to_a_local_claim() {
    let spec = fixture_spec();
    let corpus = corpus(&[
        r#"{"id": "crosscheck", "setup": {"state": "held"}, "trace": [{"action": "crosscheck"}]}"#,
    ]);
    let records = classify_traces(&spec, &corpus, false);
    assert_eq!(
        records[0].verdict,
        Verdict::Permitted,
        "reason: {}",
        records[0].reason
    );
    assert!(
        !records[0].reason.contains("audited"),
        "a foreign citation must not misattribute the local prose claim: {}",
        records[0].reason
    );
}

// ---------------------------------------------------------------------------
// f4sx item 2 — prose-only evidence degrades to the ADVISORY tier
// ---------------------------------------------------------------------------
//
// The closed verdict vocabulary is unchanged: prose-only covering evidence
// is still `Unknown` (prose is not interpretable — it neither permits nor
// forbids). What changes is its TIER. The record carries an explicit
// `advisory: true` marker and the report counts it in a separate
// `advisory_counts` map keyed by the verdict's snake_case spelling,
// EXCLUDED from the hard `verdict_counts`. Advisory unknowns therefore
// never read as a contradiction and never enter the
// `forbidden + unsupported` predicate `cmd_conform` uses to choose exit 1.
//
// Current behavior these tests pin against: every prose-only trace lands in
// `verdict_counts["unknown"]` as if it were a hard verdict, and no advisory
// marker or separate count exists anywhere in the envelope — so a
// prose-heavy corpus' summary reads as if it carried unknowns/disagreements.

/// Prose-only covering evidence → `Unknown` (vocabulary kept), marked
/// `advisory: true` — advisory-tier, never a hard unknown, never a
/// contradiction.
#[test]
fn prose_only_covering_claim_is_advisory_not_a_hard_unknown() {
    let spec = fixture_spec();
    let traces = corpus(&[
        r#"{"id": "audit-prose", "setup": {"state": "held"}, "trace": [{"action": "audit"}]}"#,
    ]);
    let records = classify_traces(&spec, &traces, false);
    assert_eq!(records.len(), 1);
    let r = &records[0];
    assert_eq!(
        r.verdict,
        Verdict::Unknown,
        "the verdict vocabulary is unchanged: prose evidence is not interpretable"
    );
    assert!(
        r.reason.contains("prose_claim_not_interpretable"),
        "reason names the prose rule: {}",
        r.reason
    );
    assert!(
        r.reason.contains("audited"),
        "reason names the prose claim: {}",
        r.reason
    );
    assert!(
        r.advisory,
        "prose-only evidence must be advisory-tier, not a hard unknown: {r:?}"
    );
    assert_ne!(
        r.verdict,
        Verdict::Forbidden,
        "advisory evidence is never a contradiction"
    );
}

/// A prose-only corpus is advisory-only: the report counts it in
/// `advisory_counts` (a separate envelope field), excludes it from the
/// hard `verdict_counts`, and the `forbidden + unsupported` predicate
/// `cmd_conform` uses for exit 1 does not fire.
#[test]
fn prose_advisory_never_counts_as_a_failing_verdict() {
    let spec = fixture_spec();
    let lines =
        [r#"{"id": "audit-prose", "setup": {"state": "held"}, "trace": [{"action": "audit"}]}"#];
    let bytes = lines.join("\n");
    let traces = corpus(&lines);
    let report = build_report(&spec, bytes.as_bytes(), &traces, false);

    assert_eq!(
        report.advisory_counts.get("unknown"),
        Some(&1),
        "the advisory tier is counted separately by verdict spelling: {:?}",
        report.advisory_counts
    );
    assert_eq!(
        report.verdict_counts.get("unknown"),
        None,
        "advisory unknowns must not pollute the hard verdict counts: {:?}",
        report.verdict_counts
    );
    assert_eq!(
        report.verdict_counts.get("forbidden"),
        None,
        "an advisory prose trace is never counted as a contradiction"
    );
    let gating: usize = ["forbidden", "unsupported"]
        .iter()
        .map(|key| report.verdict_counts.get(*key).copied().unwrap_or(0))
        .sum();
    assert_eq!(
        gating, 0,
        "advisory-only corpora never force a failing run summary"
    );

    // The distinction rides the ENVELOPE (data), not just Rust fields.
    let json = serde_json::to_value(&report).expect("report serializes");
    assert_eq!(
        json["records"][0]["advisory"],
        serde_json::json!(true),
        "the record carries the advisory marker: {json}"
    );
    assert_eq!(
        json["advisory_counts"]["unknown"],
        serde_json::json!(1),
        "the report carries the separate advisory count: {json}"
    );

    // The human summary names the advisory tier instead of hiding it in
    // the hard verdict counts.
    let human = human_view(&report);
    assert!(
        human.contains("advisory"),
        "the human summary separates advisory from gating verdicts: {human}"
    );
}

/// An executable claim that genuinely disagrees still contradicts
/// (`forbidden`, not advisory) and still enters the failing predicate.
#[test]
fn executable_contradiction_still_counts_as_a_failing_verdict() {
    let spec = fixture_spec();
    let lines = [
        r#"{"id": "wrong-state-reopen", "setup": {"state": "held"}, "trace": [{"action": "reopen"}]}"#,
    ];
    let bytes = lines.join("\n");
    let traces = corpus(&lines);
    let report = build_report(&spec, bytes.as_bytes(), &traces, false);

    let r = report
        .records
        .iter()
        .find(|r| r.scenario_id == "wrong-state-reopen")
        .expect("the contradiction record exists");
    assert_eq!(r.verdict, Verdict::Forbidden);
    assert!(
        r.reason.contains("contradiction_forbidden_any_mode") && r.reason.contains("locked"),
        "the contradiction still names the executable claim: {}",
        r.reason
    );
    assert!(
        !r.advisory,
        "an executable contradiction is gating, never advisory: {r:?}"
    );
    assert_eq!(
        report.verdict_counts.get("forbidden"),
        Some(&1),
        "the contradiction is counted in the hard verdict counts: {:?}",
        report.verdict_counts
    );
    assert_eq!(
        report.advisory_counts.get("forbidden"),
        None,
        "a contradiction never lands in the advisory count"
    );
    let gating: usize = ["forbidden", "unsupported"]
        .iter()
        .map(|key| report.verdict_counts.get(*key).copied().unwrap_or(0))
        .sum();
    assert!(
        gating > 0,
        "a genuine executable disagreement still fails the run"
    );
}
