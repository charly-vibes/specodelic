//! Purpose: the verify-runner seams of specodelic-u7kc — the
//! tool-dependency seam (a compiled `**rust:**` fragment may call the
//! real tool), the property fixture API (a generator name may materialize
//! a concrete spec), and the properties-only scope verdict (a scoped CI
//! gate may read the properties verdict without the model stage hiding
//! it). Responsibilities: prove each seam against the REAL tool — the
//! real scratch-crate runner, the real fixture registry, the real verdict
//! mapping — with no test doubles and no re-implemented checkers.
//! Rationale: the lf3 scoped run measured 0/537 corpus property rows
//! honestly expressible because the scratch crate could not see the tool
//! (M1), a named generator yielded its own name (M2), and the verdict
//! hid the properties stage behind the model stage (M3'); these tests
//! are the executable proof that each seam is now reachable.

use specodelic::compile::{artifact_stem, properties_to_proptest};
use specodelic::spec::parse_str;
use specodelic::verify::{
    CargoRunner, ModelGateState, PropsGateState, VerifyScope, evaluate_properties_gate,
    properties_only_verdict, verdict, verdict_in_scope,
};

mod common;
use common::UnwrapPanic;

/// A spec whose single property row opts into an executable fragment that
/// calls the REAL linter: the fragment parses an inline spec and asks
/// the real `lint_corpus` whether the real rule `linter.model_present`
/// fired (the inline spec declares no Model section).
///
/// The generator (`sample_row()`) is deliberately NOT a fixture name: this
/// test is about the tool-dependency seam alone — the fixture seam has
/// its own test below.
const TOOL_CALLING_SPEC: &str = r#"---
id: seamdemo
kind: intent
statement: "THE seamdemo SHALL call the real tool from a compiled fragment."
---

## Constraints

| id | kind | expr | traces_to |
|----|------|------|------------|
| c1 | invariant | `holds` | [[seamdemo]] |

## Model

### States

- s1
- s2

### Transitions

| id | from | to | guard |
|----|------|----|-------|
| t | s1 | s2 | [[seamdemo.c1]] |

## Properties

| id | kind | derives_from | generator | predicate |
|----|------|--------------|-----------|------------|
| p_tool_call | unit | [[seamdemo.c1]] | `sample_row()` | `**rust:** specodelic::lint::lint_corpus(&[specodelic::spec::parse_str("---\nid: inl\nkind: intent\nstatement: \"THE inl SHALL be linted.\"\n---\n").expect("the inline spec parses")]).issues.iter().any(|i| i.rule_id == "linter.model_present")` |
"#;

/// Compile the spec, stage its artifact, and run the real properties
/// gate over it — the seam's end-to-end path.
fn run_properties_gate(spec_text: &str) -> specodelic::verify::PropertiesGate {
    let spec = parse_str(spec_text).unwrap_p();
    let dir = tempfile::tempdir().unwrap_p();
    let stem = artifact_stem(&spec);
    let artifact = dir.path().join(format!("{stem}_props.rs"));
    std::fs::write(&artifact, properties_to_proptest(&spec)).unwrap_p();
    evaluate_properties_gate(&spec, dir.path(), &CargoRunner::default())
}

#[test]
fn scratch_artifact_calling_the_tool_compiles_and_executes() {
    // P1 — the tool-dependency seam: the scratch crate the runner
    // materializes must be able to depend on the specodelic lib, so a
    // compiled fragment can call the REAL linter.
    //
    // RED (before the seam): the scratch manifest declares `proptest`
    // only, so the artifact fails to build with "unresolved crate
    // `specodelic`" and the gate reports `properties_uncompilable`.
    // GREEN (after the seam): the artifact builds, the fragment runs the
    // real linter, the real rule id is observed, and the block passes.
    let gate = run_properties_gate(TOOL_CALLING_SPEC);
    assert!(
        matches!(gate.state, PropsGateState::Pass),
        "the tool-calling fragment must compile and pass; state: {:?}, blocks: {:?}",
        gate.state,
        gate.blocks
    );
}

/// A spec whose generator cell names a FIXTURE
/// (`spec_file_with_undeclared_transition_target`) and whose predicate
/// fragment asserts the REAL linter's verdict on the materialized
/// fixture — the expected rule id must be reported.
const FIXTURE_CALLING_SPEC: &str = r#"---
id: seamfixture
kind: intent
statement: "THE seamfixture SHALL assert the real linter's verdict on a materialized fixture."
---

## Constraints

| id | kind | expr | traces_to |
|----|------|------|------------|
| c1 | invariant | `holds` | [[seamfixture]] |

## Model

### States

- s1
- s2

### Transitions

| id | from | to | guard |
|----|------|----|-------|
| t | s1 | s2 | [[seamfixture.c1]] |

## Properties

| id | kind | derives_from | generator | predicate |
|----|------|--------------|-----------|------------|
| p_fixture_call | unit | [[seamfixture.c1]] | `spec_file_with_undeclared_transition_target()` | `**rust:** specodelic::lint::lint_corpus(&[specodelic::spec::parse_str(&v0).expect("the materialized fixture parses")]).issues.iter().any(|i| i.rule_id == "linter.every_transition_valid")` |
"#;

#[test]
fn fixture_named_generator_materializes_a_concrete_spec() {
    // P2 — the fixture API: a generator name that names a fixture
    // materializes the fixture's concrete spec text (not the
    // generator's own name), and the REAL linter judges it.
    let gate = run_properties_gate(FIXTURE_CALLING_SPEC);
    assert!(
        matches!(gate.state, PropsGateState::Pass),
        "the fixture-materializing fragment must run against the real linter; state: {:?}, blocks: {:?}",
        gate.state,
        gate.blocks
    );
}

#[test]
fn fixture_registry_resolves_a_concrete_parseable_spec() {
    // P2 — the registry itself: every fixture name resolves to a
    // concrete spec text the REAL parser accepts, and the fixture's
    // text is a real spec (not a placeholder).
    let names = specodelic::fixtures::names();
    assert!(
        names.contains(&"property_row_with_one_property"),
        "the registry must carry the demo fixture; names: {names:?}"
    );
    for name in names {
        let fixture = specodelic::fixtures::lookup(name).unwrap_p();
        let text = specodelic::fixtures::materialize(name).unwrap_p();
        assert_eq!(
            text, fixture.text,
            "materialize must yield the fixture text"
        );
        let spec = parse_str(&text).unwrap_p();
        assert!(
            !spec.properties.is_empty(),
            "the fixture `{name}` must carry a Property row"
        );
    }
}

#[test]
fn emitted_strategy_materializes_a_fixture_generator() {
    // P2 — the emission: a fixture-named generator yields the fixture
    // through the registry; every other generator keeps the
    // placeholder emission byte-for-byte.
    let spec = parse_str(FIXTURE_CALLING_SPEC).unwrap_p();
    let artifact = properties_to_proptest(&spec);
    assert!(
        artifact.contains(
            "specodelic::fixtures::materialize(\"spec_file_with_undeclared_transition_target\")"
        ),
        "the fixture-named strategy must materialize through the registry; artifact:\n{artifact}"
    );
    assert!(
        !artifact.contains("Just(\"spec_file_with_undeclared_transition_target\")"),
        "a fixture-named generator must not yield its own name; artifact:\n{artifact}"
    );

    let plain = parse_str(TOOL_CALLING_SPEC).unwrap_p();
    let plain_artifact = properties_to_proptest(&plain);
    assert!(
        plain_artifact.contains("Just(\"sample_row\".into())"),
        "a non-fixture generator keeps the placeholder emission; artifact:\n{plain_artifact}"
    );
}

#[test]
fn properties_scope_reports_failed_when_a_block_failed() {
    // P3 (a) — the properties-only scope verdict reports
    // `properties_failed` exactly when a compiled block failed.
    let failed =
        properties_only_verdict(&PropsGateState::Failed("1 of 2 blocks failed".to_string()));
    assert_eq!(failed.status, "properties_failed");
    assert_eq!(
        properties_only_verdict(&PropsGateState::Pass).status,
        "properties_passed"
    );
}

#[test]
fn properties_scope_ignores_a_missing_or_stale_model_run() {
    // P3 (b) — a missing or stale model run never produces
    // `properties_failed`: the scoped verdict does not read the model
    // gate at all, and the full verdict keeps its own precedence.
    for model in [
        ModelGateState::Missing {
            detail: "no report".to_string(),
        },
        ModelGateState::Stale {
            detail: "not current".to_string(),
        },
    ] {
        assert_ne!(
            properties_only_verdict(&PropsGateState::Pass).status,
            "properties_failed",
            "a skipped or stale model stage must not read as a properties failure"
        );
        assert_ne!(
            verdict(&PropsGateState::Pass, &model).status,
            "properties_failed",
            "the full verdict keeps its own precedence for a skipped model stage"
        );
    }
}

#[test]
fn properties_scope_pass_is_never_verified() {
    // P3 (c) — the scoped verdict never reports `verified`, and the full
    // verdict's both-gates-required semantics are unchanged.
    assert_eq!(
        properties_only_verdict(&PropsGateState::Pass).status,
        "properties_passed"
    );
    assert_ne!(
        properties_only_verdict(&PropsGateState::Pass).status,
        "verified",
        "the properties-only scope must never report verified"
    );
    assert_eq!(
        verdict(&PropsGateState::Pass, &ModelGateState::Clean).status,
        "verified"
    );
    assert_eq!(
        verdict(
            &PropsGateState::Pass,
            &ModelGateState::Missing {
                detail: "no report".to_string()
            }
        )
        .status,
        "missing_model_run"
    );
    assert_eq!(
        verdict_in_scope(
            &PropsGateState::Pass,
            &ModelGateState::Clean,
            VerifyScope::BothGates
        )
        .status,
        "verified"
    );
    assert_eq!(
        verdict_in_scope(
            &PropsGateState::Pass,
            &ModelGateState::Clean,
            VerifyScope::PropertiesOnly
        )
        .status,
        "properties_passed"
    );
}

/// The corpus's own compile property row, migrated to a real
/// tool-calling fragment — the demonstration that a SHIPPED corpus row
/// executes against the real compiler (and not only against a
/// hand-written spec fixture).
#[test]
fn corpus_property_row_executes_against_the_real_compiler() {
    let path =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("openspec/specs/compile/spec.md");
    let text = std::fs::read_to_string(&path).unwrap_p();
    let spec = parse_str(&text).unwrap_p();
    let dir = tempfile::tempdir().unwrap_p();
    let stem = artifact_stem(&spec);
    std::fs::write(
        dir.path().join(format!("{stem}_props.rs")),
        properties_to_proptest(&spec),
    )
    .unwrap_p();
    let gate = evaluate_properties_gate(&spec, dir.path(), &CargoRunner::default());
    let block = gate
        .blocks
        .iter()
        .find(|b| b.id == "proptest_block_per_property")
        .expect("the corpus carries the migrated property row");
    assert!(
        block.passed,
        "the migrated corpus row must execute against the real compiler; detail: {:?}",
        block.detail
    );
}
