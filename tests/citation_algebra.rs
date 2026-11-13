//! Guard citation semantics — slice 1 of add-min-expr-kernel
//! (specodelic-txo, tasks.md §2, design D3/CORR-002).
//!
//! Integration-scope tests for the citation algebra and the
//! status-bearing run report: the grammar parses over the closed
//! slice-1 set ([[id]], ∧, ¬; prose stays prose), composition is
//! Kleene three-valued (unknown absorbs, never coerces to pass),
//! undischargable citations report unknown, and RunReport persists
//! backend, bound, and per-invariant status honestly.
//!
//! These live out-of-module because src/model_check.rs is pinned by
//! the pretender file_lines ratchet (shrink-only, pretender.toml).

#[path = "common/mod.rs"]
mod common;
use std::collections::BTreeMap;

use common::UnwrapPanic;
use specodelic::model_check::{
    BACKEND_VERSION, Backend, Bound, CitationExpr, InvariantStatus, Outcome, RunReport,
    ThreeValued, evaluate_citation, parse_citation_expr,
};

#[test]
fn citation_expr_parses_cite_conjunction_negation_prose_stays_prose() {
    // Scenario: Citation algebra evaluates — [[a]] ∧ [[b]] and ¬[[a]]
    // parse into the closed citation grammar; a prose cell is not a
    // citation expression (pure widening, prose untouched).
    assert_eq!(
        parse_citation_expr("[[a]] ∧ [[b]]"),
        Some(CitationExpr::And(
            Box::new(CitationExpr::Cite("a".into())),
            Box::new(CitationExpr::Cite("b".into())),
        ))
    );
    assert_eq!(
        parse_citation_expr("¬[[a]]"),
        Some(CitationExpr::Not(Box::new(CitationExpr::Cite("a".into()))))
    );
    assert_eq!(
        parse_citation_expr("[[a]]"),
        Some(CitationExpr::Cite("a".into()))
    );
    assert!(
        parse_citation_expr("`x holds for every row`").is_none(),
        "prose expr cells are not citation expressions"
    );
}

#[test]
fn citation_conjunction_of_verified_discharges_verified() {
    // Scenario: Citation algebra evaluates — citations resolve against
    // compiled property outcomes and compose.
    let expr = parse_citation_expr("[[a]] ∧ [[b]]").unwrap_p();
    // A ∧ B verifies only when BOTH conjuncts verify — the composite
    // must be an And node over the two citations.
    assert!(matches!(
        &expr,
        specodelic::compile::CitationExpr::And(_, _)
    ));
    let outcomes = BTreeMap::from([
        ("a".to_string(), ThreeValued::Verified),
        ("b".to_string(), ThreeValued::Verified),
    ]);
    assert_eq!(evaluate_citation(&expr, &outcomes), ThreeValued::Verified);
}

#[test]
fn citation_conjunction_absorbs_unknown_never_coerces_to_pass() {
    // dl/1 Kleene absorb: one unknown disjunct makes the whole
    // conjunction unknown — never pass (spec.run_report_status).
    let expr = parse_citation_expr("[[a]] ∧ [[b]]").unwrap_p();
    let outcomes = BTreeMap::from([
        ("a".to_string(), ThreeValued::Verified),
        ("b".to_string(), ThreeValued::Unknown),
    ]);
    assert_eq!(evaluate_citation(&expr, &outcomes), ThreeValued::Unknown);
}

#[test]
fn counterexample_propagates_through_conjunction() {
    let expr = parse_citation_expr("[[a]] ∧ [[b]]").unwrap_p();
    let outcomes = BTreeMap::from([
        ("a".to_string(), ThreeValued::Verified),
        ("b".to_string(), ThreeValued::Counterexample),
    ]);
    assert_eq!(
        evaluate_citation(&expr, &outcomes),
        ThreeValued::Counterexample
    );
}

#[test]
fn undischargable_citation_reports_unknown_never_pass() {
    // Scenario: Undischargable citation is honest — a cited id with
    // no known outcome (prose property, nonexistent id) is unknown.
    let conj = parse_citation_expr("[[prose_prop]] ∧ [[no_such_id]]").unwrap_p();
    assert_eq!(
        evaluate_citation(&conj, &BTreeMap::new()),
        ThreeValued::Unknown
    );
    let bare = parse_citation_expr("[[nope]]").unwrap_p();
    assert_eq!(
        evaluate_citation(&bare, &BTreeMap::new()),
        ThreeValued::Unknown
    );
}

#[test]
fn malformed_bracket_in_citation_id_stays_prose() {
    // RO5U F-1 (slice-1 review): a citation id containing a bracket can
    // never match a real Constraints id (ids are filename stems) — the
    // cell is a malformed citation expression and stays prose, never a
    // silent citation upgrade that would evaluate unknown forever.
    assert!(parse_citation_expr("[[a]b]]").is_none());
    assert!(parse_citation_expr("[[a[b]]").is_none());
}

#[test]
fn citation_negation_flips_verified_counterexample_keeps_unknown() {
    let expr = parse_citation_expr("¬[[a]]").unwrap_p();
    let verified = BTreeMap::from([("a".to_string(), ThreeValued::Verified)]);
    assert_eq!(
        evaluate_citation(&expr, &verified),
        ThreeValued::Counterexample
    );
    let counterexample = BTreeMap::from([("a".to_string(), ThreeValued::Counterexample)]);
    assert_eq!(
        evaluate_citation(&expr, &counterexample),
        ThreeValued::Verified
    );
    let unknown = BTreeMap::from([("a".to_string(), ThreeValued::Unknown)]);
    assert_eq!(evaluate_citation(&expr, &unknown), ThreeValued::Unknown);
}

#[test]
fn report_persists_per_invariant_status_with_honest_unknown() {
    // Scenario: Report names its status — backend, bound, and
    // per-invariant status persist; unknown is persisted verbatim,
    // never coerced.
    let report = RunReport {
        backend: Backend {
            engine: "stateright".into(),
            version: BACKEND_VERSION.into(),
        },
        bound: Bound::default(),
        outcome: Outcome::NoCounterexample,
        invariants_checked: vec!["c_exec".into(), "c_cite".into()],
        invariant_statuses: vec![
            InvariantStatus {
                id: "c_exec".into(),
                status: ThreeValued::Verified,
            },
            InvariantStatus {
                id: "c_cite".into(),
                status: ThreeValued::Unknown,
            },
        ],
        violated_invariant_id: None,
        trace: None,
        artifact_sha256: "sha256".into(),
        states_explored: 3,
    };
    let json = serde_json::to_string(&report).unwrap_p();
    assert!(json.contains("\"status\":\"verified\""), "{}", json);
    assert!(json.contains("\"status\":\"unknown\""), "{}", json);
    let round: RunReport = serde_json::from_str(&json).unwrap_p();
    assert_eq!(round, report);
}

// ---- specodelic-9o1h: cross-evaluator composition in the resolver ----
// The citation algebra resolves a target against the same invocation's
// EXEC outcomes AND the corpus-static KERNEL claim statuses (E2 finding
// F2). These tests drive the resolver directly: the composition adds an
// evidence source, it never rewrites the other legs, and an empty
// kernel map behaves exactly as before the composition.

/// The cross-evaluator fixture: one exec row, one kernel row, and one
/// citation row over both.
const CROSS_EVALUATOR_SPEC: &str = "---\nid: cx.algebra\nkind: intent\nstatement: \"THE cx SHALL compose evaluators\"\n---\n\n## Constraints\n\n| id | kind | expr | traces_to |\n|----|------|------|-----------|\n| c1 | invariant | `**rust:** state != \"blackhole\"` | [[cx.algebra]] |\n| ka | invariant | `**kernel:** |State| == 2` | [[cx.algebra]] |\n| kc | invariant | `[[c1]] ∧ [[ka]]` | [[cx.algebra]] |\n\n## Model\n\n### States\n\n- s1\n- s2\n\n### Transitions\n\n| id | from | to | guard |\n|----|------|----|-------|\n| t | s1 | s2 | [[cx.algebra.c1]] |\n\n## Properties\n\n| id | kind | derives_from | generator | predicate |\n|----|------|--------------|-----------|-----------|\n| p_c1 | unit | [[cx.algebra.c1]] | `word()` | `**rust:** v0.len() >= 1` |\n| p_ka | unit | [[cx.algebra.ka]] | `word()` | `**rust:** v0.len() >= 1` |\n| p_kc | unit | [[cx.algebra.kc]] | `word()` | `**rust:** v0.len() >= 1` |\n";

/// The parsed cross-evaluator fixture.
fn cross_evaluator_spec() -> specodelic::spec::Spec {
    specodelic::spec::parse_str(CROSS_EVALUATOR_SPEC).unwrap_p()
}

/// The fixture's exec evidence: the c1 row verified by the backend.
fn exec_evidence() -> Vec<BTreeMap<String, ThreeValued>> {
    vec![BTreeMap::from([("c1".to_string(), ThreeValued::Verified)])]
}

/// The fixture's kernel evidence: the ka row evaluated by the kernel
/// evaluator over the same invocation.
fn kernel_evidence() -> Vec<BTreeMap<String, specodelic::citation_corpus::ClaimEvidence>> {
    let spec = cross_evaluator_spec();
    let claims = specodelic::kernel::evaluate_corpus_claims(
        std::slice::from_ref(&spec),
        specodelic::kernel::CorpusBackend::Native,
    );
    vec![specodelic::kernel::citation_evidence(&claims[0])]
}

/// The resolved status of the fixture's citation row `kc`.
fn resolved_citation(
    kernel_evidence: &[BTreeMap<String, specodelic::citation_corpus::ClaimEvidence>],
) -> specodelic::citation_corpus::ResolvedStatus {
    let spec = cross_evaluator_spec();
    let resolved = specodelic::citation_corpus::resolve_corpus_citations(
        std::slice::from_ref(&spec),
        &exec_evidence(),
        kernel_evidence,
    );
    resolved[0].iter().find(|r| r.id == "kc").unwrap_p().clone()
}

#[test]
fn kernel_evidence_composes_into_the_citation_target() {
    // The citation over a kernel-evaluated claim resolves against the
    // claim's same-invocation status — `[[exec]] ∧ [[kernel]]` verifies
    // when both legs do.
    let resolved = resolved_citation(&kernel_evidence());
    assert_eq!(resolved.id, "kc");
    assert_eq!(
        resolved.status,
        ThreeValued::Verified,
        "the composed citation verifies: {resolved:?}"
    );
    assert!(
        resolved.reason.is_none(),
        "a verified row carries no reason"
    );
}

#[test]
fn empty_kernel_evidence_behaves_exactly_as_before_the_composition() {
    // The refactor seam: an empty kernel map is the pre-composition
    // behavior — the kernel claim is not evidence, so the citation stays
    // unknown with the unchanged missing-target reason.
    let resolved = resolved_citation(&[BTreeMap::new()]);
    assert_eq!(resolved.status, ThreeValued::Unknown);
    let reason = resolved.reason.unwrap_p();
    assert!(
        reason.contains("missing target `ka`"),
        "the missing-target reason is unchanged: {reason}"
    );
}

#[test]
fn unknown_kernel_claim_composes_its_labeled_reason() {
    // Kleene absorb across evaluators: an undischargable kernel claim
    // keeps the citation unknown, and the composed reason NAMES the
    // kernel cause.
    let spec = cross_evaluator_spec();
    let unknown = specodelic::kernel::evaluate_corpus_claims(
        std::slice::from_ref(&spec),
        specodelic::kernel::CorpusBackend::Tlc,
    );
    let evidence = vec![specodelic::kernel::citation_evidence(&unknown[0])];
    let resolved = resolved_citation(&evidence);
    assert_eq!(resolved.status, ThreeValued::Unknown);
    let reason = resolved.reason.unwrap_p();
    assert!(
        reason.contains("kernel claim `ka` is unknown"),
        "the composed reason names the kernel claim: {reason}"
    );
    assert!(
        reason.contains("unsupported"),
        "the composed reason carries the kernel cause: {reason}"
    );
}

#[test]
fn counterexample_kernel_claim_propagates_to_the_citing_citation() {
    // Kleene dominance across evaluators: a refuted kernel claim makes
    // the citing citation a counterexample.
    let mut evidence = kernel_evidence();
    evidence[0].insert(
        "ka".to_string(),
        specodelic::citation_corpus::ClaimEvidence {
            status: ThreeValued::Counterexample,
            reason: Some("the claim evaluated to counterexample".to_string()),
        },
    );
    let resolved = resolved_citation(&evidence);
    assert_eq!(resolved.status, ThreeValued::Counterexample);
}
