---
id: linter.external_completeness
kind: intent
checked_against_core: clear
statement: "WHEN a spec repo declares an external checklist, THE linter SHALL reject it if any checklist item lacks an explicit mapping to a resolved constraint/property id or an explicit waiver with a stated rationale."
---

# Linter: External Completeness Check

`STATUS.md` §4's long-standing unsolved problem: every gap folded into
`specodelic.md`'s Revision 2 — `id_matches_file`, `ref_kind_compatible`,
`single_root_reachable`, the model well-formedness pair — was found by a
human comparing a new checker file against the existing constraint list by
eye. `linter.coverage` mechanizes *internal* consistency (every present
constraint has a deriving property) but says so explicitly in its own
Notes: it can tell you a present constraint lacks a test, never that a
constraint is missing entirely. This file is the structurally different
thing `STATUS.md` called for — a checker that diffs against an external
reference (a domain checklist, a list of known past incidents, a
stakeholder sign-off list) instead of checking the repo against itself.

Unlike the six checkers in the Checker Ownership table, this one is
**optional per repo**: it only runs when a repo declares a checklist to
check against. A repo with no checklist has nothing external to be
incomplete *relative to* — that's a repo this checker doesn't apply to, not
one it passes vacuously. No file's `linted` state depends on this checker;
see Notes for what it does gate instead.

## Constraints

| id                       | kind      | expr                                                                                                                                      | traces_to | satisfies |
|----------------------------|-----------|------------------------------------------------------------------------------------------------------------------------------------------|----------------------|
| checklist_well_formed      | invariant | `a declared checklist is a flat list of items, each with a stable id and a plain-text description`                                        | [[linter.external_completeness]] |          |
| every_item_accounted       | invariant | `∀ checklist item c: ∃ exactly one mapping row m with m.item == c.id and m.status ∈ {covered, waived}`                                     | [[linter.external_completeness]] |          |
| covered_maps_resolve       | invariant | `∀ mapping row m where m.status == "covered": m.mapped_ids is non-empty, and every id in it resolves to a real constraint or property row` | [[linter.external_completeness]] |          |
| waiver_has_rationale       | invariant | `∀ mapping row m where m.status == "waived": m.rationale is non-empty prose`                                                                | [[linter.external_completeness]] |          |
| no_duplicate_claim         | invariant | `∀ checklist item c: no two mapping rows both target c.id`                                                                                  | [[linter.external_completeness]] |          |
| manifest_failure | effect | `linter.external_completeness.manifest_failure(detail)` | [[linter.external_completeness]] | [[errors.envelope_error_kind]] ∧ [[errors.exit_code_mapping]] ∧ [[errors.remediation_hint_present]] |
| mapping_failure | effect | `linter.external_completeness.mapping_failure(detail)` | [[linter.external_completeness]] | [[errors.envelope_error_kind]] ∧ [[errors.exit_code_mapping]] ∧ [[errors.remediation_hint_present]] |
| resolution_failure | effect | `linter.external_completeness.resolution_failure(detail)` | [[linter.external_completeness]] | [[errors.envelope_error_kind]] ∧ [[errors.exit_code_mapping]] ∧ [[errors.remediation_hint_present]] |

## Model

### States
- `not_applicable`
- `loaded`
- `mapping_checked`
- `passed`
- `load_failed` (emits: `[[linter.external_completeness.manifest_failure]]`)
- `mapping_failed` (emits: `[[linter.external_completeness.mapping_failure]]`)
- `resolution_failed` (emits: `[[linter.external_completeness.resolution_failure]]`)

### Transitions

| id            | from             | to               | guard                                                                                              |
|---------------|------------------|------------------|-----------------------------------------------------------------------------------------------------|
| load          | not_applicable   | loaded           | `repo declares a checklist` ∧ [[linter.external_completeness.checklist_well_formed]]                 |
| load_fail | not_applicable | load_failed | `repo declares a checklist` ∧ `¬([[linter.external_completeness.checklist_well_formed]])` |
| check_mapping | loaded           | mapping_checked  | [[linter.external_completeness.every_item_accounted]] ∧ [[linter.external_completeness.no_duplicate_claim]] |
| check_fail | loaded | mapping_failed | `¬([[linter.external_completeness.every_item_accounted]] ∧ [[linter.external_completeness.no_duplicate_claim]])` |
| accept        | mapping_checked  | passed           | [[linter.external_completeness.covered_maps_resolve]] ∧ [[linter.external_completeness.waiver_has_rationale]] |
| reject | mapping_checked | resolution_failed | `¬([[linter.external_completeness.covered_maps_resolve]] ∧ [[linter.external_completeness.waiver_has_rationale]])` |


## Properties

| id                            | kind | derives_from                                          | generator                                                          | predicate                                                                 |
|---------------------------------|------|------------------------------------------------------------|--------------------------------------------------------------------------|-----------------------------------------------------------------------------|
| unmapped_item_rejected           | unit | [[linter.external_completeness.every_item_accounted]]      | `checklist_item_with_no_mapping_row()`                                    | `check(repo) == failed`                                                    |
| dangling_mapped_id_rejected      | unit | [[linter.external_completeness.covered_maps_resolve]]      | `mapping_row_with(status: "covered", mapped_ids: ["nonexistent.id"])`     | `check(repo) == failed`                                                    |
| unrationalized_waiver_rejected   | unit | [[linter.external_completeness.waiver_has_rationale]]      | `mapping_row_with(status: "waived", rationale: "")`                       | `check(repo) == failed`                                                    |
| duplicate_claim_rejected         | unit | [[linter.external_completeness.no_duplicate_claim]]        | `two_mapping_rows_targeting_the_same_checklist_item()`                    | `check(repo) == failed`                                                    |
| fully_mapped_checklist_passes    | unit | [[linter.external_completeness.every_item_accounted]]      | `checklist_where_every_item_has_a_covered_or_waived_mapping()`            | `check(repo) == passed`                                                    |
| no_checklist_not_applicable      | unit | [[linter.external_completeness.every_item_accounted]]                            | `repo_with_no_declared_checklist()`                                       | `check(repo) == not_applicable` — distinct from `passed`, see Notes         |
| mapping_naturality               | law  | [[specodelic.rename_naturality]]                            | `arbitrary_repo_with_checklist(), arbitrary_id_rename()`                  | **identity:** `mapped(repo renamed to itself) == mapped(repo)` — the rename identity case instantiated at the mapped-ids observation point  **naturality:** `mapped(rename(I)) == rename(mapped(I))` — renaming a constraint or property id updates every `mapped_ids` cell claiming it, the same as any other reference |
| malformed_checklist_rejected     | unit | [[linter.external_completeness.checklist_well_formed]]      | `declared_checklist_with(a_nested_item, an_item_missing_its_id)`          | `check(repo) == failed` |
| manifest_failure_label_asserted | unit | [[linter.external_completeness.manifest_failure]] | `manifest_failure_raised()` | `error_label == "linter.external_completeness.manifest_failure"` — renaming the label touches the error Constraint, this property, and its note together (EDGE-002) |
| mapping_failure_label_asserted | unit | [[linter.external_completeness.mapping_failure]] | `mapping_failure_raised()` | `error_label == "linter.external_completeness.mapping_failure"` — renaming the label touches the error Constraint, this property, and its note together (EDGE-002) |
| resolution_failure_label_asserted | unit | [[linter.external_completeness.resolution_failure]] | `resolution_failure_raised()` | `error_label == "linter.external_completeness.resolution_failure"` — renaming the label touches the error Constraint, this property, and its note together (EDGE-002) |
## Notes

**What this checker can and can't prove — read carefully, this is the
whole point of the file.** `every_item_accounted`, `covered_maps_resolve`,
and `waiver_has_rationale` together prove that a human explicitly asserted,
for every checklist item, either *"constraint X covers this"* (checked
structurally: X exists) or *"this doesn't apply, because ___"* (checked
structurally: a rationale was written). What none of this proves is that
the assertion is *true* — that constraint X actually, semantically,
addresses the checklist item it's mapped to, or that the waiver's stated
reason is a good one. That gap is irreducible from inside this framework:
whether a mapping is semantically correct is a judgment call about meaning,
the same category of thing `STATUS.md` §2 keeps out of scope for prose
quality. What this checker mechanizes isn't "the repo is complete" — it's
"someone was forced to look at every item and commit to a claim about it,
on the record, instead of the checklist silently going unconsulted." That
is a real, checkable weakening of the original problem, not a full
solution to it — the same honest half-measure `linter.coverage` makes about
internal completeness, one level out.

**Why this doesn't gate `linted`, `compiled`, or anything else in
`specodelic.md`'s lifecycle.** The six Checker Ownership table checkers
and `linter.coverage` all check something every spec file necessarily has
(frontmatter, refs, a model, constraints). A checklist is an artifact nothing
requires a repo to declare — bolting a mandatory gate onto an optional
input would make every repo without a checklist unable to reach `verified`,
which is wrong. Instead this stands as an independent, addressable gate: a
repo *may* declare a checklist and run this against it, and CI or an
orchestrator (`STATUS.md` §4's remaining P1 item) can choose to require it
for release the same way a real org's ship gate might require a security
sign-off — a policy decision layered on top of `specodelic`, not a fact
`specodelic.md` itself asserts about every repo.

**Decision of record (mp1 row 10, 2026-09-29): the checklist is a
dedicated `*.checklist.md` artifact OUTSIDE `𝒦`.** A checklist manifest
is not a spec file — no frontmatter, no four-layer shape, exempt like
`AGENTS.md`/`STATUS.md`. Its format: a `## Items` section holding a flat
list of `- **<id>**: <plain-text description>` bullets (ids are stable
row-id tokens: alphanumeric segments joined by `.`/`_`/`-`), plus a
`## Mapping` section holding exactly one table with the columns
`item`/`status`/`mapped_ids`/`rationale`. Presence of the file IS the
declaration — a repo with no `*.checklist.md` has nothing external to be
incomplete relative to (`not_applicable`, not a vacuous pass). Mapping
rows must reference declared items; statuses outside {covered, waived}
leave the item unaccounted-for (`every_item_accounted`'s beat); a
duplicated item claim is `no_duplicate_claim`'s beat, never double-
reported as unaccounted. A manifest that declares zero items is itself
a `checklist_well_formed` defect — a checklist nothing can be consulted
against is not a checklist, and a silent pass would be a false green.
The linter tolerates a UTF-8 BOM and `### `-depth headers inside a
section (prose, never section switches). The degenerate-spec-file option (empty
Model/Constraints, frontmatter `id: spec`) was rejected: it would grow
`𝒦` after all, drag the naming law and dual-format machinery onto an
artifact that has no intent to state, for no benefit this checker needs.

**`mapping_naturality` is enforced by the rename tool.** The checklist
sits outside `𝒦`, so `linter.referential_integrity` itself never parses
the manifest — but `rename.md`'s `old_id_fully_replaced` reaches into
`mapped_ids` cells (bare and `[[…]]` spellings alike, children
following their parent), and `rename`'s verify gate re-runs this
checker's resolution (`covered_maps_resolve`, plus manifest
well-formedness) over the post-rename corpus. A rename that misses a
cell is rejected before any byte is written, and a rename that would
dangle a cell is equally refused.

`checked_against_core: clear` (see `AGENTS.md`'s convention). This file's
constraints are local to what an *external-completeness check*
specifically needs, the same way `compile.md`/`model_check.md`/
`verify.md`/`rename.md` each carry lifecycle-step-specific invariants
without requiring new top-level entries.
