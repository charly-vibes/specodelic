//! Claim-coverage section tests (openspec/changes/add-check-claim-coverage,
//! scope A): the placement rule (D8), the class fold (D9), the
//! materialization site (D10), the scope statement, the residual bucket,
//! and the non-gating promise.
//!
//! Purpose: pin the section's semantics at the library level, over
//! fixtures — a spec file with declared claims and a report with rows —
//! never over the corpus (the corpus is the self-hosting exercise, and
//! `spk check` cannot judge a corpus file yet: specodelic-qbox).
//! Responsibilities: one test per decided rule of the change's delta —
//! placement through a property's `derives_from` hop, direct naming,
//! residuals, the strongest-class fold, the file-qualified claim id, the
//! machine-readable scope field, the envelope's shape, the human view's
//! verdict line, the closed vocabulary, and the byte-stability of two
//! identical runs.
//! Rationale: the section's whole value is that a reader can count the
//! gap; a section that miscounts, abbreviates or gates would be worse
//! than no section. The rules are pinned before the implementation so the
//! implementation cannot invent its own placement or fold.

use std::collections::BTreeMap;

use specodelic::audit::{AuditClass, AuditRecord, AuditReport, RungAggregate};
use specodelic::check;
use specodelic::claim_coverage::{self, ClaimCoverage};
use specodelic::conform::Verdict;
use specodelic::spec::{Spec, parse_str};

/// A lint-shaped fixture: two declared invariant claims and one property
/// deriving from the first of them. The second claim has no property and
/// no row — the shape the `unchecked` class exists for.
const FIXTURE: &str = r#"---
id: demo.claim_coverage
kind: intent
statement: "THE system SHALL hold"
---

## Constraints

| id | kind | expr | traces_to |
|----|------|------|-----------|
| alpha | invariant | `a holds` | [[demo.claim_coverage]] |
| beta | invariant | `b holds` | [[demo.claim_coverage]] |

## Model

### States

- s1

### Transitions

| id | from | to | guard |
|----|------|----|-------|
| t | s1 | s1 | `g` |

## Properties

| id | kind | derives_from | generator | predicate |
|----|------|--------------|-----------|-----------|
| p_alpha | unit | [[demo.claim_coverage.alpha]] | `g()` | `x` |
"#;

fn spec() -> Spec {
    parse_str(FIXTURE).expect("the fixture spec parses")
}

/// A report carrying exactly the given rows.
fn report(rows: Vec<AuditRecord>) -> AuditReport {
    AuditReport {
        report_schema_version: 1,
        evidence_scope: "the fixture's evidence scope".to_string(),
        scope_sha256: "0".repeat(64),
        expectation_declared: false,
        verdict_counts: BTreeMap::new(),
        class_counts: BTreeMap::new(),
        ladder_rung: RungAggregate::default(),
        records: rows,
    }
}

/// One evidence row: its key, its class, and the ids it names.
fn row(key: &str, class: AuditClass, names: &[&str]) -> AuditRecord {
    let verdict = match class {
        AuditClass::Current | AuditClass::New => Verdict::Permitted,
        AuditClass::Stale => Verdict::Underspecified,
        AuditClass::Contradicted => Verdict::Forbidden,
        AuditClass::BindsNothing => Verdict::Agree,
        AuditClass::Unchecked => Verdict::Underspecified,
    };
    AuditRecord {
        scenario_id: key.to_string(),
        class,
        conform_verdict: verdict,
        reason: "a fixture row".to_string(),
        evaluated_claim_ids: names.iter().map(|n| n.to_string()).collect(),
        closed_world: false,
    }
}

/// The claim ids the section lists, in order.
fn claim_ids(coverage: &ClaimCoverage) -> Vec<&str> {
    coverage
        .claims
        .iter()
        .map(|c| c.claim_id.as_str())
        .collect()
}

/// The class the section gives a claim.
fn class_of(coverage: &ClaimCoverage, claim_id: &str) -> Option<AuditClass> {
    coverage
        .claims
        .iter()
        .find(|c| c.claim_id == claim_id)
        .map(|c| c.class)
}

/// Placement through a property's `derives_from` hop: a row naming the
/// property places the claim the property derives from — and only that
/// claim.
#[test]
fn a_property_row_places_the_claim_it_derives_from() {
    let spec = spec();
    let rows = vec![row(
        "verify:prop:p_alpha",
        AuditClass::Current,
        &["p_alpha"],
    )];
    let coverage = claim_coverage::coverage(&spec, &report(rows));

    assert_eq!(
        class_of(&coverage, "demo.claim_coverage.alpha"),
        Some(AuditClass::Current),
        "the property's claim is placed by the property row"
    );
    assert_eq!(
        class_of(&coverage, "demo.claim_coverage.beta"),
        Some(AuditClass::Unchecked),
        "the other declared claim is placed by no row"
    );
    assert!(
        coverage.residuals.is_empty(),
        "both rows placed a claim: {:?}",
        coverage.residuals
    );
}

/// Direct naming: a model row's key names the claim it contradicts, so the
/// claim is placed by that row.
#[test]
fn a_model_row_places_the_claim_it_names() {
    let spec = spec();
    let rows = vec![
        row("verify:prop:p_alpha", AuditClass::Current, &["p_alpha"]),
        row("verify:model:beta", AuditClass::Contradicted, &[]),
    ];
    let coverage = claim_coverage::coverage(&spec, &report(rows));

    assert_eq!(
        class_of(&coverage, "demo.claim_coverage.beta"),
        Some(AuditClass::Contradicted),
        "the model row names the claim directly"
    );
}

/// Residuals: a row that places no claim is accounted for, never dropped,
/// and no claim carries it.
#[test]
fn a_row_that_places_no_claim_lands_in_the_residual_bucket() {
    let spec = spec();
    let rows = vec![
        row("verify:prop:p_alpha", AuditClass::Current, &["p_alpha"]),
        row("verify:model", AuditClass::Unchecked, &[]),
    ];
    let coverage = claim_coverage::coverage(&spec, &report(rows));

    assert_eq!(
        coverage
            .residuals
            .iter()
            .map(|r| r.row_key.as_str())
            .collect::<Vec<_>>(),
        vec!["verify:model"],
        "the model-level row is accounted for: {:?}",
        coverage.residuals
    );
    assert_eq!(
        class_of(&coverage, "demo.claim_coverage.beta"),
        Some(AuditClass::Unchecked),
        "the residual row places no claim"
    );
}

/// The fold: the strongest class among the placing rows wins, a vacuous
/// placement is `binds_nothing` (not `unchecked`), and a claim placed by
/// no row is `unchecked`.
#[test]
fn a_claim_carries_the_strongest_class_among_its_rows() {
    let spec = spec();
    let rows = vec![
        row("verify:prop:p_alpha", AuditClass::Current, &["p_alpha"]),
        row("verify:model:alpha", AuditClass::Contradicted, &[]),
        row("bind:beta", AuditClass::BindsNothing, &[]),
    ];
    let coverage = claim_coverage::coverage(&spec, &report(rows));

    assert_eq!(
        class_of(&coverage, "demo.claim_coverage.alpha"),
        Some(AuditClass::Contradicted),
        "a contradiction outranks a pass"
    );
    assert_eq!(
        class_of(&coverage, "demo.claim_coverage.beta"),
        Some(AuditClass::BindsNothing),
        "a vacuous placement is not an unchecked claim"
    );
}

/// The file-qualified claim id: two files declaring the same claim id keep
/// distinct entries, and a near-miss slug places nothing.
#[test]
fn claims_are_named_by_their_file_qualified_id() {
    let other = FIXTURE.replace("demo.claim_coverage", "demo.other");
    let other_spec = parse_str(&other).expect("the second fixture parses");
    let rows = vec![row(
        "verify:prop:p_alpha",
        AuditClass::Current,
        &["p_alpha"],
    )];

    let first = claim_coverage::coverage(&spec(), &report(rows.clone()));
    let second = claim_coverage::coverage(&other_spec, &report(rows));

    assert_eq!(
        claim_ids(&first),
        vec!["demo.claim_coverage.alpha", "demo.claim_coverage.beta"],
        "the first file's claims carry its own id"
    );
    assert_eq!(
        claim_ids(&second),
        vec!["demo.other.alpha", "demo.other.beta"],
        "the second file's claims carry its own id"
    );
}

/// A resemblance is not a placement: a row whose name merely resembles a
/// claim places no claim.
#[test]
fn a_near_miss_name_places_no_claim() {
    let spec = spec();
    let rows = vec![row(
        "verify:prop:alphaish",
        AuditClass::Current,
        &["alphaish"],
    )];
    let coverage = claim_coverage::coverage(&spec, &report(rows));

    assert_eq!(
        class_of(&coverage, "demo.claim_coverage.alpha"),
        Some(AuditClass::Unchecked),
        "a near-miss name places nothing"
    );
    assert_eq!(
        coverage.residuals.len(),
        1,
        "the near-miss row is a residual: {:?}",
        coverage.residuals
    );
}

/// The scope statement is machine-readable: the section carries a scope
/// id a consumer can compare, plus the human statement.
#[test]
fn the_section_carries_a_machine_readable_scope() {
    let spec = spec();
    let coverage = claim_coverage::coverage(&spec, &report(vec![]));

    assert_eq!(
        coverage.scope.id, "this_run_producers",
        "the scope id names the run's own producers"
    );
    assert!(
        coverage.scope.statement.contains("this run"),
        "the statement says whose evidence the classes describe: {}",
        coverage.scope.statement
    );
}

/// The materialization site: the section rides the check envelope beside
/// the report, the audit report's field set is unchanged, and the audit
/// report still parses from the envelope's payload (a check report stays
/// usable as a `--before` prior).
#[test]
fn the_section_rides_the_check_envelope_beside_an_unchanged_report() {
    let spec = spec();
    let rows = vec![row(
        "verify:prop:p_alpha",
        AuditClass::Current,
        &["p_alpha"],
    )];
    let report = report(rows);
    let payload = check::payload(&spec, &report);

    let json = serde_json::to_value(&payload).expect("the payload serializes");
    let mut keys: Vec<&str> = json
        .as_object()
        .expect("the payload is an object")
        .keys()
        .map(String::as_str)
        .collect();
    keys.sort_unstable();

    let mut expected = vec![
        "report_schema_version",
        "evidence_scope",
        "scope_sha256",
        "expectation_declared",
        "verdict_counts",
        "class_counts",
        "ladder_rung",
        "records",
        "claim_coverage",
    ];
    expected.sort_unstable();
    assert_eq!(
        keys, expected,
        "the audit report's field set is unchanged and the section is its one sibling"
    );
    assert_eq!(
        json["claim_coverage"]["scope"]["id"],
        serde_json::json!("this_run_producers"),
        "the scope id is machine-readable: {json}"
    );

    let reparsed: AuditReport = serde_json::from_value(json.clone())
        .expect("the audit report still parses from the payload");
    assert_eq!(reparsed, report, "the audit report round-trips unchanged");
}

/// The human view: the coverage counts ride the verdict line — not a
/// trailing block a reader misses — and the section states its scope.
#[test]
fn the_human_view_carries_the_coverage_counts_on_the_verdict_line() {
    let spec = spec();
    let rows = vec![row(
        "verify:prop:p_alpha",
        AuditClass::Current,
        &["p_alpha"],
    )];
    let payload = check::payload(&spec, &report(rows));
    let human = check::human_view(&payload);

    let verdict_line = human.lines().next().unwrap_or_default();
    assert!(
        verdict_line.contains("claims:"),
        "the verdict line carries the coverage counts: {verdict_line}"
    );
    assert!(
        verdict_line.contains("unchecked 1"),
        "the unchecked count is on the verdict line: {verdict_line}"
    );
    assert!(
        human.contains("this run"),
        "the section states its scope: {human}"
    );
    assert!(
        human.contains("demo.claim_coverage.alpha"),
        "the section names the claims it lists: {human}"
    );
}

/// The closed vocabulary: the section introduces no new class value.
#[test]
fn the_section_uses_only_audits_closed_class_set() {
    let spec = spec();
    let rows = vec![
        row("verify:prop:p_alpha", AuditClass::Current, &["p_alpha"]),
        row("verify:model", AuditClass::Unchecked, &[]),
    ];
    let coverage = claim_coverage::coverage(&spec, &report(rows));

    const CLOSED: [&str; 6] = [
        "current",
        "stale",
        "unchecked",
        "contradicted",
        "binds_nothing",
        "new",
    ];
    for class in coverage
        .claims
        .iter()
        .map(|c| c.class.as_str())
        .chain(coverage.residuals.iter().map(|r| r.class.as_str()))
    {
        assert!(
            CLOSED.contains(&class),
            "`{class}` is in audit's closed set: {CLOSED:?}"
        );
    }
}

/// Byte-stability: the same inputs judged twice produce the same section,
/// in the same order, with the same classes.
#[test]
fn the_section_is_byte_identical_across_runs() {
    let spec = spec();
    let rows = vec![
        row("verify:prop:p_alpha", AuditClass::Current, &["p_alpha"]),
        row("verify:model:beta", AuditClass::Contradicted, &[]),
    ];
    let first = claim_coverage::coverage(&spec, &report(rows.clone()));
    let second = claim_coverage::coverage(&spec, &report(rows));

    assert_eq!(
        serde_json::to_string(&first).unwrap(),
        serde_json::to_string(&second).unwrap(),
        "the section is byte-identical across identical runs"
    );
    assert_eq!(
        claim_ids(&first),
        vec!["demo.claim_coverage.alpha", "demo.claim_coverage.beta"],
        "the claims are listed in the corpus's declaration order: {:?}",
        claim_ids(&first)
    );
}

/// Non-gating: a claim with no evidence row is `unchecked` and the run
/// still exits 0 — the section contributes counts, never an exit code.
#[test]
fn an_unchecked_claim_does_not_fail_the_run() {
    use specodelic::audit;

    let spec = spec();
    let rows = vec![row(
        "verify:prop:p_alpha",
        AuditClass::Current,
        &["p_alpha"],
    )];
    let report = audit::fold_rows(audit::build_report(&spec, b"", &[], false, false), rows);
    let coverage = claim_coverage::coverage(&spec, &report);

    assert_eq!(
        class_of(&coverage, "demo.claim_coverage.beta"),
        Some(AuditClass::Unchecked),
        "the unevidenced claim is listed as unchecked"
    );
    assert_eq!(
        audit::exit_code(&report, false),
        0,
        "the unchecked claim changes nothing about the exit code"
    );
    assert_eq!(
        coverage.class_counts.get("unchecked"),
        Some(&1),
        "the unchecked claim is counted: {:?}",
        coverage.class_counts
    );
}
