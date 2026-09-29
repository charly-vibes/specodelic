---
id: spec
kind: intent
statement: "WHEN a specodelic tool stage fails, THE error-contract capability SHALL report a single file-id-namespaced labeled error carried by an emitting failure state, with a remediation hint and a falsifying property asserting the exact label."
---

# error-contract Specification

## Purpose
Make errors in the specodelic corpus typed, namespaced, and falsifiable
facts instead of CHANGELOG lore and mute `failed` states: the cross-cutting
output contract (envelope kind, exit codes, one labeled failure per stage,
remediation hints) becomes a published `extension_point` contract, every
tool file's failure terminal emits a file-owned labeled error Constraint,
and every label carries a unit property asserting the exact label — the
same red→green discipline every other corpus fact already has.

## Constraints

| id                       | kind            | expr                                                                                                                                                                                                                   | traces_to |
|--------------------------|-----------------|------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|-----------|
| error_expr_shape         | extension_point | `every error Constraint's expr is a file-id-namespaced variant head: <owning-file-id>.<variant_head>(field, …) — the label names its owning file, so label uniqueness is a per-file property and cross-file collisions are structurally impossible` | [[spec]]  |
| failure_state_emits      | extension_point | `every failure terminal state in a tool file's Model carries an emits edge to an effect Constraint owned by that file, whose expr satisfies error_expr_shape — a failure state emits nothing is a malformed model` (phase-2 terminals timed_out / exploration_only are excluded in v1) | [[spec]]  |
| failure_class_is_state   | invariant       | `where a file's own prose distinguishes failure classes, each class is its own failure state — a single undifferentiated failed state under multi-class prose is the flattened-generic-error pathology at the state-machine level` | [[spec]]  |
| guard_negation_typed     | invariant       | `a failure transition's guard is the typed negation of the intra-file constraints it fails against — ¬([[c1]] ∧ [[c2]] ∧ …); files whose failure guards deliberately cite no intra-file rows (orchestrate.md's stage pipeline, per its own Notes) are a stated carve-out, recorded in linter-failure_shape.md, not silent` | [[spec]]  |
| single_labeled_failure   | extension_point | `a failing tool stage reports exactly one labeled error naming the failing stage and class — never a silent partial result and never an unlabeled failure` (restates compile_is_total at contract altitude)             | [[spec]]  |
| remediation_hint_present | extension_point | `every error report carries a non-empty remediation hint — the output contract's exit-code mapping (0 ok, 1 findings-or-failure, 2 invocation error) and envelope kind (ok:false ↔ "error") are part of this contract row` | [[spec]]  |
| error_property_names_label | invariant     | `every error Constraint has ≥1 unit property whose predicate asserts the exact label string — a label with no falsifying property is not specced, only named`                                                            | [[spec]]  |
| contract_published       | invariant       | `the output contract rows live in specs/errors.md as kind == extension_point; tool files point satisfies at them from their own files — consumers never edit the contract file, per the extension_point mechanism`        | [[spec]]  |

## Model

### States
- `proposed`
- `approved`
- `implemented`
- `archived`

### Transitions

| id         | from        | to          | guard                                                                                       |
|------------|-------------|-------------|----------------------------------------------------------------------------------------------|
| approve    | proposed    | approved    | `proposal reviewed and approved by the maintainer`                                            |
| implement  | approved    | implemented | `all tasks.md items complete; spk lint and openspec validate --strict pass over the change`   |
| archive    | implemented | archived    | `just archive-change id=add-error-contract ran with the dual-format recipe`                   |

## Properties

| id                                  | kind | derives_from                        | generator                                                   | predicate                                                                                       |
|-------------------------------------|------|-------------------------------------|-------------------------------------------------------------|-------------------------------------------------------------------------------------------------|
| unnamespaced_label_rejected         | unit | [[spec.error_expr_shape]]           | `error_constraint_with_label_lacking_file_id_prefix()`       | `check(file) == failed` — a bare `extraction_failure(...)` head is not a valid label              |
| mute_failure_state_detected         | unit | [[spec.failure_state_emits]]        | `tool_file_with_failure_terminal_and_no_emits_edge()`        | `check(file) == failed` — the corpus's current shape, asserting the red is real                   |
| emits_wrong_kind_rejected           | unit | [[spec.failure_state_emits]]        | `failure_state_emitting_an_invariant_constraint()`           | `check(file) == failed` — existing ref_kind_compatible typing, restated as the contract's edge    |
| multiclass_single_state_detected    | unit | [[spec.failure_class_is_state]]     | `file_arguing_two_failure_classes_with_one_failed_state()`   | `check(file) == failed` — compile.md's pre-change shape                                           |
| untyped_negation_detected           | unit | [[spec.guard_negation_typed]]       | `failure_transition_with_prose_negation_guard()`             | `check(file) == failed` — ¬x_ok.guard with no [[id]] citations                                     |
| carved_out_guard_not_flagged        | unit | [[spec.guard_negation_typed]]       | `orchestrate_stage_fail_transition()`                        | `check(file) == passed` — the D2 carve-out is checked, not assumed                                |
| labeled_failure_is_single           | unit | [[spec.single_labeled_failure]]     | `failing_stage_with_two_extractable_errors()`                | `report(stage) == one labeled error naming stage and class` — never a partial bundle               |
| hint_missing_rejected               | unit | [[spec.remediation_hint_present]]   | `error_report_with_empty_remediation_hint()`                 | `check(report) == failed`                                                                         |
| label_property_asserts_exact_label  | unit | [[spec.error_property_names_label]] | `error_constraint_with_property_predicating_a_different_label()` | `check(file) == failed` — coverage alone cannot catch this; the property must name the label   |
| contract_satisfied_from_consumer    | unit | [[spec.contract_published]]         | `tool_file_with_satisfies_pointing_at_contract_row()`        | `extraction yields exactly one satisfies edge and lint passes` — the satisfies precedent's shape   |

## ADDED Requirements

### Requirement: Namespaced labeled errors
The system SHALL require every error Constraint in the corpus to carry a file-id-namespaced variant-head label (`<owning-file-id>.<variant_head>(field, …)`), and SHALL reject any error Constraint whose label lacks the owning file's id prefix or whose variant head collides within one file.

#### Scenario: Bare label rejected
- **WHEN** an error Constraint's expr is `extraction_failure(row_id)` without the owning file's id prefix
- **THEN** `spk lint` fails the file, naming the label and the expected `<file-id>.` prefix

#### Scenario: Namespaced label accepted
- **WHEN** an error Constraint in `compile.md` declares expr `compile.extraction_failure(row_id, reason)`
- **THEN** the file lints clean and the label is unique corpus-wide by construction

### Requirement: Failure states emit labeled errors
The system SHALL require every failure terminal state in a tool file's Model to carry an `emits` edge to that file's labeled error Constraint, and SHALL require a distinct failure state per failure class wherever the file's own prose distinguishes classes.

#### Scenario: Mute failure state rejected
- **WHEN** a tool file's Model contains a failure terminal state with no `emits` edge
- **THEN** `spk lint` fails the file with a remediation hint pointing at the missing emits target

#### Scenario: Split failure states carry distinct labels
- **WHEN** `compile.md`'s Model distinguishes extraction and emission failure classes as `extract_failed` and `emit_failed`
- **THEN** each state emits its own labeled error Constraint and `every_state_used` passes

### Requirement: Typed failure guards
The system SHALL require a failure transition's guard to be the typed negation of the intra-file constraints it fails against (`¬([[c1]] ∧ [[c2]] ∧ …)`), with a stated carve-out for files whose guards deliberately cite no intra-file rows.

#### Scenario: Prose negation rejected
- **WHEN** a failure transition's guard is `¬x_ok.guard` — no `[[id]]` citations
- **THEN** `spk lint` fails the file, since the guard creates no typed edge for the reference typing to check

#### Scenario: Carve-out file passes
- **WHEN** `orchestrate.md`'s stage-fail transitions keep their prose guards per that file's own Notes
- **THEN** lint passes — the carve-out is recorded in `linter-failure_shape.md`, not silent

### Requirement: Output contract published and satisfied
The system SHALL publish the cross-cutting output contract — envelope kind `ok:false` ↔ `"error"`, exit codes 0/1/2, exactly one labeled failure per failing stage, and a non-empty remediation hint — as `extension_point` rows in `specs/errors.md`, with each tool file pointing `satisfies` at them from its own file.

#### Scenario: Consumer satisfies the contract
- **WHEN** a tool file carries a `satisfies` edge pointing at a contract row in `specs/errors.md`
- **THEN** extraction yields exactly one cross-file edge and lint passes — no edit to the contract file

#### Scenario: Label without falsifying property rejected
- **WHEN** an error Constraint has no unit property whose predicate asserts its exact label string
- **THEN** `spk lint` fails the file — a named-but-unfalsifiable label is not a specced error

## Requirements

### Requirement: Namespaced labeled errors
The system SHALL require every error Constraint in the corpus to carry a file-id-namespaced variant-head label (`<owning-file-id>.<variant_head>(field, …)`), and SHALL reject any error Constraint whose label lacks the owning file's id prefix or whose variant head collides within one file.

#### Scenario: Bare label rejected
- **WHEN** an error Constraint's expr is `extraction_failure(row_id)` without the owning file's id prefix
- **THEN** `spk lint` fails the file, naming the label and the expected `<file-id>.` prefix

#### Scenario: Namespaced label accepted
- **WHEN** an error Constraint in `compile.md` declares expr `compile.extraction_failure(row_id, reason)`
- **THEN** the file lints clean and the label is unique corpus-wide by construction

### Requirement: Failure states emit labeled errors
The system SHALL require every failure terminal state in a tool file's Model to carry an `emits` edge to that file's labeled error Constraint, and SHALL require a distinct failure state per failure class wherever the file's own prose distinguishes classes.

#### Scenario: Mute failure state rejected
- **WHEN** a tool file's Model contains a failure terminal state with no `emits` edge
- **THEN** `spk lint` fails the file with a remediation hint pointing at the missing emits target

#### Scenario: Split failure states carry distinct labels
- **WHEN** `compile.md`'s Model distinguishes extraction and emission failure classes as `extract_failed` and `emit_failed`
- **THEN** each state emits its own labeled error Constraint and `every_state_used` passes

### Requirement: Typed failure guards
The system SHALL require a failure transition's guard to be the typed negation of the intra-file constraints it fails against (`¬([[c1]] ∧ [[c2]] ∧ …)`), with a stated carve-out for files whose guards deliberately cite no intra-file rows.

#### Scenario: Prose negation rejected
- **WHEN** a failure transition's guard is `¬x_ok.guard` — no `[[id]]` citations
- **THEN** `spk lint` fails the file, since the guard creates no typed edge for the reference typing to check

#### Scenario: Carve-out file passes
- **WHEN** `orchestrate.md`'s stage-fail transitions keep their prose guards per that file's own Notes
- **THEN** lint passes — the carve-out is recorded in `linter-failure_shape.md`, not silent

### Requirement: Output contract published and satisfied
The system SHALL publish the cross-cutting output contract — envelope kind `ok:false` ↔ `"error"`, exit codes 0/1/2, exactly one labeled failure per failing stage, and a non-empty remediation hint — as `extension_point` rows in `specs/errors.md`, with each tool file pointing `satisfies` at them from its own file.

#### Scenario: Consumer satisfies the contract
- **WHEN** a tool file carries a `satisfies` edge pointing at a contract row in `specs/errors.md`
- **THEN** extraction yields exactly one cross-file edge and lint passes — no edit to the contract file

#### Scenario: Label without falsifying property rejected
- **WHEN** an error Constraint has no unit property whose predicate asserts its exact label string
- **THEN** `spk lint` fails the file — a named-but-unfalsifiable label is not a specced error
