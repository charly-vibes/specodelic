//! The compile spec's property fixtures — the registry the 35 property
//! rows of `openspec/specs/compile/spec.md` materialize through.
//!
//! Purpose: the scoped migration of the compile corpus file (specodelic-lf3)
//! makes every property row name a fixture; this file proves the registry
//! actually carries those inputs: every generator name the corpus's property
//! table names resolves to a fixture, every fixture is a real spec the real
//! parser accepts, and every fixture that is supposed to be lint-clean
//! lints clean.
//!
//! Responsibilities: assert the corpus↔registry correspondence, assert each
//! fixture parses and carries a Property row, assert the lint-clean
//! fixtures really lint clean, and assert the fixtures that exist to fail
//! fail the way their row claims.
//!
//! Rationale: the corpus's property cells are executable only if the
//! generator names they carry materialize real inputs — a name with no
//! fixture would silently yield the generator's own name (a placeholder
//! input), which is the failure mode the migration exists to remove.

use specodelic::fixtures;
use specodelic::lint::lint_corpus;
use specodelic::spec::parse_str;

#[path = "common/mod.rs"]
mod common;
use common::UnwrapPanic;

/// The corpus file whose property rows the registry serves.
const COMPILE_SPEC: &str = "openspec/specs/compile/spec.md";

/// The generator names in a generator cell — the same `name(` shape the
/// compiler's `generator_names` extracts (a test may restate it; the
/// compiler's own is private).
fn generator_names(cell: &str) -> Vec<String> {
    let mut names = vec![];
    let mut rest = cell;
    while let Some(open) = rest.find('(') {
        let head = &rest[..open];
        let name: String = head
            .chars()
            .rev()
            .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
            .collect::<Vec<char>>()
            .into_iter()
            .rev()
            .collect();
        if !name.is_empty() && name.chars().next().is_some_and(|c| c.is_ascii_alphabetic()) {
            if !names.contains(&name) {
                names.push(name);
            }
        }
        rest = &rest[open + 1..];
    }
    names
}

#[test]
fn every_property_row_generator_resolves_to_a_fixture() {
    // The corpus↔registry correspondence: every generator name the
    // compile spec's property table names must materialize a real
    // fixture — otherwise the row's compiled strategy yields the
    // generator's own name (a placeholder input).
    let text = std::fs::read_to_string(COMPILE_SPEC).unwrap_p();
    let spec = parse_str(&text).unwrap_p();
    assert_eq!(spec.properties.len(), 35, "the compile file's 35 rows");
    let mut missing: Vec<String> = vec![];
    for row in &spec.properties {
        let cell = row.cells.get("generator").cloned().unwrap_or_default();
        for name in generator_names(&cell) {
            if !fixtures::is_known(&name) && !missing.contains(&name) {
                missing.push(name);
            }
        }
    }
    assert!(
        missing.is_empty(),
        "every generator name must resolve to a registered fixture; missing: {missing:?}"
    );
}

#[test]
fn every_fixture_is_a_real_spec_with_a_property_row() {
    // The registry's own contract: each fixture's text is a real spec the
    // real parser accepts, and it carries the Property row a compiled
    // block needs.
    for name in fixtures::names() {
        let text = fixtures::materialize(name).unwrap_p();
        let spec = parse_str(&text).unwrap_p();
        assert!(
            !spec.properties.is_empty(),
            "the fixture `{name}` must carry a Property row"
        );
        assert_eq!(spec.intent.kind, "intent", "the fixture `{name}`");
        assert!(
            spec.intent.statement.starts_with("THE "),
            "the fixture `{name}` needs an EARS-shaped statement"
        );
    }
}

/// The fixtures that exist to FAIL the compiler (or the linter) — every
/// other fixture must lint clean.
///
/// `transition_row_with_fragment_guard` is here for a structural reason:
/// its transition's guard carries a fragment, so the guard cites no
/// constraint and the model's rows are an orphaned island under
/// `linter.single_root_reachable` — the fragment is exactly what the row
/// under test is about, and the fixture is never a corpus file.
const FAILING_FIXTURES: &[&str] = &[
    "file_that_has_not_passed_lint",
    "spec_file_with_undeclared_transition_target",
    "transition_row_with_fragment_guard",
];

#[test]
fn every_other_fixture_lints_clean() {
    // The lint-clean fixtures: a fixture that trips a lint rule would
    // make the corpus-wide lint gate fail for a reason that has nothing
    // to do with the property under test.
    for name in fixtures::names() {
        if FAILING_FIXTURES.contains(&name) {
            continue;
        }
        let text = fixtures::materialize(name).unwrap_p();
        let spec = parse_str(&text).unwrap_p();
        let report = lint_corpus(std::slice::from_ref(&spec));
        assert_eq!(
            report.issues.len(),
            0,
            "the fixture `{name}` must lint clean; findings: {:?}",
            report
                .issues
                .iter()
                .map(|i| format!("{}: {}", i.rule_id, i.message))
                .collect::<Vec<_>>()
        );
    }
}

#[test]
fn the_lint_failing_fixture_really_fails_lint() {
    // The counterpart: the fixture the precondition row is about must
    // carry the lint finding its row claims.
    let text = fixtures::materialize("file_that_has_not_passed_lint").unwrap_p();
    let spec = parse_str(&text).unwrap_p();
    let report = lint_corpus(std::slice::from_ref(&spec));
    assert!(
        report
            .issues
            .iter()
            .any(|i| i.rule_id == "linter.every_transition_valid"),
        "the fixture must carry the undeclared-transition finding; findings: {:?}",
        report.issues
    );
}
