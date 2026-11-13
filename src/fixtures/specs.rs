//! The corpus fixtures of the `specodelic` and `kinds` corpus files —
//! the inputs their property rows materialize through.
//!
//! Purpose: these two corpus files describe the format itself and the
//! five kinds, so their claims are claims about the TOOL's behavior on
//! a real spec file — a real parse, a real lint finding, a real
//! rename, a real model-check run. Every fixture here is a real spec
//! text (or, for the two deliberately malformed ones, a real text the
//! real parser rejects), never a re-implementation of a checker.
//!
//! Responsibilities: hold the registry of named fixtures (one entry
//! per generator name `openspec/specs/specodelic/spec.md` and
//! `openspec/specs/kinds/spec.md` name), build the real spec texts,
//! and expose the seams an executable fragment needs but may not
//! build itself (the lint verdict helpers, the corpus-text reader,
//! the properties-gate runner).
//!
//! Rationale: an executable fragment may not read files or spawn
//! processes (`fragment_hygiene`), so the file-reading and
//! process-spawning seams live HERE — in the library the verify
//! runner's scratch crate depends on — and the fragment calls them.
//!
//! Naming: a generator name shared by two rows (the corpus names one
//! generator with different arguments in different rows) carries the
//! NEUTRAL fixture here; the row-specific shape is built by the
//! fragment from that neutral input, so every row's input is derived
//! from a real fixture rather than from a second, divergent copy.

use crate::spec::Spec;

/// One named corpus fixture: the generator name that materializes
/// it and the builder that renders the real spec text.
///
/// The builder's own doc comment says what the fixture is for —
/// the registry stays a flat table of names and builders.
pub struct SpecFixture {
    /// The fixture's name — the generator name that materializes it.
    pub name: &'static str,
    /// The builder that renders the fixture's real spec text.
    pub build: fn() -> String,
}

// -----------------------------------------------------------------
// The fixture texts
// -----------------------------------------------------------------

/// The canonical fixture spec: three Constraint rows (an invariant one, the invariant one
/// a transition guard may cite, and an effect one a state may emit), two States, one
/// Transition and three Property rows — lint-clean, so a row that claims "the tool
/// accepts this shape" has a clean input to accept.
fn base() -> String {
    spec_text(
        "fx",
        "THE fx SHALL hold",
        &[
            "| c1 | invariant | `the fixture holds` | [[fx]] |",
            "| c2 | invariant | `the guard cites this` | [[fx]] |",
            "| c3 | effect | `the effect fires` | [[fx]] |",
        ],
        &["- s1", "- s2"],
        &[("t1", "s1", "s2", "[[fx.c2]]")],
        &[
            ("p1", "unit", "[[fx.c1]]", "`fx()`", "`the fixture holds`"),
            (
                "p2",
                "unit",
                "[[fx.c2]]",
                "`fx()`",
                "`the guard cites this`",
            ),
            ("p3", "unit", "[[fx.c3]]", "`fx()`", "`the effect fires`"),
        ],
        "",
    )
}

/// The id a rename targets — the Property row id the rename
/// fixture carries (`fx.p_rename_me`).
///
/// Not a spec file: the rename law rows take the corpus and the id to rename as two
/// separate inputs, so the id travels as its own fixture.
fn rename_target_id() -> String {
    "fx.p_rename_me".to_string()
}

/// The fixture whose Constraint row carries the ADVISORY kind — a
/// shape the kind enum accepts.
fn advisory_kind() -> String {
    kind_variant("advisory")
}

/// The fixture whose Constraint row carries the EFFECT kind — a
/// shape the kind enum accepts.
fn effect_kind() -> String {
    kind_variant("effect")
}

/// The fixture whose Constraint row carries the EXTENSION_POINT
/// kind — a shape the kind enum accepts.
fn extension_point_kind() -> String {
    kind_variant("extension_point")
}

/// The base fixture with `kind` on its first Constraint row — the
/// kind variants the kind enum accepts.
fn kind_variant(kind: &str) -> String {
    spec_text(
        "fx",
        "THE fx SHALL hold",
        &[
            &format!("| c1 | {kind} | `the fixture holds` | [[fx]] |"),
            "| c2 | invariant | `the guard cites this` | [[fx]] |",
            "| c3 | effect | `the effect fires` | [[fx]] |",
        ],
        &["- s1", "- s2"],
        &[("t1", "s1", "s2", "[[fx.c2]]")],
        &[
            ("p1", "unit", "[[fx.c1]]", "`fx()`", "`the fixture holds`"),
            (
                "p2",
                "unit",
                "[[fx.c2]]",
                "`fx()`",
                "`the guard cites this`",
            ),
            ("p3", "unit", "[[fx.c3]]", "`fx()`", "`the effect fires`"),
        ],
        "",
    )
}

/// The fixture whose transition row carries an EMPTY guard cell —
/// the transition that may not gate on nothing.
fn empty_guard_cell() -> String {
    spec_text(
        "fx",
        "THE fx SHALL hold",
        &[
            "| c1 | invariant | `the fixture holds` | [[fx]] |",
            "| c2 | invariant | `the guard cites this` | [[fx]] |",
        ],
        &["- s1", "- s2"],
        &[("t1", "s1", "s2", "")],
        &[
            ("p1", "unit", "[[fx.c1]]", "`fx()`", "`the fixture holds`"),
            (
                "p2",
                "unit",
                "[[fx.c2]]",
                "`fx()`",
                "`the guard cites this`",
            ),
        ],
        "",
    )
}

/// The fixture whose transition guard cites an ADVISORY constraint —
/// the guard that may not gate.
fn advisory_guard() -> String {
    spec_text(
        "fx",
        "THE fx SHALL hold",
        &[
            "| c1 | advisory | `the fixture advises` | [[fx]] |",
            "| c2 | invariant | `the guard cites this` | [[fx]] |",
        ],
        &["- s1", "- s2"],
        &[("t1", "s1", "s2", "[[fx.c1]]")],
        &[
            ("p1", "unit", "[[fx.c1]]", "`fx()`", "`the fixture advises`"),
            (
                "p2",
                "unit",
                "[[fx.c2]]",
                "`fx()`",
                "`the guard cites this`",
            ),
        ],
        "",
    )
}

/// The fixture carrying a renameable Property row — the input the
/// two `rename_naturality` law rows rename.
fn renameable() -> String {
    spec_text(
        "fx",
        "THE fx SHALL hold",
        &["| c1 | invariant | `the fixture holds` | [[fx]] |"],
        &["- s1", "- s2"],
        &[("t1", "s1", "s2", "[[fx.c1]]")],
        &[
            ("p1", "unit", "[[fx.c1]]", "`fx()`", "`the fixture holds`"),
            (
                "p_rename_me",
                "unit",
                "[[fx.c1]]",
                "`fx()`",
                "`the fixture holds`",
            ),
        ],
        "",
    )
}

/// The fixture whose transition guard cites a DECLARED STATE — the legal typed citation
/// (the state is reachable another way, so the whole file stays lint-clean).
fn state_guard() -> String {
    spec_text(
        "fx",
        "THE fx SHALL hold",
        &[
            "| c1 | invariant | `the fixture holds` | [[fx]] |",
            "| c2 | effect | `the effect fires` | [[fx]] |",
        ],
        &["- s1", "- s2 (emits: [[fx.c2]])"],
        &[("t1", "s1", "s2", "[[fx.s1]]")],
        &[
            ("p1", "unit", "[[fx.c1]]", "`fx()`", "`the fixture holds`"),
            ("p2", "unit", "[[fx.c2]]", "`fx()`", "`the effect fires`"),
        ],
        "",
    )
}

/// The fixture whose state EMITS an effect constraint — the legal
/// emits shape.
fn state_effect_emits() -> String {
    spec_text(
        "fx",
        "THE fx SHALL hold",
        &[
            "| c1 | invariant | `the fixture holds` | [[fx]] |",
            "| c2 | invariant | `the guard cites this` | [[fx]] |",
            "| c3 | effect | `the effect fires` | [[fx]] |",
        ],
        &["- s1", "- s2 (emits: [[fx.c3]])"],
        &[("t1", "s1", "s2", "[[fx.c2]]")],
        &[
            ("p1", "unit", "[[fx.c1]]", "`fx()`", "`the fixture holds`"),
            (
                "p2",
                "unit",
                "[[fx.c2]]",
                "`fx()`",
                "`the guard cites this`",
            ),
            ("p3", "unit", "[[fx.c3]]", "`fx()`", "`the effect fires`"),
        ],
        "",
    )
}

/// The fixture whose state EMITS an invariant constraint — the
/// illegal emits shape.
fn state_invariant_emits() -> String {
    spec_text(
        "fx",
        "THE fx SHALL hold",
        &[
            "| c1 | invariant | `the fixture holds` | [[fx]] |",
            "| c2 | invariant | `the guard cites this` | [[fx]] |",
        ],
        &["- s1", "- s2 (emits: [[fx.c1]])"],
        &[("t1", "s1", "s2", "[[fx.c2]]")],
        &[
            ("p1", "unit", "[[fx.c1]]", "`fx()`", "`the fixture holds`"),
            (
                "p2",
                "unit",
                "[[fx.c2]]",
                "`fx()`",
                "`the guard cites this`",
            ),
        ],
        "",
    )
}

/// The fixture whose Constraint row SUPERSEDES another one — the
/// corpus in which the supersession status is DERIVED.
fn superseding() -> String {
    spec_text_with_header(
        "fx",
        "THE fx SHALL hold",
        "| id | kind | expr | traces_to | supersedes |\n|----|------|------|-----------|------------|\n",
        &[
            "| c1 | invariant | `the fixture holds` | [[fx]] |  |",
            "| c2 | invariant | `the fixture supersedes` | [[fx]] | [[fx.c1]] |",
        ],
        &["- s1", "- s2"],
        &[("t1", "s1", "s2", "[[fx.c1]]")],
        &[
            ("p1", "unit", "[[fx.c1]]", "`fx()`", "`the fixture holds`"),
            (
                "p2",
                "unit",
                "[[fx.c2]]",
                "`fx()`",
                "`the fixture supersedes`",
            ),
        ],
        "",
    )
}

/// The fixture whose two Constraint rows SUPERSEDE each other — the
/// corpus the supersedes-cycle claim reads.
fn supersedes_cycle() -> String {
    spec_text_with_header(
        "fx",
        "THE fx SHALL hold",
        "| id | kind | expr | traces_to | supersedes |\n|----|------|------|-----------|------------|\n",
        &[
            "| c1 | invariant | `the fixture holds` | [[fx]] | [[fx.c2]] |",
            "| c2 | invariant | `the fixture supersedes` | [[fx]] | [[fx.c1]] |",
        ],
        &["- s1", "- s2"],
        &[("t1", "s1", "s2", "[[fx.c1]]")],
        &[
            ("p1", "unit", "[[fx.c1]]", "`fx()`", "`the fixture holds`"),
            (
                "p2",
                "unit",
                "[[fx.c2]]",
                "`fx()`",
                "`the fixture supersedes`",
            ),
        ],
        "",
    )
}

/// The fixture whose Constraint row TRACES TO a foreign intent — the
/// disconnected cluster the reachability claim reads.
fn disconnected_cluster() -> String {
    spec_text(
        "fx",
        "THE fx SHALL hold",
        &["| c1 | invariant | `the fixture holds` | [[other]] |"],
        &["- s1", "- s2"],
        &[("t1", "s1", "s2", "[[fx.c1]]")],
        &[("p1", "unit", "[[fx.c1]]", "`fx()`", "`the fixture holds`")],
        "",
    )
}

/// The fixture whose two Constraint rows TRACE TO each other — the
/// corpus the acyclic claim reads.
fn traces_cycle() -> String {
    spec_text(
        "fx",
        "THE fx SHALL hold",
        &[
            "| c1 | invariant | `the fixture holds` | [[fx.c2]] |",
            "| c2 | invariant | `the fixture supersedes` | [[fx.c1]] |",
        ],
        &["- s1", "- s2"],
        &[("t1", "s1", "s2", "[[fx.c1]]")],
        &[
            ("p1", "unit", "[[fx.c1]]", "`fx()`", "`the fixture holds`"),
            (
                "p2",
                "unit",
                "[[fx.c2]]",
                "`fx()`",
                "`the fixture supersedes`",
            ),
        ],
        "",
    )
}

/// The fixture carrying a Constraint row with NO deriving Property
/// row — the coverage gap.
fn coverage_gap() -> String {
    spec_text(
        "fx",
        "THE fx SHALL hold",
        &[
            "| c1 | invariant | `the fixture holds` | [[fx]] |",
            "| c2 | invariant | `the second constraint holds` | [[fx]] |",
        ],
        &["- s1", "- s2"],
        &[("t1", "s1", "s2", "[[fx.c1]]")],
        &[("p1", "unit", "[[fx.c1]]", "`fx()`", "`the fixture holds`")],
        "",
    )
}

/// The fixture carrying a reference that resolves to nothing — the
/// dangling reference the total_refs claim reads.
fn dangling_reference() -> String {
    spec_text(
        "fx",
        "THE fx SHALL hold",
        &["| c1 | invariant | `the fixture holds` | [[fx]] |"],
        &["- s1", "- s2"],
        &[("t1", "s1", "s2", "[[fx.never_defined]]")],
        &[("p1", "unit", "[[fx.c1]]", "`fx()`", "`the fixture holds`")],
        "",
    )
}

/// The fixture whose transition guard cites a PROPERTY row — the
/// guard that may not gate on a property.
fn property_guard() -> String {
    spec_text(
        "fx",
        "THE fx SHALL hold",
        &[
            "| c1 | invariant | `the fixture holds` | [[fx]] |",
            "| c2 | invariant | `the guard cites this` | [[fx]] |",
        ],
        &["- s1", "- s2"],
        &[("t1", "s1", "s2", "[[fx.p1]]")],
        &[
            ("p1", "unit", "[[fx.c1]]", "`fx()`", "`the fixture holds`"),
            (
                "p2",
                "unit",
                "[[fx.c2]]",
                "`fx()`",
                "`the guard cites this`",
            ),
        ],
        "",
    )
}

/// The fixture whose Constraint row SUPERSEDES a PROPERTY row — the
/// cross-kind supersedes claim.
fn supersedes_property_row() -> String {
    spec_text_with_header(
        "fx",
        "THE fx SHALL hold",
        "| id | kind | expr | traces_to | supersedes |\n|----|------|------|-----------|------------|\n",
        &[
            "| c1 | invariant | `the fixture holds` | [[fx]] | [[fx.p1]] |",
            "| c2 | invariant | `the guard cites this` | [[fx]] |  |",
        ],
        &["- s1", "- s2"],
        &[("t1", "s1", "s2", "[[fx.c2]]")],
        &[
            ("p1", "unit", "[[fx.c1]]", "`fx()`", "`the fixture holds`"),
            (
                "p2",
                "unit",
                "[[fx.c2]]",
                "`fx()`",
                "`the guard cites this`",
            ),
        ],
        "",
    )
}

/// The fixture whose Constraint row carries a `derives_from` column
/// — a column that appears on Property rows only.
fn constraint_with_derives_from() -> String {
    spec_text_with_header(
        "fx",
        "THE fx SHALL hold",
        "| id | kind | expr | traces_to | derives_from |\n|----|------|------|-----------|--------------|\n",
        &[
            "| c1 | invariant | `the fixture holds` | [[fx]] | [[fx.c1]] |",
            "| c2 | invariant | `the guard cites this` | [[fx]] |  |",
        ],
        &["- s1", "- s2"],
        &[("t1", "s1", "s2", "[[fx.c2]]")],
        &[
            ("p1", "unit", "[[fx.c1]]", "`fx()`", "`the fixture holds`"),
            (
                "p2",
                "unit",
                "[[fx.c2]]",
                "`fx()`",
                "`the guard cites this`",
            ),
        ],
        "",
    )
}

/// The fixture whose Constraint row SATISFIES an invariant
/// constraint — the satisfies claim's wrong-kind target.
fn satisfies_invariant() -> String {
    spec_text_with_header(
        "fx",
        "THE fx SHALL hold",
        "| id | kind | expr | traces_to | satisfies |\n|----|------|------|-----------|-----------|\n",
        &[
            "| c1 | invariant | `the fixture holds` | [[fx]] | [[fx.c2]] |",
            "| c2 | invariant | `the guard cites this` | [[fx]] |  |",
        ],
        &["- s1", "- s2"],
        &[("t1", "s1", "s2", "[[fx.c2]]")],
        &[
            ("p1", "unit", "[[fx.c1]]", "`fx()`", "`the fixture holds`"),
            (
                "p2",
                "unit",
                "[[fx.c2]]",
                "`fx()`",
                "`the guard cites this`",
            ),
        ],
        "",
    )
}

/// The fixture whose Constraint row SATISFIES a FOREIGN file's extension point — the
/// cross-file corpus the reachability carve-out claim reads.
fn satisfies_foreign_extension_point() -> String {
    spec_text_with_header(
        "fx",
        "THE fx SHALL hold",
        "| id | kind | expr | traces_to | satisfies |\n|----|------|------|-----------|-----------|\n",
        &[
            "| c1 | invariant | `the fixture holds` | [[fx]] | [[host.c_host]] |",
            "| c2 | invariant | `the guard cites this` | [[fx]] |  |",
        ],
        &["- s1", "- s2"],
        &[("t1", "s1", "s2", "[[fx.c2]]")],
        &[
            ("p1", "unit", "[[fx.c1]]", "`fx()`", "`the fixture holds`"),
            (
                "p2",
                "unit",
                "[[fx.c2]]",
                "`fx()`",
                "`the guard cites this`",
            ),
        ],
        "",
    )
}

/// The FOREIGN file the extension-point corpus carries — a file
/// whose Constraint row is an extension point.
fn extension_point_host() -> String {
    spec_text(
        "host",
        "THE host SHALL hold",
        &[
            "| c_host | extension_point | `the host extends` | [[host]] |",
            "| c_invariant | invariant | `the host holds` | [[host]] |",
        ],
        &["- h1"],
        &[("t_host", "h1", "h1", "[[host.c_invariant]]")],
        &[
            (
                "p_host",
                "unit",
                "[[host.c_host]]",
                "`fx()`",
                "`the host extends`",
            ),
            (
                "p_invariant",
                "unit",
                "[[host.c_invariant]]",
                "`fx()`",
                "`the host holds`",
            ),
        ],
        "",
    )
}

/// The fixture with a SIXTH top-level section — a section the five
/// kinds do not name.
fn sixth_section() -> String {
    let mut text = base();
    text.push_str("\n## Actions\n\n| id | what |\n|----|------|\n| a1 | do it |\n");
    text
}

/// The fixture whose Property row carries a kind outside the closed
/// set — the kind-assignment failure.
fn made_up_property_kind() -> String {
    spec_text(
        "fx",
        "THE fx SHALL hold",
        &["| c1 | invariant | `the fixture holds` | [[fx]] |"],
        &["- s1", "- s2"],
        &[("t1", "s1", "s2", "[[fx.c1]]")],
        &[("p1", "audit", "[[fx.c1]]", "`fx()`", "`the fixture holds`")],
        "",
    )
}

/// The fixture whose Constraint row carries a kind outside the
/// closed set — the kind-assignment failure on the constraint side.
fn made_up_constraint_kind() -> String {
    spec_text(
        "fx",
        "THE fx SHALL hold",
        &["| c1 | made_up | `the fixture holds` | [[fx]] |"],
        &["- s1", "- s2"],
        &[("t1", "s1", "s2", "[[fx.c1]]")],
        &[("p1", "unit", "[[fx.c1]]", "`fx()`", "`the fixture holds`")],
        "",
    )
}

/// The fixture whose transition names an UNDECLARED state — the
/// shape failure.
fn undeclared_target() -> String {
    spec_text(
        "fx",
        "THE fx SHALL hold",
        &["| c1 | invariant | `the fixture holds` | [[fx]] |"],
        &["- s1"],
        &[("t1", "s1", "s_never_declared", "[[fx.c1]]")],
        &[("p1", "unit", "[[fx.c1]]", "`fx()`", "`the fixture holds`")],
        "",
    )
}

/// The fixture whose frontmatter is MISSING the `statement` field —
/// a text the real parser rejects.
fn missing_statement() -> String {
    spec_text(
        "fx",
        "",
        &["| c1 | invariant | `the fixture holds` | [[fx]] |"],
        &["- s1", "- s2"],
        &[("t1", "s1", "s2", "[[fx.c1]]")],
        &[("p1", "unit", "[[fx.c1]]", "`fx()`", "`the fixture holds`")],
        "",
    )
}

/// The fixture whose frontmatter carries a NON-EARS statement.
fn non_ears_statement() -> String {
    spec_text(
        "fx",
        "the system should maybe work",
        &["| c1 | invariant | `the fixture holds` | [[fx]] |"],
        &["- s1", "- s2"],
        &[("t1", "s1", "s2", "[[fx.c1]]")],
        &[("p1", "unit", "[[fx.c1]]", "`fx()`", "`the fixture holds`")],
        "",
    )
}

/// The fixture whose intent id JOINS two capabilities with `and`.
fn conjoined_id() -> String {
    spec_text(
        "fx.cancel_and_refund",
        "THE fx SHALL hold",
        &["| c1 | invariant | `the fixture holds` | [[fx.cancel_and_refund]] |"],
        &["- s1", "- s2"],
        &[("t1", "s1", "s2", "[[fx.cancel_and_refund.c1]]")],
        &[(
            "p1",
            "unit",
            "[[fx.cancel_and_refund.c1]]",
            "`fx()`",
            "`the fixture holds`",
        )],
        "",
    )
}

/// The fixture whose Property row is a LAW row missing the
/// associativity case.
fn law_missing_case() -> String {
    spec_text(
        "fx",
        "THE fx SHALL hold",
        &["| c1 | invariant | `the fixture holds` | [[fx]] |"],
        &["- s1", "- s2"],
        &[("t1", "s1", "s2", "[[fx.c1]]")],
        &[(
            "p1",
            "law",
            "[[fx.c1]]",
            "`fx()`",
            "**identity:** `f(a) == a`",
        )],
        "",
    )
}

/// The fixture carrying a STATE no transition reaches.
fn orphan_state() -> String {
    spec_text(
        "fx",
        "THE fx SHALL hold",
        &["| c1 | invariant | `the fixture holds` | [[fx]] |"],
        &["- s1", "- s2", "- s3"],
        &[("t1", "s1", "s2", "[[fx.c1]]")],
        &[("p1", "unit", "[[fx.c1]]", "`fx()`", "`the fixture holds`")],
        "",
    )
}

/// The fixture with NO Model section at all.
fn without_model() -> String {
    spec_text(
        "fx",
        "THE fx SHALL hold",
        &["| c1 | invariant | `the fixture holds` | [[fx]] |"],
        &[],
        &[],
        &[("p1", "unit", "[[fx.c1]]", "`fx()`", "`the fixture holds`")],
        "",
    )
}

/// The fixture whose Transitions table has NO guard column.
fn without_guard_column() -> String {
    let mut text = String::new();
    text.push_str("---\nid: fx\nkind: intent\nstatement: \"THE fx SHALL hold\"\n---\n\n");
    text.push_str(
        "## Constraints\n\n| id | kind | expr | traces_to |\n|----|------|------|-----------|\n",
    );
    text.push_str("| c1 | invariant | `the fixture holds` | [[fx]] |\n");
    text.push_str("\n## Model\n\n### States\n\n- s1\n- s2\n");
    text.push_str("\n### Transitions\n\n| id | from | to |\n|----|------|----|\n");
    text.push_str("| t1 | s1 | s2 |\n");
    text.push_str("\n## Properties\n\n| id | kind | derives_from | generator | predicate |\n|----|------|--------------|-----------|------------|\n");
    text.push_str("| p1 | unit | [[fx.c1]] | `base_fixture()` | `the fixture holds` |\n");
    text
}

/// The fixture whose Constraint row carries an EXECUTABLE invariant
/// fragment — the model the executable-invariant run judges.
fn violated_invariant() -> String {
    spec_text(
        "fx",
        "THE fx SHALL hold",
        &["| c1 | invariant | **rust:** state != \"s2\" | [[fx]] |"],
        &["- s1", "- s2"],
        &[("t1", "s1", "s2", "[[fx.c1]]")],
        &[("p1", "unit", "[[fx.c1]]", "`fx()`", "`the fixture holds`")],
        "",
    )
}

/// The fixture whose Property row carries a FAILING executable
/// predicate — the block the properties gate must report as failed.
fn failing_block() -> String {
    spec_text(
        "fx",
        "THE fx SHALL hold",
        &["| c1 | invariant | `the fixture holds` | [[fx]] |"],
        &["- s1", "- s2"],
        &[("t1", "s1", "s2", "[[fx.c1]]")],
        &[(
            "p1",
            "unit",
            "[[fx.c1]]",
            "`failing_block()`",
            "**rust:** { false }",
        )],
        "",
    )
}

/// The fixture whose frontmatter carries RATIONALE PROSE — the
/// input the prose-invariance claim reads.
fn with_rationale() -> String {
    let mut text = String::new();
    text.push_str("---\nid: fx\nkind: intent\nstatement: \"THE fx SHALL hold\"\nrationale: \"the fixture carries a rationale paragraph the linter must ignore\"\n---\n\n");
    text.push_str(
        "## Constraints\n\n| id | kind | expr | traces_to |\n|----|------|------|-----------|\n",
    );
    text.push_str("| c1 | invariant | `the fixture holds` | [[fx]] |\n");
    text.push_str("\n## Model\n\n### States\n\n- s1\n- s2\n");
    text.push_str("\n### Transitions\n\n| id | from | to | guard |\n|----|------|----|-------|\n");
    text.push_str("| t1 | s1 | s2 | [[fx.c1]] |\n");
    text.push_str("\n## Properties\n\n| id | kind | derives_from | generator | predicate |\n|----|------|--------------|-----------|------------|\n");
    text.push_str("| p1 | unit | [[fx.c1]] | `base_fixture()` | `the fixture holds` |\n");
    text
}

/// Render a fixture spec file from its parts — the real four-layer
/// shape (frontmatter, Constraints, Model, Properties).
fn spec_text(
    id: &str,
    statement: &str,
    constraints: &[&str],
    states: &[&str],
    transitions: &[(&str, &str, &str, &str)],
    properties: &[(&str, &str, &str, &str, &str)],
    extra: &str,
) -> String {
    spec_text_with_header(
        id,
        statement,
        "| id | kind | expr | traces_to |\n|----|------|------|-----------|\n",
        constraints,
        states,
        transitions,
        properties,
        extra,
    )
}

/// Render a fixture spec file whose Constraints table carries an
/// extra reference column.
#[allow(clippy::too_many_arguments)]
fn spec_text_with_header(
    id: &str,
    statement: &str,
    constraints_header: &str,
    constraints: &[&str],
    states: &[&str],
    transitions: &[(&str, &str, &str, &str)],
    properties: &[(&str, &str, &str, &str, &str)],
    extra: &str,
) -> String {
    let mut text = String::new();
    if statement.is_empty() {
        text.push_str(&format!("---\nid: {id}\nkind: intent\n---\n\n"));
    } else {
        text.push_str(&format!(
            "---\nid: {id}\nkind: intent\nstatement: \"{statement}\"\n---\n\n"
        ));
    }
    text.push_str("## Constraints\n\n");
    text.push_str(constraints_header);
    for row in constraints {
        text.push_str(row);
        text.push('\n');
    }
    text.push_str("\n## Model\n\n### States\n\n");
    for state in states {
        text.push_str(state);
        text.push('\n');
    }
    text.push_str("\n### Transitions\n\n| id | from | to | guard |\n|----|------|----|-------|\n");
    for (id, from, to, guard) in transitions {
        text.push_str(&format!("| {id} | {from} | {to} | {guard} |\n"));
    }
    text.push_str("\n## Properties\n\n| id | kind | derives_from | generator | predicate |\n|----|------|--------------|-----------|------------|\n");
    for (id, kind, derives, generator, predicate) in properties {
        text.push_str(&format!(
            "| {id} | {kind} | {derives} | {generator} | {predicate} |\n"
        ));
    }
    text.push('\n');
    text.push_str(extra);
    text
}

// -----------------------------------------------------------------
// The registry
// -----------------------------------------------------------------

/// The registry — one entry per generator name the two corpus
/// files name.
///
/// Every entry materializes a REAL input: a spec file the tool itself parses and lints,
/// or (for `specodelic_rename_id`) the id a rename targets. Nothing here is a placeholder
/// string — the emitted `spec_gen` strategies must carry real inputs for the executable
/// fragments to test anything.
///
/// The registry — (name, builder) per fixture.
type Entry = (&'static str, fn() -> String);

const REGISTRY: &[Entry] = &[
    ("specodelic_spec_file", base),
    ("specodelic_spec_repo", renameable),
    ("specodelic_rename_id", rename_target_id),
    ("kinds_rename_id", rename_target_id),
    ("arbitrary_row_of_one_kind", renameable),
    ("arbitrary_well_formed_row_of_one_kind", base),
    ("specodelic_property_row_guard", property_guard),
    (
        "guard_field_pointing_at_an_advisory_constraint",
        advisory_guard,
    ),
    ("transition_guard_citing_a_declared_state", state_guard),
    (
        "constraint_row_with_supersedes_property_row",
        supersedes_property_row,
    ),
    (
        "constraint_row_with_derives_from_column",
        constraint_with_derives_from,
    ),
    ("constraint_row_with_kind_made_up", made_up_constraint_kind),
    ("constraint_row_with_kind_advisory", advisory_kind),
    ("constraint_row_with_kind_effect", effect_kind),
    (
        "constraint_row_with_kind_extension_point",
        extension_point_kind,
    ),
    (
        "constraint_row_satisfying_a_foreign_extension_point",
        satisfies_foreign_extension_point,
    ),
    (
        "foreign_file_declaring_an_extension_point",
        extension_point_host,
    ),
    ("schema_with_is_superseded_column", superseding),
    (
        "emits_field_pointing_at_an_invariant_constraint",
        state_invariant_emits,
    ),
    ("state_row_with_effect_emits", state_effect_emits),
    ("state_row_with_invariant_emits", state_invariant_emits),
    ("state_row_without_emits", base),
    (
        "satisfies_field_pointing_at_an_invariant_constraint",
        satisfies_invariant,
    ),
    ("frontmatter_missing_statement", missing_statement),
    ("intent_row_without_statement", missing_statement),
    ("transition_row_with_empty_guard", empty_guard_cell),
    ("transition_row_without_guard", without_guard_column),
    (
        "statement_outside_the_five_ears_patterns",
        non_ears_statement,
    ),
    ("row_id_joined_to_two_intents_via_and", conjoined_id),
    ("spec_file_with_coverage_gap", coverage_gap),
    ("law_property_missing_associativity", law_missing_case),
    ("state_appearing_in_no_transition", orphan_state),
    ("spec_file_without_states_or_transitions", without_model),
    (
        "spec_file_with_transition_target_not_in_states",
        undeclared_target,
    ),
    ("model_with_a_violated_invariant", violated_invariant),
    ("compiled_proptest_block_that_fails", failing_block),
    ("two_specs_identical_except_rationale_prose", with_rationale),
    ("property_row_with_kind_audit", made_up_property_kind),
    ("row_belonging_to_a_sixth_section", sixth_section),
    ("kind_assignment_failure_raised", made_up_property_kind),
    ("shape_check_failure_raised", undeclared_target),
    ("spec_repo_with_traces_cycle", traces_cycle),
    (
        "spec_repo_with_disconnected_constraint_cluster",
        disconnected_cluster,
    ),
    ("spec_repo_with_supersedes_cycle", supersedes_cycle),
    ("spec_file_with_dangling_reference", dangling_reference),
];

/// The fixture named `name`, if the registry carries one.
pub fn lookup(name: &str) -> Option<SpecFixture> {
    REGISTRY
        .iter()
        .find(|(n, _)| *n == name)
        .map(|(n, build)| SpecFixture {
            name: n,
            build: *build,
        })
}

/// The fixture named `name`'s real spec text.
pub fn materialize(name: &str) -> Option<String> {
    lookup(name).map(|f| (f.build)())
}

/// Every fixture name in this registry.
pub fn names() -> Vec<&'static str> {
    REGISTRY.iter().map(|(n, _)| *n).collect()
}

// -----------------------------------------------------------------
// The seams an executable fragment may not build itself
// -----------------------------------------------------------------

/// The lint findings for `spec` — `(rule_id, message)` per issue.
pub fn lint_issues(spec: &Spec) -> Vec<(String, String)> {
    let report = crate::lint::lint_corpus(std::slice::from_ref(spec));
    report
        .issues
        .iter()
        .map(|i| (i.rule_id.clone(), i.message.clone()))
        .collect()
}

/// The lint findings for a CORPUS of specs — `(rule_id, message)`
/// per issue, in corpus order.
pub fn lint_issues_of(specs: &[Spec]) -> Vec<(String, String)> {
    let report = crate::lint::lint_corpus(specs);
    report
        .issues
        .iter()
        .map(|i| (i.rule_id.clone(), i.message.clone()))
        .collect()
}

/// Did the linter fire `rule_id` on `spec`?
pub fn rule_fired(spec: &Spec, rule_id: &str) -> bool {
    lint_issues(spec).iter().any(|(r, _)| r == rule_id)
}

/// Did the linter fire `rule_id` on the corpus `specs`?
pub fn rule_fired_in(specs: &[Spec], rule_id: &str) -> bool {
    lint_issues_of(specs).iter().any(|(r, _)| r == rule_id)
}

/// Is `spec` free of lint findings (the warnings channel does not
/// gate — a warning is never a failure)?
pub fn lint_clean(spec: &Spec) -> bool {
    lint_issues(spec).is_empty()
}

/// The spec text without its frontmatter `rationale` key — the
/// prose-free twin of the rationale fixture.
pub fn strip_rationale(text: &str) -> String {
    text.lines()
        .filter(|line| !line.starts_with("rationale:"))
        .collect::<Vec<_>>()
        .join("\n")
        + "\n"
}

/// The corpus text of `file_id`'s spec file (e.g. `kinds`) — the
/// real file this repository ships, read from the crate root.
///
/// The seam exists because an executable fragment may not read files
/// (`fragment_hygiene`): the label-consistency claims are claims about the corpus text
/// itself, so the corpus seam lives in the library.
pub fn corpus_text(file_id: &str) -> Option<String> {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("openspec")
        .join("specs")
        .join(file_id)
        .join("spec.md");
    std::fs::read_to_string(path).ok()
}

/// The three sites an error label must appear in: the error Constraint row, the Property
/// row that asserts it, and the scenario note (the EDGE-002 three-site rename rule).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LabelSites {
    /// The error Constraint row carries the label.
    pub constraint_row: bool,
    /// The Property row carries the label.
    pub property_row: bool,
    /// The scenario note carries the label.
    pub note: bool,
}

/// Where `label` appears in the corpus text of `file_id`.
pub fn label_sites(file_id: &str, label: &str) -> Option<LabelSites> {
    let text = corpus_text(file_id)?;
    let constraint_row = text
        .lines()
        .any(|line| line.starts_with("| ") && line.contains(&format!("{label}(")));
    let property_row = text.lines().any(|line| {
        line.starts_with("| ") && line.contains(&format!("error_label == \"{label}\""))
    });
    let note = text
        .lines()
        .any(|line| line.trim_start().starts_with("- **THEN**") && line.contains(label));
    Some(LabelSites {
        constraint_row,
        property_row,
        note,
    })
}

/// The properties-gate verdict over a fixture whose compiled block fails — `(verdict
/// status, the artifact really carries a failing assertion)`.
///
/// The gate itself is the real one (`verify::evaluate_properties_gate` over the real
/// compiled artifact); the RUNNER is the tool's own documented substitution seam
/// (`verify::PropertiesRunner`), so the gate's own verdict mapping — not a
/// re-implementation of it — is what this reads.
pub fn failing_properties_verdict() -> Option<(String, bool)> {
    let spec = crate::spec::parse_str(&failing_block()).ok()?;
    let compiled = crate::compile::compile_spec(&spec).ok()?;
    let dir = std::env::temp_dir().join(format!(
        "specodelic-failing-properties-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0)
    ));
    std::fs::create_dir_all(&dir).ok()?;
    let stem = crate::compile::artifact_stem(&spec);
    std::fs::write(dir.join(format!("{stem}_props.rs")), &compiled.props).ok()?;
    let gate = crate::verify::evaluate_properties_gate(&spec, &dir, &FailingBlockRunner);
    let verdict = crate::scope_verdict::properties_only_verdict(&gate.state);
    let fragment = spec
        .properties
        .first()
        .and_then(|row| row.cells.get("predicate").cloned())
        .and_then(|cell| crate::compile::fragment_of_strict(&cell).ok().flatten())
        .map(|f| f.text);
    let really_fails = fragment
        .map(|text| compiled.props.contains(&text) && compiled.props.contains("assert!("))
        .unwrap_or(false);
    let _ = std::fs::remove_dir_all(&dir);
    Some((verdict.status.to_string(), really_fails))
}

/// The runner that reports every compiled block as FAILED — the tool's own runner seam,
/// used to drive the real properties gate over a fixture whose block genuinely fails.
struct FailingBlockRunner;

impl crate::verify::PropertiesRunner for FailingBlockRunner {
    fn run_blocks(
        &self,
        _props_path: &std::path::Path,
        blocks: &[crate::verify::PropBlockMeta],
    ) -> Result<Vec<crate::verify::BlockResult>, crate::verify::RunnerError> {
        Ok(blocks
            .iter()
            .map(|b| crate::verify::BlockResult {
                id: b.id.clone(),
                case: b.case.clone(),
                fn_name: b.fn_name.clone(),
                passed: false,
                detail: Some("the runner seam reports this block as failed".into()),
            })
            .collect())
    }
}
