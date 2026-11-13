//! Purpose: per-case fragment bodies for law-kind Property rows
//! (specodelic.md Revision 21) — a law row's `**name:**` case labels
//! stay the enumeration of record, and each named case may carry its
//! own executable body.
//! Responsibilities: pin the per-case extraction (the bodies form a
//! map keyed by the enumerated case labels), the per-case emission (one
//! block per case — unchanged cardinality — carrying that case's body
//! verbatim, the honest `todo_predicate!` placeholder where a case has
//! no body), the labeled failures (a fragment not associated with a
//! case, a second body for one case, a body naming a case the
//! predicate does not enumerate, a case body on a non-law row), the
//! label/marker interaction (the `**rust:**` marker is never counted as
//! a case label) and the pre-Revision byte-compatibility.
//! Rationale: the case grammar is shared between the linter and the
//! compiler, so every claim here is observable through the public
//! compile/lint API — nothing reaches into crate-private items.

use specodelic::compile::{compile_spec, properties_to_proptest};
use specodelic::lint::lint_corpus;
use specodelic::spec::parse_str;
use specodelic::verify::blocks_from_source;

mod common;
use common::{UnwrapPanic, UnwrapPanicErr};

/// A law row whose predicate enumerates three cases and carries a body
/// for two of them: the third case (`naturality`) has no body and must
/// keep the honest placeholder.
const LAW_WITH_BODIES: &str = "---\nid: demo.cases\nkind: intent\nstatement: \"THE demo SHALL carry per-case bodies\"\n---\n\n## Constraints\n\n| id | kind | expr | traces_to |\n|----|------|------|-----------|\n| a | invariant | `holds` | [[demo.cases]] |\n\n## Model\n\n### States\n\n- s1\n- s2\n\n### Transitions\n\n| id | from | to | guard |\n|----|------|----|-------|\n| t1 | s1 | s2 | [[demo.cases.a]] |\n\n## Properties\n\n| id | kind | derives_from | generator | predicate |\n|----|------|--------------|-----------|------------|\n| p_natural | law | [[demo.cases.a]] | `word()` | **identity:** `f(a) == a` **identity::** `**rust:** v0.len() >= 1` **associativity:** `f(f(a)) == f(a)` **associativity::** `**rust:** v0.len() >= 2` **naturality:** `g(f(a)) == g(a)` |\n";

/// The pre-Revision-21 shape: a law row with case labels and no bodies.
/// Its compiled artifact must stay byte-identical (rust_back_compat).
const LAW_LEGACY: &str = "---\nid: demo.law.legacy\nkind: intent\nstatement: \"THE demo SHALL carry a legacy law row\"\n---\n\n## Constraints\n\n| id | kind | expr | traces_to |\n|----|------|------|-----------|\n| a | invariant | `holds` | [[demo.law.legacy]] |\n\n## Model\n\n### States\n\n- s1\n- s2\n\n### Transitions\n\n| id | from | to | guard |\n|----|------|----|-------|\n| t1 | s1 | s2 | [[demo.law.legacy.a]] |\n\n## Properties\n\n| id | kind | derives_from | generator | predicate |\n|----|------|--------------|-----------|------------|\n| p_natural | law | [[demo.law.legacy.a]] | `word()` | **identity:** `f(a) == a` **associativity:** `f(f(a)) == f(a)` |\n";

/// The legacy artifact's block section — the golden the pre-Revision-21
/// emission must keep byte-for-byte.
const LAW_LEGACY_BLOCKS: &str = "proptest! {\n    // id: p_natural\n    // case: identity\n    // generator: `word()`\n    // predicate: **identity:** `f(a) == a` **associativity:** `f(f(a)) == f(a)`\n    #[test]\n    fn p_natural_identity(v0 in spec_gen::word()) {\n        todo_predicate!(\"**identity:** `f(a) == a` **associativity:** `f(f(a)) == f(a)`\");\n    }\n    // id: p_natural\n    // case: associativity\n    // generator: `word()`\n    // predicate: **identity:** `f(a) == a` **associativity:** `f(f(a)) == f(a)`\n    #[test]\n    fn p_natural_associativity(v0 in spec_gen::word()) {\n        todo_predicate!(\"**identity:** `f(a) == a` **associativity:** `f(f(a)) == f(a)`\");\n    }\n}\n\n";

/// The emitted block section of a compiled artifact — everything from
/// the `proptest!` macro on.
fn blocks_section(src: &str) -> &str {
    src.find("proptest! {")
        .map(|i| &src[i..])
        .unwrap_or_default()
}

/// The emitted block that carries the given case label.
fn block_for_case<'a>(src: &'a str, case: &str) -> &'a str {
    let marker = format!("// case: {case}\n");
    let start = src
        .find(&marker)
        .unwrap_or_else(|| panic!("no block for case {case}"));
    let rest = &src[start..];
    let end = rest[1..]
        .find("// case: ")
        .map(|i| i + 1)
        .unwrap_or(rest.len());
    &rest[..end]
}

/// The BODY of the emitted block that carries the given case label — the
/// assertion or the placeholder, without the metadata comments (the
/// `// predicate:` comment quotes the whole cell, bodies included).
fn block_body_for_case<'a>(src: &'a str, case: &str) -> &'a str {
    let block = block_for_case(src, case);
    let open = block
        .find("{\n")
        .unwrap_or_else(|| panic!("no block body for case {case}"));
    let body = &block[open + 2..];
    let end = body.rfind('}').unwrap_or(body.len());
    &body[..end]
}

// --- per-case extraction and emission ---

#[test]
fn law_case_bodies_emit_one_block_per_case() {
    // properties_to_proptest: a law row compiles to one block per
    // enumerated case — the cardinality is unchanged by Revision 21.
    let spec = parse_str(LAW_WITH_BODIES).unwrap_p();
    let src = properties_to_proptest(&spec);
    let blocks = blocks_from_source(&src);
    assert_eq!(blocks.len(), 3, "one block per case: {src}");
    let cases: Vec<Option<&str>> = blocks.iter().map(|b| b.case.as_deref()).collect();
    assert_eq!(
        cases,
        vec![Some("identity"), Some("associativity"), Some("naturality")],
        "the case labels stay the enumeration of record: {src}"
    );
}

#[test]
fn law_case_body_is_emitted_verbatim_for_its_case() {
    // The body of a case is that case's block body, verbatim — never
    // another case's body, never a placeholder.
    let spec = parse_str(LAW_WITH_BODIES).unwrap_p();
    let src = properties_to_proptest(&spec);
    let identity = block_body_for_case(&src, "identity");
    assert!(
        identity.contains("assert!(\n            v0.len() >= 1,"),
        "the identity case carries its own body: {identity}"
    );
    assert!(
        !identity.contains("v0.len() >= 2"),
        "a case never carries another case's body: {identity}"
    );
    let associativity = block_body_for_case(&src, "associativity");
    assert!(
        associativity.contains("assert!(\n            v0.len() >= 2,"),
        "the associativity case carries its own body: {associativity}"
    );
}

#[test]
fn law_case_without_body_keeps_the_placeholder() {
    // A case with no body keeps the honest `todo_predicate!`
    // placeholder — never a passing stub, never a fabricated body.
    let spec = parse_str(LAW_WITH_BODIES).unwrap_p();
    let src = properties_to_proptest(&spec);
    let naturality = block_body_for_case(&src, "naturality");
    assert!(
        naturality.contains("todo_predicate!("),
        "a case without a body keeps the placeholder: {naturality}"
    );
    assert!(
        !naturality.contains("assert!("),
        "a case without a body never gets a fabricated assertion: {naturality}"
    );
}

// --- labeled failures ---

#[test]
fn fragment_without_a_case_label_is_labeled() {
    // A fragment in fragment position that no case key introduces is a
    // labeled extraction failure naming the rule and the case — never a
    // silently ignored marker.
    let text = LAW_WITH_BODIES.replace(
        "**identity::** `**rust:** v0.len() >= 1`",
        "`**rust:** v0.len() >= 1`",
    );
    let spec = parse_str(&text).unwrap_p();
    let e = compile_spec(&spec).unwrap_p_err();
    assert_eq!(e.stage, "fragment_extraction", "{}", e.message);
    assert!(e.message.contains("law"), "{}", e.message);
    assert!(e.message.contains("case"), "{}", e.message);
}

#[test]
fn duplicate_body_for_one_case_is_labeled() {
    // Two bodies for one case are a labeled failure naming the case.
    let text = LAW_WITH_BODIES.replace(
        "**identity::** `**rust:** v0.len() >= 1`",
        "**identity::** `**rust:** v0.len() >= 1` **identity::** `**rust:** v0.len() >= 3`",
    );
    let spec = parse_str(&text).unwrap_p();
    let e = compile_spec(&spec).unwrap_p_err();
    assert_eq!(e.stage, "fragment_extraction", "{}", e.message);
    assert!(e.message.contains("law"), "{}", e.message);
    assert!(e.message.contains("case"), "{}", e.message);
    assert!(e.message.contains("identity"), "{}", e.message);
}

#[test]
fn body_for_an_unenumerated_case_is_labeled() {
    // A body whose key names a case the predicate does not enumerate is
    // a labeled failure naming the case.
    let text = LAW_WITH_BODIES.replace(
        "**naturality:** `g(f(a)) == g(a)`",
        "**naturality:** `g(f(a)) == g(a)` **commutativity::** `**rust:** v0.len() >= 4`",
    );
    let spec = parse_str(&text).unwrap_p();
    let e = compile_spec(&spec).unwrap_p_err();
    assert_eq!(e.stage, "fragment_extraction", "{}", e.message);
    assert!(e.message.contains("law"), "{}", e.message);
    assert!(e.message.contains("case"), "{}", e.message);
    assert!(e.message.contains("commutativity"), "{}", e.message);
}

#[test]
fn empty_case_body_is_labeled() {
    // A case key whose body is empty is a labeled failure naming the
    // case — never an emitted empty assertion.
    let text = LAW_WITH_BODIES.replace(
        "**identity::** `**rust:** v0.len() >= 1`",
        "**identity::** `**rust:**`",
    );
    let spec = parse_str(&text).unwrap_p();
    let e = compile_spec(&spec).unwrap_p_err();
    assert_eq!(e.stage, "fragment_extraction", "{}", e.message);
    assert!(e.message.contains("law"), "{}", e.message);
    assert!(e.message.contains("case"), "{}", e.message);
    assert!(e.message.contains("identity"), "{}", e.message);
}

#[test]
fn case_key_without_a_body_is_labeled() {
    // A case key that introduces no body is a labeled failure naming
    // the case — never a silently ignored marker.
    let text =
        LAW_WITH_BODIES.replace("**identity::** `**rust:** v0.len() >= 1`", "**identity::**");
    let spec = parse_str(&text).unwrap_p();
    let e = compile_spec(&spec).unwrap_p_err();
    assert_eq!(e.stage, "fragment_extraction", "{}", e.message);
    assert!(e.message.contains("law"), "{}", e.message);
    assert!(e.message.contains("case"), "{}", e.message);
    assert!(e.message.contains("identity"), "{}", e.message);
}

#[test]
fn case_body_on_a_non_law_row_is_labeled() {
    // The per-case body grammar is law-only — a unit row carrying a
    // case key is a labeled failure, never a silently ignored marker.
    let text = LAW_WITH_BODIES.replace(
        "| p_natural | law | [[demo.cases.a]] |",
        "| p_natural | unit | [[demo.cases.a]] |",
    );
    let spec = parse_str(&text).unwrap_p();
    let e = compile_spec(&spec).unwrap_p_err();
    assert_eq!(e.stage, "fragment_extraction", "{}", e.message);
    assert!(e.message.contains("law"), "{}", e.message);
    assert!(e.message.contains("case"), "{}", e.message);
    assert!(e.message.contains("identity"), "{}", e.message);
}

// --- label/marker interaction ---

#[test]
fn the_fragment_marker_is_never_a_case_label() {
    // The `**rust:**` marker is a fragment marker, never a case label:
    // a law row whose only `**name:**` occurrence is the marker does not
    // enumerate the identity/associativity floor.
    let text = LAW_WITH_BODIES.replace("**identity:** `f(a) == a` ", "");
    let spec = parse_str(&text).unwrap_p();
    let report = lint_corpus(std::slice::from_ref(&spec));
    assert!(
        report
            .issues
            .iter()
            .any(|i| i.rule_id == "linter.law_cases"),
        "the marker is not a case label: {:?}",
        report.issues
    );
}

#[test]
fn law_row_with_bodies_but_no_labels_fails_law_cases() {
    // A law row that carries bodies but enumerates no case label still
    // fails `law_cases` — the bodies never replace the enumeration.
    let text = LAW_WITH_BODIES
        .replace("**identity:** `f(a) == a` ", "")
        .replace("**associativity:** `f(f(a)) == f(a)` ", "")
        .replace("**naturality:** `g(f(a)) == g(a)` ", "");
    let spec = parse_str(&text).unwrap_p();
    let report = lint_corpus(std::slice::from_ref(&spec));
    assert!(
        report
            .issues
            .iter()
            .any(|i| i.rule_id == "linter.law_cases"),
        "bodies never replace the enumeration: {:?}",
        report.issues
    );
}

#[test]
fn empty_label_set_falls_back_to_the_floor() {
    // Defense in depth: a law row with no case label at all still
    // compiles to the identity/associativity floor blocks.
    let text = LAW_WITH_BODIES
        .replace("**identity:** `f(a) == a` ", "")
        .replace("**associativity:** `f(f(a)) == f(a)` ", "")
        .replace("**naturality:** `g(f(a)) == g(a)` ", "");
    let spec = parse_str(&text).unwrap_p();
    let src = properties_to_proptest(&spec);
    let blocks = blocks_from_source(&src);
    let cases: Vec<Option<&str>> = blocks.iter().map(|b| b.case.as_deref()).collect();
    assert_eq!(
        cases,
        vec![Some("identity"), Some("associativity")],
        "the floor survives an empty label set: {src}"
    );
}

// --- the helper rule reaches per-case bodies ---

/// A law row whose case body references a shared helper, plus the
/// `## Fragments` section that defines it.
const LAW_WITH_HELPER: &str = "---\nid: demo.cases.helper\nkind: intent\nstatement: \"THE demo SHALL carry a per-case body that uses a helper\"\n---\n\n## Constraints\n\n| id | kind | expr | traces_to |\n|----|------|------|-----------|\n| a | invariant | `holds` | [[demo.cases.helper]] |\n\n## Model\n\n### States\n\n- s1\n- s2\n\n### Transitions\n\n| id | from | to | guard |\n|----|------|----|-------|\n| t1 | s1 | s2 | [[demo.cases.helper.a]] |\n\n## Properties\n\n| id | kind | derives_from | generator | predicate |\n|----|------|--------------|-----------|------------|\n| p_law | law | [[demo.cases.helper.a]] | `word()` | **identity:** `f(a) == a` **identity::** `**rust:** @is_short(&v0)` **associativity:** `f(f(a)) == f(a)` |\n\n## Fragments\n\ndef is_short:\n\n```rust\nfn is_short(s: &str) -> bool { s.len() < 3 }\n```\n";

#[test]
fn a_helper_referenced_by_a_case_body_inlines_at_the_use_site() {
    // The helper rule reaches per-case bodies: the referenced body
    // lands in that case's block, and no `@` reference survives.
    let spec = parse_str(LAW_WITH_HELPER).unwrap_p();
    let src = properties_to_proptest(&spec);
    let identity = block_body_for_case(&src, "identity");
    assert!(
        identity.contains("fn is_short(s: &str) -> bool { s.len() < 3 }"),
        "the helper body inlines at the use site: {identity}"
    );
    assert!(
        !identity.contains('@'),
        "no helper reference marker survives into the artifact: {identity}"
    );
}

#[test]
fn an_unknown_helper_reference_in_a_case_body_is_labeled() {
    // An unresolvable helper reference in a per-case body is a labeled
    // failure — never a silently emitted `@name`.
    let text = LAW_WITH_HELPER.replace("@is_short(&v0)", "@nope(&v0)");
    let spec = parse_str(&text).unwrap_p();
    let e = compile_spec(&spec).unwrap_p_err();
    assert_eq!(e.stage, "fragment_extraction", "{}", e.message);
    assert!(e.message.contains("nope"), "{}", e.message);
}

#[test]
fn a_helper_used_only_by_a_case_body_is_not_dead() {
    // A helper a per-case body references is live — never reported as
    // a dead helper on compile's warnings channel.
    let spec = parse_str(LAW_WITH_HELPER).unwrap_p();
    let compiled = compile_spec(&spec).unwrap_p();
    assert!(
        !compiled.warnings.iter().any(|w| w.contains("is_short")),
        "a helper a case body uses is live: {:?}",
        compiled.warnings
    );
}

// --- back-compat ---

#[test]
fn legacy_law_row_compiles_byte_identically() {
    // rust_back_compat: a cell that carried no case body compiles
    // exactly as it did before Revision 21.
    let spec = parse_str(LAW_LEGACY).unwrap_p();
    let src = properties_to_proptest(&spec);
    assert_eq!(
        blocks_section(&src),
        LAW_LEGACY_BLOCKS,
        "the pre-Revision-21 artifact stays byte-identical"
    );
}
