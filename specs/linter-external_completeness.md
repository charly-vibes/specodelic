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

| id                       | kind      | expr                                                                                                                                      | traces_to |
|----------------------------|-----------|------------------------------------------------------------------------------------------------------------------------------------------|-----------|
| checklist_well_formed      | invariant | `a declared checklist is a flat list of items, each with a stable id and a plain-text description`                                        | [[linter.external_completeness]] |
| every_item_accounted       | invariant | `∀ checklist item c: ∃ exactly one mapping row m with m.item == c.id and m.status ∈ {covered, waived}`                                     | [[linter.external_completeness]] |
| covered_maps_resolve       | invariant | `∀ mapping row m where m.status == "covered": m.mapped_ids is non-empty, and every id in it resolves to a real constraint or property row` | [[linter.external_completeness]] |
| waiver_has_rationale       | invariant | `∀ mapping row m where m.status == "waived": m.rationale is non-empty prose`                                                                | [[linter.external_completeness]] |
| no_duplicate_claim         | invariant | `∀ checklist item c: no two mapping rows both target c.id`                                                                                  | [[linter.external_completeness]] |

## Model

### States
- `not_applicable`
- `loaded`
- `mapping_checked`
- `passed`
- `failed`

### Transitions

| id            | from             | to               | guard                                                                                              |
|---------------|------------------|------------------|-----------------------------------------------------------------------------------------------------|
| load          | not_applicable   | loaded           | `repo declares a checklist` ∧ [[linter.external_completeness.checklist_well_formed]]                 |
| load_fail     | not_applicable   | failed           | `repo declares a checklist` ∧ `¬load.guard`                                                          |
| check_mapping | loaded           | mapping_checked  | [[linter.external_completeness.every_item_accounted]] ∧ [[linter.external_completeness.no_duplicate_claim]] |
| check_fail    | loaded           | failed           | `¬check_mapping.guard`                                                                                |
| accept        | mapping_checked  | passed           | [[linter.external_completeness.covered_maps_resolve]] ∧ [[linter.external_completeness.waiver_has_rationale]] |
| reject        | mapping_checked  | failed           | `¬accept.guard`                                                                                       |

## Properties

| id                            | kind | derives_from                                          | generator                                                          | predicate                                                                 |
|---------------------------------|------|------------------------------------------------------------|--------------------------------------------------------------------------|-----------------------------------------------------------------------------|
| unmapped_item_rejected           | unit | [[linter.external_completeness.every_item_accounted]]      | `checklist_item_with_no_mapping_row()`                                    | `check(repo) == failed`                                                    |
| dangling_mapped_id_rejected      | unit | [[linter.external_completeness.covered_maps_resolve]]      | `mapping_row_with(status: "covered", mapped_ids: ["nonexistent.id"])`     | `check(repo) == failed`                                                    |
| unrationalized_waiver_rejected   | unit | [[linter.external_completeness.waiver_has_rationale]]      | `mapping_row_with(status: "waived", rationale: "")`                       | `check(repo) == failed`                                                    |
| duplicate_claim_rejected         | unit | [[linter.external_completeness.no_duplicate_claim]]        | `two_mapping_rows_targeting_the_same_checklist_item()`                    | `check(repo) == failed`                                                    |
| fully_mapped_checklist_passes    | unit | [[linter.external_completeness.every_item_accounted]]      | `checklist_where_every_item_has_a_covered_or_waived_mapping()`            | `check(repo) == passed`                                                    |
| no_checklist_not_applicable      | unit | [[linter.external_completeness]]                            | `repo_with_no_declared_checklist()`                                       | `check(repo) == not_applicable` — distinct from `passed`, see Notes         |
| mapping_naturality               | law  | [[specodelic.rename_naturality]]                            | `arbitrary_repo_with_checklist(), arbitrary_id_rename()`                  | **naturality:** `mapped(rename(I)) == rename(mapped(I))` — renaming a constraint or property id updates every `mapped_ids` cell claiming it, the same as any other reference |
| malformed_checklist_rejected     | unit | [[linter.external_completeness.checklist_well_formed]]      | `declared_checklist_with(a_nested_item, an_item_missing_its_id)`          | `check(repo) == failed` |

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

**`mapped_ids` is a reference the existing Reference Typing table doesn't
name — flagged, not resolved here.** A `covered` mapping row's `mapped_ids`
field points at a Constraint or Property row exactly the way `traces_to`
or `derives_from` do, and `mapping_naturality` above only holds if the
rename tool and `linter.referential_integrity` actually walk into a
checklist's mapping table when rewriting `[[old_id]]`. But a checklist
file isn't a specodelic file — it has no frontmatter `kind: intent`, no
Constraints/Model/Properties layers — so it sits outside the four-layer
shape `AGENTS.md` requires of "every new file." Extending Reference Typing
to cover it means deciding whether a checklist manifest is a sixth *kind*
of artifact outside `𝒦` (STATUS.md §1 calls the whole schema category `𝒦`
fixed) or a degenerate spec file that reuses the existing shape with empty
Model/Constraints sections and only a Properties-like mapping table. Both
are real options with different consequences for `rename.md`'s
`old_id_fully_replaced` guarantee; **Needs Human Review**, not decided by
this file. Until it is, treat `mapping_naturality` above as aspirational —
asserted the way `specodelic.md` asserts laws before their enforcing
checker exists, not yet backed by a rename tool that actually reaches this
far.

`checked_against_core: clear` (see `AGENTS.md`'s convention). This file's
constraints are local to what an *external-completeness check*
specifically needs, the same way `compile.md`/`model_check.md`/
`verify.md`/`rename.md` each carry lifecycle-step-specific invariants
without requiring new top-level entries.
