//! specodelic — the Specodelic format library shared by the `specodelic` CLI (binary `specodelic`, alias `spk`).
//!
//! Purpose: model the four-layer markdown spec format (Intent /
//! Constraints / Model / Properties) so the CLI commands and tests all
//! speak the same parse. Responsibilities: expose `spec` (the parser),
//! `ears` (the statement grammar), `lint` (the invariants), `graph` (the
//! derived reference graph), `compile` (the Constraints→TOML /
//! Model→IR / Properties→proptest! functor), `fixtures` (the named
//! property fixtures a compiled property cell may materialize), and
//! `guide` (the closed
//! value sets + format revision + embedded primer). Rationale: keep the domain core in a
//! library so the binary stays thin and the corpus can be dogfooded from
//! integration tests.

pub mod acset;
pub mod archive_companion;
pub mod audit;
pub mod blocks;
pub mod checklist;
pub mod citation_corpus;
pub mod compile;
pub mod conform;
pub mod doctor;
pub mod ears;
pub mod graph;
pub mod guide;
pub mod hooks;
pub mod human;
pub mod kernel;
pub mod lint;
pub mod merge;
pub mod migrate;
pub mod model_check;
pub mod orchestrate;
pub mod packs;
pub mod parse;
pub mod refactor;
pub mod rename;
pub mod skill_pack;
pub mod spec;
#[cfg(test)]
pub mod test_panic;
pub mod update_notice;
pub mod verify;

/// The named property fixtures a compiled property cell may draw on.
///
/// Purpose: turn a generator NAME into a concrete input. A corpus
/// property row's generator cell names a fixture
/// (`property_row_with_one_property()`), and the compiled artifact's
/// strategy materializes it through this registry — so the row's
/// predicate fragment runs against a real spec instead of against the
/// generator's own name.
/// Responsibilities: hold the registry of named fixtures, resolve a
/// name to its fixture, and materialize a fixture's concrete spec text.
/// Rationale: the materialization happens inside the compiled artifact,
/// which runs in the verify runner's scratch crate — so the fixtures
/// must live in the library that scratch crate depends on (the
/// tool-dependency seam, openspec/changes/add-verify-runner-seams design
/// D1/D2). Every fixture is a real four-layer spec text: the real
/// parser consumes it and the real checkers judge it — no test doubles,
/// no re-implemented checkers.
pub mod fixtures {
    /// One named fixture: the concrete spec text a property cell's
    /// generator yields.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct Fixture {
        /// The fixture's name — the generator name that materializes it.
        pub name: &'static str,
        /// One line: what the fixture is for.
        pub summary: &'static str,
        /// The fixture's concrete spec text — a real four-layer spec file.
        pub text: &'static str,
    }

    /// Build a fixture's spec text — the real four-layer shape every
    /// fixture shares: frontmatter, a Constraints table, a Model section
    /// and a Properties table. The `constraints`, `states`,
    /// `transitions` and `properties` arguments carry the table's ROWS
    /// (the headers are added here, so a fixture never drifts from the
    /// format); `fragments` carries an optional `## Fragments` section
    /// (the opaque prose section compile's helper extraction reads).
    ///
    /// The two binding fixtures carry an extra `kernel.binding` column,
    /// so their Constraints header differs; they use the explicit-header
    /// arm (the header is the third argument).
    macro_rules! fixture_spec {
        (
            $id:literal,
            $statement:literal,
            $constraints:literal,
            $states:literal,
            $transitions:literal,
            $properties:literal
            $(,)?
        ) => {
            fixture_spec!(
                $id,
                $statement,
                "| id | kind | expr | traces_to |\n|----|------|------|------------|\n",
                $constraints,
                $states,
                $transitions,
                $properties,
                ""
            )
        };
        (
            $id:literal,
            $statement:literal,
            $constraints:literal,
            $states:literal,
            $transitions:literal,
            $properties:literal,
            $fragments:literal
            $(,)?
        ) => {
            fixture_spec!(
                $id,
                $statement,
                "| id | kind | expr | traces_to |\n|----|------|------|------------|\n",
                $constraints,
                $states,
                $transitions,
                $properties,
                $fragments
            )
        };
        (
            $id:literal,
            $statement:literal,
            $constraints_header:literal,
            $constraints:literal,
            $states:literal,
            $transitions:literal,
            $properties:literal,
            $fragments:literal
            $(,)?
        ) => {
            concat!(
                "---\nid: ", $id, "\nkind: intent\nstatement: \"", $statement, "\"\n---\n\n",
                "## Constraints\n\n", $constraints_header,
                $constraints,
                "\n## Model\n\n### States\n\n",
                $states,
                "\n### Transitions\n\n| id | from | to | guard |\n|----|------|----|-------|\n",
                $transitions,
                "\n## Properties\n\n| id | kind | derives_from | generator | predicate |\n|----|------|--------------|-----------|------------|\n",
                $properties,
                "\n",
                $fragments
            )
        };
    }

    /// The registry — the one list of named fixtures.
    ///
    /// Every fixture is a REAL spec text: the real parser consumes it
    /// and the real checkers judge it. The names are the generator names
    /// the corpus's property rows name (`compile.md`'s 35 property rows
    /// each name one), so a row's generator cell materializes the input
    /// its claim is about.
    const REGISTRY: &[Fixture] = &[
        Fixture {
            name: "property_row_with_one_property",
            summary: "a spec file with exactly one unit Property row",
            text: fixture_spec!(
                "fixture.property_row",
                "THE fixture SHALL carry exactly one unit Property row.",
                "| c1 | invariant | `holds` | [[fixture.property_row]] |\n",
                "- s1\n- s2\n",
                "| t | s1 | s2 | [[fixture.property_row.c1]] |\n",
                "| p_one_property_row | unit | [[fixture.property_row.c1]] | `sample_row()` | `count(compile(row).blocks) >= 1` |\n",
            ),
        },
        Fixture {
            name: "spec_file_with_undeclared_transition_target",
            summary: "a spec file whose transition target its States list never declares",
            text: fixture_spec!(
                "fixture.undeclared_target",
                "THE fixture SHALL name a transition target its States list does not declare.",
                "| c1 | invariant | `holds` | [[fixture.undeclared_target]] |\n",
                "- s1\n",
                "| t | s1 | s_never_declared | [[fixture.undeclared_target.c1]] |\n",
                "| p_declared_target | unit | [[fixture.undeclared_target.c1]] | `sample_row()` | `every transition target is a declared state` |\n",
            ),
        },
        Fixture {
            name: "file_that_has_not_passed_lint",
            summary: "a spec file with one lint finding — its transition names an undeclared state",
            text: fixture_spec!(
                "fixture.not_linted",
                "THE fixture SHALL carry a lint finding the compiler refuses to compile.",
                "| c_one | invariant | `holds` | [[fixture.not_linted]] |\n",
                "- s1\n",
                "| t_one | s1 | s_never_declared | [[fixture.not_linted.c_one]] |\n",
                "| p_one | unit | [[fixture.not_linted.c_one]] | `sample()` | `the constraint holds` |\n",
            ),
        },
        Fixture {
            name: "arbitrary_constraint_row",
            summary: "a spec file carrying exactly one Constraint row",
            text: fixture_spec!(
                "fixture.constraint_row",
                "THE fixture SHALL carry exactly one Constraint row.",
                "| c_one | invariant | `the constraint holds` | [[fixture.constraint_row]] |\n",
                "- s1\n- s2\n",
                "| t_one | s1 | s2 | [[fixture.constraint_row.c_one]] |\n",
                "| p_one | unit | [[fixture.constraint_row.c_one]] | `sample()` | `the constraint holds` |\n",
            ),
        },
        Fixture {
            name: "arbitrary_model_section",
            summary: "a spec file whose Model has two states (one emitting) and two transitions",
            text: fixture_spec!(
                "fixture.model_section",
                "THE fixture SHALL carry a Model section with two transitions and one emitting state.",
                "| c_one | invariant | `the constraint holds` | [[fixture.model_section]] |\n\
                 | c_effect | effect | `the effect fires` | [[fixture.model_section]] |\n",
                "- s1\n- s2 (emits: `[[fixture.model_section.c_effect]]`)\n",
                "| t_one | s1 | s2 | [[fixture.model_section.c_one]] |\n\
                 | t_two | s2 | s1 | [[fixture.model_section.c_one]] |\n",
                "| p_one | unit | [[fixture.model_section.c_one]] | `sample()` | `the constraint holds` |\n\
                 | p_effect | unit | [[fixture.model_section.c_effect]] | `sample()` | `the effect fires` |\n",
            ),
        },
        Fixture {
            name: "law_property_row_with",
            summary: "a spec file whose Property row is law-kind with the identity and associativity cases",
            text: fixture_spec!(
                "fixture.law_row",
                "THE fixture SHALL carry one law-kind Property row with its required cases.",
                "| c_one | invariant | `holds` | [[fixture.law_row]] |\n",
                "- s1\n- s2\n",
                "| t_one | s1 | s2 | [[fixture.law_row.c_one]] |\n",
                "| p_law | law | [[fixture.law_row.c_one]] | `sample()` | **identity:** `f(a) == a` **associativity:** `f(f(a)) == f(a)` |\n",
            ),
        },
        Fixture {
            name: "arbitrary_linted_and_covered_file",
            summary: "a lint-clean, coverage-clean spec file the compiler accepts",
            text: fixture_spec!(
                "fixture.covered_file",
                "THE fixture SHALL lint clean and carry coverage for its constraint.",
                "| c_one | invariant | `the constraint holds` | [[fixture.covered_file]] |\n",
                "- s1\n- s2\n",
                "| t_one | s1 | s2 | [[fixture.covered_file.c_one]] |\n",
                "| p_one | unit | [[fixture.covered_file.c_one]] | `sample()` | `the constraint holds` |\n",
            ),
        },
        Fixture {
            name: "extraction_stage_fails_on_row",
            summary: "a spec file whose Property row carries an empty **rust:** fragment",
            text: fixture_spec!(
                "fixture.empty_fragment",
                "THE fixture SHALL carry a Property row whose fragment is empty.",
                "| c_one | invariant | `holds` | [[fixture.empty_fragment]] |\n",
                "- s1\n- s2\n",
                "| t_one | s1 | s2 | [[fixture.empty_fragment.c_one]] |\n",
                "| p_empty_fragment | unit | [[fixture.empty_fragment.c_one]] | `sample()` | `**rust:**` |\n",
            ),
        },
        Fixture {
            name: "emission_stage_fails",
            summary: "a spec file whose Property row carries a **py:** fragment (no emitter yet)",
            text: fixture_spec!(
                "fixture.py_fragment",
                "THE fixture SHALL carry a Property row with a py-tagged fragment.",
                "| c_one | invariant | `holds` | [[fixture.py_fragment]] |\n",
                "- s1\n- s2\n",
                "| t_one | s1 | s2 | [[fixture.py_fragment.c_one]] |\n",
                "| p_py_fragment | unit | [[fixture.py_fragment.c_one]] | `sample()` | `**py:** py_predicate()` |\n",
            ),
        },
        Fixture {
            name: "property_row_with_rust_fragment",
            summary: "a spec file whose Property row opts into executable translation with a **rust:** fragment",
            text: fixture_spec!(
                "fixture.rust_fragment",
                "THE fixture SHALL carry a Property row with a rust-tagged fragment.",
                "| c_one | invariant | `holds` | [[fixture.rust_fragment]] |\n",
                "- s1\n- s2\n",
                "| t_one | s1 | s2 | [[fixture.rust_fragment.c_one]] |\n",
                "| p_rust_fragment | unit | [[fixture.rust_fragment.c_one]] | `sample()` | `**rust:** verbatim_marker == \"verbatim\"` |\n",
            ),
        },
        Fixture {
            name: "property_row_without_fragment",
            summary: "a spec file whose Property row carries prose only (no fragment marker)",
            text: fixture_spec!(
                "fixture.prose_predicate",
                "THE fixture SHALL carry a Property row with a prose predicate.",
                "| c_one | invariant | `holds` | [[fixture.prose_predicate]] |\n",
                "- s1\n- s2\n",
                "| t_one | s1 | s2 | [[fixture.prose_predicate.c_one]] |\n",
                "| p_prose | unit | [[fixture.prose_predicate.c_one]] | `sample()` | `the prose predicate holds` |\n",
            ),
        },
        Fixture {
            name: "property_row_with_two_generators",
            summary: "a spec file whose Property row names two generators",
            text: fixture_spec!(
                "fixture.two_generators",
                "THE fixture SHALL carry a Property row with two named generators.",
                "| c_one | invariant | `holds` | [[fixture.two_generators]] |\n",
                "- s1\n- s2\n",
                "| t_one | s1 | s2 | [[fixture.two_generators.c_one]] |\n",
                "| p_two_generators | unit | [[fixture.two_generators.c_one]] | `g_one(), g_two()` | `the prose predicate holds` |\n",
            ),
        },
        Fixture {
            name: "revision_15_rust_fragment_cell",
            summary: "a spec file whose Property row carries a Revision-15 shaped **rust:** cell",
            text: fixture_spec!(
                "fixture.revision_15",
                "THE fixture SHALL carry a Revision-15 shaped rust fragment cell.",
                "| c_one | invariant | `holds` | [[fixture.revision_15]] |\n",
                "- s1\n- s2\n",
                "| t_one | s1 | s2 | [[fixture.revision_15.c_one]] |\n",
                "| p_revision_15 | unit | [[fixture.revision_15.c_one]] | `sample()` | `**rust:** revision_marker == \"r15\"` |\n",
            ),
        },
        Fixture {
            name: "property_row_with_marker_tag",
            summary: "a spec file with one unknown-tag row (**go:**) and one py-tagged row (**py:**) — the py row comes first, so the emitter gate fires before the unknown-tag rejection",
            text: fixture_spec!(
                "fixture.marker_tags",
                "THE fixture SHALL carry an unknown-tag row and a py-tagged row.",
                "| c_one | invariant | `holds` | [[fixture.marker_tags]] |\n",
                "- s1\n- s2\n",
                "| t_one | s1 | s2 | [[fixture.marker_tags.c_one]] |\n",
                "| p_py_tag | unit | [[fixture.marker_tags.c_one]] | `sample()` | `**py:** py_predicate()` |\n\
                 | p_go_tag | unit | [[fixture.marker_tags.c_one]] | `sample()` | `**go:** go_predicate()` |\n",
            ),
        },
        Fixture {
            name: "property_row_with_mid_span_marker_tag",
            summary: "a spec file whose Property row mentions **py:** mid-span (never a fragment position)",
            text: fixture_spec!(
                "fixture.mid_span_marker",
                "THE fixture SHALL mention the py marker mid-span in a Property row.",
                "| c_one | invariant | `holds` | [[fixture.mid_span_marker]] |\n",
                "- s1\n- s2\n",
                "| t_one | s1 | s2 | [[fixture.mid_span_marker.c_one]] |\n",
                "| p_mid_span | unit | [[fixture.mid_span_marker.c_one]] | `sample()` | `count(x) == 1 **py:** is only a mention` |\n",
            ),
        },
        Fixture {
            name: "law_row_with_fragment",
            summary: "a spec file whose law-kind Property row carries a **rust:** fragment",
            text: fixture_spec!(
                "fixture.law_fragment",
                "THE fixture SHALL carry a law-kind Property row with a rust fragment.",
                "| c_one | invariant | `holds` | [[fixture.law_fragment]] |\n",
                "- s1\n- s2\n",
                "| t_one | s1 | s2 | [[fixture.law_fragment.c_one]] |\n",
                "| p_law_fragment | law | [[fixture.law_fragment.c_one]] | `sample()` | `**rust:** law_marker == \"x\"` **identity:** `f(a) == a` **associativity:** `f(f(a)) == f(a)` |\n",
            ),
        },
        Fixture {
            name: "fragment_with",
            summary: "a spec file whose Property fragment carries the banned token std::fs",
            text: fixture_spec!(
                "fixture.banned_token",
                "THE fixture SHALL carry a Property fragment with a banned token.",
                "| c_one | invariant | `holds` | [[fixture.banned_token]] |\n",
                "- s1\n- s2\n",
                "| t_one | s1 | s2 | [[fixture.banned_token.c_one]] |\n",
                "| p_banned | unit | [[fixture.banned_token.c_one]] | `sample()` | `**rust:** std::fs::read_to_string(\"x\").is_ok()` |\n",
            ),
        },
        Fixture {
            name: "transition_row_with_fragment_guard",
            summary: "a spec file whose Transition guard carries a **rust:** fragment",
            text: fixture_spec!(
                "fixture.guard_fragment",
                "THE fixture SHALL carry a Transition guard with a rust fragment.",
                "| c_one | invariant | `holds` | [[fixture.guard_fragment]] |\n",
                "- s1\n- s2\n",
                "| t_one | s1 | s2 | `**rust:** guard_marker == \"x\"` |\n",
                "| p_guard | unit | [[fixture.guard_fragment.c_one]] | `sample()` | `the prose predicate holds` |\n",
            ),
        },
        Fixture {
            name: "effect_row_with_fragment",
            summary: "a spec file whose effect-kind Constraint carries a **rust:** fragment",
            text: fixture_spec!(
                "fixture.effect_fragment",
                "THE fixture SHALL carry an effect Constraint with a rust fragment.",
                "| c_one | invariant | `holds` | [[fixture.effect_fragment]] |\n\
                 | c_effect | effect | `**rust:** effect_marker == \"x\"` | [[fixture.effect_fragment]] |\n",
                "- s1\n- s2\n",
                "| t_one | s1 | s2 | [[fixture.effect_fragment.c_one]] |\n",
                "| p_one | unit | [[fixture.effect_fragment.c_one]] | `sample()` | `the constraint holds` |\n\
                 | p_effect | unit | [[fixture.effect_fragment.c_effect]] | `sample()` | `the effect fires` |\n",
            ),
        },
        Fixture {
            name: "invariant_cell_with_kernel_marker",
            summary: "a spec file whose invariant Constraint opts into kernel translation",
            text: fixture_spec!(
                "fixture.kernel_marker",
                "THE fixture SHALL carry an invariant Constraint with a kernel expression.",
                "| c_kernel | invariant | `**kernel:** |State| == 2` | [[fixture.kernel_marker]] |\n",
                "- s1\n- s2\n",
                "| t_one | s1 | s2 | [[fixture.kernel_marker.c_kernel]] |\n",
                "| p_kernel | unit | [[fixture.kernel_marker.c_kernel]] | `sample()` | `the constraint holds` |\n",
            ),
        },
        Fixture {
            name: "invariant_cell_with_nonmember_atomic",
            summary: "a spec file whose invariant Constraint opts into a kernel expression with a non-member atomic",
            text: fixture_spec!(
                "fixture.kernel_nonmember",
                "THE fixture SHALL carry a kernel expression with a non-member atomic.",
                "| c_kernel | invariant | `**kernel:** frobnicate(x)` | [[fixture.kernel_nonmember]] |\n",
                "- s1\n- s2\n",
                "| t_one | s1 | s2 | [[fixture.kernel_nonmember.c_kernel]] |\n",
                "| p_kernel | unit | [[fixture.kernel_nonmember.c_kernel]] | `sample()` | `the constraint holds` |\n",
            ),
        },
        Fixture {
            name: "constraint_row_with_arbitrary_binding_text",
            summary: "a spec file whose Constraint row carries an arbitrary kernel.binding text",
            text: fixture_spec!(
                "fixture.arbitrary_binding",
                "THE fixture SHALL carry a Constraint row with an arbitrary binding text.",
                "| id | kind | expr | traces_to | kernel.binding |\n|----|------|------|------------|----------------|\n",
                "| c_one | invariant | `holds` | [[fixture.arbitrary_binding]] | `arbitrary binding text 42 == answer` |\n",
                "- s1\n- s2\n",
                "| t_one | s1 | s2 | [[fixture.arbitrary_binding.c_one]] |\n",
                "| p_one | unit | [[fixture.arbitrary_binding.c_one]] | `sample()` | `the constraint holds` |\n",
                ""
            ),
        },
        Fixture {
            name: "external_checker_claiming_a_constraint",
            summary: "a spec file whose Constraint row binds an external checker's claim",
            text: fixture_spec!(
                "fixture.external_claim",
                "THE fixture SHALL carry a Constraint row bound to an external checker.",
                "| id | kind | expr | traces_to | kernel.binding |\n|----|------|------|------------|----------------|\n",
                "| c_one | invariant | `holds` | [[fixture.external_claim]] | `pytest -k external_claim tests/test_claims.py::test_external_claim` |\n",
                "- s1\n- s2\n",
                "| t_one | s1 | s2 | [[fixture.external_claim.c_one]] |\n",
                "| p_one | unit | [[fixture.external_claim.c_one]] | `sample()` | `the constraint holds` |\n",
                ""
            ),
        },
        Fixture {
            name: "spec_with_one_helper_and_two_referencing_fragments",
            summary: "a spec file with one `## Fragments` helper and two fragments referencing it",
            text: fixture_spec!(
                "fixture.one_helper",
                "THE fixture SHALL carry one shared helper two fragments reference.",
                "| c_one | invariant | `holds` | [[fixture.one_helper]] |\n",
                "- s1\n- s2\n",
                "| t_one | s1 | s2 | [[fixture.one_helper.c_one]] |\n",
                "| p_first | unit | [[fixture.one_helper.c_one]] | `sample()` | `**rust:** @shared_helper(&v0)` |\n\
                 | p_second | unit | [[fixture.one_helper.c_one]] | `sample()` | `**rust:** @shared_helper(&v0)` |\n",
                "## Fragments\n\ndef shared_helper:\n```rust\nfn shared_helper(text: &str) -> bool {\n    text.contains(\"fixture\")\n}\n```\n",
            ),
        },
        Fixture {
            name: "property_fragment_referencing",
            summary: "a spec file whose Property fragment references an undefined helper",
            text: fixture_spec!(
                "fixture.unknown_helper",
                "THE fixture SHALL carry a fragment referencing an undefined helper.",
                "| c_one | invariant | `holds` | [[fixture.unknown_helper]] |\n",
                "- s1\n- s2\n",
                "| t_one | s1 | s2 | [[fixture.unknown_helper.c_one]] |\n",
                "| p_ghost | unit | [[fixture.unknown_helper.c_one]] | `sample()` | `**rust:** @ghost(&v0)` |\n",
            ),
        },
        Fixture {
            name: "ghost",
            summary: "a spec file whose Property row references the undefined ghost helper (the corpus's generator cell for that row names `ghost` as a second generator)",
            text: fixture_spec!(
                "fixture.ghost_helper",
                "THE fixture SHALL carry a fragment referencing the undefined ghost helper.",
                "| c_one | invariant | `holds` | [[fixture.ghost_helper]] |\n",
                "- s1\n- s2\n",
                "| t_one | s1 | s2 | [[fixture.ghost_helper.c_one]] |\n",
                "| p_ghost | unit | [[fixture.ghost_helper.c_one]] | `sample()` | `**rust:** @ghost(&v0)` |\n",
            ),
        },
        Fixture {
            name: "helper_named",
            summary: "a spec file whose `## Fragments` helper is named v0 (the strategy-binding namespace)",
            text: fixture_spec!(
                "fixture.helper_collision",
                "THE fixture SHALL carry a helper whose name collides with a generated symbol.",
                "| c_one | invariant | `holds` | [[fixture.helper_collision]] |\n",
                "- s1\n- s2\n",
                "| t_one | s1 | s2 | [[fixture.helper_collision.c_one]] |\n",
                "| p_one | unit | [[fixture.helper_collision.c_one]] | `sample()` | `the prose predicate holds` |\n",
                "## Fragments\n\ndef v0:\n```rust\nfn v0(text: &str) -> bool {\n    text.is_empty()\n}\n```\n",
            ),
        },
        Fixture {
            name: "helper_body_with",
            summary: "a spec file whose `## Fragments` helper body carries the banned token std::fs",
            text: fixture_spec!(
                "fixture.helper_hygiene",
                "THE fixture SHALL carry a helper body with a banned token.",
                "| c_one | invariant | `holds` | [[fixture.helper_hygiene]] |\n",
                "- s1\n- s2\n",
                "| t_one | s1 | s2 | [[fixture.helper_hygiene.c_one]] |\n",
                "| p_one | unit | [[fixture.helper_hygiene.c_one]] | `sample()` | `the prose predicate holds` |\n",
                "## Fragments\n\ndef dirty_helper:\n```rust\nfn dirty_helper(text: &str) -> bool {\n    std::fs::read_to_string(text).is_ok()\n}\n```\n",
            ),
        },
        Fixture {
            name: "mutually_referencing_helpers",
            summary: "a spec file whose two `## Fragments` helpers reference each other",
            text: fixture_spec!(
                "fixture.helper_cycle",
                "THE fixture SHALL carry two helpers that reference each other.",
                "| c_one | invariant | `holds` | [[fixture.helper_cycle]] |\n",
                "- s1\n- s2\n",
                "| t_one | s1 | s2 | [[fixture.helper_cycle.c_one]] |\n",
                "| p_one | unit | [[fixture.helper_cycle.c_one]] | `sample()` | `the prose predicate holds` |\n",
                "## Fragments\n\ndef is_a:\n```rust\nfn is_a(text: &str) -> bool {\n    @is_b(text)\n}\n```\n\ndef is_b:\n```rust\nfn is_b(text: &str) -> bool {\n    @is_a(text)\n}\n```\n",
            ),
        },
        Fixture {
            name: "spec_with_an_unreferenced_def",
            summary: "a spec file with one `## Fragments` helper no fragment cell references",
            text: fixture_spec!(
                "fixture.dead_helper",
                "THE fixture SHALL carry a helper no fragment cell references.",
                "| c_one | invariant | `holds` | [[fixture.dead_helper]] |\n",
                "- s1\n- s2\n",
                "| t_one | s1 | s2 | [[fixture.dead_helper.c_one]] |\n",
                "| p_one | unit | [[fixture.dead_helper.c_one]] | `sample()` | `the prose predicate holds` |\n",
                "## Fragments\n\ndef unused_helper:\n```rust\nfn unused_helper(text: &str) -> bool {\n    text.is_empty()\n}\n```\n",
            ),
        },
        Fixture {
            name: "linted_and_covered_file_without_a_fragments_section",
            summary: "the lint-clean covered file, with no Fragments section at all",
            text: fixture_spec!(
                "fixture.covered_file",
                "THE fixture SHALL carry no Fragments section.",
                "| c_one | invariant | `the constraint holds` | [[fixture.covered_file]] |\n",
                "- s1\n- s2\n",
                "| t_one | s1 | s2 | [[fixture.covered_file.c_one]] |\n",
                "| p_one | unit | [[fixture.covered_file.c_one]] | `sample()` | `the constraint holds` |\n",
            ),
        },
        Fixture {
            name: "linted_and_covered_file_with_a_fragments_section",
            summary: "the same lint-clean covered file, plus a Fragments section the linter treats as opaque prose",
            text: fixture_spec!(
                "fixture.covered_file",
                "THE fixture SHALL carry a Fragments section the linter treats as opaque prose.",
                "| c_one | invariant | `the constraint holds` | [[fixture.covered_file]] |\n",
                "- s1\n- s2\n",
                "| t_one | s1 | s2 | [[fixture.covered_file.c_one]] |\n",
                "| p_one | unit | [[fixture.covered_file.c_one]] | `sample()` | `the constraint holds` |\n",
                "## Fragments\n\ndef opaque_helper:\n```rust\nfn opaque_helper(text: &str) -> bool {\n    text.is_empty()\n}\n```\n",
            ),
        },
    ];

    /// The registry — every named fixture, in registry order.
    pub fn registry() -> &'static [Fixture] {
        REGISTRY
    }

    /// The fixture named `name`, if the registry carries one.
    pub fn lookup(name: &str) -> Option<&'static Fixture> {
        REGISTRY.iter().find(|f| f.name == name)
    }

    /// The fixture named `name`'s concrete spec text — what a compiled
    /// property cell's strategy yields.
    ///
    /// The pack corpus file's fixtures live in [`packs`]: a pack
    /// property row's input is a real workspace, not one spec text, so
    /// its fixture is built by the pack registry's builder. Both
    /// registries answer here — the compiler asks one question ("is
    /// this generator a fixture?"), never two.
    pub fn materialize(name: &str) -> Option<String> {
        lookup(name)
            .map(|f| f.text.to_string())
            .or_else(|| packs::materialize(name))
    }

    /// Every fixture name in the compile registry.
    ///
    /// The pack registry's names live in [`packs::names`] — the two
    /// registries carry different kinds of input (a spec text vs a
    /// workspace), so each keeps its own name list.
    pub fn names() -> Vec<&'static str> {
        REGISTRY.iter().map(|f| f.name).collect()
    }

    /// Is `name` a fixture name? The compiler's branch condition: a
    /// generator whose name is a fixture name materializes through the
    /// registry; every other generator keeps its placeholder emission.
    pub fn is_known(name: &str) -> bool {
        lookup(name).is_some() || packs::lookup(name).is_some()
    }

    /// The pack corpus fixtures — the inputs the `packs` corpus file's
    /// property rows materialize through.
    ///
    /// Purpose: the pack mechanism's claims are claims about the TOOL
    /// (the workspace scanner, the pack pass, the base closed-set
    /// walkers), never about generated values — so a pack property row's
    /// input is a real workspace: real pack files and real consumer
    /// specs on disk, which the real scanner and the real linter read.
    ///
    /// Responsibilities: hold the registry of named pack fixtures (one
    /// per generator name `openspec/specs/packs/spec.md` names), build
    /// the real pack and consumer file texts, and run a real corpus (an
    /// on-disk, git-anchored workspace the tool's own pack pass reads).
    ///
    /// Rationale: an executable fragment may not read files or spawn
    /// processes (`fragment_hygiene`), so the workspace seam lives HERE
    /// — in the library the verify runner's scratch crate depends on —
    /// and the fragment calls it. The corpus is built by the tool's own
    /// parser and judged by the tool's own linter: no test doubles, no
    /// re-implemented checkers.
    pub mod packs {
        use crate::lint::Report;
        use crate::packs::PackInfo;
        use crate::spec::Spec;
        use std::path::PathBuf;
        use std::sync::atomic::{AtomicU64, Ordering};

        /// One named pack fixture: the generator name that materializes
        /// it, and the builder that renders the real file text.
        pub struct PackFixture {
            /// The fixture's name — the generator name that materializes it.
            pub name: &'static str,
            /// One line: what the fixture is for.
            pub summary: &'static str,
            /// The builder that renders the fixture's real file text.
            pub build: fn() -> String,
        }

        /// A well-formed pack file — `kind: profile`, the four standard
        /// layers, and the six manifest tables.
        fn build_pack_file() -> String {
            pack_file("data.pack", 20)
        }

        /// A well-formed pack file, one of three in a workspace.
        fn build_pack_file_alpha() -> String {
            pack_file("alpha.pack", 20)
        }

        /// A well-formed pack file that declares one extra kind — the
        /// shared vocabulary two packs claim.
        fn build_pack_file_alpha_shared() -> String {
            pack_file_with_kinds("alpha", 20, &["shared.tolerance"])
        }

        /// A pack-free consumer spec — the corpus that must lint
        /// byte-identically with and without the pack mechanism.
        fn build_consumer() -> String {
            consumer_spec("consumer", "", "plain prose only")
        }

        /// A consumer whose `uses` edge targets a pack no workspace
        /// declares — the orphan half of the vocabulary signal.
        fn build_consumer_orphan() -> String {
            consumer_spec(
                "consumer",
                &uses_link("ghost.pack"),
                "ghost.pack vocabulary applies here",
            )
        }

        /// A consumer whose `uses` edge targets a profile pack.
        fn build_consumer_uses_pack() -> String {
            consumer_spec("consumer", &uses_link("data.pack"), "plain prose only")
        }

        /// A pack file pinned to an older base revision.
        fn build_pack_file_older() -> String {
            pack_file("data.pack", 10)
        }

        /// A pack file whose `## Kinds` row names a base closed-set
        /// member — the narrowing rejection.
        fn build_pack_file_narrowing() -> String {
            pack_file_with_kinds("data.pack", 20, &["invariant"])
        }

        /// A consumer whose Constraint row's `kind` cell carries a
        /// pack-qualified fiber kind.
        fn build_fiber_kind_consumer() -> String {
            fiber_kind_consumer("consumer", &uses_link("data.pack"), "data.dataset")
        }

        /// The registry — one fixture per generator name the packs
        /// corpus file's property table names.
        const REGISTRY: &[PackFixture] = &[
            PackFixture {
                name: "pack_file_parsed_then_manifest_read",
                summary: "a well-formed pack file with all six manifest tables",
                build: build_pack_file,
            },
            PackFixture {
                name: "corpus_with_n_profile_files",
                summary: "a well-formed pack file, one of three in a workspace",
                build: build_pack_file_alpha,
            },
            PackFixture {
                name: "lint_subdirectory_with_pack_at_workspace_root",
                summary: "a well-formed pack file at a workspace root",
                build: build_pack_file,
            },
            PackFixture {
                name: "corpus_linted_before_and_after_mechanism",
                summary: "a pack-free consumer spec",
                build: build_consumer,
            },
            PackFixture {
                name: "two_packs_declare_identical_vocabulary_and_a_file_uses_it",
                summary: "a pack file declaring the shared vocabulary token",
                build: build_pack_file_alpha_shared,
            },
            PackFixture {
                name: "corpus_using_vocabulary_without_pack",
                summary: "a consumer whose `uses` edge targets an undeclared pack",
                build: build_consumer_orphan,
            },
            PackFixture {
                name: "file_with_uses_edge_to_profile_id",
                summary: "a consumer whose `uses` edge targets a profile pack",
                build: build_consumer_uses_pack,
            },
            PackFixture {
                name: "pack_file_linted_alone",
                summary: "a well-formed pack file",
                build: build_pack_file,
            },
            PackFixture {
                name: "pack_pinned_to_older_revision",
                summary: "a pack file pinned to an older base revision",
                build: build_pack_file_older,
            },
            PackFixture {
                name: "pack_enabled_without_instances",
                summary: "a well-formed pack file, enabled by a consumer",
                build: build_pack_file,
            },
            PackFixture {
                name: "pack_removing_previously_declared_kind",
                summary: "a pack file whose `## Kinds` row names a base kind",
                build: build_pack_file_narrowing,
            },
            PackFixture {
                name: "pack_parsed_as_standard_spec",
                summary: "a well-formed pack file — the four layers",
                build: build_pack_file,
            },
            PackFixture {
                name: "two_packs_claim_same_mechanism",
                summary: "a pack file declaring the shared vocabulary token",
                build: build_pack_file_alpha_shared,
            },
            PackFixture {
                name: "guide_sets_compared_across_pack_enable",
                summary: "a well-formed pack file",
                build: build_pack_file,
            },
            PackFixture {
                name: "pack_model_section_parsed",
                summary: "a pack file with the full lifecycle machine",
                build: build_pack_file,
            },
            PackFixture {
                name: "base_table_kind_column_with_active_fiber_kind",
                summary: "a consumer whose Constraint row carries a fiber kind",
                build: build_fiber_kind_consumer,
            },
        ];

        /// The registry — every named pack fixture, in registry order.
        pub fn registry() -> &'static [PackFixture] {
            REGISTRY
        }

        /// The pack fixture named `name`, if the registry carries one.
        pub fn lookup(name: &str) -> Option<&'static PackFixture> {
            REGISTRY.iter().find(|f| f.name == name)
        }

        /// The pack fixture named `name`'s real file text.
        pub fn materialize(name: &str) -> Option<String> {
            lookup(name).map(|f| (f.build)())
        }

        /// Every pack fixture name in the registry.
        pub fn names() -> Vec<&'static str> {
            REGISTRY.iter().map(|f| f.name).collect()
        }

        /// The pack file's text: `kind: profile` frontmatter, the four
        /// standard layers, and the six manifest tables — the real input
        /// the workspace scanner and the `pack_shape` rule read.
        ///
        /// `extra_kinds` are extra `## Kinds` rows (the shared token two
        /// packs claim, or the base-kind row the narrowing rejection
        /// fires on); `model` is the Model section — the lifecycle
        /// machine, or a single-state stand.
        fn pack_file_text(id: &str, revision: u32, extra_kinds: &str, model: &str) -> String {
            format!(
                "---\nid: {id}\nkind: profile\nstatement: \"WHEN a workspace enables this pack, THE format SHALL provide the {id} vocabulary and its declared checkers\"\n---\n\
                 \n## Constraints\n\
                 \n| id | kind | expr | traces_to |\n\
                 |----|------|------|-----------|\n\
                 | vocab_declared | invariant | `the six manifest tables declare the vocabulary` | [[{id}]] |\n\
                 \n## Model\n\
                 \n{model}\
                 \n## Properties\n\
                 \n| id | kind | derives_from | generator | predicate |\n\
                 |----|------|--------------|-----------|------------|\n\
                 | p | unit | [[{id}.vocab_declared]] | `g()` | `the pack declares its vocabulary` |\n\
                 \n## Sections\n\
                 \n| section | row_shape |\n\
                 |---------|-----------|\n\
                 | Data | `\\| name \\| dtype \\|` |\n\
                 \n## Kinds\n\
                 \n| kind | vocabulary |\n\
                 |------|------------|\n\
                 | {id}.tolerance | `an empirical bound` |\n\
                 {extra_kinds}\
                 \n## References\n\
                 \n| field | resolves_to |\n\
                 |-------|-------------|\n\
                 | measures | `a {id}.tolerance kind row` |\n\
                 \n## Checkers\n\
                 \n| rule | semantics |\n\
                 |------|-----------|\n\
                 | {id}.bound_present | `every {id}.tolerance row carries a bound` |\n\
                 \n## Floors\n\
                 \n| kind | required_cases |\n\
                 |------|----------------|\n\
                 | {id}.tolerance | `identity` |\n\
                 \n## Requires\n\
                 \n| dep | revision |\n\
                 |-----|----------|\n\
                 | base | specodelic.md Revision {revision} |\n"
            )
        }

        /// The Model section of a pack in one lifecycle state — the
        /// single-state stand for `draft` and `deprecated`.
        fn single_state_model(id: &str, state: &str) -> String {
            format!(
                "### States\n\
                 \n- {state}\n\
                 \n### Transitions\n\
                 \n| id | from | to | guard |\n\
                 |----|------|----|-------|\n\
                 | hold | {state} | {state} | [[{id}.vocab_declared]] |\n"
            )
        }

        /// The Model section of a pack in the full lifecycle machine.
        fn lifecycle_model(id: &str) -> String {
            format!(
                "### States\n\
                 \n- draft\n- published\n- deprecated\n\
                 \n### Transitions\n\
                 \n| id | from | to | guard |\n\
                 |----|------|----|-------|\n\
                 | publish | draft | published | [[{id}.vocab_declared]] |\n\
                 | deprecate | published | deprecated | [[{id}.vocab_declared]] |\n"
            )
        }

        /// A well-formed pack file with extra `## Kinds` rows.
        pub fn pack_file_with_kinds(id: &str, revision: u32, extra_kinds: &[&str]) -> String {
            let mut kinds = String::new();
            for kind in extra_kinds {
                kinds.push_str(&format!("| {kind} | `a declared pack kind` |\n"));
            }
            pack_file_text(id, revision, &kinds, &lifecycle_model(id))
        }

        /// A well-formed pack file pinned to `revision`.
        pub fn pack_file(id: &str, revision: u32) -> String {
            pack_file_with_kinds(id, revision, &[])
        }

        /// A pack file in one lifecycle state (a single-state Model names
        /// exactly where the pack stands — `draft` or `deprecated`).
        pub fn pack_file_in_state(id: &str, revision: u32, state: &str) -> String {
            pack_file_text(id, revision, "", &single_state_model(id, state))
        }

        /// The `uses`-column link cell for a pack id — the reference a
        /// consumer declares to enable a pack.
        ///
        /// The link is BUILT here, never written into a corpus cell as
        /// literal `[[...]]` text: a wiki link inside a structured cell
        /// is the linter's `total_refs` beat, so a fragment that spelled
        /// the link out would dangle against the corpus.
        pub fn uses_link(pack_id: &str) -> String {
            format!("[[{pack_id}]]")
        }

        /// A consumer intent spec: a lint-clean four-layer file whose
        /// Constraint row carries a `uses` column (the declared pack
        /// enablement) and whose Constraint `kind` cell carries
        /// `kind`.
        fn consumer_text(id: &str, uses: &str, kind: &str, expr: &str) -> String {
            format!(
                "---\nid: {id}\nkind: intent\nstatement: \"THE system SHALL hold\"\n---\n\
                 \n## Constraints\n\
                 \n| id | kind | expr | traces_to | uses |\n\
                 |----|------|------|-----------|------|\n\
                 | c1 | {kind} | `{expr}` | [[{id}]] | {uses} |\n\
                 \n## Model\n\
                 \n### States\n\
                 \n- s1\n- s2\n\
                 \n### Transitions\n\
                 \n| id | from | to | guard |\n\
                 |----|------|----|-------|\n\
                 | t | s1 | s2 | [[{id}.c1]] |\n\
                 \n## Properties\n\
                 \n| id | kind | derives_from | generator | predicate |\n\
                 |----|------|--------------|-----------|------------|\n\
                 | p | unit | [[{id}.c1]] | `g()` | `the consumer holds` |\n"
            )
        }

        /// A consumer intent spec whose Constraint row carries a
        /// `uses` column (the declared pack enablement) and whose expr
        /// carries the pack's vocabulary.
        pub fn consumer_spec(id: &str, uses: &str, vocab: &str) -> String {
            consumer_text(id, uses, "invariant", vocab)
        }

        /// A consumer whose Constraint row's `kind` cell carries a
        /// pack-qualified fiber kind — the base closed-set walkers' fiber
        /// arm.
        pub fn fiber_kind_consumer(id: &str, uses: &str, kind: &str) -> String {
            consumer_text(id, uses, kind, "holds")
        }

        /// A real workspace corpus: the files on disk under one root,
        /// parsed with their paths set.
        ///
        /// The root is a fresh temp directory, git-anchored (a workspace
        /// is a git repository — the pack scanner anchors its discovery
        /// at the git toplevel), and every file is written to disk and
        /// parsed from disk, so the real scanner and the real linter read
        /// the files themselves.
        pub struct Corpus {
            root: PathBuf,
            specs: Vec<Spec>,
        }

        impl Corpus {
            /// Build the corpus: `files` are `(relative path, content)`
            /// pairs, written under one workspace root.
            pub fn new(files: &[(&str, &str)]) -> Corpus {
                let root = unique_root();
                std::fs::create_dir_all(&root).expect("the corpus root is created");
                let _ = std::process::Command::new("git")
                    .args(["init", "-q"])
                    .current_dir(&root)
                    .output();
                let mut specs = Vec::with_capacity(files.len());
                for (rel, text) in files {
                    let path = root.join(rel);
                    if let Some(parent) = path.parent() {
                        std::fs::create_dir_all(parent)
                            .expect("the corpus file's directory is created");
                    }
                    std::fs::write(&path, text).expect("the corpus file is written");
                    let mut spec = crate::spec::parse_str(text).expect("the corpus file parses");
                    spec.path = Some(path);
                    specs.push(spec);
                }
                Corpus { root, specs }
            }

            /// The corpus's parsed files, in file order.
            pub fn specs(&self) -> &[Spec] {
                &self.specs
            }

            /// The real lint pass over the corpus.
            pub fn lint(&self) -> Report {
                crate::lint::lint_corpus(&self.specs)
            }

            /// The real pack discovery over the corpus.
            pub fn discovered(&self) -> Vec<PackInfo> {
                crate::packs::discover(&self.specs)
            }

            /// The corpus's lint findings, as `rule_id: message` strings.
            pub fn issue_messages(&self) -> Vec<String> {
                self.lint()
                    .issues
                    .iter()
                    .map(|i| format!("{}: {}", i.rule_id, i.message))
                    .collect()
            }

            /// The corpus's advisory findings, as `rule_id: message`
            /// strings.
            pub fn warning_messages(&self) -> Vec<String> {
                self.lint()
                    .warnings
                    .iter()
                    .map(|i| format!("{}: {}", i.rule_id, i.message))
                    .collect()
            }

            /// The workspace root the corpus lives in.
            pub fn root(&self) -> &std::path::Path {
                &self.root
            }
        }

        impl Drop for Corpus {
            fn drop(&mut self) {
                let _ = std::fs::remove_dir_all(&self.root);
            }
        }

        /// The corpus counter — every corpus gets its own root.
        static CORPUS_COUNTER: AtomicU64 = AtomicU64::new(0);

        /// A fresh, unique workspace root under the system temp dir.
        fn unique_root() -> PathBuf {
            let n = CORPUS_COUNTER.fetch_add(1, Ordering::SeqCst);
            let stamp = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0);
            std::env::temp_dir().join(format!(
                "specodelic-pack-corpus-{}-{stamp}-{n}",
                std::process::id()
            ))
        }
    }
}

/// The scoped verdict a CI gate reads: the properties stage alone.
///
/// Purpose: `openspec/specs/verify/spec.md`'s properties-only scope
/// requirement — `properties_failed` exactly when a compiled block
/// failed, never for a missing or stale model run.
/// Responsibilities: hold the scope selector, map the properties gate
/// state to the scoped verdict, and share ONE implementation of the
/// properties branch with the full verdict.
/// Rationale: the scope lives at the library level (a library consumer
/// and the `just verify-properties` recipe read the same mapping), and
/// the shared branch means the scoped verdict and the full verdict can
/// never drift on what "properties failed" means. The `verified`
/// transition keeps its own both-gates-required semantics — the scoped
/// verdict never reports `verified`.
pub mod scope_verdict {
    use crate::verify::{ModelGateState, PropsGateState, Verdict};

    /// The properties branch of a verdict: the blocking verdict when the
    /// properties gate is not a pass, `None` when it passed (the caller
    /// decides on the model gate). ONE implementation — the full verdict
    /// and the properties-only scope verdict read the same mapping, so the
    /// two can never drift on what "properties failed" means.
    pub(crate) fn properties_verdict(properties: &PropsGateState) -> Option<Verdict> {
        let (status, message, hint) = match properties {
        PropsGateState::Pass => return None,
        PropsGateState::Failed(summary) => (
            "properties_failed",
            format!("{summary} — a partial pass is a failure of the property"),
            "fix the failing predicates (see each block's detail for the shrunk failing input)".to_string(),
        ),
        PropsGateState::Stale => (
            "stale_properties_artifact",
            "the on-disk *_props.rs artifact predates the current spec — properties from a stale artifact never count".to_string(),
            "run: specodelic compile <files> to regenerate the artifact".to_string(),
        ),
        PropsGateState::MissingArtifact => (
            "missing_properties_artifact",
            "no compiled proptest artifact for this spec — verify consumes compile's output, it never re-compiles".to_string(),
            "run: specodelic compile <files>".to_string(),
        ),
        PropsGateState::Uncompilable(output) => (
            "properties_uncompilable",
            "the *_props.rs artifact did not compile as a Rust test crate".to_string(),
            format!("fix the artifact's Rust (first errors): {output}"),
        ),
        PropsGateState::RunnerUnavailable(detail) => (
            "runner_unavailable",
            "the properties gate executes the compiled proptest! blocks and needs the Rust toolchain".to_string(),
            format!("install the Rust toolchain so cargo is on PATH ({detail})"),
        ),
        PropsGateState::TimedOut { secs, detail } => (
            "properties_timed_out",
            format!(
                "the cargo test run exceeded its {secs}s wall-clock bound and was killed — a hanging (likely pathological) predicate is an unbounded DoS on verify, and no block verdicts exist yet"
            ),
            format!(
                "raise --timeout-secs (or pass --timeout-secs 0 to run unbounded) and re-run; the partial output names the block that hung: {detail}"
            ),
        ),
    };
        Some(Verdict {
            status,
            message,
            hint,
        })
    }

    /// The scope a verdict is computed in.
    ///
    /// `BothGates` is the full `verified` transition
    /// (`both_gates_required`); `PropertiesOnly` is the scoped CI verdict —
    /// the properties gate alone, for a recipe that gates on the properties
    /// stage without the model stage hiding it.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum VerifyScope {
        /// Both gates: the full `verified` transition.
        BothGates,
        /// The properties gate alone — never reports `verified`.
        PropertiesOnly,
    }

    /// The verdict for `scope`. The scope selects WHICH gates the verdict
    /// reads; it never weakens a gate: the properties-only scope reports the
    /// properties verdict alone and can never report `verified`.
    pub fn verdict_in_scope(
        properties: &PropsGateState,
        model: &ModelGateState,
        scope: VerifyScope,
    ) -> Verdict {
        match scope {
            VerifyScope::BothGates => crate::verify::verdict(properties, model),
            VerifyScope::PropertiesOnly => properties_only_verdict(properties),
        }
    }

    /// The properties-only scope verdict: `properties_failed` exactly when a
    /// compiled block failed, and the properties gate's own status for every
    /// other state.
    ///
    /// The model gate is NOT read here: a missing or stale model run never
    /// produces `properties_failed` — a scoped CI gate reads the properties
    /// stage, not the model stage. And this verdict NEVER reports `verified`:
    /// the `verified` transition keeps its own semantics
    /// (`both_gates_required`), so a properties-only pass can never be
    /// mistaken for a verified file.
    pub fn properties_only_verdict(properties: &PropsGateState) -> Verdict {
        match properties_verdict(properties) {
        Some(blocker) => blocker,
        None => Verdict {
            status: "properties_passed",
            message: "every compiled proptest! block passed — the properties-only scope does not evaluate the model gate, so this file is NOT verified".to_string(),
            hint: "run: specodelic model-check <files>, then specodelic verify <files> for the full both-gates verdict".to_string(),
        },
    }
    }
}
