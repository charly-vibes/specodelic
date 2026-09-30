---
id: linter.failure_shape
kind: intent
checked_against_core: clear
statement: "WHEN a spec repo's tool files are linted, THE failure-shape checker SHALL reject a tool file whose failure terminal states emit nothing, whose error labels collide within a file, or whose failure guards neither cite their negated transition's citation set nor appear on the recorded carve-out list."
---

# Linter: Failure Shape Check

The checker half of the error contract (`errors.md`): it turns the
contract's tier-2 rules — mute failure terminals, label collisions, and
typed-negation guard totals — from graph-observable facts into lint
findings. Everything it checks is decidable from the reference graph the
`graph` tool already derives; nothing here reads prose, so the checker
cannot over-reject a file whose prose happens to argue for failure
classes the graph already separates (the D2a decision: classes ⟺
distinct guard citation sets, never a prose judgment).

v1 scope is failure terminals only. `model_check.md`'s `timed_out` and
`exploration_only` are real terminal outcomes but are a **stated
non-goal** of this version: typing them raises the exit-code mapping
question (does an exhausted bound map to 1?) that `errors.md`'s
`exit_code_mapping` row defers to a later Revision — the gap is stated
here so it is visible rather than forgotten.

This file is **spec-only** (design D6): no Checker Ownership row lands
until the checker is implemented — the table's invariant is that every
listed checker exists. The implementation ticket carries the enforcement
note; until it ships, these rules are normative for the future checker,
not claims about today's `spk lint`.

## Constraints

| id                       | kind      | expr                                                                                                                                                                   | traces_to | satisfies |
|--------------------------|-----------|------------------------------------------------------------------------------------------------------------------------------------------------------------------------|-----------|-----------|
| terminal_states_emit     | invariant | `every failure terminal state in a tool file's Model (a state with inbound transitions, no outbound transitions, carrying an emits edge obligation per errors.md's failure_state_emits) emits exactly one effect Constraint owned by that file — a mute failure terminal is a finding` (v1 scope: failure terminals only; timed_out / exploration_only are the stated non-goal above) | [[linter.failure_shape]] |           |
| error_labels_unique      | invariant | `within one file, no two error Constraints carry the same variant head — cross-file collisions are structurally impossible by errors.md's error_expr_shape (the label is file-id-namespaced), so this check is per-file and trivially green corpus-wide today; it exists so a future format change cannot silently drop the namespacing law` | [[linter.failure_shape]] |           |
| guard_negation_total     | invariant | `every failure transition either cites exactly the union of its success siblings' citation sets (the negated disjunction, errors.md's guard_negation_typed), or is on the recorded carve-out list — which is exactly orchestrate.md's stage-fail transitions (lint_fail, compile_fail, model_check_fail, verify_fail), per that file's own Notes' don't-restate discipline; a zero-citation failure guard off the list is a finding, never a silent pass` | [[linter.failure_shape]] |           |
| check_failure | effect | `linter.failure_shape.check_failure(detail)` | [[linter.failure_shape]] | [[errors.envelope_error_kind]] ∧ [[errors.exit_code_mapping]] ∧ [[errors.remediation_hint_present]] |

## Model

### States
- `unchecked`
- `checking`
- `passed`
- `failed` (emits: `[[linter.failure_shape.check_failure]]`)

### Transitions

| id      | from     | to      | guard                                                                                    |
|---------|----------|---------|-------------------------------------------------------------------------------------------|
| begin   | unchecked | checking | `the repo declares a specs corpus to check`                                             |
| accept  | checking | passed  | [[linter.failure_shape.terminal_states_emit]] ∧ [[linter.failure_shape.error_labels_unique]] ∧ [[linter.failure_shape.guard_negation_total]] |
| reject  | checking | failed  | `¬([[linter.failure_shape.terminal_states_emit]] ∧ [[linter.failure_shape.error_labels_unique]] ∧ [[linter.failure_shape.guard_negation_total]])`          |

## Properties

| id                            | kind | derives_from                                              | generator                                                        | predicate                                                                                    |
|-------------------------------|------|------------------------------------------------------------|--------------------------------------------------------------------|------------------------------------------------------------------------------------------------|
| mute_terminal_rejected        | unit | [[linter.failure_shape.terminal_states_emit]]              | `tool_file_with_failure_terminal_and_no_emits_edge()`              | `check(file) == failed` — the corpus's pre-add-error-contract shape, kept as the negative fixture |
| label_collision_rejected      | unit | [[linter.failure_shape.error_labels_unique]]               | `file_with_two_error_constraints_sharing_a_variant_head()`         | `check(file) == failed`                                                                        |
| zero_citation_flagged         | unit | [[linter.failure_shape.guard_negation_total]]              | `failure_transition_citing_nothing_off_the_carveout_list()`        | `check(file) == failed` — the carve-out is a checked list, not an escape hatch                 |
| carved_out_guard_passes       | unit | [[linter.failure_shape.guard_negation_total]]              | `orchestrate_stage_fail_transition()`                              | `check(file) == passed` — the carve-out is checked, not assumed                                 |
| negation_set_mismatch_flagged | unit | [[linter.failure_shape.guard_negation_total]]              | `failure_transition_whose_citations_differ_from_its_siblings()`    | `check(file) == failed` — citation-set inequality is graph-decidable                            |
| clean_repo_passes             | unit | [[linter.failure_shape.terminal_states_emit]]              | `the_repo_itself()`                                                | `check(repo) == passed` — the corpus dogfoods this checker's contract                          |
| check_failure_label_asserted  | unit | [[linter.failure_shape.check_failure]]                     | `failure_shape_violation_found()`                                  | `error_label == "linter.failure_shape.check_failure"` — renaming the label touches the error Constraint, this property, and its note together (EDGE-002) |

## Notes

**The class rule is the graph-decidable form.** `errors.md`'s
`failure_class_is_state` owns the corpus-side rule (a state with ≥2
inbound failure transitions whose guard citation sets differ is
malformed); this checker never re-derives classes from prose —
"classes ⟺ distinct citation sets" is the only reading it implements,
so a single-class file (one failure state, one inbound failure
transition, however many prose paragraphs argue about it) cannot be
over-rejected.

**Carve-out list, restated.** The only recorded carve-out is
`orchestrate.md`'s stage-fail transitions. That file's Notes decline to
restate upstream files' logic as guard-citable rows; retyping their
guards would reintroduce exactly the duplication it argues against. The
list is data to this checker (checked membership), not an assumption —
a new zero-citation failure guard anywhere else is a finding.

**Ownership is deliberately absent.** Unlike the seven checkers in
`specodelic.md`'s Checker Ownership table, this file ships no table row
until its implementation exists; see design D6 of the error-contract
change. The implementation ticket notes that `AGENTS.md`'s
sibling-tool constraint ("specodelic-6pi must be fixed before any CI
gate chains lint") is stale — 6pi is closed — and that the staleness
belongs to governance, not to this file.