//! The compiler's in-module test block (specodelic-1nhe: moved out of
//! `src/compile.rs` — pretender's `file_lines` ratchet is shrink-only, and
//! the Revision-21 law-case grammar needed the headroom; the
//! `specodelic-ez9a.1` precedent moved the shared-fragment tests the same
//! way).
//!
//! Purpose: pin the compiler's fragment extraction, emission and IR
//! behaviour against the crate-private API.
//! Responsibilities: the tests that need `super::*` (the crate-private
//! helpers) live here; the public-API tests live in `tests/`.
//! Rationale: a child module keeps the parent's private items in scope, so
//! the move is behaviour-preserving.

use crate::test_panic::UnwrapPanic;
use crate::test_panic::UnwrapPanicErr;

use super::*;
use crate::spec::parse_str;

#[test]
fn guard_citations_extracted_from_invariant_rows_prose_excluded() {
    // Slice 1 (specodelic-txo): an invariant-kind Constraint whose
    // expr cell is a citation expression lands in the IR's
    // guard_citations; prose expr cells stay out (inert as before).
    let text = SAMPLE
        .replace(
            "| a | invariant | `x holds` |",
            "| a | invariant | `[[p1]] ∧ [[p2]]` |",
        )
        .replace(
            "| b | effect | `y fires` |",
            "| b | invariant | `x holds` |",
        );
    let spec = parse_str(&text).unwrap_p();
    let ir = extract_model_ir(&spec);
    assert_eq!(
        ir.guard_citations.get("a").map(String::as_str),
        Some("[[p1]] ∧ [[p2]]"),
        "citation expr is carried verbatim"
    );
    assert!(
        !ir.guard_citations.contains_key("b"),
        "prose expr cells are not citations"
    );
    assert!(ir.invariants.is_empty(), "no fragments in this spec");
}

#[test]
fn sanitize_ident_leading_digit_and_unicode_are_valid_rust_idents() {
    // specodelic-suz (pinning): a digit-leading id gets an underscore
    // prefix; unicode chars are replaced — both results are valid
    // Rust identifiers.
    assert_eq!(sanitize_ident("9lives"), "_9lives");
    assert_eq!(sanitize_ident("café"), "caf_");
    for s in [sanitize_ident("9lives"), sanitize_ident("café")] {
        let first = s.chars().next().unwrap_p();
        assert!(first == '_' || first.is_ascii_alphabetic());
        assert!(s.chars().all(|c| c.is_ascii_alphanumeric() || c == '_'));
    }
}

#[test]
fn sanitize_ident_never_yields_a_rust_keyword_or_empty() {
    // specodelic-suz: a reserved word passes the char map untouched
    // and must still not become a fn name — `fn fn(…)` does not
    // compile; the empty id must not become `fn ()` either.
    for kw in [
        "fn", "match", "async", "await", "impl", "type", "self", "Self",
    ] {
        assert_eq!(sanitize_ident(kw), format!("_{kw}"), "keyword {kw}");
    }
    assert_eq!(sanitize_ident(""), "_");
}

#[test]
fn sanitize_ident_caps_extremely_long_ids_and_stays_unique_deterministic() {
    // specodelic-suz: a hostile 200-char id must not emit a 200-char
    // fn name — capped, and two ids sharing a prefix beyond the cap
    // must not collide (the suffix hash disambiguates them).
    let prefix = "a".repeat(200);
    let a = sanitize_ident(&prefix);
    let b = sanitize_ident(&format!("{prefix}x"));
    assert!(a.len() <= 64, "capped: {}", a.len());
    assert!(b.len() <= 64, "capped: {}", b.len());
    assert_ne!(a, b, "same-prefix long ids must not collide");
    assert_eq!(a, sanitize_ident(&prefix), "deterministic");
}

const SAMPLE: &str = "---\nid: demo.thing\nkind: intent\nstatement: \"THE system SHALL work\"\n---\n\n## Constraints\n\n| id | kind | expr | traces_to |\n|----|------|------|-----------|\n| a | invariant | `x holds` | [[demo.thing]] |\n| b | effect | `y fires` | [[demo.thing]] |\n\n## Model\n\n### States\n\n- s1\n- s2 (emits: `[[demo.thing.b]]`)\n\n### Transitions\n\n| id | from | to | guard |\n|----|------|----|-------|\n| t1 | s1 | s2 | [[demo.thing.a]] |\n| t2 | s2 | s1 | `done` |\n\n## Properties\n\n| id | kind | derives_from | generator | predicate |\n|----|------|--------------|-----------|-----------|\n| p1 | unit | [[demo.thing.a]] | `arbitrary_row()` | `check(x) == ok` |\n| p2 | law | [[demo.thing.a]] | `arbitrary_row(), other()` | **identity:** `f(a) == a` **associativity:** `f(f(a)) == f(a)` |\n";

fn sample() -> Spec {
    parse_str(SAMPLE).unwrap_p()
}

// --- 1.2 / 1.3 Constraints → TOML ---

#[test]
fn toml_roundtrips_field_for_field() {
    let spec = sample();
    let toml = constraints_to_toml(&spec);
    let doc: ConstraintsDoc = toml::from_str(&toml).unwrap_p();
    assert_eq!(doc.source, "demo.thing");
    assert_eq!(doc.constraints.len(), 2);
    let a = &doc.constraints[0];
    assert_eq!(a.id, "a");
    assert_eq!(a.kind, "invariant");
    assert_eq!(a.expr, "`x holds`");
    assert_eq!(a.traces_to, "[[demo.thing]]");
}

#[test]
fn toml_ids_preserved_verbatim() {
    let spec = sample();
    let toml = constraints_to_toml(&spec);
    assert!(toml.contains("id = \"a\""));
    assert!(toml.contains("id = \"b\""));
}

// --- 2.1 / 2.2 Model → ModelIR ---

#[test]
fn ir_transition_count_matches() {
    let spec = sample();
    let ir = extract_model_ir(&spec);
    assert_eq!(ir.transitions.len(), 2);
    let t1 = &ir.transitions[0];
    assert_eq!(t1.from, "s1");
    assert_eq!(t1.to, "s2");
    assert_eq!(t1.guard.as_deref(), Some("[[demo.thing.a]]"));
}

#[test]
fn ir_emits_covers_exactly_emitting_states() {
    let spec = sample();
    let ir = extract_model_ir(&spec);
    // exactly one entry — no entry for s1, never a default or null
    assert_eq!(ir.emits.len(), 1);
    assert_eq!(ir.emits.get("s2").map(String::as_str), Some("b"));
    assert!(!ir.emits.contains_key("s1"));
}

// --- 3.2 / 3.3 Properties → proptest! ---

#[test]
fn unit_property_yields_at_least_one_block() {
    let spec = sample();
    let src = properties_to_proptest(&spec);
    assert!(src.contains("proptest!"));
    assert!(src.contains("fn p1("));
    assert!(src.contains("todo_predicate!"));
}

#[test]
fn law_expands_to_required_cases() {
    let spec = sample();
    let src = properties_to_proptest(&spec);
    // p1 (unit) → 1 block; p2 (law, identity+associativity) → 2 blocks
    assert_eq!(src.matches("#[test]").count(), 3);
    assert!(src.contains("fn p2_identity("));
    assert!(src.contains("fn p2_associativity("));
}

#[test]
fn law_without_labels_falls_back_to_floor() {
    let spec = parse_str(
        "---\nid: demo.law\nkind: intent\nstatement: \"THE system SHALL hold\"\n---\n\n## Constraints\n\n| id | kind | expr | traces_to |\n|----|------|------|-----------|\n| a | invariant | `x` | [[demo.law]] |\n\n## Model\n\n### States\n\n- s\n\n### Transitions\n\n| id | from | to | guard |\n|----|------|----|-------|\n| t | s | s | [[demo.law.a]] |\n\n## Properties\n\n| id | kind | derives_from | generator | predicate |\n|----|------|--------------|-----------|-----------|\n| p | law | [[demo.law.a]] | `g()` | `f holds` |\n",
    )
    .unwrap_p();
    let src = properties_to_proptest(&spec);
    // floor: identity + associativity
    assert_eq!(src.matches("#[test]").count(), 2);
    assert!(src.contains("fn p_identity("));
    assert!(src.contains("fn p_associativity("));
}

#[test]
fn generators_get_spec_gen_helpers() {
    let spec = sample();
    let src = properties_to_proptest(&spec);
    assert!(src.contains("pub fn arbitrary_row()"));
    assert!(src.contains("pub fn other()"));
    assert!(src.contains("v0 in spec_gen::arbitrary_row()"));
    assert!(src.contains("v1 in spec_gen::other()"));
}

// --- specodelic-8aq: parameterless property blocks ---

const PARAMETERLESS_SAMPLE: &str = "---\nid: demo.par\nkind: intent\nstatement: \"THE system SHALL hold\"\n---\n\n## Constraints\n\n| id | kind | expr | traces_to |\n|----|------|------|-----------|\n| a | invariant | `x` | [[demo.par]] |\n\n## Model\n\n### States\n\n- s\n\n### Transitions\n\n| id | from | to | guard |\n|----|------|----|-------|\n| t | s | s | [[demo.par.a]] |\n\n## Properties\n\n| id | kind | derives_from | generator | predicate |\n|----|------|--------------|-----------|------------|\n| p | unit | [[demo.par.a]] | `(a, b)` where one was removed | `x holds` |\n| l | law | [[demo.par.a]] |  | `f holds` |\n";

#[test]
fn parameterless_generator_rows_emit_plain_test_fns() {
    // specodelic-8aq: a generator cell yielding no strategy params (no
    // `name(` occurrences — the corpus shape, e.g. linter-schema_shape's
    // `(a, b)` where-prose) must NOT emit `fn p()` inside proptest! —
    // proptest! requires (pat in strategy) on every fn, so a bare fn is
    // a parse error that left the scratch crate properties_uncompilable.
    // The block goes outside the macro as a plain #[test] fn, honest via
    // todo_predicate!, with the metadata comments verify fingerprints.
    let spec = parse_str(PARAMETERLESS_SAMPLE).unwrap_p();
    let src = properties_to_proptest(&spec);
    // No proptest! macro at all: every block in this fixture is
    // parameterless.
    assert!(
        !src.contains("proptest!"),
        "bare fn inside proptest! is a parse error: {src}"
    );
    // Plain #[test] fns at top level (unindented `fn`).
    assert!(src.contains("#[test]\nfn p("), "unit block: {src}");
    // Law-case splitting intact: one plain fn per required case.
    assert!(
        src.contains("#[test]\nfn l_identity("),
        "law identity: {src}"
    );
    assert!(
        src.contains("#[test]\nfn l_associativity("),
        "law associativity: {src}"
    );
    // Honest failure preserved — never a silent pass.
    assert_eq!(
        src.matches("todo_predicate!(").count(),
        3,
        "placeholder bodies: {src}"
    );
    // Metadata comments preserved for verify's staleness fingerprint.
    assert!(src.contains("// id: p"));
    assert!(src.contains("// id: l"));
    // verify's block discovery still finds every emitted block.
    let blocks = crate::verify::blocks_from_source(&src);
    assert_eq!(blocks.len(), 3, "all blocks discoverable: {src}");
}

#[test]
fn parameterless_fragment_row_asserts_outside_proptest() {
    // A parameterless row that opts in with **rust:** keeps its verbatim
    // fragment body (fragment rows unchanged) — just outside proptest!.
    let text = PARAMETERLESS_SAMPLE.replace(
        "| p | unit | [[demo.par.a]] | `(a, b)` where one was removed | `x holds` |",
        "| p | unit | [[demo.par.a]] | `(a, b)` where one was removed | `**rust:** state != \"blackhole\"` |",
    );
    let spec = parse_str(&text).unwrap_p();
    let src = properties_to_proptest(&spec);
    assert!(src.contains("#[test]\nfn p("), "plain fn: {src}");
    assert!(!src.contains("proptest!"), "no macro fn: {src}");
    assert!(
        src.contains("assert!(\n            state != \"blackhole\","),
        "verbatim fragment body: {src}"
    );
}

// --- model_to_tla ---

#[test]
fn tla_disjunct_count_matches_transitions_plus_stuttering() {
    let spec = sample();
    let src = model_to_tla(&spec);
    // one disjunct per transition + the closing stutter disjunct
    // (specodelic-len: terminal states must not be engine-side
    // deadlocks for opt-in TLC)
    assert_eq!(src.matches("\\/").count(), 3);
    assert!(src.contains("UNCHANGED vpc"));
    assert!(src.contains("vpc = \"s1\" /\\ vpc' = \"s2\""));
}

#[test]
fn tla_module_header_is_single_line_tla_plus_form() {
    // specodelic-len: the header must be the TLA+ module-header form
    // (dashes + MODULE name + dashes on ONE line) so opt-in TLC can
    // parse the module — the previous three-line form is not a valid
    // TLA+ module header.
    let spec = sample();
    let src = model_to_tla(&spec);
    let header = src.lines().next().unwrap_p();
    assert!(header.starts_with("---- MODULE demothing ----"));
    assert!(header.trim_end().ends_with("----"));
    assert!(!src.contains("\nMODULE"));
}

#[test]
fn tla_output_covers_emitting_states_only() {
    let spec = sample();
    let src = model_to_tla(&spec);
    // domain is exactly the emitting states — s1 has no entry; the
    // value is the effect-Constraint's `expr`, not its id
    assert!(src.contains("\"s2\" :> \"`y fires`\""));
    assert!(!src.contains("\"s1\" :>"));
}

#[test]
fn tla_no_emits_means_empty_output() {
    let spec = parse_str(
        "---\nid: demo.quiet\nkind: intent\nstatement: \"THE system SHALL hold\"\n---\n\n## Constraints\n\n| id | kind | expr | traces_to |\n|----|------|------|-----------|\n| a | invariant | `x` | [[demo.quiet]] |\n\n## Model\n\n### States\n\n- s\n\n### Transitions\n\n| id | from | to | guard |\n|----|------|----|-------|\n| t | s | s | [[demo.quiet.a]] |\n\n## Properties\n\n| id | kind | derives_from | generator | predicate |\n|----|------|--------------|-----------|------------|\n| p | unit | [[demo.quiet.a]] | `g()` | `x` |\n",
    )
    .unwrap_p();
    let src = model_to_tla(&spec);
    assert!(src.contains("Output == << >>"));
}

#[test]
fn tla_module_name_sanitized_and_guard_verbatim() {
    let mut spec = sample();
    spec.path = Some(std::path::PathBuf::from("specs/demo-thing.md"));
    let src = model_to_tla(&spec);
    assert!(src.contains("MODULE demothing"));
    // the spec's own guard text travels verbatim in the disjunct comment
    assert!(src.contains("(guard: [[demo.thing.a]])"));
}

#[test]
fn tla_artifact_is_byte_stable() {
    let spec = sample();
    assert_eq!(model_to_tla(&spec), model_to_tla(&spec));
}

// --- 4.2 contract invariants ---

#[test]
fn compile_refuses_stateless_spec_with_labeled_failure() {
    // lib-level totality: lint's model_present gate covers CLI runs,
    // but compile_spec itself must fail labeled, never emit a
    // malformed module.
    let spec = parse_str(
        "---\nid: demo.bare\nkind: intent\nstatement: \"THE system SHALL work\"\n---\n\n## Constraints\n\n| id | kind | expr | traces_to |\n|----|------|------|-----------|\n| a | invariant | `x` | [[demo.bare]] |\n",
    )
    .unwrap_p();
    let err = compile_spec(&spec).unwrap_p_err();
    assert_eq!(err.stage, "model_to_tla");
}

#[test]
fn colliding_fn_names_are_disambiguated() {
    // `p.1` and `p_1` sanitize to the same fn name — the second gets a
    // suffix instead of emitting duplicate fns.
    let spec = parse_str(
        "---\nid: demo.coll\nkind: intent\nstatement: \"THE system SHALL work\"\n---\n\n## Constraints\n\n| id | kind | expr | traces_to |\n|----|------|------|-----------|\n| a | invariant | `x` | [[demo.coll]] |\n\n## Model\n\n### States\n\n- s\n\n### Transitions\n\n| id | from | to | guard |\n|----|------|----|-------|\n| t | s | s | [[demo.coll.a]] |\n\n## Properties\n\n| id | kind | derives_from | generator | predicate |\n|----|------|--------------|-----------|------------|\n| p.1 | unit | [[demo.coll.a]] | `g()` | `x` |\n| p_1 | unit | [[demo.coll.a]] | `g()` | `x` |\n",
    )
    .unwrap_p();
    let src = properties_to_proptest(&spec);
    assert!(src.contains("fn p_1("));
    assert!(src.contains("fn p_1_2("));
    assert_eq!(src.matches("fn p_1(").count(), 1);
}

#[test]
fn compile_preserves_every_source_id() {
    let spec = sample();
    let compiled = compile_spec(&spec).unwrap_p();
    let text = format!("{}{}{:?}", compiled.toml, compiled.props, compiled.model_ir);
    for id in spec.defined_ids() {
        assert!(
            text.contains(&id) || ir_contains_id(&compiled.model_ir, &id),
            "id `{id}` dropped"
        );
    }
}
