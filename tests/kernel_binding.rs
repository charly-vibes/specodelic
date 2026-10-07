//! The opaque binding claim carrier — Tier C of add-min-expr-kernel
//! (specodelic-bf5, tasks.md §5.1 RED; design D4).
//!
//! `kernel.binding` extraction tests: an invariant-kind Constraint with a
//! `kernel.binding` cell extracts the claim as an opaque string; the
//! toolchain never interprets its contents (no parse, no validation, no
//! execution); a constraint claimed by an external checker surfaces in
//! contract-TOML `flags` binding shape (node id / `-k` /
//! `[[tests.shell]]`) via the compiled Constraints TOML artifact. No
//! registry — the carrier is the artifact surface and nothing more.
//!
//! Hard gate (tasks.md §5.1): at least three fixtures preserve ordinary,
//! arbitrary, and empty binding strings exactly.
//!
//! These live out-of-module as new integration tests (the ticket's
//! preferred split; src/model_check.rs is pinned by the pretender
//! file_lines ratchet — shrink-only).

use specodelic::compile::{compile_spec, constraints_to_toml};
use specodelic::spec::parse_str;

/// The pre-Revision lint-clean shape (the proven parity fixture shape),
/// extended with an optional `kernel.binding` column. `rows` are
/// verbatim constraint rows.
fn binding_spec(rows: &str, with_binding_col: bool) -> String {
    let cols = if with_binding_col {
        "| id | kind | expr | traces_to | kernel.binding |\n\
         |----|------|------|----------|----------------|\n"
    } else {
        "| id | kind | expr | traces_to |\n\
         |----|------|------|-----------|\n"
    };
    format!(
        "---\nid: demo.binding\nkind: intent\nstatement: \"THE demo SHALL carry binding fixtures\"\n---\n\
         \n## Constraints\n\
         \n{cols}\
         {rows}\
         \n## Model\n\
         \n### States\n\
         \n- s1\n\
         - s2\n\
         \n### Transitions\n\
         \n| id | from | to | guard |\n\
         |----|------|----|-------|\n\
         | t | s1 | s2 | [[demo.binding.c1]] |\n\
         \n## Properties\n\
         \n| id | kind | derives_from | generator | predicate |\n\
         |----|------|--------------|-----------|------------|\n\
         | p_c1 | unit | [[demo.binding.c1]] | `g()` | `x` |\n\
         | p_b1 | unit | [[demo.binding.b1]] | `g()` | `x` |\n"
    )
}

/// Parse + compile a fixture and return the constraints TOML artifact.
fn compiled_toml(rows: &str, with_binding_col: bool) -> String {
    let text = binding_spec(rows, with_binding_col);
    let spec = parse_str(&text).expect("binding fixture parses");
    compile_spec(&spec).expect("binding fixture compiles").toml
}

/// Extract the `binding` value of one constraint entry from the artifact
/// TOML (panics if the entry lacks the field — the extraction meter).
fn binding_of(toml: &str, id: &str) -> String {
    let doc: toml::Value = toml::from_str(toml).expect("artifact re-parses");
    doc["constraints"]
        .as_array()
        .expect("constraints array")
        .iter()
        .find(|c| c["id"].as_str() == Some(id))
        .unwrap_or_else(|| panic!("constraint {id} missing from artifact"))
        .get("binding")
        .and_then(|b| b.as_str())
        .unwrap_or_else(|| panic!("constraint {id} carries no binding field"))
        .to_string()
}

// ---- §5.1 RED: the hard gate — three fixture classes preserved exactly ----

#[test]
fn ordinary_binding_text_preserved_exactly() {
    // Scenario: Binding extracts verbatim — an ordinary binding string
    // (a scenario node id, the plainest carrier form) extracts
    // byte-exact, never reshaped.
    let rows = "| c1 | invariant | `holds` | [[demo.binding]] |  |\n\
                | b1 | invariant | `**kernel:** |State| == 2` | [[demo.binding]] | `scenario:demo.binding.p_c1` |\n";
    let toml = compiled_toml(rows, true);
    assert_eq!(
        binding_of(&toml, "b1"),
        "scenario:demo.binding.p_c1",
        "ordinary binding text must be preserved exactly"
    );
}

#[test]
fn arbitrary_binding_text_preserved_exactly() {
    // Scenario: Binding extracts verbatim — arbitrary checker-language
    // text (pytest `-k` expression, `[[tests.shell]]` command payload,
    // punctuation, unicode, backslashes) extracts byte-exact. The
    // toolchain never interprets the contents: this text would fail any
    // parse, validation, or execution — extraction still succeeds.
    let rows = "| c1 | invariant | `holds` | [[demo.binding]] |  |\n\
                | b1 | invariant | `**kernel:** |State| == 2` | [[demo.binding]] | `-k kernel_binding and not slow` |\n\
                | b2 | invariant | `**kernel:** |Intent| == 1` | [[demo.binding]] | `[[tests.shell]] printf '%s' 'a|b{c}\\\\' && exit 1 # {drop}` |\n\
                | b3 | invariant | `**kernel:** resolves(traces_to)` | [[demo.binding]] | `λx.„quote” — †† ←→ \\n\\t ≠ ==` |\n";
    let toml = compiled_toml(rows, true);
    assert_eq!(binding_of(&toml, "b1"), "-k kernel_binding and not slow");
    assert_eq!(
        binding_of(&toml, "b2"),
        "[[tests.shell]] printf '%s' 'a|b{c}\\\\' && exit 1 # {drop}",
        "shell payload preserved byte-exact incl. backslashes and pipes"
    );
    assert_eq!(binding_of(&toml, "b3"), "λx.„quote” — †† ←→ \\n\\t ≠ ==");
}

#[test]
fn empty_binding_cell_preserved_as_empty_string() {
    // Scenario: Binding extracts verbatim — a present-but-empty
    // kernel.binding cell extracts as the empty string, not as absence
    // and not as an error (the empty-claim case external checkers hit
    // on scaffolded tables).
    let rows = "| c1 | invariant | `holds` | [[demo.binding]] |  |\n\
                | b1 | invariant | `**kernel:** |State| == 2` | [[demo.binding]] | |\n";
    let toml = compiled_toml(rows, true);
    assert_eq!(
        binding_of(&toml, "b1"),
        "",
        "empty binding cell must surface as the empty string, exactly"
    );
}

// ---- §5.1 RED: the claim-carrier surface — never interpreted, no registry ----

#[test]
fn binding_surfaces_in_the_compiled_toml_only() {
    // Scenario: Claim path is contract flags — the binding text is the
    // compiled Constraints TOML artifact (the material contract-TOML
    // `flags` authors bind against: node id / `-k` / [[tests.shell]]).
    // The extraction is the carrier surface; nothing else is built.
    let rows = "| c1 | invariant | `holds` | [[demo.binding]] |  |\n\
                | b1 | invariant | `**kernel:** |State| == 2` | [[demo.binding]] | `scenario:demo.binding.p_c1` |\n";
    let text = binding_spec(rows, true);
    let spec = parse_str(&text).expect("fixture parses");
    let toml = constraints_to_toml(&spec);
    assert!(
        toml.contains("scenario:demo.binding.p_c1"),
        "binding surfaces verbatim in the constraints TOML artifact"
    );
    // Direct TOML round-trip: the artifact parses back to the same
    // string — carried, not transformed.
    assert_eq!(
        binding_of(&toml, "b1"),
        "scenario:demo.binding.p_c1",
        "constraints_to_toml carries the binding verbatim"
    );
}

#[test]
fn binding_contents_never_interpreted_or_validated() {
    // Scenario: Binding extracts verbatim — contents that are invalid
    // TOML, invalid shell, invalid everything still extract: the
    // toolchain never parses, validates, or executes binding internals
    // (bridge-never-absorb, design D4).
    let rows = "| c1 | invariant | `holds` | [[demo.binding]] |  |\n\
                | b1 | invariant | `**kernel:** |State| == 2` | [[demo.binding]] | `\x01\x02 raw bytes ]]} {{{ ]` |\n";
    let toml = compiled_toml(rows, true);
    assert_eq!(
        binding_of(&toml, "b1"),
        "\u{1}\u{2} raw bytes ]]} {{{ ]",
        "garbage binding text extracts verbatim — no interpretation, no rejection"
    );
}

// ---- §5.1 RED: pure widening — pre-Revision artifacts byte-identical ----

#[test]
fn no_binding_column_compiles_byte_identically() {
    // Scenario: Pure widening — a spec without a kernel.binding column
    // (every pre-Revision spec) compiles with NO binding field anywhere;
    // the artifact is exactly the pre-Revision artifact shape.
    let rows = "| c1 | invariant | `holds` | [[demo.binding]] |\n\
                | b1 | invariant | `**kernel:** |State| == 2` | [[demo.binding]] |\n";
    let toml = compiled_toml(rows, false);
    let doc: toml::Value = toml::from_str(&toml).expect("artifact re-parses");
    let any_binding = doc["constraints"]
        .as_array()
        .unwrap()
        .iter()
        .any(|c| c.get("binding").is_some());
    assert!(
        !any_binding,
        "a spec without the kernel.binding column gains no binding field (pure widening): {toml}"
    );
}

// ---- §5.1 RED: the invariant-kind scope — the spec's exact words ----

#[test]
fn non_invariant_row_binding_stays_out() {
    // Scenario: Binding extracts verbatim — the requirement scopes to
    // invariant-kind Constraints; an effect-kind row carrying the cell
    // extracts nothing (no widening of the scope).
    let rows = "| c1 | invariant | `holds` | [[demo.binding]] |  |\n\
                | fx | effect | `y fires` | [[demo.binding]] | `scenario:must.not.extract` |\n\
                | b1 | invariant | `**kernel:** |State| == 2` | [[demo.binding]] | `scenario:demo.binding.p_c1` |\n";
    let toml = compiled_toml(rows, true);
    let doc: toml::Value = toml::from_str(&toml).expect("artifact re-parses");
    let fx = doc["constraints"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["id"].as_str() == Some("fx"))
        .expect("effect row present");
    assert!(
        fx.get("binding").is_none(),
        "non-invariant row with a kernel.binding cell must not extract: {fx:?}"
    );
    assert_eq!(binding_of(&toml, "b1"), "scenario:demo.binding.p_c1");
}
