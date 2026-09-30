---
id: linter.observability
kind: intent
statement: "WHEN a spec declares an effect Constraint, THE linter SHALL warn — advisory, exit 0 — on every effect within the lint invocation's file set that no `observes` reference targets, and SHALL never gate a lifecycle transition on this warning."
---

# Linter: Observability

`specodelic.md` Revision 9 added the `observes` typed reference (a
Constraint row's outbound pointer to an effect Constraint, any file) —
the vocabulary for "this output must be observable". Vocabulary alone
does not make observability a *checked* fact: nothing distinguishes an
effect that something watches from one nobody declared an interest in.
This file specifies the check that closes that gap, shaped on the
Revision 7 precedent (`satisfies`) for typing and on
`linter-external_completeness.md` for cross-file check shape.

## Constraints

| id                            | kind      | expr                                                                                                                                                                                                                                                                       | traces_to |
|-------------------------------|-----------|-----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|-----------|
| observation_universe          | invariant | `the check's universe is the lint invocation's file set — corpus-wide lint (just lint-specs) is the canonical run; every effect Constraint in that set is either the target of ≥1 observes edge resolved within the same set, or is reported as an advisory warning naming the unobserved row`                                                                                                             | [[linter.observability]]  |
| advisory_severity             | invariant | `the unobserved-effect warning is emitted on the success envelope's warnings channel with exit 0 — never an Issue, never a nonzero exit; a warning is not a failure and never gates a lifecycle transition (whether it ever gates is a follow-up change, decided on dogfood friction evidence)`                                                                                                                         | [[linter.observability]]  |
| self_observation_not_counted  | invariant | `an effect row does not observe itself — an observation counts only when the observes edge is sourced at a different row (intra-file or cross-file); a row pointing observes at its own output is vacuous and is still warned`                                                                                                                           | [[linter.observability]]  |
| dangling_observes_is_referential | invariant | `an observes edge whose target does not resolve is a linter.total_refs finding, never an observability warning — the two checks compose without double-reporting the same row`                                                                                                            | [[linter.observability]]  |
| no_waivers_in_v1              | invariant | `no waiver machinery exists in this Revision — when the need is demonstrated, linter-external_completeness.md's covered/waived claim structure is the candidate, decided in a follow-up change`                                                                                             | [[linter.observability]]  |

## Model

### States

- `scanning`
- `observed`
- `unobserved_warned`

### Transitions

| id          | from       | to                 | guard                                                       |
|-------------|------------|--------------------|--------------------------------------------------------------|
| scan_effects | scanning   | observed           | `every effect in the invocation's file set has ≥1 observer`   |
| warn_unobserved | scanning | unobserved_warned | [[linter.observability.observation_universe]]                 |

## Properties

| id                              | kind | derives_from                                        | generator                                        | predicate                                                                       |
|---------------------------------|------|-----------------------------------------------------|--------------------------------------------------|----------------------------------------------------------------------------------|
| unobserved_effect_warned        | unit | [[linter.observability.observation_universe]]       | `effect_with_zero_observes_edges()`               | `exit 0 ∧ a warning naming the row id and the rule id linter.observability`      |
| observed_effect_silent          | unit | [[linter.observability.observation_universe]]       | `effect_with_at_least_one_observes_edge()`        | `no observability warning`                                                       |
| warning_never_gates             | unit | [[linter.observability.advisory_severity]]          | `corpus_with_unobserved_effects()`                | `exit code 0 ∧ zero Issue findings attributable to this check`                   |
| mutual_observation_passes       | unit | [[linter.observability.observation_universe]]       | `two_files_mutually_observing_each_others_effects()` | `both effects counted as observed — no warning`                               |
| self_observation_still_warned   | unit | [[linter.observability.self_observation_not_counted]] | `effect_row_pointing_observes_at_itself()`     | `warning fires — vacuous self-observation is not an observer`                    |
| dangling_observes_not_doubled   | unit | [[linter.observability.dangling_observes_is_referential]] | `observes_pointing_at_an_absent_row()`      | `exactly one total_refs finding; zero observability warnings`                    |
| waiver_input_ignored            | unit | [[linter.observability.no_waivers_in_v1]]           | `corpus_with_unobserved_effects()`               | `the warning fires regardless of any claimed-waiver text — no waiver channel exists` |

## Notes

`checked_against_core: clear` (see `AGENTS.md`'s convention). The
self-observation decision above resolves the question left open in the
change design's Open Questions: intra-file observation by a *different*
row counts; a row observing itself does not. Whether this check ever
becomes a gate (and joins `orchestrate.md`'s stage guards) is deliberately
undecided — the advisory-vs-gating decision needs dogfood friction
evidence first, per the Revision 9 text.