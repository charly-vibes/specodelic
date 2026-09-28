---
id: model_check
kind: intent
checked_against_core: clear
statement: "THE model_check step SHALL run a model checker (TLC or Alloy) against the module compile.md's model_to_tla produces, and SHALL report either that no counterexample was found within a stated bound, or a minimal counterexample trace naming the violated invariant."
---

# Model check

`specodelic.md` names `model_check` as the `compiled → model_checked`
transition, guarded only by `model_present` — a *structural* check that
a Model section exists. Whether the model actually holds
(`no_counterexample`) is a separate invariant, consumed later by the
`verify` transition's guard, not by `model_check` itself. That split was
never made explicit anywhere (`STATUS.md` §4, P0): this file specifies
the run `model_check` names — what "invoking the checker" actually does
to the module `compile.md` produces, and what a stated result must
contain.

## Constraints

| id                                | kind      | expr                                                                                                                                        | traces_to     |
|--------------------------------------|-----------|----------------------------------------------------------------------------------------------------------------------------------------------------|---------------|
| checker_invoked                       | invariant | `reaching model_checked always corresponds to an actual TLC/Alloy run against the current compiled module — model_present's structural check is not itself that run` | [[model_check]] |
| exhaustive_within_bound               | invariant | `a run explores all reachable states up to a stated bound (state-space size or step count) before it may report no counterexample; the bound is part of the report, since unbounded checking of a spec with no finite state space never terminates` | [[model_check]] |
| counterexample_is_minimal             | invariant | `when a run finds a counterexample, it reports the shortest violating transition sequence found, not merely the first one encountered`               | [[model_check]] |
| counterexample_names_violated_invariant | invariant | `a counterexample report names exactly one violated invariant, by its Constraints-table id, together with the full state trace leading to it`     | [[model_check]] |
| no_counterexample_feeds_verify        | invariant | `[[specodelic.no_counterexample]] holds iff the most recent run on the current compiled module reported clean within its stated bound`             | [[model_check]] |
| rerun_on_model_change                 | invariant | `a run's clean result does not satisfy [[specodelic.no_counterexample]] once the Model section (States or Transitions) has changed since that run — the model must be re-run, not assumed still clean` | [[model_check]] |

## Model

### States
- `not_run`
- `running`
- `clean`
- `counterexample_found`
- `timed_out`

### Transitions

| id               | from      | to                   | guard                                                                                                  |
|------------------|-----------|----------------------|---------------------------------------------------------------------------------------------------------|
| begin            | not_run   | running              | [[model_check.checker_invoked]]                                                                         |
| finish_clean     | running   | clean                | [[model_check.exhaustive_within_bound]] ∧ `no violation found within the bound`                          |
| finish_violation | running   | counterexample_found | [[model_check.counterexample_is_minimal]] ∧ [[model_check.counterexample_names_violated_invariant]]       |
| finish_timeout   | running   | timed_out            | `the stated bound was not reached before the run's time/state budget expired`                            |

## Properties

| id                                   | kind | derives_from                                          | generator                                                          | predicate                                                                |
|-----------------------------------------|------|-------------------------------------------------------------|--------------------------------------------------------------------------|------------------------------------------------------------------------------|
| stale_claim_rejected                     | unit | [[model_check.checker_invoked]]                             | `compiled_module_with_zero_runs_against_it()`                             | `no_counterexample(file) == undetermined` — never defaults to true         |
| bound_must_be_stated                     | unit | [[model_check.exhaustive_within_bound]]                     | `run_report_with_no_stated_bound()`                                       | `check(report) == invalid`                                                 |
| shortest_counterexample_reported         | unit | [[model_check.counterexample_is_minimal]]                   | `model_with(two_violating_traces_of_different_length: true)`              | `reported_trace(check(model)) == the_shorter_of_the_two`                   |
| violated_invariant_named                 | unit | [[model_check.counterexample_names_violated_invariant]]     | `model_with_exactly_one_violated_invariant()`                             | `check(model).violated_invariant_id == the_expected_id`                   |
| clean_run_satisfies_verify_gate          | unit | [[model_check.no_counterexample_feeds_verify]]              | `arbitrary_model_with_no_violation_within_bound()`                        | `no_counterexample(file) == true`                                          |
| stale_result_invalidated_by_edit         | unit | [[model_check.rerun_on_model_change]]                       | `(clean_run, model_edited_afterward_with_no_rerun)`                       | `no_counterexample(file) == false` — until re-run                          |
| clean_model_passes                       | unit | [[model_check.checker_invoked]]                             | `arbitrary_model_with_no_violation_within_bound()`                        | `check(model) == clean`                                                    |

## Notes

**This clarifies a naming trap in `specodelic.md`'s lifecycle,** flagged
as **CLAR-003** (same shape as CLAR-001, CLAR-002 — not resolved here for
the same reason: renaming a live state name should go through
`rename_naturality`, not be done by hand). `model_checked` sounds like it
means "the model was checked and holds," but its guard,
`[[specodelic.model_present]]`, only confirms a Model section exists —
`checker_invoked` above is what "a run actually happened" means, and
whether that run found a counterexample (`no_counterexample`) is a fact
consumed only by the *next* transition, `verify`. A file can be in state
`model_checked` while its most recent run found a counterexample, or
before any run has happened at all if `model_present` alone were treated
as sufficient. `no_counterexample_feeds_verify` and
`rerun_on_model_change` above are what close that gap: no one may claim
`no_counterexample` without having actually run this state machine to
`clean`, and a `clean` result older than the last Model edit doesn't
count.

`timed_out` is a real terminal outcome, not a failure this file
papers over as "counterexample" or "clean" by default: an unbounded or
very large state space may exhaust its budget before `exhaustive_within_bound`
is satisfied either way. `verify`'s downstream guard treats `timed_out`
the same as `counterexample_found` for the purpose of
`no_counterexample` (both are not-clean), but they are reported
differently, since a timeout is not evidence of a violation — just of an
inconclusive run.

`checked_against_core: clear` (see `AGENTS.md`'s convention) —
`no_counterexample` and `model_present` already existed there and are
referenced, not restated. `verify` (`STATUS.md` §4, now done — see
`verify.md`) consumes this file's `clean`/`counterexample_found`
outcome alongside `compile.md`'s proptest! blocks.
