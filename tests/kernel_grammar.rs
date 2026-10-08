//! The kernel grammar — Tier B core of add-min-expr-kernel
//! (specodelic-7ga, tasks.md §3.1 RED; design D1/CORR-001).
//!
//! Grammar-scope tests for the closed kernel atomic set: kernel
//! expressions in expr cells parse under the closed grammar (per-cell
//! **kernel:** marker opt-in per kernel_expr_opt_in, specs/compile.md —
//! the citation precedent that a marker decides), a non-member atomic fails
//! labeled naming the atomic AND the closed set (sd1 discipline), prose
//! expr cells stay prose and compile byte-identically (pure widening),
//! and the slice-1 citation grammar is never stolen by the kernel.
//!
//! These live out-of-module as new integration tests (the ticket's
//! preferred split; src/model_check.rs is pinned by the pretender
//! file_lines ratchet — shrink-only).

use specodelic::compile::{ThreeValued, compile_spec, extract_model_ir};
use specodelic::kernel::{
    Atomic, KernelError, KernelExpr, Term, parse_kernel_expr, parse_kernel_str,
};
use specodelic::spec::parse_str;

/// A prose fixture: one invariant row with a prose expr cell, one
/// invariant row with a slice-1 citation expr cell, one effect row —
/// the exact pre-kernel shape nothing in phase 3 may disturb.
const PROSE_SAMPLE: &str = "---\nid: demo.prose\nkind: intent\nstatement: \"THE system SHALL hold\"\n---\n\n## Constraints\n\n| id | kind | expr | traces_to |\n|----|------|------|-----------|\n| pr | invariant | `every row holds its own shape` | [[demo.prose]] |\n| ci | invariant | `[[demo.prose.pr]] ∧ ¬[[demo.prose.fx]]` | [[demo.prose]] |\n| fx | effect | `y fires` | [[demo.prose]] |\n\n## Model\n\n### States\n\n- s1\n- s2 (emits: `[[demo.prose.fx]]`)\n\n### Transitions\n\n| id | from | to | guard |\n|----|------|----|-------|\n| t1 | s1 | s2 | [[demo.prose.pr]] |\n\n## Properties\n\n| id | kind | derives_from | generator | predicate |\n|----|------|--------------|-----------|-----------|\n| p1 | unit | [[demo.prose.pr]] | `arbitrary_row()` | `check(x) == ok` |\n";

fn prose_sample() -> specodelic::spec::Spec {
    parse_str(PROSE_SAMPLE).expect("prose fixture parses")
}

// --- 3.1 RED: closed atomic set accepts ---

#[test]
fn closed_atomic_set_parses() {
    // Scenario: Closed atomic set accepts and rejects — every closed
    // form parses into the kernel grammar.
    // Reference atomics.
    assert!(matches!(
        parse_kernel_str("resolves(traces_to)"),
        Ok(Some(KernelExpr::Atomic(_)))
    ));
    assert!(matches!(
        parse_kernel_str("unique(traces_to)"),
        Ok(Some(KernelExpr::Atomic(_)))
    ));
    assert!(matches!(
        parse_kernel_str("acyclic(supersedes)"),
        Ok(Some(KernelExpr::Atomic(_)))
    ));
    assert!(matches!(
        parse_kernel_str("reachable(demo.prose.pr, demo.prose.ci, supersedes)"),
        Ok(Some(KernelExpr::Atomic(_)))
    ));
    // Bounded quantifiers over I(k) with conjunction and negation.
    assert!(matches!(
        parse_kernel_str("∀ c ∈ Constraint: resolves(traces_to)"),
        Ok(Some(KernelExpr::ForAll(_, _, _)))
    ));
    assert!(matches!(
        parse_kernel_str("∃ c ∈ Constraint: unique(supersedes)"),
        Ok(Some(KernelExpr::Exists(_, _, _)))
    ));
    assert!(matches!(
        parse_kernel_str("¬acyclic(supersedes)"),
        Ok(Some(KernelExpr::Not(_)))
    ));
    // Equality and comparisons over bounded terms.
    assert!(matches!(
        parse_kernel_str("|Constraint| == 3"),
        Ok(Some(KernelExpr::Atomic(_)))
    ));
    assert!(matches!(
        parse_kernel_str("∀ c ∈ Constraint: |Constraint| >= 1"),
        Ok(Some(KernelExpr::ForAll(_, _, _)))
    ));
}

// --- 3.1 RED: non-member atomic fails labeled (sd1 discipline) ---

#[test]
fn nonmember_atomic_fails_labeled_naming_atomic_and_set() {
    // Scenario: Closed atomic set accepts and rejects — an expression
    // using a non-member atomic is a labeled failure naming the atomic
    // AND the closed set, never prose, never silently ignored.
    let err = parse_kernel_str("∀ c ∈ Constraint: frobnicate(c)")
        .expect_err("non-member atomic must fail labeled");
    let message = match err {
        KernelError::Labeled { message } => message,
    };
    assert!(
        message.contains("frobnicate"),
        "label must name the offending atomic: {message}"
    );
    assert!(
        message.contains("resolves")
            && message.contains("unique")
            && message.contains("acyclic")
            && message.contains("reachable"),
        "label must name the closed set: {message}"
    );
}

#[test]
fn malformed_kernel_shaped_cell_fails_labeled_not_prose() {
    // An opted-in cell (a **kernel:** marker in fragment position)
    // whose content breaks the grammar is a labeled failure — never a
    // silent prose fallback.
    assert!(parse_kernel_str("∀ c ∈ Constraint:").is_err());
    assert!(parse_kernel_str("resolves(").is_err());
    // An empty marker is labeled too — never a silent no-op.
    assert!(parse_kernel_expr("`**kernel:**`").is_err());
}

// --- 3.1 RED: row-typing edge cases (RO5U stage-4 pins) ---

#[test]
fn row_typing_edge_cases_fail_labeled() {
    // acyclic requires an endo morphism — traces_to is Constraint →
    // Intent, not endo: labeled, never silent.
    let err = parse_kernel_str("acyclic(traces_to)").expect_err("non-endo");
    match err {
        KernelError::Labeled { message } => {
            assert!(message.contains("endo"), "{message}");
        }
    }
    // An unknown morphism name is labeled, naming the Reference Typing
    // rows.
    let err = parse_kernel_str("resolves(bogus)").expect_err("unknown morphism");
    match err {
        KernelError::Labeled { message } => {
            assert!(message.contains("bogus"), "{message}");
        }
    }
    // An unknown quantifier object is labeled, naming the closed five.
    let err = parse_kernel_str("∀ c ∈ Ghost: resolves(traces_to)").expect_err("unknown object");
    match err {
        KernelError::Labeled { message } => {
            assert!(message.contains("Ghost"), "{message}");
        }
    }
    // A projection of a morphism not sourced on the bounding object is
    // labeled (Reference Typing row-typing).
    let err = parse_kernel_str("∀ i ∈ Intent: i.supersedes == x").expect_err("ill-typed proj");
    match err {
        KernelError::Labeled { message } => {
            assert!(message.contains("not sourced on Intent"), "{message}");
        }
    }
    // Rule of record: a dot after an UNBOUND identifier is an id
    // literal's file-qualification, not a projection — the binding
    // decides. `c.supersedes == x` with no quantifier is the id
    // comparison `"c.supersedes" == "x"`, refuted over any instance
    // (distinct ids), never an unbound-variable crash.
    assert_eq!(
        parse_kernel_str("c.supersedes == x"),
        Ok(Some(KernelExpr::Atomic(Atomic::Eq(
            Term::Id("c.supersedes".into()),
            Term::Id("x".into()),
        ))))
    );
    // Nested shadowing binds innermost (the rev() lookup, pinned).
    assert!(matches!(
        parse_kernel_str("∀ c ∈ Constraint: ∀ c ∈ Intent: |Intent| >= 0"),
        Ok(Some(KernelExpr::ForAll(_, _, _)))
    ));
}

// --- 3.1 RED: pure widening — prose stays prose ---

#[test]
fn prose_expr_cell_stays_prose() {
    // Scenario: Prose expr cell untouched — a cell with no kernel
    // content parses to None (stays prose, inert as before).
    assert_eq!(parse_kernel_expr("every row holds its own shape"), Ok(None));
    assert_eq!(parse_kernel_expr("`x holds`"), Ok(None));
    assert_eq!(parse_kernel_expr(""), Ok(None));
}

#[test]
fn citation_cells_are_not_stolen_by_the_kernel() {
    // Slice-1 cells keep their owner: an unmarked citation expression
    // is a citation, never a kernel expression (the marker decides).
    assert_eq!(parse_kernel_expr("¬[[demo.prose.pr]]"), Ok(None));
    assert_eq!(parse_kernel_expr("[[demo.prose.pr]] ∧ [[a]]"), Ok(None));
}

#[test]
fn corpus_quantifier_prose_stays_prose() {
    // The corpus's existing ∀-led prose cells (specodelic.md's
    // unique_id, merge.md's no_new_id_collision) carry no marker and
    // stay prose — pure widening; phase 6 migrates them, not phase 3.
    assert_eq!(
        parse_kernel_expr("`∀ row ∈ file: unique(row.id)`"),
        Ok(None)
    );
    assert_eq!(
        parse_kernel_expr("`∀ id ∈ index(branch_A) ∩ index(branch_B): id was already defined`"),
        Ok(None)
    );
}

#[test]
fn midspan_kernel_syntax_is_a_mention() {
    // Scenario: Mid-span occurrence is a mention — a cell that mentions
    // kernel syntax mid-span extracts nothing (sd1 fragment-position
    // rule carries over).
    assert_eq!(
        parse_kernel_expr("the ∀ c ∈ Constraint: form is kernel syntax"),
        Ok(None)
    );
    assert_eq!(
        parse_kernel_expr("rows defining `acyclic(supersedes)` mid-span"),
        Ok(None)
    );
}

// --- 3.1 RED: prose expr cells compile byte-identically ---

#[test]
fn prose_expr_cells_compile_byte_identically() {
    // Scenario: Prose expr cell untouched / Nothing valid is
    // invalidated — the compiled artifacts (TOML, proptest, TLA) of a
    // prose corpus are byte-identical to the pre-kernel compilation.
    // Goldens captured from the pre-phase-3 compiler.
    let spec = prose_sample();
    let compiled = compile_spec(&spec).expect("prose fixture compiles");
    assert_eq!(
        compiled.toml,
        "source = \"demo.prose\"\n\n[[constraints]]\nid = \"pr\"\nkind = \"invariant\"\nexpr = \"`every row holds its own shape`\"\ntraces_to = \"[[demo.prose]]\"\n\n[[constraints]]\nid = \"ci\"\nkind = \"invariant\"\nexpr = \"`[[demo.prose.pr]] ∧ ¬[[demo.prose.fx]]`\"\ntraces_to = \"[[demo.prose]]\"\n\n[[constraints]]\nid = \"fx\"\nkind = \"effect\"\nexpr = \"`y fires`\"\ntraces_to = \"[[demo.prose]]\"\n"
    );
}

#[test]
fn prose_and_citation_extraction_unchanged() {
    // The IR keeps exactly the slice-1 extraction: the citation cell is
    // in guard_citations, the prose and effect cells stay out, and no
    // kernel field carries prose (pure widening, no behavior change for
    // the existing corpus).
    let spec = prose_sample();
    let ir = extract_model_ir(&spec);
    assert_eq!(
        ir.guard_citations.get("ci").map(String::as_str),
        Some("[[demo.prose.pr]] ∧ ¬[[demo.prose.fx]]")
    );
    assert!(!ir.guard_citations.contains_key("pr"));
    assert!(!ir.guard_citations.contains_key("fx"));
}

// --- 3.1 RED: opt-in lands in the compile IR (the extraction seam the
// later steps evaluate over) ---

#[test]
fn kernel_expr_cell_extracts_into_the_ir() {
    // Scenario: Opt-in extracts under the closed grammar — an invariant
    // expr cell carrying a kernel expression extracts into the IR's
    // kernel claims, keyed by constraint id; prose stays out.
    let text = PROSE_SAMPLE.replace(
        "| pr | invariant | `every row holds its own shape` | [[demo.prose]] |",
        "| pr | invariant | `**kernel:** acyclic(supersedes)` | [[demo.prose]] |",
    );
    let spec = parse_str(&text).expect("kernel fixture parses");
    let ir = extract_model_ir(&spec);
    assert_eq!(
        ir.guard_kernel.get("pr").map(String::as_str),
        Some("acyclic(supersedes)"),
        "kernel expr cell must extract into guard_kernel"
    );
    assert!(
        !ir.guard_kernel.contains_key("ci"),
        "citation cell stays a citation"
    );
    assert!(!ir.guard_kernel.contains_key("fx"), "effect expr stays out");
}

#[test]
fn three_valued_exists_and_composes() {
    // ThreeValued is reused from the slice-1 citation algebra, never
    // re-invented: the kernel's statuses ARE compile::ThreeValued.
    let _v: ThreeValued = ThreeValued::Verified;
    let _c: ThreeValued = ThreeValued::Counterexample;
    let _u: ThreeValued = ThreeValued::Unknown;
}
