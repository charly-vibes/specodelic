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

use std::collections::BTreeMap;

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
    let expr = parse_citation_expr("[[a]] ∧ [[b]]").expect("parses");
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
    let expr = parse_citation_expr("[[a]] ∧ [[b]]").expect("parses");
    let outcomes = BTreeMap::from([
        ("a".to_string(), ThreeValued::Verified),
        ("b".to_string(), ThreeValued::Unknown),
    ]);
    assert_eq!(evaluate_citation(&expr, &outcomes), ThreeValued::Unknown);
}

#[test]
fn counterexample_propagates_through_conjunction() {
    let expr = parse_citation_expr("[[a]] ∧ [[b]]").expect("parses");
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
    let conj = parse_citation_expr("[[prose_prop]] ∧ [[no_such_id]]").expect("parses");
    assert_eq!(
        evaluate_citation(&conj, &BTreeMap::new()),
        ThreeValued::Unknown
    );
    let bare = parse_citation_expr("[[nope]]").expect("parses");
    assert_eq!(
        evaluate_citation(&bare, &BTreeMap::new()),
        ThreeValued::Unknown
    );
}

#[test]
fn citation_negation_flips_verified_counterexample_keeps_unknown() {
    let expr = parse_citation_expr("¬[[a]]").expect("parses");
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
    let json = serde_json::to_string(&report).expect("serializes");
    assert!(json.contains("\"status\":\"verified\""), "{}", json);
    assert!(json.contains("\"status\":\"unknown\""), "{}", json);
    let round: RunReport = serde_json::from_str(&json).expect("deserializes");
    assert_eq!(round, report);
}
