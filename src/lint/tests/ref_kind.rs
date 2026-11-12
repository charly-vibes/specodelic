//! ref_kind_compatible fixtures (specodelic-d8c): one firing fixture per
//! typed Reference Typing field family plus pass-side controls, off one
//! CLEAN baseline that exercises every typed column. Lives in a submodule
//! to stay under the library file_lines ratchet (revision18.rs pattern).

use super::*;

/// CLEAN baseline: every typed column populated with kind-compatible
/// targets. Lints with zero issues (asserted below) — the control for
/// every mutation.
const CLEAN: &str = r#"---
id: rk.t
kind: intent
statement: "THE rk SHALL type"
---

## Constraints

| id | kind | expr | traces_to | supersedes | satisfies | observes |
|----|------|------|-----------|------------|-----------|----------|
| inv | invariant | `x` | [[rk.t]] | | | | |
| eff | effect | `out == {x}` | [[rk.t]] | | | | |
| ext | extension_point | `contract` | [[rk.t]] | | [[rk.t.ext]] | | |
| soft | advisory | `hint` | [[rk.t]] | | | | |
| watch | invariant | `seen` | [[rk.t]] | | | [[rk.t.eff]] |

## Model

### States

- s1
- f (emits: `[[rk.t.eff]]`)

### Transitions

| id | from | to | guard |
|----|------|----|-------|
| ok | s1 | s1 | [[rk.t.inv]] |
| boom | s1 | f | [[rk.t.inv]] |

## Properties

| id | kind | derives_from | generator | predicate |
|----|------|--------------|-----------|------------|
| p_inv | unit | [[rk.t.inv]] | `g()` | `x` |
| p_eff | unit | [[rk.t.eff]] | `g()` | `x` |
| p_ext | unit | [[rk.t.ext]] | `g()` | `x` |
| p_soft | unit | [[rk.t.soft]] | `g()` | `x` |
| p_watch | unit | [[rk.t.watch]] | `g()` | `x` |
| nat2 | law | [[rk.t.inv]] | `g()` | **identity:** `f(a) == a` **associativity:** `h(g(f)) == f` |
| nat | law | [[rk.t.nat2]] | `g()` | **identity:** `f(a) == a` **associativity:** `h(g(f)) == f` |
"#;

/// The profile pack file the `uses` cell cites.
const PACK: &str = r#"---
id: rk.pack
kind: profile
statement: "THE rk SHALL pack"
---
"#;

/// The intent (non-profile) file the `uses`-violation fixture cites.
const NOT_PACK: &str = r#"---
id: rk.pack
kind: intent
statement: "THE rk SHALL not be a pack"
---
"#;

/// CLEAN with a `uses` column: the header gains the eighth column and a
/// `user` constraint row carries the pack-enablement edge (plus its
/// covering property, keeping the fixture otherwise clean).
fn uses_fixture() -> String {
    CLEAN.replace(
        "| id | kind | expr | traces_to | supersedes | satisfies | observes |",
        "| id | kind | expr | traces_to | supersedes | satisfies | observes | uses |",
    )
    .replace(
        "| watch | invariant | `seen` | [[rk.t]] | | | [[rk.t.eff]] |",
        "| watch | invariant | `seen` | [[rk.t]] | | | [[rk.t.eff]] |\n| user | invariant | `u` | [[rk.t]] | | | | [[rk.pack]] |",
    )
    .replace(
        "| p_watch | unit | [[rk.t.watch]] | `g()` | `x` |",
        "| p_watch | unit | [[rk.t.watch]] | `g()` | `x` |\n| p_user | unit | [[rk.t.user]] | `g()` | `x` |",
    )
}

fn clean_corpus() -> Vec<Spec> {
    vec![spec_at(CLEAN, "rk-t.md")]
}

fn ref_kind_hits(report: &Report) -> Vec<&Issue> {
    report
        .issues
        .iter()
        .filter(|i| i.rule_id == "linter.ref_kind_compatible")
        .collect()
}

// ---------- the control ----------

/// The CLEAN baseline fires nothing: every typed column cites a
/// kind-compatible target, including the law→law derives_from endo edge.
#[test]
fn ref_kind_clean_baseline_lints_clean() {
    let report = lint_corpus(&clean_corpus());
    assert!(
        report.issues.is_empty(),
        "clean baseline must lint clean: {:?}",
        report.issues
    );
    assert!(
        ref_kind_hits(&report).is_empty(),
        "no ref_kind_compatible finding on clean corpus"
    );
}

// ---------- firing: one fixture per typed field family ----------

/// guard → advisory constraint (doc: invariant Constraint or State only —
/// the d8c ticket's probe case).
#[test]
fn guard_citing_advisory_constraint_fires() {
    let specs = vec![spec_at(
        &CLEAN.replace(
            "| ok | s1 | s1 | [[rk.t.inv]] |",
            "| ok | s1 | s1 | [[rk.t.soft]] |",
        ),
        "rk-t.md",
    )];
    let report = lint_corpus(&specs);
    let hits = ref_kind_hits(&report);
    assert_eq!(hits.len(), 1, "guard→advisory must fire once: {hits:?}");
    assert!(
        hits[0].message.contains("[[rk.t.soft]]"),
        "finding names the target: {}",
        hits[0].message
    );
    assert!(
        hits[0].message.contains("invariant"),
        "finding names the requirement: {}",
        hits[0].message
    );
}

/// emits → invariant constraint (doc: effect only).
#[test]
fn emits_citing_invariant_constraint_fires() {
    let specs = vec![spec_at(
        &CLEAN.replace("- f (emits: `[[rk.t.eff]]`)", "- f (emits: `[[rk.t.inv]]`)"),
        "rk-t.md",
    )];
    let report = lint_corpus(&specs);
    let hits = ref_kind_hits(&report);
    assert_eq!(hits.len(), 1, "emits→invariant must fire once: {hits:?}");
    assert!(
        hits[0].message.contains("effect"),
        "finding names the effect requirement: {}",
        hits[0].message
    );
}

/// traces_to → Constraint (doc: Intent only).
#[test]
fn traces_to_constraint_fires() {
    let specs = vec![spec_at(
        &CLEAN.replace(
            "| inv | invariant | `x` | [[rk.t]] |",
            "| inv | invariant | `x` | [[rk.t.eff]] |",
        ),
        "rk-t.md",
    )];
    let report = lint_corpus(&specs);
    let hits = ref_kind_hits(&report);
    assert_eq!(
        hits.len(),
        1,
        "traces_to→Constraint must fire once: {hits:?}"
    );
}

/// derives_from → State from a unit property (doc: Constraint, or the
/// same Property when the deriving row is itself a law).
#[test]
fn derives_from_state_from_unit_property_fires() {
    let specs = vec![spec_at(
        &CLEAN.replace(
            "| p_inv | unit | [[rk.t.inv]] | `g()` | `x` |",
            "| p_inv | unit | [[rk.t.s1]] | `g()` | `x` |",
        ),
        "rk-t.md",
    )];
    let report = lint_corpus(&specs);
    let hits = ref_kind_hits(&report);
    assert_eq!(hits.len(), 1, "derives_from→State must fire once: {hits:?}");
}

/// supersedes across kinds: Constraint row superseding a Property.
#[test]
fn supersedes_across_kinds_fires() {
    let specs = vec![spec_at(
        &CLEAN.replace(
            "| inv | invariant | `x` | [[rk.t]] | | | | |",
            "| inv | invariant | `x` | [[rk.t]] | [[rk.t.p_inv]] | | | |",
        ),
        "rk-t.md",
    )];
    let report = lint_corpus(&specs);
    let hits = ref_kind_hits(&report);
    assert_eq!(
        hits.len(),
        1,
        "cross-kind supersedes must fire once: {hits:?}"
    );
}

/// satisfies → non-extension_point constraint.
#[test]
fn satisfies_non_extension_point_fires() {
    let specs = vec![spec_at(
        &CLEAN.replace(
            "| ext | extension_point | `contract` | [[rk.t]] | | [[rk.t.ext]] | | |",
            "| ext | extension_point | `contract` | [[rk.t]] | | [[rk.t.eff]] | | |",
        ),
        "rk-t.md",
    )];
    let report = lint_corpus(&specs);
    let hits = ref_kind_hits(&report);
    assert_eq!(hits.len(), 1, "satisfies→effect must fire once: {hits:?}");
    assert!(
        hits[0].message.contains("extension_point"),
        "finding names the requirement: {}",
        hits[0].message
    );
}

/// observes → non-effect constraint.
#[test]
fn observes_non_effect_fires() {
    let specs = vec![spec_at(
        &CLEAN.replace(
            "| watch | invariant | `seen` | [[rk.t]] | | | [[rk.t.eff]] |",
            "| watch | invariant | `seen` | [[rk.t]] | | | [[rk.t.soft]] | [[rk.pack]] |",
        ),
        "rk-t.md",
    )];
    let report = lint_corpus(&specs);
    let hits = ref_kind_hits(&report);
    assert_eq!(hits.len(), 1, "observes→advisory must fire once: {hits:?}");
}

/// uses → non-profile file (frontmatter kind `intent`): the schema's
/// `uses` row refines the Intent target with kind == profile.
#[test]
fn uses_non_profile_file_fires() {
    let with_uses = uses_fixture();
    let specs = vec![
        spec_at(&with_uses, "rk-t.md"),
        spec_at(NOT_PACK, "rk-pack.md"),
    ];
    let report = lint_corpus(&specs);
    let hits = ref_kind_hits(&report);
    assert_eq!(hits.len(), 1, "uses→intent file must fire once: {hits:?}");
    assert!(
        hits[0].message.contains("profile"),
        "finding names the profile requirement: {}",
        hits[0].message
    );
}

/// uses → profile file passes (no ref_kind_compatible finding; the
/// pack mechanism's own advisories are not this rule's beat).
#[test]
fn uses_profile_file_passes() {
    let with_uses = uses_fixture();
    let specs = vec![spec_at(&with_uses, "rk-t.md"), spec_at(PACK, "rk-pack.md")];
    assert!(
        ref_kind_hits(&lint_corpus(&specs)).is_empty(),
        "uses→profile Intent is typed-legal"
    );
}

/// Member-arm target: `file.row.member` canonicalizes to the row and is
/// typed as the row — a guard citing an advisory row's member fires.
#[test]
fn member_arm_is_typed_as_its_row() {
    let specs = vec![spec_at(
        &CLEAN.replace(
            "| ok | s1 | s1 | [[rk.t.inv]] |",
            "| ok | s1 | s1 | [[rk.t.soft.expr]] |",
        ),
        "rk-t.md",
    )];
    let report = lint_corpus(&specs);
    let hits = ref_kind_hits(&report);
    assert_eq!(hits.len(), 1, "member arm must type as the row: {hits:?}");
}

// ---------- pass-side controls ----------

/// guard → State: the "has reached state X" pattern (Revision 12).
#[test]
fn guard_citing_state_passes() {
    let specs = vec![spec_at(
        &CLEAN.replace(
            "| ok | s1 | s1 | [[rk.t.inv]] |",
            "| ok | s1 | s1 | [[rk.t.s1]] |",
        ),
        "rk-t.md",
    )];
    assert!(
        ref_kind_hits(&lint_corpus(&specs)).is_empty(),
        "guard→State is typed-legal"
    );
}

/// law → law-Property derives_from: the endo restatement edge.
#[test]
fn law_derives_from_law_property_passes() {
    let specs = vec![spec_at(CLEAN, "rk-t.md")];
    assert!(
        ref_kind_hits(&lint_corpus(&specs)).is_empty(),
        "law→law endo derives_from is typed-legal"
    );
}

/// supersedes same-kind: Constraint → Constraint.
#[test]
fn supersedes_same_kind_passes() {
    let specs = vec![spec_at(
        &CLEAN.replace(
            "| inv | invariant | `x` | [[rk.t]] | | | | |",
            "| inv | invariant | `x` | [[rk.t]] | [[rk.t.soft]] | | | |",
        ),
        "rk-t.md",
    )];
    assert!(
        ref_kind_hits(&lint_corpus(&specs)).is_empty(),
        "same-kind supersedes is typed-legal"
    );
}

/// Dangling target: exactly the total_refs finding — no duplicate typing
/// finding (dangling targets are unresolved, typing is not their beat).
#[test]
fn dangling_target_stays_total_refs_beat() {
    let specs = vec![spec_at(
        &CLEAN.replace(
            "| ok | s1 | s1 | [[rk.t.inv]] |",
            "| ok | s1 | s1 | [[rk.t.nope]] |",
        ),
        "rk-t.md",
    )];
    let report = lint_corpus(&specs);
    assert!(
        ref_kind_hits(&lint_corpus(&specs)).is_empty(),
        "no typing finding for a dangling target"
    );
    assert!(
        report
            .issues
            .iter()
            .any(|i| i.rule_id == "linter.total_refs"),
        "total_refs fires for the dangling target"
    );
}

/// Untyped columns never type: a link in an expr cell is format prose.
#[test]
fn expr_cell_links_are_not_typed() {
    let specs = vec![spec_at(
        &CLEAN.replace(
            "| inv | invariant | `x` |",
            "| inv | invariant | `[[rk.t.soft]]` |",
        ),
        "rk-t.md",
    )];
    assert!(
        ref_kind_hits(&lint_corpus(&specs)).is_empty(),
        "expr cell is not a typed reference field"
    );
}

/// The finding is self-describing: rule_semantics from RULE_TABLE and a
/// remediation hint in the message (output discipline).
#[test]
fn finding_carries_semantics_and_remediation_hint() {
    let specs = vec![spec_at(
        &CLEAN.replace(
            "| ok | s1 | s1 | [[rk.t.inv]] |",
            "| ok | s1 | s1 | [[rk.t.soft]] |",
        ),
        "rk-t.md",
    )];
    let report = lint_corpus(&specs);
    let hits = ref_kind_hits(&report);
    assert_eq!(hits.len(), 1);
    assert_eq!(
        hits[0].rule_semantics,
        rule_semantics("ref_kind_compatible").unwrap_p()
    );
    assert!(
        hits[0].message.contains("Reference Typing"),
        "remediation hint names the table: {}",
        hits[0].message
    );
}
