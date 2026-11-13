---
tags: [pipeline-step:research, pipeline-run:epic-orchestrator-2026-11-13-specodelic-lf3-scoped-verify-migration]
---

# LF3 feasibility — `compile` property rows

**Ticket:** specodelic-lf3 (scoped-first) · **run:** 2026-11-13 ·
**Method:** row-by-row classification of every Property row in
`openspec/specs/compile/spec.md` against the runner as it exists at HEAD.

## Verdict

| metric | value |
|--------|-------|
| property rows | 35 |
| expressible now | **0** |
| needs runner harness | **35** |

**No row of this file is honestly expressible today.** Every row is a
claim about the tool's own behavior (the linter, the compiler, the pack
scanner, the verify gates) or about corpus content — never a closed-form
predicate over generated values. See
[the summary](./2026-11-13-lf3-feasibility-summary.md) for the runner
measurements behind this verdict and the corpus-wide fallback sweep.

## Blockers

| code | meaning |
|------|---------|
| **T** | tool dependency — the row asserts the tool's own checker behavior; an honest executable form must CALL the tool |
| **F** | corpus/fixture access — the row asserts something about corpus content; the fragment would have to read the corpus (`std::fs` is banned by `fragment_hygiene`) or use a fixture API |
| **G** | generator vocabulary — the row needs varied generated values; today a named generator compiles to `Just(name)` — one constant input per property |
| **L** | law-case fragments — a law-kind row cannot carry a `**rust:**` fragment at all (`fragment_law_rejected`), so each named case needs its own body (a format Revision) |

## Row-by-row

| # | row id | kind | generator (fixture named) | predicate (gist) | verdict | blockers | why |
|---|--------|------|---------------------------|------------------|---------|----------|-----|
| 1 | `precondition_violation_rejected` | unit | `file_that_has_not_passed_lint()` | `check(file) == failed` | needs-harness | **T** | the compiler must refuse a file that has not passed lint |
| 2 | `toml_roundtrips` | unit | `arbitrary_constraint_row()` | `parse_toml(compile(row)) == row` | needs-harness | **T** | the compiler's TOML emission must round-trip a constraint row |
| 3 | `tla_disjunct_count_matches` | unit | `arbitrary_model_section(n_transitions)` | `count(disjuncts(compile(model).Next)) == n_transitions + 1` — t | needs-harness | **T** | the compiler's TLA+ emission must carry one disjunct per transition |
| 4 | `output_function_covers_emitting_states_only` | unit | `arbitrary_model_section(n_states_with_emits, n_states_w | `domain(compile(model).Output) == the_states_with_emits` — no en | needs-harness | **T** | the compiler's TLA+ `Output` domain must be exactly the emitting states |
| 5 | `proptest_block_per_property` | unit | `arbitrary_property_row()` | `count(compile(row).blocks) ≥ 1` | needs-harness | **T** | the compiler must emit at least one block per property row |
| 6 | `law_property_compiles_required_cases` | unit | `law_property_row_with(cases: ["identity", "associativit | `count(compile(row).blocks) == 2` | needs-harness | **T** | the compiler must emit one block per named law case |
| 7 | `compile_completes_or_fails_cleanly` | unit | `arbitrary_linted_and_covered_file()` | `compile(file) ∈ {all_three_artifacts, single_labeled_failure}`  | needs-harness | **T** | the compiler must produce all artifacts or one labeled failure |
| 8 | `id_preservation_holds` | unit | `arbitrary_linted_and_covered_file()` | `ids_in(file) ⊆ ids_in(compile(file))` | needs-harness | **T** | the compiler's artifacts must carry every input id |
| 9 | `roundtrip_stable` | unit | `arbitrary_linted_and_covered_file()` | `compile(file) == compile(parse(compile(file)))` | needs-harness | **T** | compile must be idempotent under re-parse |
| 10 | `extraction_failure_label_asserted` | unit | `extraction_stage_fails_on_row()` | `error_label == "compile.extraction_failure"` — renaming the lab | needs-harness | **F** | the label `"compile.extraction_failure"` must hold in three corpus sites |
| 11 | `emission_failure_label_asserted` | unit | `emission_stage_fails()` | `error_label == "compile.emission_failure"` — same three-site re | needs-harness | **F** | the label `"compile.emission_failure"` must hold in three corpus sites |
| 12 | `fragment_body_emitted_verbatim` | unit | `property_row_with_rust_fragment()` | `the emitted block body contains the fragment verbatim` | needs-harness | **T** | the compiler must emit the fragment body verbatim into the block |
| 13 | `prose_predicate_stays_placeholder` | unit | `property_row_without_fragment()` | `compile(row).body contains todo_predicate!` — no widening of pr | needs-harness | **T** | a markerless predicate must keep the `todo_predicate!` placeholder |
| 14 | `fragment_values_bind_generators` | unit | `property_row_with_two_generators()` | `the emitted block's args are v0, v1 — one per named generator,  | needs-harness | **T** **G** | the compiler must bind one argument per named generator — the arguments are `Just(name)` constants today, so the claim is only observable on the constant |
| 15 | `rust_fragment_semantics_unchanged` | unit | `revision_15_rust_fragment_cell()` | `extract(cell) == revision_15_extract(cell)` — byte-identical, p | needs-harness | **T** | the Revision-16 extractor must extract byte-identically to the Revision-15 extractor |
| 16 | `unknown_tag_labeled` | unit | `property_row_with_marker_tag("**go:")` | `error_stage == fragment_extraction ∧ message names the tag and  | needs-harness | **T** | extraction must fail labeled for an unknown tag |
| 17 | `non_fragment_tag_is_mention` | unit | `property_row_with_mid_span_marker_tag("**py:")` | `compile(row) == compile(row_without_marker)` — mid-span occurre | needs-harness | **T** | a mid-span marker must never extract |
| 18 | `one_tag_extracts_per_cell` | unit | `property_row_with_marker_tag("**py:")` | `extract(cell) == Some(fragment_under("py"))` — a fragment-posit | needs-harness | **T** | exactly one tag may extract per cell |
| 19 | `missing_emitter_labeled` | unit | `property_row_with_marker_tag("**py:")` | `error_label == "compile.emission_failure" ∧ the remediation hin | needs-harness | **T** | an accepted-but-unemitted tag must fail labeled at emission |
| 20 | `law_fragment_labeled` | unit | `law_row_with_fragment()` | `error_stage == fragment_extraction` | needs-harness | **T** | a fragment on a law row must fail extraction — the rule this row asserts is itself the blocker for migrating law rows |
| 21 | `hygiene_violation_labeled` | unit | `fragment_with(banned_token: "std::fs")` | `error_stage == fragment_extraction ∧ message names the token` | needs-harness | **T** | a banned token in a fragment must fail extraction |
| 22 | `guard_fragment_labeled` | unit | `transition_row_with_fragment_guard()` | `error_stage == fragment_extraction` | needs-harness | **T** | a fragment on a transition guard must fail extraction |
| 23 | `non_invariant_fragment_labeled` | unit | `effect_row_with_fragment()` | `error_stage == fragment_extraction` | needs-harness | **T** | a fragment on a non-invariant constraint must fail extraction |
| 24 | `kernel_expr_extracts` | unit | `invariant_cell_with_kernel_marker()` | `the expression extracts under the closed grammar and evaluates  | needs-harness | **T** | the kernel grammar must extract and evaluate to one of three statuses |
| 25 | `kernel_nonmember_labeled` | unit | `invariant_cell_with_nonmember_atomic()` | `error_stage == kernel_grammar ∧ the message names the offending | needs-harness | **T** | a non-member atomic must fail the kernel grammar |
| 26 | `binding_text_surfaces_verbatim` | unit | `constraint_row_with_arbitrary_binding_text()` | `the compiled TOML carries the binding cell text verbatim — no i | needs-harness | **T** | the compiled TOML must carry the binding cell text verbatim |
| 27 | `claim_path_is_flags` | unit | `external_checker_claiming_a_constraint()` | `the claim is carried by contract-TOML flags binding; the toolch | needs-harness | **T** | the claim path must be carried by contract-TOML flags |
| 28 | `helper_inlines_at_each_use_site` | unit | `spec_with_one_helper_and_two_referencing_fragments()` | `the helper body appears in the emitted props artifact once per  | needs-harness | **T** | the compiler must inline a `## Fragments` helper body at each use site |
| 29 | `unknown_helper_reference_labeled` | unit | `property_fragment_referencing("@ghost(&v0)")` | `error_stage == fragment_extraction ∧ the message names the unre | needs-harness | **T** | an unresolved `@name` must fail extraction |
| 30 | `helper_collision_labeled_asserted` | unit | `helper_named("v0")` | `error_stage == fragment_extraction ∧ the message names both the | needs-harness | **T** | a helper colliding with a generated symbol must fail extraction |
| 31 | `helper_hygiene_labeled` | unit | `helper_body_with(banned_token: "std::fs")` | `error_stage == fragment_extraction ∧ the message names the toke | needs-harness | **T** | a banned token in a helper body must fail extraction |
| 32 | `helper_cycle_labeled_asserted` | unit | `mutually_referencing_helpers("is_a", "is_b")` | `error_stage == fragment_extraction ∧ the message names the cycl | needs-harness | **T** | a helper reference cycle must fail extraction |
| 33 | `dead_helper_warns_asserted` | unit | `spec_with_an_unreferenced_def()` | `compile succeeds ∧ the warnings channel names the dead def ∧ re | needs-harness | **T** | an unreferenced helper def must warn, and referenced ones must not |
| 34 | `absent_section_preserves_bytes_p` | unit | `linted_and_covered_file_without_a_fragments_section()` | `every compiled artifact is byte-identical to the pre-change com | needs-harness | **T** | a file without a `## Fragments` section must compile byte-identically to the pre-change compiler |
| 35 | `fragments_section_lints_clean_p` | unit | `linted_and_covered_file_with_a_fragments_section()` | `lint issues == 0 — the section is opaque prose, no gating rule  | needs-harness | **T** | a file with a `## Fragments` section must lint clean |

## What would unblock this file

1. **Tool-dependency seam** — the scratch crate must be able to depend on
   the `specodelic` lib (`src/verify.rs` `SCRATCH_MANIFEST` is
   `proptest`-only today), so a fragment can call `specodelic::lint::lint_corpus`
   / `specodelic::compile::*` on a fixture instead of re-implementing them.
2. **Fixture API** — the generators in this file name *fixtures*
   (`spec_file_with(transition.to_not_in_states: true)`,
   `row_belonging_to_a_sixth_section("Actions")`, …). Nothing in the tree
   turns a generator name into a concrete spec/row today.
3. **Generator vocabulary** — a named generator yields `Just(name)`
   (`src/compile.rs:1323`), i.e. exactly one input per property. Owned by
   specodelic-jwjj.

