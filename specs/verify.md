---
id: verify
kind: intent
checked_against_core: clear
statement: "THE verify step SHALL run every proptest! block compile.md's properties_to_proptest produced to completion, and SHALL transition a file to verified only when all of them pass and model_check.md's most recent run against the current compiled artifact reported clean."
---

# Verify

`specodelic.md` names `verify` as the `model_checked → verified`
transition, guarded by `no_counterexample ∧ properties_pass` — but neither
what "running the properties" means nor how that conjunction is actually
enforced was ever specified (`STATUS.md` §4, P0, last of the three
pipeline items). This file closes it: `verify` is what actually executes
`compile.md`'s output and combines it with `model_check.md`'s outcome
into the one fact `specodelic.md`'s lifecycle calls `verified`.

## Constraints

| id                              | kind      | expr                                                                                                                                          | traces_to  satisfies |
|------------------------------------|-----------|------------------------------------------------------------------------------------------------------------------------------------------------------|----------------------|
| blocks_run_to_completion            | invariant | `verify actually executes every proptest! block compile.md produced against the current compiled artifact — properties_pass is never satisfied by a partial run or a cached prior result` | [[verify]] |          |
| properties_pass_reflects_latest_run | invariant | `[[specodelic.properties_pass]] holds iff every block from the most recent run against the current compiled artifact passed; a run predating the last edit to Properties, Constraints, or Model does not count` | [[verify]] |          |
| failure_reports_shrunk_counterexample | invariant | `when a proptest! block fails, the report gives the shrunk (minimal) failing input, not the first randomly-generated failing input encountered` | [[verify]] |          |
| both_gates_required                 | invariant | `verify only transitions a file to verified when both [[specodelic.no_counterexample]] (model_check.md's clean outcome) and [[specodelic.properties_pass]] hold — passing properties alone, or a clean model alone, is insufficient` | [[verify]] |          |
| law_cases_all_run                   | invariant | `for a law-kind property compiled into multiple blocks (one per required case), all compiled blocks must pass — a partial pass (identity passes, associativity fails) is a failure of the property, not a partial success` | [[verify]] |          |
| verify_is_idempotent                | invariant | `re-running verify against an unchanged compiled artifact yields the same pass/fail outcome as the prior run, even though proptest may resample inputs each run for coverage` | [[verify]] |          |
| verification_failure | effect | `verify.verification_failure(detail) — the label names its owning file per error_expr_shape` | [[verify]] | [[errors.envelope_error_kind]] ∧ [[errors.exit_code_mapping]] ∧ [[errors.remediation_hint_present]] |

## Model

### States
- `not_run`
- `running`
- `properties_evaluated`
- `verified`
- `failed` (emits: `[[verify.verification_failure]]`)

### Transitions

| id       | from                 | to                   | guard                                                                                                       |
|----------|----------------------|----------------------|--------------------------------------------------------------------------------------------------------------|
| begin    | not_run              | running              | [[verify.blocks_run_to_completion]]                                                                          |
| evaluate | running              | properties_evaluated | `every compiled proptest! block has recorded pass or fail`                                                    |
| accept   | properties_evaluated | verified             | [[verify.both_gates_required]] ∧ [[verify.properties_pass_reflects_latest_run]] ∧ [[verify.law_cases_all_run]] |
| reject | properties_evaluated | failed | `¬([[verify.both_gates_required]] ∧ [[verify.properties_pass_reflects_latest_run]] ∧ [[verify.law_cases_all_run]])` |


## Properties

| id                             | kind | derives_from                                        | generator                                                                | predicate                                                       |
|-----------------------------------|------|------------------------------------------------------------|---------------------------------------------------------------------------------|----------------------------------------------------------------------|
| partial_run_rejected               | unit | [[verify.blocks_run_to_completion]]                        | `run_that_skips_one_compiled_block()`                                             | `check(run) == invalid`                                              |
| cached_result_rejected             | unit | [[verify.properties_pass_reflects_latest_run]]             | `(passing_run, properties_edited_afterward_with_no_rerun)`                        | `properties_pass(file) == false` — until re-run                     |
| shrunk_counterexample_reported     | unit | [[verify.failure_reports_shrunk_counterexample]]           | `block_with(large_random_failing_input: true, smaller_reachable_by_shrinking: true)` | `reported_input(check(block)) == the_smaller_one`                   |
| single_gate_insufficient           | unit | [[verify.both_gates_required]]                             | `file_with(properties_pass: true, no_counterexample: false)`                       | `check(file) == failed`                                              |
| both_gates_clean_passes            | unit | [[verify.both_gates_required]]                             | `file_with(properties_pass: true, no_counterexample: true)`                        | `check(file) == verified`                                            |
| partial_law_case_rejected          | unit | [[verify.law_cases_all_run]]                                | `law_property_with(identity_case: "pass", associativity_case: "fail")`             | `check(property) == failed`                                          |
| rerun_matches_prior_outcome        | unit | [[verify.verify_is_idempotent]]                             | `(run_1, run_2)` on an unchanged compiled artifact                                | `outcome(run_1) == outcome(run_2)`                                   |
| verification_failure_label_asserted | unit | [[verify.verification_failure]] | `verification_failure_raised()` | `error_label == "verify.verification_failure"` — renaming the label touches the error Constraint, this property, and its note together (EDGE-002) |
## Notes

**This closes `STATUS.md` §4's P0.** The three pipeline items —
`compile.md`, `model_check.md`, and this file — now cover the full
`linted → compiled → model_checked → verified` lifecycle
`specodelic.md` names but had never specified past the data-shape level.
`STATUS.md` §1 describes "something you can simulate before writing
code" as one of the four things a spec file gives you; `compile.md` is
what makes the simulation buildable, `model_check.md` is what makes
running it meaningful, and this file is what makes trusting its result
well-defined rather than an unstated conjunction of two booleans.

`both_gates_required` is the one constraint doing the most work here: it
restates `specodelic.md`'s bare `no_counterexample ∧ properties_pass`
guard, but as an actual mechanism — `single_gate_insufficient`'s test
case (properties pass, model doesn't) is exactly the failure mode a
project would hit if someone treated a green proptest run alone as
sufficient to ship, skipping the model checker.

`law_cases_all_run` is a deliberate echo of `specodelic.md`'s
`law_requires_cases` (which gates *compiling* a law-kind property on it
having ≥ 2 cases in the first place). That constraint ensures the cases
exist before compilation; this one ensures they're all actually run and
passing before verification — the same discipline applied at two
different pipeline stages, on either side of `compile.md`.

`checked_against_core: clear` (see `AGENTS.md`'s convention). With this
file, `STATUS.md` §4's P0 is fully done; **P1 (the rename/refactor tool)
is next.**
