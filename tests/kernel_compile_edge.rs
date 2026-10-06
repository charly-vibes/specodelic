//! Compile-level kernel edge cases — Tier B core of add-min-expr-kernel
//! (specodelic-7ga, RO5U review stage-4 pins).
//!
//! The compile leg's kernel handling at its labeled edges: a
//! `**kernel:**` marker on a non-invariant Constraint cell is NOT a
//! defined surface this Revision — it fails labeled (the kernel owns
//! invariant expr cells only); an invariant cell that opts in AND
//! breaks the closed grammar fails labeled at compile with the stage
//! name `kernel_grammar`; a double opt-in (`**kernel:**` +
//! `**rust:**`) fails labeled; and a fully valid kernel cell compiles
//! without disturbing the prose/citation rows around it.
//!
//! These live out-of-module as new integration tests (the ticket's
//! preferred split; src/model_check.rs is pinned by the pretender
//! file_lines ratchet — shrink-only).

use specodelic::compile::compile_spec;
use specodelic::spec::parse_str;

const BASE: &str = "---\nid: demo.edge\nkind: intent\nstatement: \"THE edge SHALL exist\"\n---\n\n## Constraints\n\n| id | kind | expr | traces_to |\n|----|------|------|-----------|\n| a | invariant | `holds a` | [[demo.edge]] |\n| b | effect | `y fires` | [[demo.edge]] |\n\n## Model\n\n### States\n\n- s1\n\n### Transitions\n\n| id | from | to | guard |\n|----|------|----|-------|\n| t1 | s1 | s1 | [[demo.edge.a]] |\n\n## Properties\n\n| id | kind | derives_from | generator | predicate |\n|----|------|--------------|-----------|-----------|\n| p1 | unit | [[demo.edge.a]] | `arb()` | `check(x) == ok` |\n";

#[test]
fn kernel_marker_on_a_non_invariant_cell_fails_labeled() {
    // The kernel owns invariant expr cells only this Revision; a
    // kernel marker on an effect/advisory cell is a labeled failure
    // (the unknown-tag path), never a silent ignore.
    let text = BASE.replace(
        "| b | effect | `y fires` | [[demo.edge]] |",
        "| b | effect | `**kernel:** acyclic(supersedes)` | [[demo.edge]] |",
    );
    let spec = parse_str(&text).expect("parses");
    let err = compile_spec(&spec).expect_err("non-invariant kernel marker must fail labeled");
    assert_eq!(err.stage, "fragment_extraction", "{err:?}");
    assert!(err.message.contains("kernel"), "{err:?}");
}

#[test]
fn broken_kernel_grammar_fails_labeled_at_compile() {
    // An invariant cell that opts in and breaks the closed grammar
    // fails labeled at compile, stage kernel_grammar, naming the
    // offending atomic and the closed set.
    let text = BASE.replace(
        "| a | invariant | `holds a` | [[demo.edge]] |",
        "| a | invariant | `**kernel:** ∀ c ∈ Constraint: frobnicate(c)` | [[demo.edge]] |",
    );
    let spec = parse_str(&text).expect("parses");
    let err = compile_spec(&spec).expect_err("non-member atomic must fail labeled");
    assert_eq!(err.stage, "kernel_grammar", "{err:?}");
    assert!(err.message.contains("frobnicate"), "{err:?}");
    assert!(err.message.contains("kernel_grammar_closed"), "{err:?}");
}

#[test]
fn double_opt_in_fails_labeled() {
    // A cell claiming both **kernel:** and an executable **lang:**
    // fragment is labeled — never a silent pick of one.
    let text = BASE.replace(
        "| a | invariant | `holds a` | [[demo.edge]] |",
        "| a | invariant | `**kernel:** acyclic(supersedes)` `**rust:** true` | [[demo.edge]] |",
    );
    let spec = parse_str(&text).expect("parses");
    let err = compile_spec(&spec).expect_err("double opt-in must fail labeled");
    assert!(err.message.contains("**kernel:**"), "{err:?}");
}

#[test]
fn valid_kernel_cell_compiles_besides_prose_and_citations() {
    // A valid kernel cell compiles; the prose and citation rows around
    // it keep their extraction untouched (coexistence of the three
    // expr-cell languages in one table).
    let text = BASE.replace(
        "| a | invariant | `holds a` | [[demo.edge]] |",
        "| a | invariant | `**kernel:** acyclic(supersedes)` | [[demo.edge]] |",
    );
    let spec = parse_str(&text).expect("parses");
    let compiled = compile_spec(&spec).expect("compiles");
    assert_eq!(
        compiled.model_ir.guard_kernel.get("a").map(String::as_str),
        Some("acyclic(supersedes)")
    );
    assert!(!compiled.model_ir.guard_citations.contains_key("a"));
}
