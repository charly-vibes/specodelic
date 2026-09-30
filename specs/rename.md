---
id: rename
kind: intent
checked_against_core: clear
statement: "WHEN a user requests renaming a spec row's id, THE rename tool SHALL update the defining row and every referencing [[link]] as a single atomic operation, or leave the repo unchanged."
---

# Rename Tool

`rename_naturality` is asserted as a property in three other files
(`specodelic.md`, `linter-referential_integrity.md`,
`linter-graph_shape.md`'s `topo_sort_naturality`), but the tool those
properties are *about* has never had its own Intent, Constraints, Model, or
Properties (`STATUS.md` §4, P0). This file is that spec: what a rename
request actually does, step by step, and what must hold before it's allowed
to report success.

A rename is the concrete operation whose existence makes `η : I ⇒ I'`
(`STATUS.md` §1's natural transformation between two instances of `𝒦`) more
than a formal description — it's the thing that has to actually construct
`I'` from `I` and a single `(old_id, new_id)` pair, honoring naturality
rather than merely being checked against it after the fact.

## Constraints

| id                        | kind      | expr                                                                                                                              | traces_to | satisfies |
|----------------------------|-----------|----------------------------------------------------------------------------------------------------------------------------------|----------------------|
| new_id_available           | invariant | `new_id ∉ index(repo)` — renaming never collides with an existing id                                                              | [[rename]] |          |
| new_id_matches_filename    | invariant | `if the renamed row is a file's own Intent, [[specodelic.id_matches_file]] must hold between new_id and the (possibly also renamed) filename` | [[rename]] |          |
| old_id_fully_replaced      | invariant | `∀ [[old_id]] reference anywhere in repo: rewritten to [[new_id]]; zero occurrences of old_id remain post-rename`                  | [[rename]] |          |
| atomic_operation           | invariant | `the definition-row update, every reference rewrite, and any required filename change apply as one transaction: all succeed, or the repo is left byte-identical to its pre-rename state` | [[rename]] |          |
| kind_unchanged             | invariant | `assigned_kind(row) is the same before and after rename — both 𝒦's five-object kind and, where present, the row's own kind/sub-kind column` — see [[kinds.kind_shape_naturality]] | [[rename]] |          |
| prose_untouched_by_rename  | invariant | `rename rewrites only [[id]] wiki-link syntax and structured id fields (frontmatter id, table id/traces_to/derives_from/guard/from/to cells) — it never edits rationale/description prose, even if the prose happens to mention the old name in words` | [[rename]] |          |
| validation_failure | effect | `rename.validation_failure(new_id, reason) — the label names its owning file per error_expr_shape` | [[rename]] | [[errors.envelope_error_kind]] ∧ [[errors.exit_code_mapping]] ∧ [[errors.remediation_hint_present]] |
| apply_failure | effect | `rename.apply_failure(detail) — the label names its owning file per error_expr_shape` | [[rename]] | [[errors.envelope_error_kind]] ∧ [[errors.exit_code_mapping]] ∧ [[errors.remediation_hint_present]] |
| post_check_failure | effect | `rename.post_check_failure(detail) — the label names its owning file per error_expr_shape` | [[rename]] | [[errors.envelope_error_kind]] ∧ [[errors.exit_code_mapping]] ∧ [[errors.remediation_hint_present]] |

## Model

### States
- `requested`
- `checked`
- `applying`
- `applied`
- `verifying`
- `passed`
- `validate_failed` (emits: `[[rename.validation_failure]]`)
- `apply_failed` (emits: `[[rename.apply_failure]]`)
- `check_failed` (emits: `[[rename.post_check_failure]]`)

### Transitions

| id            | from        | to          | guard                                                                                                |
|---------------|-------------|-------------|-------------------------------------------------------------------------------------------------------|
| validate      | requested   | checked     | [[rename.new_id_available]] ∧ [[rename.new_id_matches_filename]]                                      |
| validate_fail | requested | validate_failed | `¬([[rename.new_id_available]] ∧ [[rename.new_id_matches_filename]])` |
| apply         | checked     | applying    | [[rename.atomic_operation]]                                                                            |
| apply_ok      | applying    | applied     | [[rename.old_id_fully_replaced]] ∧ [[rename.kind_unchanged]] ∧ [[rename.prose_untouched_by_rename]]     |
| apply_fail | applying | apply_failed | `¬([[rename.old_id_fully_replaced]] ∧ [[rename.kind_unchanged]] ∧ [[rename.prose_untouched_by_rename]])` — atomic_operation requires this path to roll back to the pre-rename repo, not to leave a partial edit |
| verify        | applied     | verifying   | `linter.referential_integrity and linter.graph_shape are re-run against the renamed repo`               |
| accept        | verifying   | passed      | [[linter.referential_integrity.ref_resolves]] ∧ [[linter.graph_shape.acyclic]]                          |
| reject | verifying | check_failed | `¬([[linter.referential_integrity.ref_resolves]] ∧ [[linter.graph_shape.acyclic]])` |


## Properties

| id                          | kind | derives_from                          | generator                                          | predicate                                                                                                                                                                                    |
|-------------------------------|------|--------------------------------------------|---------------------------------------------------------|-----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| rename_naturality             | law  | [[rename.old_id_fully_replaced]]           | `arbitrary_spec_repo(), arbitrary_id_rename()`           | **identity:** `rename(I, a, a) == I`  **associativity:** `rename(rename(I,a,b), b,c) == rename(I,a,c)`  **naturality:** `compile(rename(I)) == rename(compile(I))`                             |
| collision_rejected            | unit | [[rename.new_id_available]]                | `(repo_with_id: "x", new_id: "x_taken_by_other_row")`    | `check(request) == failed`                                                                                                                                                                    |
| filename_mismatch_rejected    | unit | [[rename.new_id_matches_filename]]         | `rename_of_a_files_own_intent_id_without_a_matching_filename_change()` | `check(request) == failed`                                                                                                                                                                    |
| partial_failure_rolls_back    | unit | [[rename.atomic_operation]]                | `apply_interrupted_after_definition_row_updated_but_before_all_refs_rewritten()` | `post_state(repo) == pre_state(repo)`                                                                                                                                                          |
| stray_ref_caught_by_verify    | unit | [[rename.old_id_fully_replaced]]           | `rename_that_misses_one_[[old_id]]_occurrence()`         | `check(request) == failed` — caught at `verify`, not silently accepted                                                                                                                        |
| prose_mention_left_alone      | unit | [[rename.prose_untouched_by_rename]]       | `row_whose_rationale_prose_contains_the_old_id_as_a_word()` | `rationale_text(post_rename_row) == rationale_text(pre_rename_row)`                                                                                                                           |
| clean_rename_passes           | unit | [[rename.old_id_fully_replaced]]           | `well_formed_repo(), id_not_used_elsewhere()`            | `check(request) == passed`                                                                                                                                                                    |
| kind_preserved_by_rename      | unit | [[rename.kind_unchanged]]                  | `rename_of_a_constraint_row_to_a_new_id()`               | `assigned_kind(post_rename_row) == assigned_kind(pre_rename_row)` — both the five-object kind and, where present, the row's own kind column survive the rewrite                              |
| validation_failure_label_asserted | unit | [[rename.validation_failure]] | `validation_failure_raised()` | `error_label == "rename.validation_failure"` — renaming the label touches the error Constraint, this property, and its note together (EDGE-002) |
| apply_failure_label_asserted | unit | [[rename.apply_failure]] | `apply_failure_raised()` | `error_label == "rename.apply_failure"` — renaming the label touches the error Constraint, this property, and its note together (EDGE-002) |
| post_check_failure_label_asserted | unit | [[rename.post_check_failure]] | `post_check_failure_raised()` | `error_label == "rename.post_check_failure"` — renaming the label touches the error Constraint, this property, and its note together (EDGE-002) |
## Notes

`checked_against_core: clear` (see `AGENTS.md`'s convention). The
properties this file needs (atomicity, prose-non-interference,
kind-preservation) are local to what a *rename operation* must guarantee,
the same way `compile.md`, `model_check.md`, and `verify.md` each carry
constraints specific to their own step of the lifecycle without requiring
new entries in
`specodelic.md` itself — this file's Intent is the fourth instance of
that pattern, not an exception to it.

`rename_naturality` now exists in four places (`specodelic.md`,
`linter-referential_integrity.md`, `linter-graph_shape.md`'s
`topo_sort_naturality`, and here). That's intentional, not duplication to
clean up: each site states the same law at the altitude that site owns —
`specodelic.md` as the format's top-level refactor-safety guarantee,
the two linter files as "this is exactly the check that would catch a
naive rename tool," and this file as the actual operation those laws are
*about*. This file is the one whose `apply`/`apply_ok`/`apply_fail`
transitions give the law something to be a law of, rather than a fact
asserted with no corresponding process.

**Why `verify` re-runs two checkers instead of all six.** `apply_ok`
already establishes `old_id_fully_replaced` (no dangling old-id refs) and
`kind_unchanged` directly as this file's own constraints. What `apply_ok`
*can't* see on its own is whether the rewrite altered anything at the
repo-graph level it wasn't watching for — a same-file collision the local
check missed, or a graph edge that only becomes visible once every file's
refs are re-indexed together. `linter.referential_integrity` and
`linter.graph_shape` are exactly the two checkers whose owned constraints
(`unique_across_repo`, `ref_resolves`, `acyclic`, `single_root_reachable`)
depend on cross-file state a single-rename's local bookkeeping can't fully
self-certify. The other four checkers (`model_shape`, `ears_syntax`,
`schema_shape`, `coverage`) check properties a rename can't perturb — it
never changes a transition's guard-presence, a statement's EARS pattern,
a table's column shape, or whether a constraint has a deriving property —
so re-running them would be redundant `Needs Human Review` overhead, not
signal.

**Decision of record (2026-09-30, user-approved; advised by a typed
Jev evaluation, `jev-1.13.0`, conf 0.74):** a batch rename is **one
atomic transaction**, not `n` independent ones. All files touched by
the batch are staged and validated as a set — the same
precondition-then-apply-then-reverify shape a single rename already
uses — and any validation failure aborts the whole batch, leaving every
file byte-identical to before. Partial success is exactly the failure
mode the single-rename `atomic_operation` constraint exists to prevent:
a half-renamed namespace is a corpus where `ref_resolves` dangles by
design, and the whole-commit atomicity git already provides makes
per-file partial commits a policy choice a tool shouldn't silently make.
`atomic_operation` therefore means the same thing at batch scope as at
single-rename scope: all-or-nothing. A batch-rename implementation is a
separate follow-up ticket; nothing in this file's constraint text
changes.
