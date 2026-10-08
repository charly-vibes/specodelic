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

| id                              | kind      | expr                                                                                                                                          | traces_to | satisfies |
|------------------------------------|-----------|------------------------------------------------------------------------------------------------------------------------------------------------------|----------------------|
| blocks_run_to_completion            | invariant | `verify actually executes every proptest! block compile.md produced against the current compiled artifact — properties_pass is never satisfied by a partial run or a cached prior result` | [[verify]] |          |
| properties_pass_reflects_latest_run | invariant | `[[specodelic.properties_pass]] holds iff every block from the most recent run against the current compiled artifact passed; a run predating the last edit to Properties, Constraints, or Model does not count` | [[verify]] |          |
| failure_reports_shrunk_counterexample | invariant | `when a proptest! block fails, the report gives the shrunk (minimal) failing input, not the first randomly-generated failing input encountered` | [[verify]] |          |
| both_gates_required                 | invariant | `verify only transitions a file to verified when both [[specodelic.no_counterexample]] (model_check.md's clean outcome) and [[specodelic.properties_pass]] hold — passing properties alone, or a clean model alone, is insufficient` | [[verify]] |          |
| law_cases_unexecuted                   | invariant | `for a law-kind property compiled into multiple blocks (one per required case), all compiled blocks must pass — a partial pass (identity passes, associativity fails) is a failure of the property, not a partial success` | [[verify]] |          |
| verify_is_idempotent                | invariant | `re-running verify against an unchanged compiled artifact yields the same pass/fail outcome as the prior run, even though proptest may resample inputs each run for coverage` | [[verify]] |          |
| bounded_wall_clock                  | invariant | `the verify runner bounds its cargo test run on a wall-clock clock (default 600s, `--timeout-secs 0` disables the bound); a run exceeding the bound is killed and reported as a labeled timeout — never a silent hang, and never indistinguishable from a block failure` | [[verify]] |          |
| verification_failure | effect | `verify.verification_failure(detail)` | [[verify]] | [[errors.envelope_error_kind]] ∧ [[errors.exit_code_mapping]] ∧ [[errors.remediation_hint_present]] |
| required_claims_govern_acceptance    | invariant | `verify only transitions a file to verified when every required claim (model_check.md's required_claims_classified set) is verified in the current report AND every generated property block passes — a passing property run alone never covers an unknown, missing, or refuted claim; invalid or duplicate claim records are rejected as malformed evidence, never inferred as unknown success, and prose-only invariants stay unchecked` | [[verify]] |          |
| evidence_scope_bound                 | invariant | `verify recomputes the scope digest and the required claim set from the current inputs and compares them against the stored report: old or unrecognized claim-schema versions, absent scope fingerprints, omitted or duplicate records, changed contributing inputs, or artifact mismatch fail with a rerun hint naming the model-check command — a stored report is never rewritten to manufacture evidence, and the digest binds structured content, not filesystem paths, so swapped reports from different scopes never verify` | [[verify]] |          |
| assurance_views_agree                | invariant | `CLI JSON, human output, and the persisted report agree on evaluated, unchecked, and blocking claims — no view reports verified while another names a blocker — and release documentation distinguishes structural lint, bounded model exploration, verified declared claims, and external application-test binding without advertising pending capabilities as implemented` | [[verify]] |          |

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
| accept   | properties_evaluated | verified             | [[verify.both_gates_required]] ∧ [[verify.properties_pass_reflects_latest_run]] ∧ [[verify.law_cases_unexecuted]] ∧ [[verify.required_claims_govern_acceptance]] ∧ [[verify.evidence_scope_bound]] |
| reject | properties_evaluated | failed | `¬([[verify.both_gates_required]] ∧ [[verify.properties_pass_reflects_latest_run]] ∧ [[verify.law_cases_unexecuted]] ∧ [[verify.required_claims_govern_acceptance]] ∧ [[verify.evidence_scope_bound]])` |


## Properties

| id                             | kind | derives_from                                        | generator                                                                | predicate                                                       |
|-----------------------------------|------|------------------------------------------------------------|---------------------------------------------------------------------------------|----------------------------------------------------------------------|
| partial_run_rejected               | unit | [[verify.blocks_run_to_completion]]                        | `run_that_skips_one_compiled_block()`                                             | `check(run) == invalid`                                              |
| cached_result_rejected             | unit | [[verify.properties_pass_reflects_latest_run]]             | `(passing_run, properties_edited_afterward_with_no_rerun)`                        | `properties_pass(file) == false` — until re-run                     |
| shrunk_counterexample_reported     | unit | [[verify.failure_reports_shrunk_counterexample]]           | `block_with(large_random_failing_input: true, smaller_reachable_by_shrinking: true)` | `reported_input(check(block)) == the_smaller_one`                   |
| single_gate_insufficient           | unit | [[verify.both_gates_required]]                             | `file_with(properties_pass: true, no_counterexample: false)`                       | `check(file) == failed`                                              |
| both_gates_clean_passes            | unit | [[verify.both_gates_required]]                             | `file_with(properties_pass: true, no_counterexample: true)`                        | `check(file) == verified`                                            |
| partial_law_case_rejected          | unit | [[verify.law_cases_unexecuted]]                                | `law_property_with(identity_case: "pass", associativity_case: "fail")`             | `check(property) == failed`                                          |
| rerun_matches_prior_outcome        | unit | [[verify.verify_is_idempotent]]                             | `(run_1, run_2)` on an unchanged compiled artifact                                | `outcome(run_1) == outcome(run_2)`                                   |
| verification_failure_label_asserted | unit | [[verify.verification_failure]] | `verification_failure_raised()` | `error_label == "verify.verification_failure"` — renaming the label touches the error Constraint, this property, and its note together (EDGE-002) |
| hang_reported_as_labeled_timeout    | unit | [[verify.bounded_wall_clock]] | `runner_with(predicate_that_never_terminates, timeout: 1)` | `check(run) == timeout_labeled ∧ outcome(run) ≠ failed` — a timeout is no verdict on the property: distinct from properties_failed so a legitimately long suite can be re-run with a raised bound instead of misread as a failing predicate |
| fragments_reach_verified            | unit | [[verify.both_gates_required]] | `fixture_with(fragment_predicate_passing, executable_invariant_clean)` | `check(file) == verified` — with executable fragments (specodelic.md Revision 15) both gates are reachable: a passing executable predicate and a clean executable-invariant run verify a real file, the state the gate could never reach before this Revision |
| claim_complete_evidence_verifies     | unit | [[verify.required_claims_govern_acceptance]] | `fixture_with(all_required_claims_verified, properties_pass)` | `check(file) == verified with the exact required and unchecked sets` |
| false_claim_blocks_verified         | unit | [[verify.required_claims_govern_acceptance]] | `file_with(properties_pass: true, counterexample_kernel_claim)` | `check(file) == failed naming the false claim id` — properties never cover a refuted claim |
| malformed_records_rejected           | unit | [[verify.required_claims_govern_acceptance]] | `report_with(missing_or_duplicate_claim_record)` | `check(file) == failed with a rerun hint` — never inferred as unknown success |
| stale_report_rejected_with_rerun_hint | unit | [[verify.evidence_scope_bound]] | `(clean_report, structurally_edited_inputs_afterward)` | `check(file) == failed naming the rerun command — the report is never rewritten` |
| swapped_scope_report_rejected       | unit | [[verify.evidence_scope_bound]] | `report_from_a_different_structured_scope()` | `check(file) == failed — the digest binds structured content, not paths` |
| views_agree_on_blockers             | unit | [[verify.assurance_views_agree]] | `run_with(required_unknown_claim_and_prose_invariant)` | `json.blockers == human.blockers == report.blockers ∧ unchecked sets agree` |
| release_docs_match_implemented_capabilities | unit | [[verify.assurance_views_agree]] | `docs_built_for_a_release()` | `version derives from Cargo metadata ∧ pending capabilities not advertised as implemented` |
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

`law_cases_unexecuted` is a deliberate echo of `specodelic.md`'s
`law_requires_cases` (which gates *compiling* a law-kind property on it
having ≥ 2 cases in the first place). That constraint ensures the cases
exist before compilation; this one ensures they're all actually run and
passing before verification — the same discipline applied at two
different pipeline stages, on either side of `compile.md`.

`checked_against_core: clear` (see `AGENTS.md`'s convention). With this
file, `STATUS.md` §4's P0 is fully done; **P1 (the rename/refactor tool)
is next.**

**Executable fragments make `verified` reachable (specodelic.md
Revision 15, specodelic-rjb).** Before that Revision the verdict's
`verified` status was unreachable by construction — every compiled block
carried a `todo_predicate!` placeholder that panics at execution, and the
native backend executed zero invariant predicates, so both gates failed
honestly on every file. `fragments_reach_verified` pins the way out
without touching `both_gates_required` or any honesty rule: a predicate
opting in with `**rust:**` executes for real, an executable invariant
gives the model run a predicate to actually check, and `verified` becomes
the conjunction both gates always demanded — reachable, never redefined.
Files whose cells carry no fragment behave exactly as before.

**Claim-gated acceptance and scope-bound evidence
(define-verification-claim-gates).** `required_claims_govern_acceptance`
completes `both_gates_required` rather than replacing it: the two gates
already demanded — a clean model run and passing properties — are now
joined by the third input they were silently missing, the per-claim
statuses the model report carries. A refuted or unknown kernel/citation
claim blocks `verified` even with a green proptest run; prose-only
invariants stay explicitly unchecked, and having a deriving Property
never makes one executable. `evidence_scope_bound` is what makes the
first constraint trustworthy: verify recomputes scope and the required
set from the *current* inputs instead of trusting the stored report's
word, so a stale, swapped, or malformed report fails with a rerun hint —
old evidence is rejected, never silently re-scored or rewritten. This is
deliberately stricter than the pre-change acceptance path; the rerun
instruction ships with the release. `assurance_views_agree` keeps the
three report surfaces (CLI JSON, human output, the persisted
`<stem>.check.json`) from telling different stories about the same run.

The claim-side machine itself (required/unchecked classification, the
aggregate priority, the report schema, and dual-format command
isolation) is specified once, in `model_check.md` — this file consumes
it, mirroring how it already consumes model_check's clean outcome rather
than restating `specodelic.md`'s bare guard. No `specodelic.md`
Constraint is touched, so no Revision bump: the acceptance conjunction
lives here (see the Revision-discipline note there).
