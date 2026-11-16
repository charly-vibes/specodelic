//! Purpose: the verify-runner seams of specodelic-u7kc — the
//! tool-dependency seam (a compiled `**rust:**` fragment may call the
//! real tool), the property fixture API (a generator name may materialize
//! a concrete spec), and the properties-only scope verdict (a scoped CI
//! gate may read the properties verdict without the model stage hiding
//! it). Responsibilities: prove each seam against the REAL tool — the
//! real scratch-crate runner, the real fixture registry, the real verdict
//! mapping — with no test doubles and no re-implemented checkers. The
//! artifact-routing seam (specodelic-1ey4) is pinned the same way: the
//! paths verify reads are exactly the ones `citation_corpus::artifact_dir`
//! routes, never a flat re-derivation that a same-stem file can collapse.
//! Rationale: the lf3 scoped run measured 0/537 corpus property rows
//! honestly expressible because the scratch crate could not see the tool
//! (M1), a named generator yielded its own name (M2), and the verdict
//! hid the properties stage behind the model stage (M3'); these tests
//! are the executable proof that each seam is now reachable.

use specodelic::compile::{artifact_stem, properties_to_proptest};
use specodelic::spec::parse_str;
use specodelic::verify::{
    CargoRunner, ModelGateState, PropsGateState, VerifyScope, evaluate_model_gate,
    evaluate_properties_gate, properties_only_verdict, verdict, verdict_in_scope,
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

// ---- specodelic-1ey4: the artifact-routing seam -------------------------
//
// verify used to resolve flat `<stem>_props.rs` / `<stem>.tla` /
// `<stem>.check.json` paths under the out-dir, while the orchestrate
// compile stage and the model-check backend pass already resolve through
// the ONE shared routing decision (`citation_corpus::artifact_dir`,
// collide-only: duplicate stems route under `out_dir/<stem>/<encoded
// source path>`). `artifact_paths(out_dir, stem)` cannot route at all —
// it never sees the corpus — so the seam is observed where verify's
// resolution becomes visible: the model gate's unresolved reason names the
// exact path the gate tried to read. The test therefore survives the
// implementation widening `artifact_paths`'s inputs; it pins the routing
// decision, not a signature.

/// A path-bearing spec whose artifact stem is its file stem — the identity
/// `compile::artifact_stem` and `citation_corpus::artifact_dir` key on. The
/// file need not exist: routing is keyed on the path, never on filesystem
/// discovery.
fn path_bearing_spec(id: &str, path: std::path::PathBuf) -> specodelic::spec::Spec {
    let text = format!(
        "---\nid: {id}\nkind: intent\nstatement: \"THE {id} SHALL read only its own routed artifacts.\"\n---\n"
    );
    let mut spec = parse_str(&text).unwrap_p();
    spec.path = Some(path);
    spec
}

/// The unresolved model gate's reason — the one place the gate names the
/// artifact path it actually tried to read.
fn model_gate_detail(
    spec: &specodelic::spec::Spec,
    corpus: &[specodelic::spec::Spec],
    out_dir: &std::path::Path,
) -> String {
    match evaluate_model_gate(spec, corpus, out_dir, &artifact_stem(spec)) {
        ModelGateState::Missing { detail } | ModelGateState::Stale { detail } => detail,
        other => panic!(
            "expected an unresolved model gate for {}; got {other:?}",
            spec.intent.id
        ),
    }
}

/// A stored report the gate accepts as current-schema, so evaluation
/// proceeds to the per-spec `.tla` the scope digest recomputes. The digest
/// itself is deliberately unmatched: the gate stops at the first
/// unreadable module and names it, which is the resolution under test.
const CURRENT_SCHEMA_REPORT: &str = r#"{
  "backend": {"engine": "native", "version": "test"},
  "bound": {"max_depth": 1},
  "outcome": "exploration_only",
  "invariants_checked": [],
  "artifact_sha256": "00",
  "states_explored": 1,
  "claim_schema_version": 1,
  "scope_sha256": "stale"
}"#;

#[test]
fn verify_stage_artifact_paths_route_duplicate_stems_per_file() {
    // specodelic-1ey4, meter 1: a duplicate-stem corpus routed through
    // citation_corpus::artifact_dir must be read per file — no cross-reads.
    // RED today: the gate resolves the flat `<stem>.check.json` under the
    // out-dir, so the first assertion below names that flat path instead of
    // this file's routed report.
    let td = tempfile::tempdir().unwrap_p();
    let out_dir = td.path().join("out");
    let corpus = vec![
        path_bearing_spec("alpha.pipeline", td.path().join("corpus/alpha/pipeline.md")),
        path_bearing_spec("beta.pipeline", td.path().join("corpus/beta/pipeline.md")),
    ];
    let stem = artifact_stem(&corpus[0]);
    assert_eq!(
        stem,
        artifact_stem(&corpus[1]),
        "the fixture must collide on one artifact stem"
    );
    let out = out_dir.to_str().unwrap_p();
    let routed = [
        specodelic::citation_corpus::artifact_dir(&corpus[0], &corpus, out),
        specodelic::citation_corpus::artifact_dir(&corpus[1], &corpus, out),
    ];
    assert_ne!(
        routed[0], routed[1],
        "duplicate stems must route to distinct dirs"
    );
    for dir in &routed {
        assert!(
            dir.starts_with(&out_dir),
            "routing stays under out_dir: {}",
            dir.display()
        );
    }
    let flat_check = out_dir.join(format!("{stem}.check.json"));
    let flat_tla = out_dir.join(format!("{stem}.tla"));

    // 1. Report resolution is per file: with no report on disk, each gate
    // names ITS OWN routed report — never the flat path that a same-stem
    // sibling would collapse onto.
    for (i, spec) in corpus.iter().enumerate() {
        let detail = model_gate_detail(spec, &corpus, &out_dir);
        let own_report = routed[i].join(format!("{stem}.check.json"));
        assert!(
            !detail.contains(flat_check.to_str().unwrap_p()),
            "{} must not read the flat report {}: {detail}",
            spec.intent.id,
            flat_check.display()
        );
        assert!(
            detail.contains(own_report.to_str().unwrap_p()),
            "{} must resolve its report at {}; detail: {detail}",
            spec.intent.id,
            own_report.display()
        );
    }

    // 2. The scope digest recomputes every corpus module. With the reports
    // present and no modules on disk, the gate must name the first ROUTED
    // module — the flat `<stem>.tla` is exactly the cross-read yl39 removed
    // from the compile stage and the backend pass.
    for dir in &routed {
        std::fs::create_dir_all(dir).unwrap_p();
        std::fs::write(
            dir.join(format!("{stem}.check.json")),
            CURRENT_SCHEMA_REPORT,
        )
        .unwrap_p();
    }
    let detail = model_gate_detail(&corpus[0], &corpus, &out_dir);
    let first_module = routed[0].join(format!("{stem}.tla"));
    assert!(
        !detail.contains(flat_tla.to_str().unwrap_p()),
        "the scope digest must not read the flat module {}: {detail}",
        flat_tla.display()
    );
    assert!(
        detail.contains(first_module.to_str().unwrap_p()),
        "the scope digest must resolve its module at {}; detail: {detail}",
        first_module.display()
    );
}

#[test]
fn verify_stage_artifact_paths_stay_flat_for_unique_stems() {
    // specodelic-1ey4, meter 2 (no-regression): a corpus-unique stem keeps
    // the flat layout, byte-identical to today's behavior.
    let td = tempfile::tempdir().unwrap_p();
    let out_dir = td.path().join("out");
    let spec = path_bearing_spec("unique.routing", td.path().join("corpus/unique.md"));
    let corpus = [spec];
    let routed =
        specodelic::citation_corpus::artifact_dir(&corpus[0], &corpus, out_dir.to_str().unwrap_p());
    assert_eq!(
        routed, out_dir,
        "a corpus-unique stem must keep the flat out-dir"
    );
    let expected = out_dir.join(format!("{}.check.json", artifact_stem(&corpus[0])));
    let detail = model_gate_detail(&corpus[0], &corpus, &out_dir);
    assert!(
        detail.contains(expected.to_str().unwrap_p()),
        "a unique stem must resolve the flat report {}; detail: {detail}",
        expected.display()
    );
}
