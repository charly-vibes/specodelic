//! Content-advisory tests (openspec/specs/linter-content_advisories/spec.md):
//! whole-cell placeholders and tautological predicates. Kept as its own
//! submodule to stay under the library file_lines ratchet.

use super::*;

// ---- linter-content_advisories (openspec/specs/linter-content_advisories/spec.md) ----

/// A shape-clean corpus skeleton with ONE constraint and ONE property,
/// either of which a case varies. Its statement is prose and its
/// non-varied cells are clean, so every warning a case sees comes from
/// the cell under test — no accidental scaffold signature.
fn spec_with_cells(constraint_kind: &str, expr: &str, predicate: &str) -> Spec {
    spec_at(
        &format!(
            "---\nid: cells.form\nkind: intent\nstatement: \"THE cells SHALL hold\"\n---\n\n## Constraints\n\n| id | kind | expr | traces_to |\n|----|------|------|-----------|\n| c | {constraint_kind} | {expr} | [[cells.form]] |\n\n## Model\n\n### States\n\n- s1\n\n### Transitions\n\n| id | from | to | guard |\n|----|------|----|-------|\n| t | s1 | s1 | [[cells.form.s1]] |\n\n## Properties\n\n| id | kind | derives_from | generator | predicate |\n|----|------|--------------|-----------|------------|\n| p | unit | [[cells.form.c]] | `g()` | {predicate} |\n"
        ),
        "cells-form.md",
    )
}

/// Does linting `spec` alone warn with `rule` on the warnings channel?
fn warns(spec: &Spec, rule: &str) -> bool {
    lint_all(std::slice::from_ref(spec), &[])
        .warnings
        .iter()
        .any(|w| w.rule_id == rule)
}

const PLACEHOLDER: &str = "linter.placeholder_content";
const TAUTOLOGY: &str = "linter.tautological_predicate";

/// specodelic-xci (RED first): the two most common AI-author filler
/// shapes — an untouched `spk new` scaffold (a `scaffold_*` row with
/// a TODO expr, and the stub statement `THE system SHALL …`) and a
/// predicate that cannot fail — are warned on the warnings channel,
/// never gated as Issues.
#[test]
fn content_placeholder_and_tautology_warn_never_fail() {
    let scaffold = spec_at(
        "---\nid: ph.scaffold\nkind: intent\nstatement: \"THE ph SHALL hold\"\n---\n\n## Constraints\n\n| id | kind | expr | traces_to |\n|----|------|------|-----------|\n| scaffold_c1 | invariant | `TODO — what must always hold` | [[ph.scaffold]] |\n\n## Model\n\n### States\n\n- s1\n\n### Transitions\n\n| id | from | to | guard |\n|----|------|----|-------|\n| t | s1 | s1 | [[ph.scaffold.scaffold_c1]] |\n\n## Properties\n\n| id | kind | derives_from | generator | predicate |\n|----|------|--------------|-----------|------------|\n| p1 | unit | [[ph.scaffold.scaffold_c1]] | `g()` | `true` |\n",
        "ph-scaffold.md",
    );
    // The untouched `spk new` stub statement is a placeholder even
    // when every row around it is well-formed.
    let stub = spec_at(
        "---\nid: stub.intent\nkind: intent\nstatement: \"THE system SHALL …\"\n---\n\n## Constraints\n\n| id | kind | expr | traces_to |\n|----|------|------|-----------|\n| inv | invariant | `x` | [[stub.intent]] |\n\n## Model\n\n### States\n\n- s1\n\n### Transitions\n\n| id | from | to | guard |\n|----|------|----|-------|\n| t | s1 | s1 | [[stub.intent.inv]] |\n\n## Properties\n\n| id | kind | derives_from | generator | predicate |\n|----|------|--------------|-----------|------------|\n| p | unit | [[stub.intent.inv]] | `g()` | `x` |\n",
        "stub-intent.md",
    );
    let report = lint_all(&[scaffold, stub], &[]);
    assert_eq!(
        report.failures(),
        0,
        "content advisories must never be Issues: {:?}",
        report.issues
    );
    // Two placeholder classes fire on that file, and the assertions are
    // kept distinct — a TODO-cell message also contains the row id, so a
    // bare `contains("scaffold_c1")` would not prove the id class ran.
    assert!(
        report.warnings.iter().any(|w| w.rule_id == PLACEHOLDER
            && w.message.contains("whole-cell placeholder")
            && w.message.contains("scaffold_c1")
            && w.message.contains("TODO")),
        "the TODO cell warns as a whole-cell placeholder: {:?}",
        report.warnings
    );
    assert!(
        report.warnings.iter().any(|w| w.rule_id == PLACEHOLDER
            && w.message
                .contains("carries the `scaffold_` placeholder prefix")),
        "the scaffold row id warns on the id class: {:?}",
        report.warnings
    );
    let stub_warning = report
        .warnings
        .iter()
        .find(|w| {
            w.rule_id == PLACEHOLDER
                && (w.message.contains("stub.intent") || w.message.contains("THE system SHALL …"))
        })
        .unwrap_p();
    assert!(
        !stub_warning.message.is_empty(),
        "the stub-statement warning carries a message"
    );
    let tautology = report
        .warnings
        .iter()
        .find(|w| w.rule_id == TAUTOLOGY)
        .unwrap_p();
    assert!(
        tautology.message.contains("p1"),
        "the tautology warning names the offending row: {}",
        tautology.message
    );
}

/// The placeholder class is table-driven over every `placeholder_classes`
/// word: TODO, TBD, and FIXME are placeholders bare, with a separator
/// note, and inside a code span — while word-like prose, lowercase
/// markers, and a mid-cell word stay silent.
#[test]
fn placeholder_words_warn_table_driven() {
    for cell in [
        "`TODO`",
        "`TBD`",
        "`FIXME`",
        "`TODO — fill me in`",
        "`TBD: pending review`",
        "FIXME - name the rule",
    ] {
        let spec = spec_with_cells("invariant", cell, "`x`");
        assert!(
            warns(&spec, PLACEHOLDER),
            "`{cell}` is a whole-cell placeholder and must warn"
        );
    }
    for cell in [
        "`TODOESCAPE_IS_FINE`",
        "`todo`",
        "`x TODO y`",
        "`nothing to fill`",
    ] {
        let spec = spec_with_cells("invariant", cell, "`x`");
        assert!(
            !warns(&spec, PLACEHOLDER),
            "`{cell}` is not a whole-cell placeholder and must stay silent"
        );
    }
}

/// The tautology class is NARROW and table-driven: the whole cell (after
/// stripping its code span and any `**rust:**` marker) is one of the
/// accepted `true` spellings or `⊤`, or one identifier compared equal to
/// itself. A second identifier or arithmetic is not judged.
#[test]
fn tautology_forms_are_narrow() {
    for form in [
        "`true`",
        "`TRUE`",
        "`True`",
        "`⊤`",
        "`x == x`",
        "`**rust:** true`",
    ] {
        let spec = spec_with_cells("invariant", "`x`", form);
        assert!(
            warns(&spec, TAUTOLOGY),
            "`{form}` is one of the narrow tautology forms and must warn"
        );
    }
    for form in ["`x == y`", "`x == x + 1`"] {
        let spec = spec_with_cells("invariant", "`x`", form);
        assert!(
            !warns(&spec, TAUTOLOGY),
            "`{form}` is outside the narrow tautology class and must stay silent"
        );
    }
}

/// H1: the tautology class judges an INVARIANT constraint expr and a
/// property predicate — never an `effect`/`advisory` expr, which is a
/// label or a hint rather than a claim that must be able to fail.
#[test]
fn tautology_judges_invariant_exprs_not_effect_exprs() {
    for kind in ["effect", "advisory"] {
        let spec = spec_with_cells(kind, "`true`", "`x`");
        assert!(
            !warns(&spec, TAUTOLOGY),
            "a `{kind}` expr is never tautology-judged"
        );
    }
    let invariant = spec_with_cells("invariant", "`true`", "`x`");
    assert!(
        warns(&invariant, TAUTOLOGY),
        "an invariant expr that cannot fail must warn"
    );
}

/// M1: the tautology class excludes statements — a statement that is
/// exactly `true` is judged by the EARS grammar, not here.
#[test]
fn statement_is_never_tautology_judged() {
    let spec = spec_at(
        "---\nid: stmt.form\nkind: intent\nstatement: \"true\"\n---\n\n## Constraints\n\n| id | kind | expr | traces_to |\n|----|------|------|-----------|\n| c | invariant | `x` | [[stmt.form]] |\n\n## Model\n\n### States\n\n- s1\n\n### Transitions\n\n| id | from | to | guard |\n|----|------|----|-------|\n| t | s1 | s1 | [[stmt.form.c]] |\n\n## Properties\n\n| id | kind | derives_from | generator | predicate |\n|----|------|--------------|-----------|------------|\n| p | unit | [[stmt.form.c]] | `g()` | `x` |\n",
        "stmt-form.md",
    );
    assert!(
        !warns(&spec, TAUTOLOGY),
        "an intent statement is outside the tautology class"
    );
}

/// M3: the `scaffold_` id class is gated on the SIGNATURE, not on the
/// row's own cells. A `spk new` file whose only scaffold evidence is the
/// stub statement still warns on its otherwise-clean `scaffold_c1` row —
/// and that id-class warning is distinct from any whole-cell message.
#[test]
fn stub_statement_alone_makes_scaffold_ids_warn() {
    let spec = spec_at(
        "---\nid: stub.signature\nkind: intent\nstatement: \"THE system SHALL …\"\n---\n\n## Constraints\n\n| id | kind | expr | traces_to |\n|----|------|------|-----------|\n| scaffold_c1 | invariant | `x` | [[stub.signature]] |\n\n## Model\n\n### States\n\n- s1\n\n### Transitions\n\n| id | from | to | guard |\n|----|------|----|-------|\n| t | s1 | s1 | [[stub.signature.scaffold_c1]] |\n\n## Properties\n\n| id | kind | derives_from | generator | predicate |\n|----|------|--------------|-----------|------------|\n| p | unit | [[stub.signature.scaffold_c1]] | `g()` | `x` |\n",
        "stub-signature.md",
    );
    let report = lint_all(&[spec], &[]);
    assert!(
        report.warnings.iter().any(|w| w.rule_id == PLACEHOLDER
            && w.message
                .contains("carries the `scaffold_` placeholder prefix")),
        "the stub-only signature must gate the scaffold id class: {:?}",
        report.warnings
    );
    assert!(
        report
            .warnings
            .iter()
            .all(|w| !w.message.contains("expr is a whole-cell placeholder")
                && !w.message.contains("predicate is a whole-cell placeholder")),
        "the row's own cells are clean, so no cell-class warning rides along: {:?}",
        report.warnings
    );
}

/// Fragment interiors are never scanned: `todo_predicate!("TODO")`
/// inside a `**rust:**` cell is legitimate executable code, not a
/// whole-cell placeholder.
#[test]
fn fragment_interior_is_not_a_placeholder() {
    let spec = spec_at(
        "---\nid: frag.form\nkind: intent\nstatement: \"THE frag SHALL hold\"\n---\n\n## Constraints\n\n| id | kind | expr | traces_to |\n|----|------|------|-----------|\n| inv | invariant | `x` | [[frag.form]] |\n\n## Model\n\n### States\n\n- s1\n\n### Transitions\n\n| id | from | to | guard |\n|----|------|----|-------|\n| t | s1 | s1 | [[frag.form.inv]] |\n\n## Properties\n\n| id | kind | derives_from | generator | predicate |\n|----|------|--------------|-----------|------------|\n| p | unit | [[frag.form.inv]] | `g()` | `**rust:** todo_predicate!(\"TODO\")` |\n",
        "frag-form.md",
    );
    let report = lint_all(&[spec], &[]);
    assert!(
        report.warnings.iter().all(|w| w.rule_id != PLACEHOLDER),
        "a **rust:** fragment interior is never a placeholder: {:?}",
        report.warnings
    );
}

/// A real row that merely NAMES the scaffold (`scaffold_teaches` — the
/// corpus's own embedded-guide shape) is documentation, not filler:
/// `placeholder_classes` gates the `scaffold_` id class on the file
/// still carrying the scaffold signature (a placeholder statement, or a
/// placeholder/tautological content cell). Without the gate a
/// legitimate corpus row would warn forever.
#[test]
fn scaffold_id_prefix_without_scaffold_signature_is_silent() {
    let spec = spec_at(
        "---\nid: guide.real\nkind: intent\nstatement: \"THE guide SHALL teach the scaffold\"\n---\n\n## Constraints\n\n| id | kind | expr | traces_to |\n|----|------|------|-----------|\n| scaffold_teaches | invariant | `the generated scaffold is documented` | [[guide.real]] |\n\n## Model\n\n### States\n\n- s1\n\n### Transitions\n\n| id | from | to | guard |\n|----|------|----|-------|\n| t | s1 | s1 | [[guide.real.scaffold_teaches]] |\n\n## Properties\n\n| id | kind | derives_from | generator | predicate |\n|----|------|--------------|-----------|------------|\n| p | unit | [[guide.real.scaffold_teaches]] | `g()` | `x` |\n",
        "guide-real.md",
    );
    let report = lint_all(&[spec], &[]);
    assert_eq!(report.failures(), 0, "{:?}", report.issues);
    assert!(
        report.warnings.iter().all(|w| w.rule_id != PLACEHOLDER),
        "a scaffold-named real row is not a placeholder: {:?}",
        report.warnings
    );
}
