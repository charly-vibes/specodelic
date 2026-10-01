---
id: linter.coverage
kind: intent
statement: "WHEN a spec file's graph shape and model shape have both passed, THE linter SHALL reject it if any constraint has no property deriving from it, or if any law-kind property fails to enumerate its required cases as machine-findable `**name:**` labels with the identity and associativity floor present."
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

| id                      | kind      | expr                                                                                     | traces_to | satisfies |
|---------------------------|-----------|-----------------------------------------------------------------------------------------------|-----------------------------------------------|
| every_constraint_covered  | invariant | `∀ constraint c: ∃ property p. p.derives_from == c.id` — the rule this row re-owns from `specodelic.md`'s higher-altitude restatement (see Notes) | [[linter.coverage]]         |          |
| every_law_has_cases       | invariant | `∀ property p where p.kind == "law": p's predicate enumerates its required cases as **name:** case labels, and the label set includes identity and associativity` — the rule this row re-owns from `specodelic.md`'s higher-altitude restatement (see Notes; machine form ratified in specodelic.md Revision 13) | [[linter.coverage]] |          |
| no_orphan_property        | invariant | `∀ property p: p.derives_from resolves to a real constraint` (restates total_refs, scoped to this edge) | [[linter.coverage]]         |          |
| coverage_is_computable    | invariant | `the derives_from multiplicity per constraint is countable in finite time from the parsed AST alone` | [[linter.coverage]]         |          |
| count_failure | effect | `linter.coverage.count_failure(detail)` | [[linter.coverage]] | [[errors.envelope_error_kind]] ∧ [[errors.exit_code_mapping]] ∧ [[errors.remediation_hint_present]] |
| law_case_failure | effect | `linter.coverage.law_case_failure(detail)` | [[linter.coverage]] | [[errors.envelope_error_kind]] ∧ [[errors.exit_code_mapping]] ∧ [[errors.remediation_hint_present]] |

## Model

### States
- `unchecked`
- `counting`
- `law_checking`
- `passed`
- `count_failed` (emits: `[[linter.coverage.count_failure]]`)
- `law_check_failed` (emits: `[[linter.coverage.law_case_failure]]`)

### Transitions

| id             | from          | to            | guard                                                                                     |
|----------------|---------------|---------------|-----------------------------------------------------------------------------------------------|
| begin          | unchecked     | counting      | `file passed linter.graph_shape and linter.model_shape`                                       |
| count_ok       | counting      | law_checking  | [[linter.coverage.every_constraint_covered]] ∧ [[linter.coverage.no_orphan_property]]           |
| count_fail | counting | count_failed | `¬([[linter.coverage.every_constraint_covered]] ∧ [[linter.coverage.no_orphan_property]])` |
| accept         | law_checking  | passed        | [[linter.coverage.every_law_has_cases]]                                                        |
| reject | law_checking | law_check_failed | `¬([[linter.coverage.every_law_has_cases]])` |


## Properties

| id                        | kind | derives_from                                     | generator                                                     | predicate                                                                 |
|-----------------------------|------|-------------------------------------------------------|--------------------------------------------------------------------|-----------------------------------------------------------------------------|
| uncovered_constraint_rejected | unit | [[linter.coverage.every_constraint_covered]]         | `spec_file_with(constraint_with_no_deriving_property: true)`         | `check(file) == failed`                                                    |
| orphan_property_rejected      | unit | [[linter.coverage.no_orphan_property]]               | `spec_file_with(property.derives_from_pointing_at_nonexistent_id: true)` | `check(file) == failed`                                                 |
| incomplete_law_rejected       | unit | [[linter.coverage.every_law_has_cases]]              | `law_property_missing("identity")`                                    | `check(file) == failed`                                                   |
| full_coverage_passes          | unit | [[linter.coverage.every_constraint_covered]]         | `arbitrary_fully_covered_spec_file()`                                 | `check(file) == passed`                                                   |
| coverage_naturality           | law  | [[specodelic.rename_naturality]]                    | `arbitrary_spec_file(), arbitrary_id_rename()`                        | **identity:** `coverage_ratio(coverage_ratio_placeholder_renamed_to_itself) == coverage_ratio(placeholder)` — the rename identity case, instantiated at the coverage_ratio observation point  **associativity:** `coverage_ratio(rename(rename(I, a, b), b, c)) == coverage_ratio(rename(I, a, c))` — the rename associativity case at the same observation point  **naturality:** `coverage_ratio(rename(I)) == coverage_ratio(I)` — renaming a constraint doesn't change whether it's covered |
| computed_derives_from_rejected | unit | [[linter.coverage.coverage_is_computable]]          | `spec_file_with(computed_or_templated_derives_from_id: true)`         | `check(file) == failed` — a non-literal id cannot be counted from the parsed AST alone |
| count_failure_label_asserted | unit | [[linter.coverage.count_failure]] | `count_failure_raised()` | `error_label == "linter.coverage.count_failure"` — renaming the label touches the error Constraint, this property, and its note together (EDGE-002) |
| law_case_failure_label_asserted | unit | [[linter.coverage.law_case_failure]] | `law_case_failure_raised()` | `error_label == "linter.coverage.law_case_failure"` — renaming the label touches the error Constraint, this property, and its note together (EDGE-002) |
| unlabeled_law_rejected       | unit | [[linter.coverage.every_law_has_cases]]              | `law_predicate_mentioning_cases_without_labels()`                     | `check(file) == failed` — a prose mention of a case name is not an enumeration (Revision 13 machine form) |
| extra_case_is_declaration    | unit | [[linter.coverage.every_law_has_cases]]              | `law_property_with_extra_case("commutativity")`                       | `compile(file) emits one block per enumerated case` — an extra named case is a first-class checkable declaration, never prose |
## Notes

**Machine-form ratification (2026-10-01, `specodelic-9qw`):**
`every_law_has_cases` now executes `specodelic.md` Revision 13: a law
row's required cases are whatever its predicate enumerates as
`**name:**` case labels — the exact form `compile`'s
`required_law_cases` has always parsed — with the identity and
associativity floor mandatory. A predicate that only mentions a case
name in prose is not an enumeration and fails the check; the label set
may exceed the floor freely (see `extra_case_is_declaration`), which
is what makes extra cases (commutativity, idempotence, ...) checkable
declarations that compile one proptest block each.

**Reference-typing reconciliation (2026-09-30, `specodelic-cxq`):**
`every_constraint_covered`, `every_law_has_cases`, `no_orphan_property`,
and `coverage_is_computable` previously pointed their `traces_to` at the
`specodelic.md` rows they re-own (`[[specodelic.coverage]]`,
`[[specodelic.law_requires_cases]]`) — a Constraint→Constraint target,
which the Reference Typing table forbids (`traces_to` resolves to Intent
only). Each now traces to `[[linter.coverage]]`, its own file's intent —
the same re-anchoring `linter-referential_integrity.md`'s
`rename_naturality` Notes call the "same claim, two altitudes" pattern:
`specodelic.md` keeps the corpus-wide statement of record, this file owns
the checkable one.

**Decision of record (2026-09-30, `specodelic-cxq`):** `coverage_naturality`
was one of the three law rows lacking the identity/associativity cases
`[[specodelic.law_requires_cases]]` requires. Its identity case is the
trivial instantiations `rename(I, a, a) == I` at the `coverage_ratio`
observation point; the naturality case remains the substantive one —
renaming a constraint doesn't change whether it's covered.

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
