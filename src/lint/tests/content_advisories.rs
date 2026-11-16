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
/// specodelic-h5ks: an executable `**rust:**` fragment that cites no
/// declared model element (state/transition/constraint/property) — green
/// lint, nothing actually checked (the bowling-rev1 shape).
const FRAGMENT_BINDS_NOTHING: &str = "linter.fragment_binds_nothing";
/// specodelic-h5ks: an invariant whose forbidden state is not declared —
/// the bad state cannot exist, so the invariant is structurally vacuous.
const VACUOUS_INVARIANT: &str = "linter.vacuous_invariant";

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

// ---- specodelic-h5ks: vacuity advisories (extension of xci) ----
//
// The kata study's vacuity class: a corpus can lint green while nothing
// is actually checked. Two shapes carry it, and both are advisories —
// warnings-channel only (exit 0, folded into `Report::warnings`, never
// `Report::issues`), file-scoped, and never a nudge off legitimate
// design pressure. Declared model elements are the citation target:
// the Model's state and transition ids plus every Constraint/Property
// row id — never the intent id, which is the file's own identity.

/// h5ks: a `**rust:**` fragment that mentions none of the file's
/// declared model elements — states, transitions, constraints,
/// properties — binds nothing: it is green lint over a fragment that
/// never touches the model (bowling rev1's scorer was never called).
/// The finding is FILE-SCOPED (the study's review shrank a 53-row list
/// to a handful of lines): ONE summary warning per file, carrying the
/// count and the id of EVERY offending row — a summary is not a
/// weakening, so this fixture has two offenders and the assertion
/// demands both. It rides the warnings channel, so the run still exits
/// 0 (`failures() == 0` is the exit-code proxy: the CLI exits nonzero
/// only on issues).
#[test]
fn fragment_citing_no_model_element_warns_binds_nothing() {
    let spec = spec_at(
        r#"---
id: demo.vac
kind: intent
statement: "THE demo SHALL keep the ledger consistent"
---

## Constraints

| id | kind | expr | traces_to |
|----|------|------|-----------|
| vacuous | invariant | `**rust:** flag != 0` | [[demo.vac]] |

## Model

### States

- held

### Transitions

| id | from | to | guard |
|----|------|----|-------|
| advance | held | held | [[demo.vac.vacuous]] |

## Properties

| id | kind | derives_from | generator | predicate |
|----|------|--------------|-----------|------------|
| p | unit | [[demo.vac.vacuous]] | `g()` | `**rust:** score >= 0` |
"#,
        "demo-vac.md",
    );
    let report = lint_all(&[spec], &[]);
    assert_eq!(
        report.failures(),
        0,
        "the vacuity advisory is warnings-only (exit 0): {:?}",
        report.issues
    );
    assert!(
        report
            .issues
            .iter()
            .all(|issue| issue.rule_id != FRAGMENT_BINDS_NOTHING),
        "a fragment advisory must never ride the issues channel: {:?}",
        report.issues
    );
    let hits: Vec<&Issue> = report
        .warnings
        .iter()
        .filter(|w| w.rule_id == FRAGMENT_BINDS_NOTHING)
        .collect();
    assert_eq!(
        hits.len(),
        1,
        "file-scoped: exactly one summary warning per file, got {:?}",
        report.warnings
    );
    let warning = hits[0];
    assert!(
        warning.message.contains("2 fragment(s)"),
        "the summary states the offender count: {}",
        warning.message
    );
    assert!(
        warning.message.contains("`vacuous`") && warning.message.contains("`p`"),
        "the summary names EVERY offending row id: {}",
        warning.message
    );
    assert_eq!(
        warning.file, "demo-vac.md",
        "the advisory is file-scoped: {:?}",
        warning
    );
}

/// h5ks (RED first): an invariant that forbids a named state which the
/// Model never declares — `aborted` appears in no `### States` bullet —
/// is structurally vacuous: the bad state cannot exist, so the claim can
/// never fail. The advisory names the undeclared state, stays on the
/// warnings channel, and never fails the run.
#[test]
fn invariant_citing_undeclared_forbidden_state_warns() {
    let spec = spec_at(
        r#"---
id: demo.unbound
kind: intent
statement: "THE demo SHALL stay within its phases"
---

## Constraints

| id | kind | expr | traces_to |
|----|------|------|-----------|
| never_aborted | invariant | `**rust:** state != "aborted"` | [[demo.unbound]] |

## Model

### States

- running
- done

### Transitions

| id | from | to | guard |
|----|------|----|-------|
| finish | running | done | [[demo.unbound.never_aborted]] |

## Properties

| id | kind | derives_from | generator | predicate |
|----|------|--------------|-----------|------------|
| p | unit | [[demo.unbound.never_aborted]] | `g()` | `x` |
"#,
        "demo-unbound.md",
    );
    let report = lint_all(&[spec], &[]);
    assert_eq!(
        report.failures(),
        0,
        "the vacuity advisory is warnings-only (exit 0): {:?}",
        report.issues
    );
    assert!(
        report
            .issues
            .iter()
            .all(|issue| issue.rule_id != VACUOUS_INVARIANT),
        "a structurally-vacuous invariant is an advisory, never an Issue: {:?}",
        report.issues
    );
    let warning = report
        .warnings
        .iter()
        .find(|w| w.rule_id == VACUOUS_INVARIANT)
        .unwrap_or_else(|| {
            panic!(
                "`aborted` is undeclared, so `never_aborted` must warn {VACUOUS_INVARIANT}: {:?}",
                report.warnings
            )
        });
    assert!(
        warning.message.contains("never_aborted") && warning.message.contains("aborted"),
        "the advisory names the row and the undeclared forbidden state: {}",
        warning.message
    );
    assert_eq!(
        warning.file, "demo-unbound.md",
        "the advisory is file-scoped: {:?}",
        warning
    );
    // The literal names an UNDECLARED state, so it is not a citation: the
    // fragment still binds nothing, and the file-scoped summary names the
    // row. The ghost literal is this rule's surface — never silently
    // accepted by the binding check.
    assert!(
        report.warnings.iter().any(|w| {
            w.rule_id == FRAGMENT_BINDS_NOTHING && w.message.contains("never_aborted")
        }),
        "an undeclared state literal must not satisfy the citation check: {:?}",
        report.warnings
    );
}

/// h5ks: the DECLARED-bad-state nuance, pinned against a reachability
/// implementation. A correct phase-style model may keep an invariant
/// over a phase its design never enters — legitimately vacuous
/// behavior, not a defect. Here `halted` IS declared in `### States`
/// but is genuinely UNREACHABLE from the first-listed state: its only
/// transition is a self-loop (`halt | halted | halted`), with no
/// from/to path from `phase_a`/`phase_b`. A check that judged
/// reachability from a seed state would flag it; the check is
/// citation-versus-declared-ids, so no vacuity advisory may fire. The
/// component still anchors to the intent through `halt`'s guard
/// citation, so the corpus stays lint-clean (no issues, exit 0); "info
/// at most" is satisfied by silence, the `Issue` model carrying no
/// severity to demote.
#[test]
fn declared_forbidden_state_in_phase_model_stays_silent() {
    let spec = spec_at(
        r#"---
id: demo.phases
kind: intent
statement: "THE demo SHALL advance through its phases"
---

## Constraints

| id | kind | expr | traces_to |
|----|------|------|-----------|
| no_halt | invariant | `**rust:** state != "halted"` | [[demo.phases]] |

## Model

### States

- phase_a
- phase_b
- halted

### Transitions

| id | from | to | guard |
|----|------|----|-------|
| enter | phase_a | phase_b | [[demo.phases.no_halt]] |
| halt | halted | halted | [[demo.phases.no_halt]] |

## Properties

| id | kind | derives_from | generator | predicate |
|----|------|--------------|-----------|------------|
| p | unit | [[demo.phases.no_halt]] | `g()` | `x` |

## Notes

The `halted` phase is declared but unreachable from the first phase:
its only transition is the self-loop `halt`, and no edge leaves
`phase_a`/`phase_b` toward it. That is the legitimate phase-model
shape — a defensive invariant over a declared phase a correct run never
enters. Its forbidden state IS declared, and the vacuity advisory
checks citation-versus-declared-ids — never reachability from any seed
state — so it stays silent: the advisory must never nudge an author off
good design pressure.
"#,
        "demo-phases.md",
    );
    let report = lint_all(&[spec], &[]);
    assert_eq!(
        report.failures(),
        0,
        "a legitimate phase model lints green: {:?}",
        report.issues
    );
    assert!(
        report
            .warnings
            .iter()
            .all(|w| w.rule_id != VACUOUS_INVARIANT),
        "a DECLARED forbidden phase is not structurally vacuous: {:?}",
        report.warnings
    );
    assert!(
        report
            .warnings
            .iter()
            .all(|w| w.rule_id != FRAGMENT_BINDS_NOTHING),
        "the fragment cites the declared `halted` state — never a vacuous binding: {:?}",
        report.warnings
    );
}

/// A shape-clean single-file fixture for the citation proxy: state
/// `held` (4 characters — a meaningful citation), transition `t`,
/// constraint `c`, and property `p` (all one character — the short ids
/// that must never satisfy the check). The constraint expr is supplied
/// by the case.
fn citation_fixture(expr: &str) -> Spec {
    spec_at(
        &format!(
            "---\nid: cite.proxy\nkind: intent\nstatement: \"THE proxy SHALL cite\"\n---\n\n## Constraints\n\n| id | kind | expr | traces_to |\n|----|------|------|-----------|\n| c | invariant | {expr} | [[cite.proxy]] |\n\n## Model\n\n### States\n\n- held\n\n### Transitions\n\n| id | from | to | guard |\n|----|------|----|-------|\n| t | held | held | [[cite.proxy.c]] |\n\n## Properties\n\n| id | kind | derives_from | generator | predicate |\n|----|------|--------------|-----------|------------|\n| p | unit | [[cite.proxy.c]] | `g()` | `x` |\n"
        ),
        "cite-proxy.md",
    )
}

/// h5ks fix-review: the citation proxy is textual and deliberately
/// narrow. String/char literals and `//` line comments are stripped
/// for IDENTIFIER matching (a declared row name inside `"…"` or a
/// comment is not an identifier coincidence — declared-state literals
/// are the separate exception pinned by
/// `declared_state_literal_counts_as_a_citation`), and a declared id
/// shorter than three characters (`c`, `t`, `p`) never satisfies the
/// check — a single letter that happens to appear in a fragment is not
/// a binding.
#[test]
fn citation_proxy_ignores_literals_comments_and_short_ids() {
    for expr in [
        "`**rust:** \"marker\" == \"marker\"`", // literal text is stripped
        "`**rust:** flag != 0 // held`",        // comment text is stripped
        "`**rust:** c >= 0`",                   // 1-char declared id: not meaningful
        "`**rust:** t >= 0`",
        "`**rust:** p >= 0`",
    ] {
        let spec = citation_fixture(expr);
        assert!(
            warns(&spec, FRAGMENT_BINDS_NOTHING),
            "{expr} must still bind nothing after the proxy's narrow scan"
        );
    }
    let meaningful = citation_fixture("`**rust:** held == held`");
    assert!(
        !warns(&meaningful, FRAGMENT_BINDS_NOTHING),
        "the 4-character declared state `held` is a real citation"
    );
}

/// h5ks follow-up: literals are stripped for identifier coincidence, but
/// a whole unescaped literal that EXACTLY names a declared `Model/States`
/// id is itself a citation — a fragment comparing the `state` binder
/// against a declared state is genuinely bound to the model. Equality is
/// exact and length-independent (`s1` counts). An UNDECLARED state
/// literal is deliberately NOT a citation: that fragment still binds
/// nothing, and the ghost literal is `vacuous_invariant`'s surface —
/// both advisories name it rather than one silently satisfying the
/// other.
#[test]
fn declared_state_literal_counts_as_a_citation() {
    let declared = citation_fixture("`**rust:** state != \"held\"`");
    assert!(
        !warns(&declared, FRAGMENT_BINDS_NOTHING),
        "a declared-state literal binds the fragment to the model"
    );

    let short_declared = spec_with_cells("invariant", "`**rust:** state != \"s1\"`", "`x`");
    assert!(
        !warns(&short_declared, FRAGMENT_BINDS_NOTHING),
        "literal equality is length-independent — the 2-character declared state `s1` counts"
    );

    let undeclared = spec_with_cells("invariant", "`**rust:** state != \"aborted\"`", "`x`");
    assert!(
        warns(&undeclared, FRAGMENT_BINDS_NOTHING),
        "an undeclared state literal is not a citation"
    );
    assert!(
        warns(&undeclared, VACUOUS_INVARIANT),
        "the same fragment is vacuous_invariant's surface"
    );
}

/// h5ks fix-review: a fragment that contains a path-qualified call
/// (`::`) invokes production code, so it is checking something and must
/// never be labeled vacuous — even when no declared row id appears in
/// its text. A call without `::` is still judged textually.
#[test]
fn path_qualified_api_call_exempts_binds_nothing() {
    let api = citation_fixture("`**rust:** specodelic::compile::fragment_of(&v0)`");
    assert!(
        !warns(&api, FRAGMENT_BINDS_NOTHING),
        "a path-qualified API call is a deliberate exemption, not a vacuous binding"
    );
    let bare = citation_fixture("`**rust:** extract(&v0)`");
    assert!(
        warns(&bare, FRAGMENT_BINDS_NOTHING),
        "an unqualified call still binds nothing textual"
    );
}

/// h5ks fix-review: the forbidden-literal scan skips `//` line
/// comments, so a commented-out `// state != "aborted"` never warns,
/// and the binder must be exactly `state` — `model.state`,
/// `state.field`, `crate::state`, and `my_state` are other names.
#[test]
fn forbidden_state_scan_skips_comments_and_requires_the_state_binder() {
    let commented = spec_with_cells(
        "invariant",
        "`**rust:** state != \"s1\" // state != \"aborted\"`",
        "`x`",
    );
    assert!(
        !warns(&commented, VACUOUS_INVARIANT),
        "a commented-out forbidden-state comparison is not a claim"
    );
    let positive = spec_with_cells("invariant", "`**rust:** state != \"aborted\"`", "`x`");
    assert!(
        warns(&positive, VACUOUS_INVARIANT),
        "the exact `state` binder over an undeclared literal still warns"
    );
    for expr in [
        "`**rust:** model.state != \"aborted\"`",
        "`**rust:** state.field != \"aborted\"`",
        "`**rust:** crate::state != \"aborted\"`",
        "`**rust:** my_state != \"aborted\"`",
    ] {
        let spec = spec_with_cells("invariant", expr, "`x`");
        assert!(
            !warns(&spec, VACUOUS_INVARIANT),
            "the binder must be exactly `state`, not {expr}"
        );
    }
}
