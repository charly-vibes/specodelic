---
id: linter.coverage
kind: intent
statement: "WHEN a spec file's graph shape and model shape have both passed, THE linter SHALL reject it if any constraint has no property deriving from it, or if any law-kind property is missing its associativity or identity case."
---

# Linter: Coverage Check

The last checker in the Checker Ownership table — it needs `graph_shape`'s
acyclicity result to know the `derives_from` edges it's counting form a
well-defined graph, and `model_shape`'s result to know every constraint it's
checking coverage for is actually attached to a real transition, not a
dangling row that would make "100% coverage" a vacuous claim.

This is the check most directly aimed at the gap this whole exercise kept
surfacing — every prior checker file found something missing from
`specodelic.md` itself. Coverage is what would have caught those
mechanically instead of by manual inspection: `id_matches_file`,
`ref_kind_compatible`, `single_root_reachable`, and the model
well-formedness pair were all constraints with no corresponding property
until Revision 2 added both together. Had this checker existed first, the
gap and its fix would have been a single failing/passing pair, not two
separate passes.

## Constraints

| id                      | kind      | expr                                                                                     | traces_to                       |
|---------------------------|-----------|-----------------------------------------------------------------------------------------------|------------------------------------|
| every_constraint_covered  | invariant | `∀ constraint c: ∃ property p. p.derives_from == c.id`                                        | [[specodelic.coverage]]         |
| every_law_has_cases       | invariant | `∀ property p where p.kind == "law": p has an associativity case and an identity case`         | [[specodelic.law_requires_cases]] |
| no_orphan_property        | invariant | `∀ property p: p.derives_from resolves to a real constraint` (restates total_refs, scoped to this edge) | [[specodelic.coverage]]         |
| coverage_is_computable    | invariant | `the derives_from multiplicity per constraint is countable in finite time from the parsed AST alone` | [[specodelic.coverage]]         |

## Model

### States
- `unchecked`
- `counting`
- `law_checking`
- `passed`
- `failed`

### Transitions

| id             | from          | to            | guard                                                                                     |
|----------------|---------------|---------------|-----------------------------------------------------------------------------------------------|
| begin          | unchecked     | counting      | `file passed linter.graph_shape and linter.model_shape`                                       |
| count_ok       | counting      | law_checking  | [[linter.coverage.every_constraint_covered]] ∧ [[linter.coverage.no_orphan_property]]           |
| count_fail     | counting      | failed        | `¬count_ok.guard`                                                                              |
| accept         | law_checking  | passed        | [[linter.coverage.every_law_has_cases]]                                                        |
| reject         | law_checking  | failed        | `¬accept.guard`                                                                                |

## Properties

| id                        | kind | derives_from                                     | generator                                                     | predicate                                                                 |
|-----------------------------|------|-------------------------------------------------------|--------------------------------------------------------------------|-----------------------------------------------------------------------------|
| uncovered_constraint_rejected | unit | [[linter.coverage.every_constraint_covered]]         | `spec_file_with(constraint_with_no_deriving_property: true)`         | `check(file) == failed`                                                    |
| orphan_property_rejected      | unit | [[linter.coverage.no_orphan_property]]               | `spec_file_with(property.derives_from_pointing_at_nonexistent_id: true)` | `check(file) == failed`                                                 |
| incomplete_law_rejected       | unit | [[linter.coverage.every_law_has_cases]]              | `law_property_missing("identity")`                                    | `check(file) == failed`                                                   |
| full_coverage_passes          | unit | [[linter.coverage.every_constraint_covered]]         | `arbitrary_fully_covered_spec_file()`                                 | `check(file) == passed`                                                   |
| coverage_naturality           | law  | [[specodelic.rename_naturality]]                    | `arbitrary_spec_file(), arbitrary_id_rename()`                        | **naturality:** `coverage_ratio(rename(I)) == coverage_ratio(I)` — renaming a constraint doesn't change whether it's covered |

## Notes

`coverage_is_computable` is worth stating explicitly even though it looks
tautological: it's the thing that fails if `expr` fields were ever allowed
to contain arbitrary logic requiring evaluation to determine
`derives_from` targets dynamically (e.g. a templated or generated id).
`specodelic.md`'s decision to keep `id` values as static string literals
in every table row — never computed — is what makes this checker decidable
at all. If that ever changes, this constraint is the one that breaks first
and should be treated as a tripwire, not just a check.

No new gaps in `specodelic.md` surfaced this time — this is the third
checker of seven to close clean, and notably the one built specifically to
catch what the first four checkers found by hand. That its own pass here
is clean is a weaker signal than it looks: it confirms `specodelic.md`'s
constraint list is self-consistent as of Revision 2, not that Revision 2
is complete — a coverage check can only tell you every constraint present
has a property; it cannot tell you a constraint is missing, which is
exactly the class of gap the first four checkers caught by inspection
rather than by any mechanism this framework has yet.

**All six checker files in the Checker Ownership table now exist**
(`linter-frontmatter`, `linter-referential_integrity`,
`linter-graph_shape`, `linter-model_shape`, `linter-ears_syntax`,
`linter-schema_shape`) — `linted` in `specodelic.md` is fully specified.
`linter-coverage.md` itself is not a row in that table; it gates `compile`
separately, one step later in the pipeline.
