//! Phase-1 verdict-engine tests for `conform` (openspec/changes/add-conform,
//! tasks 1.1–1.4): the closed five-valued taxonomy over a fixture spec +
//! fixture JSONL corpus, evidence-class separation (design D2), and
//! taxonomy totality. Pure library — no CLI.

use specodelic::conform::{Verdict, classify_claims, classify_traces, parse_corpus};
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
| reopen | confirmed | held | [[demo.conform.locked]] |
| escalate | held | held | [[demo.conform.portable]] |
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
