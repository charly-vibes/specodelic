---
id: linter.model_shape
kind: intent
statement: "WHEN a spec file's graph shape has passed, THE linter SHALL reject it if any transition lacks a guard or if the file declares a model without both a states section and a transitions section."
---

# Linter: Model Shape Check

Runs after `linter.graph_shape` — it needs the reference graph already
proven acyclic before it can trust that a `guard` field pointing at a
constraint isn't secretly part of a cycle back through the model itself.
Corresponds to `invalid-states-diagnostician`'s "unconstrained optionality"
and "ad-hoc state machine" signals, made structural rather than advisory.

## Constraints

| id                    | kind      | expr                                                                                     | traces_to                        |
|------------------------|-----------|-----------------------------------------------------------------------------------------------|-------------------------------------|
| guard_present          | invariant | `∀ transition row: guard field is non-empty`                                                   | [[specodelic.guard_required]]    |
| model_sections_paired  | invariant | `if [[model.state]] exists then [[model.transition]] exists, and vice versa`                    | [[specodelic.model_present]]     |
| every_state_used       | invariant | `∀ state: state appears as from or to in ≥ 1 transition`                                        | [[specodelic.model_present]]     |
| every_transition_valid | invariant | `∀ transition: from ∈ states and to ∈ states`                                                   | [[specodelic.model_present]]     |
| no_bool_state_field    | invariant | `no state or transition row has a column of boolean type`                                       | [[specodelic.no_boolean_columns]]|

## Model

### States
- `ungrouped`
- `pairing_checked`
- `states_checked`
- `passed`
- `failed`

### Transitions

| id                | from             | to                | guard                                                                                          |
|-------------------|------------------|-------------------|----------------------------------------------------------------------------------------------------|
| check_pairing     | ungrouped        | pairing_checked   | `file passed linter.graph_shape`                                                                    |
| pairing_ok        | pairing_checked  | states_checked    | [[linter.model_shape.model_sections_paired]] ∧ [[linter.model_shape.every_state_used]] ∧ [[linter.model_shape.every_transition_valid]] |
| pairing_fail      | pairing_checked  | failed            | `¬pairing_ok.guard`                                                                                  |
| accept            | states_checked   | passed            | [[linter.model_shape.guard_present]] ∧ [[linter.model_shape.no_bool_state_field]]                     |
| reject            | states_checked   | failed            | `¬accept.guard`                                                                                       |

## Properties

| id                       | kind | derives_from                                       | generator                                             | predicate                                                                 |
|---------------------------|------|--------------------------------------------------------|------------------------------------------------------------|-----------------------------------------------------------------------------|
| missing_guard_rejected      | unit | [[linter.model_shape.guard_present]]                  | `spec_file_with(transition_missing_guard: true)`             | `check(file) == failed`                                                    |
| unpaired_states_rejected    | unit | [[linter.model_shape.model_sections_paired]]          | `spec_file_with(states_section_but_no_transitions: true)`     | `check(file) == failed`                                                    |
| unreachable_state_rejected  | unit | [[linter.model_shape.every_state_used]]               | `spec_file_with(declared_state_never_referenced: true)`       | `check(file) == failed`                                                    |
| bad_edge_rejected           | unit | [[linter.model_shape.every_transition_valid]]         | `spec_file_with(transition.to_not_in_states: true)`           | `check(file) == failed`                                                    |
| bool_column_rejected        | unit | [[linter.model_shape.no_bool_state_field]]            | `spec_file_with(state_row_having_bool_typed_column: true)`    | `check(file) == failed`                                                    |
| well_formed_model_passes    | unit | [[linter.model_shape.every_transition_valid]]         | `arbitrary_well_formed_state_machine()`                       | `check(file) == passed`                                                    |

## Notes

`every_state_used` and `every_transition_valid` are new relative to
`specodelic.md`'s original list — `guard_required` only checked that a
guard exists, not that the state machine around it is internally
consistent (an unreferenced state, or a transition pointing at a state
that was never declared). Both are structural prerequisites for the
`model_check` transition in `specodelic.md` to mean anything: a TLA+
generator handed a dangling `to` field would either crash or silently
model-check a different graph than the author wrote. Fourth candidate to
fold back into `specodelic.md`'s constraint table.

`no_bool_state_field` restates `specodelic.md`'s `no_boolean_columns`
scoped specifically to the model section, since that's where a boolean
column would actually manifest as an ad-hoc state machine
(`invalid-states-diagnostician`'s central pathology) rather than merely a
schema-hygiene issue elsewhere in the file.

Running tally of gaps found so far, all pending a pass back into
`specodelic.md`: `id_matches_file`, `ref_kind_compatible`,
`single_root_reachable`, `every_state_used` + `every_transition_valid`.
