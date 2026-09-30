---
id: errors
kind: intent
statement: "THE error contract SHALL publish the cross-cutting output contract — error-kind envelopes with ok:false, the 0/1/2 exit-code mapping, one file-id-namespaced labeled error per failing stage, and a non-empty remediation hint — as extension_point rows that tool files satisfy from their own files, and SHALL require every failure terminal in a tool file's Model to emit a labeled error Constraint backed by a falsifying property asserting the exact label."
---

# Errors

## Purpose

Errors in the corpus become typed, namespaced, and falsifiable facts
instead of CHANGELOG lore and mute `failed` states. The cross-cutting
output contract (envelope kind, exit codes, one labeled failure per
stage, remediation hints) lives here as `extension_point` rows every
tool file points `satisfies` at; every tool file's failure terminal
emits a file-owned labeled error Constraint; every label carries a unit
property asserting the exact label — the same red→green discipline
every other corpus fact already has.

Scope note: this file spans two altitudes, kept distinct throughout —
*lint-time corpus rules* (failure states, guards: about the content of
`specs/*.md` files) and the *runtime output contract* (envelope, exit
codes: about tool behavior). "Emits" names the format mechanism;
runtime reporting is "reports". Enforcement routing is itself specced:
`enforcement_routed` below states which tier enforces each rule today,
which waits for `linter-failure_shape`, and which rides the
compile/verify pipeline.

## Constraints

| id                       | kind      | expr                                                                                                                                                                                                                   | traces_to |
|--------------------------|-----------|------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|-----------|
| error_expr_shape         | invariant | `every error Constraint's expr is a file-id-namespaced variant head: <owning-file-id>.<variant_head>(field, …) — the label names its owning file, so label uniqueness is a per-file property and cross-file collisions are structurally impossible` (pattern-checking the structured expr field follows the ears_statement precedent — expr is not rationale; prose_untouched is untouched) | [[errors]]  |
| failure_state_emits      | invariant | `every failure terminal state in a tool file's Model carries an emits edge to an effect Constraint owned by that file, whose expr satisfies error_expr_shape — a failure state emits nothing is a malformed model` (phase-2 terminals timed_out / exploration_only are excluded in v1) | [[errors]]  |
| failure_class_is_state   | invariant | `a state with ≥2 inbound failure transitions whose guard citation sets differ is malformed — each distinct failure class is its own failure state; decidable from the graph alone (compile.md's two-class prose argument is the rationale, never the check — prose is not read)` | [[errors]]  |
| guard_negation_typed     | invariant | `a failure transition whose guard cites ≥1 [[id]] must cite exactly the citation set of the success transition(s) it negates — a negated disjunction cites the union of its branches' citation sets; a failure transition citing no ids at all is either on the recorded carve-out list (orchestrate.md's stage pipeline, per its own Notes) or malformed — the carve-out is a checked list in linter-failure_shape.md, never a silent assumption` | [[errors]]  |
| single_labeled_failure   | invariant | `a failing tool stage reports exactly one labeled error naming the failing stage and class — never a silent partial result and never an unlabeled failure` (restates compile_is_total at contract altitude)             | [[errors]]  |
| envelope_error_kind      | extension_point | `a failing stage's report is an error-kind envelope with ok == false — the ok:false ↔ "error" correspondence is the published contract row, not CHANGELOG lore`                                                     | [[errors]]  |
| exit_code_mapping        | extension_point | `the output contract maps outcomes to exit codes: 0 for a clean run, 1 for findings-or-failure, 2 for invocation error — the mapping is part of the published contract, not per-tool convention`                      | [[errors]]  |
| remediation_hint_present | extension_point | `every error report carries a non-empty remediation hint` (one row per concern: envelope kind, exit codes, and hint are separate rows, so satisfies edges compose granularly and each concern has its own falsifying property)                                          | [[errors]]  |
| error_property_names_label | invariant | `every error Constraint has ≥1 unit property whose predicate asserts the exact label string — a label with no falsifying property is not specced, only named`                                                            | [[errors]]  |
| contract_published       | invariant | `the output contract rows live in specs/errors.md as kind == extension_point, and exist only there — capability constraints state requirements about them, never duplicate them; tool files point satisfies at the errors.md rows from their own files, per the extension_point mechanism` | [[errors]]  |
| enforcement_routed       | invariant | `every rule in this file states its enforcement tier and no scenario overclaims the present tool: tier 1 — enforced today by existing checkers (emits→effect typing via ref_kind_compatible, every_state_used, coverage); tier 2 — enforced by linter-failure_shape once implemented (label-shape pattern check, mute failure states, carve-out list, negation-set equality), its scenarios are normative for that checker, not claims about today's spk lint; tier 3 — runtime rows (single labeled failure, remediation hint, exit codes) enforced via the compile/verify pipeline fixtures` | [[errors]]  |

## Model

### States
- `draft`
- `published`

### Transitions

| id       | from  | to        | guard                                                          |
|----------|-------|-----------|-----------------------------------------------------------------|
| publish  | draft | published | [[errors.contract_published]] — the rows exist here, extension_point-kind, and only here |

## Properties

| id                                  | kind | derives_from                        | generator                                                   | predicate                                                                                       |
|-------------------------------------|------|-------------------------------------|-------------------------------------------------------------|-------------------------------------------------------------------------------------------------|
| unnamespaced_label_rejected         | unit | [[errors.error_expr_shape]]           | `error_constraint_with_label_lacking_file_id_prefix()`       | `check(file) == failed` — a bare `extraction_failure(...)` head is not a valid label (tier 2)     |
| mute_failure_state_detected         | unit | [[errors.failure_state_emits]]        | `tool_file_with_failure_terminal_and_no_emits_edge()`        | `check(file) == failed` — the corpus's pre-change shape, asserting the red is real (tier 2; the emits-typing half is tier 1) |
| emits_wrong_kind_rejected           | unit | [[errors.failure_state_emits]]        | `failure_state_emitting_an_invariant_constraint()`           | `check(file) == failed` — existing ref_kind_compatible typing, restated as the contract's edge (tier 1) |
| multiclass_single_state_detected    | unit | [[errors.failure_class_is_state]]     | `failed_state_with_inbound_failure_transitions_citing_different_constraint_sets()` | `check(file) == failed` — compile.md's pre-change shape, detected from the graph (tier 2)   |
| untyped_negation_detected           | unit | [[errors.guard_negation_typed]]       | `failure_transition_whose_citation_set_differs_from_its_success_transition()` | `check(file) == failed` — citation-set inequality is graph-decidable (tier 2)               |
| zero_citation_failure_flagged       | unit | [[errors.guard_negation_typed]]       | `failure_transition_citing_zero_intrafile_constraints_not_on_carveout_list()` | `check(file) == failed` — the carve-out is a checked list, not an escape hatch (tier 2)      |
| carved_out_guard_not_flagged        | unit | [[errors.guard_negation_typed]]       | `orchestrate_stage_fail_transition()`                        | `check(file) == passed` — the carve-out is checked, not assumed (tier 2)                       |
| labeled_failure_is_single           | unit | [[errors.single_labeled_failure]]     | `failing_stage_with_two_extractable_errors()`                | `report(stage) == one labeled error naming stage and class` — never a partial bundle (tier 3)      |
| envelope_kind_wrong_rejected        | unit | [[errors.envelope_error_kind]]        | `failing_stage_reported_as_ok_envelope()`                    | `check(report) == failed` — a failure must never ride a success-shaped envelope (tier 3)          |
| exit_code_mismatch_rejected         | unit | [[errors.exit_code_mapping]]          | `failure_stage_with_wrong_exit_code()`                       | `check(report) == failed` — exit codes are contract, not convention (tier 3)                      |
| hint_missing_rejected               | unit | [[errors.remediation_hint_present]]   | `error_report_with_empty_remediation_hint()`                 | `check(report) == failed` (tier 3)                                                                |
| label_property_asserts_exact_label  | unit | [[errors.error_property_names_label]] | `error_constraint_with_property_predicating_a_different_label()` | `check(file) == failed` — coverage alone cannot catch this; the property must name the label   |
| routing_matches_reality             | unit | [[errors.enforcement_routed]]         | `each_rule_checked_against_its_stated_tier()`                | `every scenario's tool claim agrees with its tier — no scenario claims today's spk lint for a tier-2 rule` |
| contract_satisfied_from_consumer    | unit | [[errors.contract_published]]         | `tool_file_with_satisfies_pointing_at_contract_row()`        | `extraction yields exactly one satisfies edge and lint passes` — the satisfies precedent's shape   |