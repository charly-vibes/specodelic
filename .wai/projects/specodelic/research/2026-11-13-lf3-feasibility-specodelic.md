---
tags: [pipeline-step:research, pipeline-run:epic-orchestrator-2026-11-13-specodelic-lf3-scoped-verify-migration]
---

# LF3 feasibility — `specodelic` property rows

**Ticket:** specodelic-lf3 (scoped-first) · **run:** 2026-11-13 ·
**Method:** row-by-row classification of every Property row in
`openspec/specs/specodelic/spec.md` against the runner as it exists at HEAD.

## Verdict

| metric | value |
|--------|-------|
| property rows | 30 |
| expressible now | **0** |
| needs runner harness | **30** |

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
| 1 | `referential_integrity` | unit | `arbitrary_spec_file()` | `∀ ref ∈ file: lookup(ref) ≠ None` | needs-harness | **T** **F** | every `[[ref]]` in a spec file resolves — a corpus-wide lookup (the linter's `total_refs`), not a pure function of one generated value |
| 2 | `acyclicity` | unit | `arbitrary_spec_repo()` | `is_dag(traces_to_graph(repo))` | needs-harness | **T** **F** | the `traces_to` graph of a corpus is a DAG — the graph builder plus corpus access |
| 3 | `rename_naturality` | law | `arbitrary_spec_repo(), arbitrary_id_rename()` | **identity:** `rename(I, a, a) == I` **associativity:** `rename( | needs-harness | **L** | T |
| 4 | `filename_convention` | unit | `(id, mismatched_filename)` pairs | `check(id, filename) == failed` | needs-harness | **T** | the linter's `id_matches_file` must fail on a mismatched pair |
| 5 | `typing_enforced` | unit | `guard_field_pointing_at_a_property_row()` | `check(file) == failed` | needs-harness | **T** | the linter's `ref_kind_compatible` must fail on a guard citing a property row |
| 6 | `own_intent_reachable` | unit | `spec_repo_with(disconnected_constraint_cluster: true)` | `check(repo) == failed` | needs-harness | **T** | the linter's `single_root_reachable` must fail on a disconnected cluster |
| 7 | `model_well_formed` | unit | `spec_file_with(transition.to_not_in_states: true)` | `check(file) == failed` | needs-harness | **T** | the linter's `every_transition_valid` must fail on an out-of-states `to` |
| 8 | `advisory_cannot_gate` | unit | `guard_field_pointing_at_an_advisory_constraint()` | `check(file) == failed` | needs-harness | **T** | the linter's `ref_kind_compatible` must fail on an advisory guard |
| 9 | `state_guard_citation_accepted` | unit | `transition_guard_citing_a_declared_state()` | `check(file) == passed` — the "has reached state X" guard patter | needs-harness | **T** | the linter must PASS a guard citing a declared state |
| 10 | `supersedes_cross_kind_rejected` | unit | `constraint_row_with(supersedes: a_property_row_id)` | `check(file) == failed` | needs-harness | **T** | the linter's same-kind `supersedes` rule must fail |
| 11 | `constraint_derives_from_rejected` | unit | `constraint_row_with(derives_from: a_constraint_row_id)` | `check(file) == failed` — the Appears-on column is normative, no | needs-harness | **T** | the linter's `derives_from` typing rule must fail |
| 12 | `supersedes_cycle_rejected` | unit | `spec_repo_with(supersedes_cycle: length ≥ 2)` | `check(repo) == failed` | needs-harness | **T** | the linter's `supersedes_acyclic` rule must fail |
| 13 | `no_stored_superseded_flag` | unit | `schema_with(explicit is_superseded: bool column)` | `check(schema) == rejected` — supersession status is derived (`s | needs-harness | **T** | the schema-shape walker must reject a stored `is_superseded` column |
| 14 | `emits_cannot_target_non_effect` | unit | `emits_field_pointing_at_an_invariant_constraint()` | `check(file) == failed` — same shape as `advisory_cannot_gate`,  | needs-harness | **T** | the linter's `emits` typing rule must fail |
| 15 | `satisfies_wrong_kind_rejected` | unit | `satisfies_field_pointing_at_an_invariant_constraint()` | `check(file) == failed` — same shape as `advisory_cannot_gate` a | needs-harness | **T** | the linter's `satisfies` typing rule must fail |
| 16 | `extension_point_needs_no_reachability_carveout` | unit | `constraint_row_with(satisfies: an_extension_point_in_an | `check(row) == passed` — `satisfies` adds no reachability obliga | needs-harness | **T** | the linter must PASS a `satisfies` edge to an extension point |
| 17 | `frontmatter_missing_field_rejected` | unit | `frontmatter_missing(one of id, kind, statement)` | `check(file) == failed` | needs-harness | **T** | the linter's `frontmatter_valid` must fail |
| 18 | `guardless_transition_rejected` | unit | `transition_row_with(empty guard cell)` | `check(file) == failed` | needs-harness | **T** | the linter's `guard_required` must fail |
| 19 | `non_ears_statement_rejected` | unit | `statement_outside_the_five_ears_patterns("the system sh | `check(file) == failed` | needs-harness | **T** | the linter's `ears_statement` must fail |
| 20 | `multi_intent_join_rejected` | unit | `row_id_joined_to_two_intents_via_and()` | `check(file) == failed` | needs-harness | **T** | the linter's `one_capability_per_row` must fail |
| 21 | `coverage_gap_flagged` | unit | `spec_file_with(constraint_with_no_deriving_property: tr | `lint(file) reports rule == failed` | needs-harness | **T** | the linter's `coverage` rule must report |
| 22 | `law_missing_case_rejected` | unit | `law_property_missing("associativity")` | `check(file) == failed` | needs-harness | **T** | the linter's `law_requires_cases` must fail |
| 23 | `orphan_state_rejected` | unit | `state_appearing_in_no_transition()` | `check(file) == failed` | needs-harness | **T** | the linter's `every_state_used` must fail |
| 24 | `missing_model_section_rejected` | unit | `spec_file_without_states_or_transitions()` | `check(file) == failed` | needs-harness | **T** | the linter's `model_present` must fail |
| 25 | `invariant_violation_blocks_verify` | unit | `model_with_a_violated_invariant()` | `check(model_check) == failed` | needs-harness | **T** | the model-check backend must report a violated invariant |
| 26 | `failing_proptest_blocks_verify` | unit | `compiled_proptest_block_that_fails()` | `check(verify) == failed` | needs-harness | **T** | the verify properties gate must fail — a claim about `spk verify` itself |
| 27 | `prose_does_not_affect_lint` | unit | `two_specs_identical_except_rationale_prose()` | `lint(a) == lint(b)` — differing prose never changes a lint resu | needs-harness | **T** | two corpora differing only in prose must lint identically |
| 28 | `silently_shrunk_id_set_rejected` | unit | `(r, r')` where a 𝒦-governed id-set member was removed w | `check(revisions) == failed` | needs-harness | **T** **F** | the linter must fail a revision diff that shrank the governed id set — needs corpus revisions |
| 29 | `constraint_kind_out_of_set_rejected` | unit | `constraint_row_with(kind: "made_up")` | `check(file) == failed` | needs-harness | **T** | the linter's `constraint_kind_closed` must fail |
| 30 | `property_kind_out_of_set_rejected` | unit | `property_row_with(kind: "audit")` | `check(file) == failed` | needs-harness | **T** | the linter's `property_kind_closed` must fail |

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
4. **Law-case fragments** — this file carries a law-kind row; law rows cannot carry a `**rust:**` fragment at all today (`fragment_law_rejected`, verified empirically this run), so the file stays unmigratable even after seams 1–3 land. A per-case fragment body is a format Revision.
