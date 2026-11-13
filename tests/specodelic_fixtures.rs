//! The fixture harness for the `specodelic` and `kinds` corpus files'
//! property rows.
//!
//! Purpose: prove that every generator the two corpus files name
//! materializes a REAL input, and that each fixture really carries the
//! shape (or the defect) the property row's claim is about — a fixture
//! that only "looks right" would make the executable predicate vacuous.
//!
//! Responsibilities: check the corpus↔registry correspondence (every
//! generator name resolves), check that the fixtures are real spec texts
//! the real parser reads, and check that each defect fixture is really
//! detected by the real tool.
//!
//! Rationale: the executable predicates in `openspec/specs/specodelic/spec.md`
//! and `openspec/specs/kinds/spec.md` call these fixtures through
//! `specodelic::fixtures`; this harness is what keeps that seam honest —
//! a fixture that stopped carrying its defect would silently weaken the
//! property.

use specodelic::acset::instance::Instance;
use specodelic::acset::query;
use specodelic::fixtures::specs;
use specodelic::lint::lint_corpus;
use specodelic::spec::parse_str;

/// The corpus files under test.
const SPECODELIC: &str = include_str!("../openspec/specs/specodelic/spec.md");
const KINDS: &str = include_str!("../openspec/specs/kinds/spec.md");

/// The generator names a generator cell names — the compiler's own rule
/// (every `name(` occurrence), mirrored here so the harness checks the
/// same correspondence the compiler binds.
fn generator_names(cell: &str) -> Vec<String> {
    let re = regex::Regex::new(r"\b([a-zA-Z_][a-zA-Z0-9_]*)\s*\(").expect("static regex");
    let mut names: Vec<String> = vec![];
    for caps in re.captures_iter(cell) {
        let name = caps[1].to_string();
        if !names.contains(&name) {
            names.push(name);
        }
    }
    names
}

/// Every generator name the corpus files' MIGRATED property rows name.
///
/// A row is migrated when its predicate cell carries an executable
/// fragment (`**rust:**`, or a law case body `**name::**`) — the rows
/// that stayed on the honest `todo_predicate!` placeholder keep their
/// original generator cell, which is a description, not a fixture.
fn corpus_generators() -> Vec<String> {
    let mut names: Vec<String> = vec![];
    for corpus in [SPECODELIC, KINDS] {
        let spec = parse_str(corpus).expect("the corpus file parses");
        for row in &spec.properties {
            let predicate = row.cells.get("predicate").cloned().unwrap_or_default();
            if !predicate.contains("**rust:**") {
                continue;
            }
            let cell = row.cells.get("generator").cloned().unwrap_or_default();
            for name in generator_names(&cell) {
                if !names.contains(&name) {
                    names.push(name);
                }
            }
        }
    }
    names
}

/// The property rows that stayed on the honest `todo_predicate!`
/// placeholder — the rows whose claim has no honest tool seam.
fn unmigrated_rows() -> Vec<String> {
    let mut ids: Vec<String> = vec![];
    for corpus in [SPECODELIC, KINDS] {
        let spec = parse_str(corpus).expect("the corpus file parses");
        for row in &spec.properties {
            let predicate = row.cells.get("predicate").cloned().unwrap_or_default();
            if !predicate.contains("**rust:**") {
                ids.push(row.id.clone());
            }
        }
    }
    ids
}

/// The fixtures that are NOT spec texts: the rename target id (an id, not
/// a file) and the frontmatter defect input (which must NOT parse).
fn is_non_spec_fixture(name: &str) -> bool {
    name == "specodelic_rename_id" || name == "kinds_rename_id"
}

#[test]
fn every_generator_name_in_the_corpus_resolves_to_a_fixture() {
    let names = corpus_generators();
    assert!(
        names.len() >= 40,
        "the corpus files name {len} generators",
        len = names.len()
    );
    for name in &names {
        assert!(
            specodelic::fixtures::is_known(name),
            "generator `{name}` has no fixture — the compiler would bind it to a placeholder"
        );
    }
}

#[test]
fn the_unmigrated_rows_are_the_documented_four() {
    let ids = unmigrated_rows();
    assert_eq!(
        ids,
        vec![
            "silently_shrunk_id_set_rejected".to_string(),
            "constraint_row_extra_field_rejected".to_string(),
            "state_row_with_extra_field_rejected".to_string(),
            "subkind_addition_without_revision_rejected".to_string(),
        ],
        "the rows without an honest tool seam — each keeps the todo_predicate! placeholder"
    );
}

#[test]
fn the_fixtures_are_real_spec_texts() {
    for name in corpus_generators() {
        let text =
            specs::materialize(&name).unwrap_or_else(|| panic!("fixture `{name}` materializes"));
        if is_non_spec_fixture(&name) {
            continue;
        }
        let parsed = parse_str(&text);
        if name == "frontmatter_missing_statement" || name == "intent_row_without_statement" {
            assert!(
                parsed.is_err(),
                "the frontmatter fixture must NOT parse — it is the missing-field input"
            );
            continue;
        }
        assert!(
            parsed.is_ok(),
            "fixture `{name}` must be a real spec text: {parsed:?}"
        );
    }
}

#[test]
fn the_neutral_fixtures_lint_clean() {
    for name in [
        "specodelic_spec_file",
        "specodelic_spec_repo",
        "arbitrary_row_of_one_kind",
        "arbitrary_well_formed_row_of_one_kind",
        "state_row_without_emits",
        "transition_guard_citing_a_declared_state",
        "schema_with_is_superseded_column",
        "two_specs_identical_except_rationale_prose",
        "row_belonging_to_a_sixth_section",
    ] {
        let text = specs::materialize(name).expect("the fixture materializes");
        let spec = parse_str(&text).expect("the fixture parses");
        let report = lint_corpus(std::slice::from_ref(&spec));
        assert!(
            report.issues.is_empty(),
            "fixture `{name}` must lint clean: {:?}",
            report
                .issues
                .iter()
                .map(|i| format!("{}: {}", i.rule_id, i.message))
                .collect::<Vec<_>>()
        );
    }
}

#[test]
fn the_dangling_reference_fixture_is_detected() {
    let text = specs::materialize("spec_file_with_dangling_reference").expect("materializes");
    let spec = parse_str(&text).expect("parses");
    assert!(specs::rule_fired(&spec, "linter.total_refs"));
}

#[test]
fn the_traces_cycle_fixture_is_detected() {
    let text = specs::materialize("spec_repo_with_traces_cycle").expect("materializes");
    let spec = parse_str(&text).expect("parses");
    assert!(specs::rule_fired(&spec, "linter.acyclic"));
}

#[test]
fn the_disconnected_cluster_fixture_is_detected() {
    let text = specs::materialize("spec_repo_with_disconnected_constraint_cluster").expect("m");
    let spec = parse_str(&text).expect("parses");
    assert!(specs::rule_fired(&spec, "linter.single_root_reachable"));
}

#[test]
fn the_supersedes_cycle_fixture_is_detected_by_the_acyclic_query() {
    let text = specs::materialize("spec_repo_with_supersedes_cycle").expect("materializes");
    let spec = parse_str(&text).expect("parses");
    let instance = Instance::from_specs(std::slice::from_ref(&spec));
    let cyclic = query::cyclic_nodes(&instance, &["supersedes"]).expect("the query runs");
    assert!(
        !cyclic.is_empty(),
        "the supersedes cycle must be found: {cyclic:?}"
    );
}

#[test]
fn the_supersession_status_is_derived_not_stored() {
    let text = specs::materialize("schema_with_is_superseded_column").expect("materializes");
    let spec = parse_str(&text).expect("parses");
    let instance = Instance::from_specs(std::slice::from_ref(&spec));
    let superseded = query::forward_closure(&instance, &["fx.c2"], &["supersedes"]).expect("q");
    assert!(
        superseded.contains("fx.c1"),
        "the supersession relation must be derivable from the edges: {superseded:?}"
    );
    let schema = specodelic::acset::schema::canonical();
    assert!(
        !schema.morphisms.iter().any(|m| m.column == "is_superseded"),
        "the schema must carry no stored supersession flag"
    );
}

#[test]
fn the_coverage_gap_fixture_is_detected() {
    let text = specs::materialize("spec_file_with_coverage_gap").expect("materializes");
    let spec = parse_str(&text).expect("parses");
    assert!(specs::rule_fired(&spec, "linter.coverage"));
}

#[test]
fn the_state_guard_fixture_lints_clean() {
    let text = specs::materialize("transition_guard_citing_a_declared_state").expect("m");
    let spec = parse_str(&text).expect("parses");
    let report = lint_corpus(std::slice::from_ref(&spec));
    assert!(
        report.issues.is_empty(),
        "a State guard is a legal typed citation: {:?}",
        report
            .issues
            .iter()
            .map(|i| format!("{}: {}", i.rule_id, i.message))
            .collect::<Vec<_>>()
    );
}

#[test]
fn the_invariant_emits_fixture_is_detected() {
    let text = specs::materialize("emits_field_pointing_at_an_invariant_constraint").expect("m");
    let spec = parse_str(&text).expect("parses");
    assert!(specs::rule_fired(&spec, "linter.ref_kind_compatible"));
}

#[test]
fn the_effect_emits_fixture_lints_clean() {
    let text = specs::materialize("state_row_with_effect_emits").expect("materializes");
    let spec = parse_str(&text).expect("parses");
    let report = lint_corpus(std::slice::from_ref(&spec));
    assert!(
        report.issues.is_empty(),
        "an effect emits is legal: {:?}",
        report
            .issues
            .iter()
            .map(|i| format!("{}: {}", i.rule_id, i.message))
            .collect::<Vec<_>>()
    );
}

#[test]
fn the_kind_shapes_lint_clean() {
    for name in [
        "constraint_row_with_kind_advisory",
        "constraint_row_with_kind_effect",
        "constraint_row_with_kind_extension_point",
    ] {
        let text = specs::materialize(name).expect("materializes");
        let spec = parse_str(&text).expect("parses");
        let report = lint_corpus(std::slice::from_ref(&spec));
        assert!(
            report.issues.is_empty(),
            "fixture `{name}` must lint clean: {:?}",
            report
                .issues
                .iter()
                .map(|i| format!("{}: {}", i.rule_id, i.message))
                .collect::<Vec<_>>()
        );
    }
}

#[test]
fn the_made_up_kind_fixtures_are_detected() {
    let text = specs::materialize("constraint_row_with_kind_made_up").expect("materializes");
    let spec = parse_str(&text).expect("parses");
    assert!(specs::rule_fired(&spec, "linter.constraint_kind_closed"));

    let text = specs::materialize("property_row_with_kind_audit").expect("materializes");
    let spec = parse_str(&text).expect("parses");
    assert!(specs::rule_fired(&spec, "linter.property_kind_closed"));
}

#[test]
fn the_typed_reference_defect_fixtures_are_detected() {
    for (name, rule) in [
        (
            "specodelic_property_row_guard",
            "linter.ref_kind_compatible",
        ),
        (
            "guard_field_pointing_at_an_advisory_constraint",
            "linter.ref_kind_compatible",
        ),
        (
            "constraint_row_with_supersedes_property_row",
            "linter.ref_kind_compatible",
        ),
        (
            "constraint_row_with_derives_from_column",
            "linter.ref_kind_compatible",
        ),
        (
            "satisfies_field_pointing_at_an_invariant_constraint",
            "linter.ref_kind_compatible",
        ),
    ] {
        let text = specs::materialize(name).expect("materializes");
        let spec = parse_str(&text).expect("parses");
        assert!(
            specs::rule_fired(&spec, rule),
            "fixture `{name}` must fire `{rule}`"
        );
    }
}

#[test]
fn the_shape_defect_fixtures_are_detected() {
    for (name, rule) in [
        (
            "spec_file_with_transition_target_not_in_states",
            "linter.every_transition_valid",
        ),
        ("transition_row_with_empty_guard", "linter.guard_required"),
        ("transition_row_without_guard", "linter.guard_required"),
        (
            "statement_outside_the_five_ears_patterns",
            "linter.ears_syntax",
        ),
        (
            "row_id_joined_to_two_intents_via_and",
            "linter.no_conjoined_id",
        ),
        ("law_property_missing_associativity", "linter.law_cases"),
        (
            "state_appearing_in_no_transition",
            "linter.every_state_used",
        ),
        (
            "spec_file_without_states_or_transitions",
            "linter.model_present",
        ),
    ] {
        let text = specs::materialize(name).expect("materializes");
        let spec = parse_str(&text).expect("parses");
        assert!(
            specs::rule_fired(&spec, rule),
            "fixture `{name}` must fire `{rule}`"
        );
    }
}

#[test]
fn the_extension_point_corpus_needs_no_reachability_carveout() {
    let text = specs::materialize("constraint_row_satisfying_a_foreign_extension_point")
        .expect("materializes");
    let spec = parse_str(&text).expect("parses");
    let host_text =
        specs::materialize("foreign_file_declaring_an_extension_point").expect("materializes");
    let host = parse_str(&host_text).expect("parses");
    let corpus = vec![spec, host];
    assert!(
        !specs::rule_fired_in(&corpus, "linter.single_root_reachable"),
        "a cross-file satisfies edge is an outbound leaf, never a reachability path: {:?}",
        specs::lint_issues_of(&corpus)
    );
}

#[test]
fn the_rationale_fixture_differs_only_in_prose() {
    let text = specs::materialize("two_specs_identical_except_rationale_prose").expect("m");
    let stripped = specs::strip_rationale(&text);
    assert_ne!(text, stripped, "the rationale fixture must carry prose");
    let with = parse_str(&text).expect("the rationale fixture parses");
    let without = parse_str(&stripped).expect("the stripped fixture parses");
    assert_eq!(
        specs::lint_issues(&with),
        specs::lint_issues(&without),
        "prose must never change a lint result"
    );
}

#[test]
fn the_violated_invariant_fixture_yields_a_counterexample() {
    let text = specs::materialize("model_with_a_violated_invariant").expect("materializes");
    let spec = parse_str(&text).expect("parses");
    let ir = specodelic::compile::extract_model_ir(&spec);
    let tla = specodelic::compile::model_to_tla(&spec);
    let bound = specodelic::model_check::Bound {
        max_depth: 10,
        max_states: Some(100),
        timeout_secs: Some(120),
    };
    let report = specodelic::model_check::run_executable(&ir, tla.as_bytes(), &bound)
        .expect("the executable-invariant run reports");
    assert_eq!(
        report.outcome,
        specodelic::model_check::Outcome::CounterexampleFound,
        "the violated invariant must be found"
    );
    assert_eq!(report.violated_invariant_id.as_deref(), Some("c1"));
}

#[test]
fn the_failing_property_gate_reports_a_failure() {
    let (status, really_fails) =
        specs::failing_properties_verdict().expect("the gate runs over the fixture");
    assert_eq!(status, "properties_failed");
    assert!(
        really_fails,
        "the compiled artifact must really carry a failing assertion"
    );
}

#[test]
fn the_error_labels_carry_all_three_sites() {
    for label in ["kinds.kind_assignment_failure", "kinds.shape_check_failure"] {
        let sites = specs::label_sites("kinds", label).expect("the corpus text is readable");
        assert!(
            sites.constraint_row,
            "the error Constraint row must carry `{label}`"
        );
        assert!(sites.property_row, "the Property row must carry `{label}`");
        assert!(sites.note, "the scenario note must carry `{label}`");
    }
}
