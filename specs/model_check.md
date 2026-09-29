---
id: model_check
kind: intent
checked_against_core: clear
statement: "THE model_check step SHALL run a model-check backend — stateright (embedded, default) or TLC (opt-in) — against the model compile.md's model_to_tla produces, and SHALL report either that no counterexample was found within a stated bound, or a minimal counterexample trace naming the violated invariant."
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
| checker_invoked                       | invariant | `reaching model_checked always corresponds to an actual checker run, by the selected backend, against the current compiled model — model_present's structural check is not itself that run` | [[model_check]] |
| exhaustive_within_bound               | invariant | `a run explores all reachable states up to a stated bound (state-space size or step count) before it may report no counterexample; the bound is part of the report, since unbounded checking of a spec with no finite state space never terminates` | [[model_check]] |
| counterexample_is_minimal             | invariant | `when a run finds a counterexample, it reports the shortest violating transition sequence found, not merely the first one encountered`               | [[model_check]] |
| counterexample_names_violated_invariant | invariant | `a counterexample report names exactly one violated invariant, by its Constraints-table id, together with the full state trace leading to it`     | [[model_check]] |
| backend_identified                     | invariant | `a run report names the backend engine and version that produced it, so two backends' reports on the same compiled model and bound are attributable and comparable` | [[model_check]] |
| no_counterexample_feeds_verify        | invariant | `[[specodelic.no_counterexample]] holds iff the most recent run on the current compiled module reported clean within its stated bound`             | [[model_check]] |
| rerun_on_model_change                 | invariant | `a run's clean result does not satisfy [[specodelic.no_counterexample]] once the Model section (States or Transitions) has changed since that run — the model must be re-run, not assumed still clean` | [[model_check]] |

## Model

### States
- `not_run`
- `running`
- `clean`
- `counterexample_found`
- `timed_out`
- `exploration_only`

### Transitions

| id               | from      | to                   | guard                                                                                                  |
|------------------|-----------|----------------------|---------------------------------------------------------------------------------------------------------|
| begin            | not_run   | running              | [[model_check.checker_invoked]]                                                                         |
| finish_clean     | running   | clean                | [[model_check.exhaustive_within_bound]] ∧ `no violation found within the bound`                          |
| finish_violation | running   | counterexample_found | [[model_check.counterexample_is_minimal]] ∧ [[model_check.counterexample_names_violated_invariant]]       |
| finish_timeout   | running   | timed_out            | `the stated bound was not reached before the run's time/state budget expired`                            |
| finish_exploration | running | exploration_only     | [[model_check.exhaustive_within_bound]] ∧ `the backend executed no invariant predicates — a completed exploration is evidence the space ends within the bound, never that the model holds` |

## Properties

| id                                   | kind | derives_from                                          | generator                                                          | predicate                                                                |
|-----------------------------------------|------|-------------------------------------------------------------|--------------------------------------------------------------------------|------------------------------------------------------------------------------|
| stale_claim_rejected                     | unit | [[model_check.checker_invoked]]                             | `compiled_module_with_zero_runs_against_it()`                             | `no_counterexample(file) == undetermined` — never defaults to true         |
| bound_must_be_stated                     | unit | [[model_check.exhaustive_within_bound]]                     | `run_report_with_no_stated_bound()`                                       | `check(report) == invalid`                                                 |
| shortest_counterexample_reported         | unit | [[model_check.counterexample_is_minimal]]                   | `model_with(two_violating_traces_of_different_length: true)`              | `reported_trace(check(model)) == the_shorter_of_the_two`                   |
| violated_invariant_named                 | unit | [[model_check.counterexample_names_violated_invariant]]     | `model_with_exactly_one_violated_invariant()`                             | `check(model).violated_invariant_id == the_expected_id`                   |
| backend_named_in_report                  | unit | [[model_check.backend_identified]]                          | `run_report_from_any_backend()`                                           | `check(report).backend == a named engine and version`                     |
| clean_run_satisfies_verify_gate          | unit | [[model_check.no_counterexample_feeds_verify]]              | `arbitrary_model_with_no_violation_within_bound()`                        | `no_counterexample(file) == true`                                          |
| stale_result_invalidated_by_edit         | unit | [[model_check.rerun_on_model_change]]                       | `(clean_run, model_edited_afterward_with_no_rerun)`                       | `no_counterexample(file) == false` — until re-run                          |
| clean_model_passes                       | unit | [[model_check.checker_invoked]]                             | `arbitrary_model_with_no_violation_within_bound()`                        | `check(model) == clean`                                                    |
| exploration_run_is_not_a_clean_verdict   | unit | [[model_check.no_counterexample_feeds_verify]]              | `model_with_only_prose_invariants_exhausted_within_bound()`               | `check(model).outcome == exploration_only` — never read as no_counterexample |

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
is satisfied either way. So is `exploration_only` (specodelic-len): the
native stateright backend interprets guards as prose (Decision 3,
Option A) and executes zero invariant predicates, so its completed runs
terminate in `finish_exploration`, NOT `finish_clean` — a completed
exploration proves the space ends within the bound and nothing more.
`no_counterexample` is reserved for a backend that actually executed
invariant predicates and found no violation; consumers (verify's
gate, the `no_counterexample_feeds_verify` property above) must treat
`exploration_only` exactly like `timed_out` — not-clean, though not
evidence of a violation either. **Backends.** The contract above is engine-neutral,
and any backend satisfying it sits behind the same state machine. Two are
specified: **stateright** (default) — an embedded Rust model-checking crate,
no external dependencies: its breadth-first exploration makes
`counterexample_is_minimal` hold by construction (the first counterexample
found is the shortest), `target_max_depth`/`timeout`/`target_state_count`
give `exhaustive_within_bound`'s stated bound and `timed_out`'s
budget-exhausted outcome directly, and its named properties give
`counterexample_names_violated_invariant` when each property is named after
the Constraint id it derives from. The native backend *interprets* the
compiled model (states as values, transitions as actions) rather than
generating Rust source. **TLC** (opt-in) — the mature JVM reference engine,
run as a subprocess against the `.tla` module `compile.md` emits, with
`-depth` as the stated bound. Alloy was dropped from the corpus language
alongside this change: with a native default and a reference TLA+ engine it
had no remaining role, and a SAT-based engine returns *an* instance, not a
minimal trace — which would fight `counterexample_is_minimal` rather than
satisfy it. `verify`'s downstream guard treats `timed_out`
the same as `counterexample_found` for the purpose of
`no_counterexample` (both are not-clean), but they are reported
differently, since a timeout is not evidence of a violation — just of an
inconclusive run.

`checked_against_core: clear` (see `AGENTS.md`'s convention) —
`no_counterexample` and `model_present` already existed there and are
referenced, not restated. `verify` (`STATUS.md` §4, now done — see
`verify.md`) consumes this file's `clean`/`counterexample_found`
outcome alongside `compile.md`'s proptest! blocks.
