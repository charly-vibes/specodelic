---
id: merge
kind: intent
checked_against_core: clear
statement: "WHEN two branches derived from a common ancestor spec repo are
  joined, THE merge tool SHALL detect id collisions and reference breaks
  introduced by the union of their edits — using [[graph]]'s blast-radius
  query rather than re-deriving reachability independently — before
  reporting the merge complete, and SHALL never report success against a
  tree that [[linter.referential_integrity]] or [[linter.graph_shape]]
  would reject."
---

# Merge

A textually clean 3-way merge (no conflict markers) can still produce a
semantically broken repo: two branches mint the same new id independently,
or one branch renames an id (`rename.md`) while the other adds a fresh
reference to the old name in a file the rename never touched. Git's
line-based merge can't see either problem — both are clean at the text
layer. This file specifies the check that runs *after* git's merge
succeeds: it reuses `graph.md`'s derived reachability rather than
re-deriving it, and `rename.md`'s actual rewrite mechanism rather than
inventing a second one, applied across two histories instead of one linear
line — the "existing mechanism, new scope" move `AGENTS.md` #3a asks for
before adding anything new.

## Constraints

| id                                    | kind      | expr                                                                                                                                                                                    | traces_to  satisfies |
|------------------------------------------|-----------|--------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|----------------------|
| graph_reused_not_rederived                | invariant | `merge's collision and blast-radius checks query [[graph]]'s artifact for both branch tips; merge never independently re-walks markdown for reachability`                                   | [[merge]] |          |
| no_new_id_collision                       | invariant | `∀ id ∈ index(branch_A) ∩ index(branch_B): id was already defined, identically, in the common ancestor` — an id newly minted by both branches independently is a collision, not a merge  | [[merge]] |          |
| blast_radii_recorded_pre_merge            | invariant | `before applying the union of edits, merge records [[graph]].blast_radius(touched_ids) for each branch separately, against the common ancestor's graph`                                    | [[merge]] |          |
| semantic_conflict_iff_blast_radius_intersects | invariant | `blast_radius_A(touched_A) ∩ blast_radius_B(touched_B) ≠ ∅ ⟹ the merge requires human review, even when the two branches' literal file diffs don't overlap`                          | [[merge]] |          |
| rename_replayed_onto_foreign_edits        | invariant | `if branch A contains a [[rename]] application of (old_id, new_id) and branch B independently adds a new [[old_id]] reference within blast_radius_A, the merge rewrites that reference to [[new_id]] using rename.md's own rewrite mechanism — it is never left dangling and never silently dropped` | [[merge]] |          |
| post_merge_relint_required                | invariant | `merge is not reported passed until [[linter.referential_integrity]] and [[linter.graph_shape]] are re-run against the merged tree and both report passed` | [[merge]] |          |
| sequential_number_reassigned_on_conflict  | invariant | `if both branches independently claim the same next sequential number (a CHANGELOG entry or Revision heading), the merge tool renumbers one of the two rather than allowing a silent duplicate` | [[merge]] |          |
| collision_failure | effect | `merge.collision_failure(detail) — the label names its owning file per error_expr_shape` | [[merge]] | [[errors.envelope_error_kind]] ∧ [[errors.exit_code_mapping]] ∧ [[errors.remediation_hint_present]] |
| reverification_failure | effect | `merge.reverification_failure(detail) — the label names its owning file per error_expr_shape` | [[merge]] | [[errors.envelope_error_kind]] ∧ [[errors.exit_code_mapping]] ∧ [[errors.remediation_hint_present]] |
| merge_aborted | effect | `merge.merge_aborted(detail) — the label names its owning file per error_expr_shape` | [[merge]] | [[errors.envelope_error_kind]] ∧ [[errors.exit_code_mapping]] ∧ [[errors.remediation_hint_present]] |

## Model

### States
- `diverged`
- `collision_checked`
- `needs_review`
- `rename_replayed`
- `ref_integrity_reverified`
- `merged`
- `collision_failed` (emits: `[[merge.collision_failure]]`)
- `reverification_failed` (emits: `[[merge.reverification_failure]]`)
- `merge_failed` (emits: `[[merge.merge_aborted]]`)

### Transitions

| id               | from                      | to                        | guard                                                                                                     |
|------------------|---------------------------|----------------------------|---------------------------------------------------------------------------------------------------------------|
| check_collisions | diverged                  | collision_checked           | [[merge.no_new_id_collision]] ∧ [[merge.graph_reused_not_rederived]]                                        |
| collision_fail | diverged | collision_failed | `¬([[merge.no_new_id_collision]] ∧ [[merge.graph_reused_not_rederived]])` |
| flag_review      | collision_checked         | needs_review                 | [[merge.semantic_conflict_iff_blast_radius_intersects]] ∧ [[merge.blast_radii_recorded_pre_merge]]           |
| auto_proceed     | collision_checked         | rename_replayed              | `¬flag_review.guard` ∧ [[merge.rename_replayed_onto_foreign_edits]]                                          |
| resolved         | needs_review              | rename_replayed              | `a human has explicitly approved merging the intersecting blast radii` ∧ [[merge.rename_replayed_onto_foreign_edits]] |
| reverify         | rename_replayed           | ref_integrity_reverified     | [[merge.post_merge_relint_required]]                                                                          |
| reverify_fail | rename_replayed | reverification_failed | `¬([[merge.post_merge_relint_required]])` |
| accept           | ref_integrity_reverified  | merged                       | [[merge.sequential_number_reassigned_on_conflict]]                                                             |
| reject | ref_integrity_reverified | merge_failed | `¬([[merge.sequential_number_reassigned_on_conflict]])` |


## Properties

| id                                       | kind | derives_from                                          | generator                                                                                       | predicate                                                                                                   |
|---------------------------------------------|------|------------------------------------------------------------|-------------------------------------------------------------------------------------------------------|-------------------------------------------------------------------------------------------------------------------|
| collision_across_branches_rejected           | unit | [[merge.no_new_id_collision]]                             | `two_branches_each_independently_defining_new_id("x")`                                                 | `check(merge) == failed`                                                                                            |
| inherited_id_not_falsely_flagged             | unit | [[merge.no_new_id_collision]]                             | `id_defined_in_common_ancestor_unchanged_in_both_branches()`                                            | `check(merge) != failed`                                                                                            |
| textually_clean_semantic_conflict_flagged    | unit | [[merge.semantic_conflict_iff_blast_radius_intersects]]   | `branch_A(renames x to y), branch_B(adds a new [[x]] reference in a file branch_A never touched)`      | `check(merge) == needs_review`                                                                                      |
| rename_replay_repairs_dangling_ref           | unit | [[merge.rename_replayed_onto_foreign_edits]]              | `same scenario, after human approval`                                                                   | `post_merge_repo has zero occurrences of [[x]]; branch_B's new reference now reads [[y]]`                            |
| disjoint_blast_radii_auto_merge              | unit | [[merge.semantic_conflict_iff_blast_radius_intersects]]   | `two_branches_touching_disjoint_namespaces_with_disjoint_blast_radii()`                                 | `check(merge) == merged` — no human review required                                                                  |
| duplicate_sequential_number_renumbered       | unit | [[merge.sequential_number_reassigned_on_conflict]]        | `both_branches_add_a_CHANGELOG_entry_claiming_the_same_next_number()`                                   | `post_merge_repo has two distinct sequential numbers; neither is duplicated`                                         |
| relint_gates_merge                           | unit | [[merge.post_merge_relint_required]]                      | `merge_where(linter.referential_integrity or linter.graph_shape: fails against the merged tree)`        | `check(merge) == failed` — mirrors `rename.md`'s `stray_ref_caught_by_verify`                                        |
| blast_radius_recorded_before_apply           | unit | [[merge.blast_radii_recorded_pre_merge]]                  | `merge_history_where_edits_were_applied_before_both_branches'_radii_were_recorded()`                    | `check(merge) == failed` — apply never precedes the recording step, for either branch                                |
| reachability_from_graph_artifact_only        | unit | [[merge.graph_reused_not_rederived]]                      | `merge_invoked_with_the_markdown_walker_patched_to_panic()`                                             | `merge of an otherwise-clean diverged repo succeeds` — collision and blast-radius checks only query [[graph]]'s artifact |
| collision_failure_label_asserted | unit | [[merge.collision_failure]] | `collision_failure_raised()` | `error_label == "merge.collision_failure"` — renaming the label touches the error Constraint, this property, and its note together (EDGE-002) |
| reverification_failure_label_asserted | unit | [[merge.reverification_failure]] | `reverification_failure_raised()` | `error_label == "merge.reverification_failure"` — renaming the label touches the error Constraint, this property, and its note together (EDGE-002) |
| merge_aborted_label_asserted | unit | [[merge.merge_aborted]] | `merge_aborted_raised()` | `error_label == "merge.merge_aborted"` — renaming the label touches the error Constraint, this property, and its note together (EDGE-002) |
## Notes

`checked_against_core: clear` (see `AGENTS.md`'s convention).
`rename_replayed_onto_foreign_edits`
was checked against `AGENTS.md` #3a specifically before being added as a
new row rather than reusing `rename.md`'s `old_id_fully_replaced` as-is:
it isn't a duplicate, because it operates over *two divergent histories*
being joined, not one linear history — `old_id_fully_replaced` guarantees
completeness within a single rename's own transaction, and says nothing
about a reference minted on a different branch that the rename operation
never saw. This file's constraint is what makes that reference get
rewritten anyway, once the branches meet.

**What this file doesn't replace.** Two edits to the same row's own field
on both branches is an ordinary textual conflict — git already marks it,
already forces a human to resolve it, and nothing here changes that. This
file exists only for the cases where the text merge is clean but the
result isn't: id collisions and rename/reference splits that don't
overlap a single line.

**Depends on `graph.md`, not a local reachability walk.** Every blast-radius
computation in this file's Constraints is a `graph` query, computed once
against the common ancestor for each branch (`blast_radii_recorded_pre_merge`)
rather than recomputed after the union is applied — recomputing post-union
would already have mixed the two branches' edges together and lost the
ability to tell which branch's change reached which id.

**Depends on `rename.md`, not a second rewrite mechanism.** `resolved`/
`auto_proceed`'s rewrite step calls the same rewrite `rename.md`'s `apply`
transition performs — this file only decides *when* a cross-branch replay
is required, never how the rewrite itself works.

**Open question, `Needs Human Review`:** what "a human has explicitly
approved" (the `resolved` transition's guard) means operationally — a
required approving review comment, a CI gate, a specific role — is left
unspecified here, the same way `rename.md` leaves batch-rename atomicity
open rather than assuming an answer.

**Open question, relationship to `refactor.md`:** a blast-radius
intersection that triggers `needs_review` here may also be exactly the
"unrelated fan-in" shape `refactor.md` flags as a tidy-first candidate —
both files answer different questions (can this merge proceed safely, vs.
should this node be split before more changes land on it) and are
deliberately left as two files rather than merged into one, but a
same-session finding from both on the same node is worth surfacing
together rather than as two unrelated notices. Left open rather than
wired automatically.
