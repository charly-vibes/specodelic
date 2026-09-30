---
id: spec
kind: intent
statement: "WHEN a specodelic tool stage fails, THE error-contract capability SHALL ensure the failure is reported as a single file-id-namespaced labeled error with a remediation hint, backed by an emitting failure state and a falsifying property asserting the exact label."
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

Scope note: this capability spans two altitudes, kept distinct throughout
— *lint-time corpus rules* (failure states, guards: about the content of
`specs/*.md` files) and the *runtime output contract* (envelope, exit
codes: about tool behavior). "Emits" names the format mechanism; runtime
reporting is "reports". Enforcement routing is itself specced:
`enforcement_routed` below states which tier enforces each rule today,
which waits for `linter-failure_shape`, and which rides the compile/verify
pipeline.

## Constraints

| id                       | kind      | expr                                                                                                                                                                                                                   | traces_to |
|--------------------------|-----------|------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|-----------|
| error_expr_shape         | invariant | `every error Constraint's expr is a file-id-namespaced variant head: <owning-file-id>.<variant_head>(field, …) — the label names its owning file, so label uniqueness is a per-file property and cross-file collisions are structurally impossible` (pattern-checking the structured expr field follows the ears_statement precedent — expr is not rationale; prose_untouched is untouched) | [[spec]]  |
| failure_state_emits      | invariant | `every failure terminal state in a tool file's Model carries an emits edge to an effect Constraint owned by that file, whose expr satisfies error_expr_shape — a failure state emits nothing is a malformed model` (phase-2 terminals timed_out / exploration_only are excluded in v1) | [[spec]]  |
| failure_class_is_state   | invariant | `a state with ≥2 inbound failure transitions whose guard citation sets differ is malformed — each distinct failure class is its own failure state; decidable from the graph alone (compile.md's two-class prose argument is the rationale, never the check — prose is not read)` | [[spec]]  |
| guard_negation_typed     | invariant | `a failure transition whose guard cites ≥1 intra-file [[id]] must cite exactly the citation set of the success transition it negates; a failure transition citing zero intra-file constraints is either on the recorded carve-out list (orchestrate.md's stage pipeline, per its own Notes) or malformed — the carve-out is a checked list in linter-failure_shape.md, never a silent assumption` | [[spec]]  |
| single_labeled_failure   | invariant | `a failing tool stage reports exactly one labeled error naming the failing stage and class — never a silent partial result and never an unlabeled failure` (restates compile_is_total at contract altitude)             | [[spec]]  |
| envelope_error_kind      | invariant | `a failing stage's report is an error-kind envelope with ok == false — the ok:false ↔ "error" correspondence is the published contract row, not CHANGELOG lore`                                                     | [[spec]]  |
| exit_code_mapping        | invariant | `the output contract maps outcomes to exit codes: 0 for a clean run, 1 for findings-or-failure, 2 for invocation error — the mapping is part of the published contract, not per-tool convention`                      | [[spec]]  |
| remediation_hint_present | invariant | `every error report carries a non-empty remediation hint` (one row per concern: envelope kind, exit codes, and hint are separate rows, resolving design.md's Open Question)                                          | [[spec]]  |
| error_property_names_label | invariant | `every error Constraint has ≥1 unit property whose predicate asserts the exact label string — a label with no falsifying property is not specced, only named`                                                            | [[spec]]  |
| contract_published       | invariant | `the output contract rows live in specs/errors.md as kind == extension_point, and exist only there — this capability's constraints state requirements about them, never duplicate them; tool files point satisfies at the errors.md rows from their own files, per the extension_point mechanism` | [[spec]]  |
| enforcement_routed       | invariant | `every rule in this capability states its enforcement tier and no scenario overclaims the present tool: tier 1 — enforced today by existing checkers (emits→effect typing via ref_kind_compatible, every_state_used, graph citation-set comparison); tier 2 — enforced by linter-failure_shape once implemented (label-shape pattern check, mute failure states, carve-out list, negation-set equality), its scenarios are normative for that checker, not claims about today's spk lint; tier 3 — runtime rows (single labeled failure, remediation hint, exit codes) enforced via the compile/verify pipeline fixtures` | [[spec]]  |

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
| unnamespaced_label_rejected         | unit | [[spec.error_expr_shape]]           | `error_constraint_with_label_lacking_file_id_prefix()`       | `check(file) == failed` — a bare `extraction_failure(...)` head is not a valid label (tier 2)     |
| mute_failure_state_detected         | unit | [[spec.failure_state_emits]]        | `tool_file_with_failure_terminal_and_no_emits_edge()`        | `check(file) == failed` — the corpus's current shape, asserting the red is real (tier 2; the emits-typing half is tier 1) |
| emits_wrong_kind_rejected           | unit | [[spec.failure_state_emits]]        | `failure_state_emitting_an_invariant_constraint()`           | `check(file) == failed` — existing ref_kind_compatible typing, restated as the contract's edge (tier 1) |
| multiclass_single_state_detected    | unit | [[spec.failure_class_is_state]]     | `failed_state_with_inbound_failure_transitions_citing_different_constraint_sets()` | `check(file) == failed` — compile.md's pre-change shape, detected from the graph (tier 2)   |
| untyped_negation_detected           | unit | [[spec.guard_negation_typed]]       | `failure_transition_whose_citation_set_differs_from_its_success_transition()` | `check(file) == failed` — citation-set inequality is graph-decidable (tier 2)               |
| zero_citation_failure_flagged       | unit | [[spec.guard_negation_typed]]       | `failure_transition_citing_zero_intrafile_constraints_not_on_carveout_list()` | `check(file) == failed` — the carve-out is a checked list, not an escape hatch (tier 2)      |
| carved_out_guard_not_flagged        | unit | [[spec.guard_negation_typed]]       | `orchestrate_stage_fail_transition()`                        | `check(file) == passed` — the D2 carve-out is checked, not assumed (tier 2)                       |
| labeled_failure_is_single           | unit | [[spec.single_labeled_failure]]     | `failing_stage_with_two_extractable_errors()`                | `report(stage) == one labeled error naming stage and class` — never a partial bundle (tier 3)      |
| envelope_kind_wrong_rejected        | unit | [[spec.envelope_error_kind]]        | `failing_stage_reported_as_ok_envelope()`                    | `check(report) == failed` — a failure must never ride a success-shaped envelope (tier 3)          |
| exit_code_mismatch_rejected         | unit | [[spec.exit_code_mapping]]          | `failure_stage_with_wrong_exit_code()`                       | `check(report) == failed` — exit codes are contract, not convention (tier 3)                      |
| hint_missing_rejected               | unit | [[spec.remediation_hint_present]]   | `error_report_with_empty_remediation_hint()`                 | `check(report) == failed` (tier 3)                                                                |
| label_property_asserts_exact_label  | unit | [[spec.error_property_names_label]] | `error_constraint_with_property_predicating_a_different_label()` | `check(file) == failed` — coverage alone cannot catch this; the property must name the label   |
| routing_matches_reality             | unit | [[spec.enforcement_routed]]         | `each_rule_checked_against_its_stated_tier()`                | `every scenario's tool claim agrees with its tier — no scenario claims today's spk lint for a tier-2 rule` |
| contract_satisfied_from_consumer    | unit | [[spec.contract_published]]         | `tool_file_with_satisfies_pointing_at_contract_row()`        | `extraction yields exactly one satisfies edge and lint passes` — the satisfies precedent's shape   |

## ADDED Requirements

### Requirement: Namespaced labeled errors
The system SHALL require every error Constraint in the corpus to carry a file-id-namespaced variant-head label (`<owning-file-id>.<variant_head>(field, …)`), and SHALL reject any error Constraint whose label lacks the owning file's id prefix or whose variant head collides within one file.

#### Scenario: Bare label rejected
- **WHEN** an error Constraint's expr is `extraction_failure(row_id)` without the owning file's id prefix
- **THEN** the tier-2 check fails the file, naming the label and the expected `<file-id>.` prefix — until `linter-failure_shape` lands, enforcement is the property fixture plus review, never a claim about today's `spk lint`

#### Scenario: Namespaced label accepted
- **WHEN** an error Constraint in `compile.md` declares expr `compile.extraction_failure(row_id, reason)`
- **THEN** the file lints clean and the label is unique corpus-wide by construction

### Requirement: Failure states emit labeled errors
The system SHALL require every failure terminal state in a tool file's Model to carry an `emits` edge to that file's labeled error Constraint, and SHALL require a distinct failure state per distinct failure class, where classes are distinguished by differing guard citation sets.

#### Scenario: Mute failure state rejected
- **WHEN** a tool file's Model contains a failure terminal state with no `emits` edge
- **THEN** the tier-2 check fails the file with a remediation hint pointing at the missing emits target; the emits-typing half (wrong target kind) is enforced today by `ref_kind_compatible`

#### Scenario: Split failure states carry distinct labels
- **WHEN** `compile.md`'s Model distinguishes extraction and emission failure classes as `extract_failed` and `emit_failed`
- **THEN** each state emits its own labeled error Constraint and `every_state_used` passes

### Requirement: Typed failure guards
The system SHALL require a failure transition whose guard cites ≥1 intra-file constraint to cite exactly the citation set of the success transition it negates, and SHALL require failure transitions citing zero intra-file constraints to be on a recorded carve-out list.

#### Scenario: Citation-set mismatch rejected
- **WHEN** a failure transition's guard cites an intra-file citation set different from the success transition it negates — or cites nothing while absent from the carve-out list
- **THEN** the tier-2 check fails the file, since the citation sets are graph-decidable and the carve-out is a checked list

#### Scenario: Carve-out file passes
- **WHEN** `orchestrate.md`'s stage-fail transitions keep their prose guards per that file's own Notes
- **THEN** the tier-2 check passes — `orchestrate.md` is on the recorded carve-out list in `linter-failure_shape.md`, not silent

### Requirement: Output contract published and satisfied
The system SHALL publish the cross-cutting output contract — envelope kind `ok:false` ↔ `"error"`, exit codes 0/1/2, exactly one labeled failure per failing stage, and a non-empty remediation hint — as `extension_point` rows in `specs/errors.md`, with each tool file pointing `satisfies` at them from its own file.

#### Scenario: Consumer satisfies the contract
- **WHEN** a tool file carries a `satisfies` edge pointing at a contract row in `specs/errors.md`
- **THEN** extraction yields exactly one cross-file edge and lint passes — no edit to the contract file

#### Scenario: Property-less constraint rejected
- **WHEN** an error Constraint has no unit property at all
- **THEN** the tier-1 coverage check fails the file — coverage over every constraint is enforced today; a named-but-unfalsifiable label is not a specced error

#### Scenario: Mismatched label predicate rejected
- **WHEN** an error Constraint's unit property predicate asserts a different label string than the constraint's own
- **THEN** the tier-2 check fails the file — exact-label assertion waits for `linter-failure_shape`; today's coverage cannot see the predicate text, so no tier-1 claim is made

## Requirements

### Requirement: Namespaced labeled errors
The system SHALL require every error Constraint in the corpus to carry a file-id-namespaced variant-head label (`<owning-file-id>.<variant_head>(field, …)`), and SHALL reject any error Constraint whose label lacks the owning file's id prefix or whose variant head collides within one file.

#### Scenario: Bare label rejected
- **WHEN** an error Constraint's expr is `extraction_failure(row_id)` without the owning file's id prefix
- **THEN** the tier-2 check fails the file, naming the label and the expected `<file-id>.` prefix — until `linter-failure_shape` lands, enforcement is the property fixture plus review, never a claim about today's `spk lint`

#### Scenario: Namespaced label accepted
- **WHEN** an error Constraint in `compile.md` declares expr `compile.extraction_failure(row_id, reason)`
- **THEN** the file lints clean and the label is unique corpus-wide by construction

### Requirement: Failure states emit labeled errors
The system SHALL require every failure terminal state in a tool file's Model to carry an `emits` edge to that file's labeled error Constraint, and SHALL require a distinct failure state per distinct failure class, where classes are distinguished by differing guard citation sets.

#### Scenario: Mute failure state rejected
- **WHEN** a tool file's Model contains a failure terminal state with no `emits` edge
- **THEN** the tier-2 check fails the file with a remediation hint pointing at the missing emits target; the emits-typing half (wrong target kind) is enforced today by `ref_kind_compatible`

#### Scenario: Split failure states carry distinct labels
- **WHEN** `compile.md`'s Model distinguishes extraction and emission failure classes as `extract_failed` and `emit_failed`
- **THEN** each state emits its own labeled error Constraint and `every_state_used` passes

### Requirement: Typed failure guards
The system SHALL require a failure transition whose guard cites ≥1 intra-file constraint to cite exactly the citation set of the success transition it negates, and SHALL require failure transitions citing zero intra-file constraints to be on a recorded carve-out list.

#### Scenario: Citation-set mismatch rejected
- **WHEN** a failure transition's guard cites an intra-file citation set different from the success transition it negates — or cites nothing while absent from the carve-out list
- **THEN** the tier-2 check fails the file, since the citation sets are graph-decidable and the carve-out is a checked list

#### Scenario: Carve-out file passes
- **WHEN** `orchestrate.md`'s stage-fail transitions keep their prose guards per that file's own Notes
- **THEN** the tier-2 check passes — `orchestrate.md` is on the recorded carve-out list in `linter-failure_shape.md`, not silent

### Requirement: Output contract published and satisfied
The system SHALL publish the cross-cutting output contract — envelope kind `ok:false` ↔ `"error"`, exit codes 0/1/2, exactly one labeled failure per failing stage, and a non-empty remediation hint — as `extension_point` rows in `specs/errors.md`, with each tool file pointing `satisfies` at them from its own file.

#### Scenario: Consumer satisfies the contract
- **WHEN** a tool file carries a `satisfies` edge pointing at a contract row in `specs/errors.md`
- **THEN** extraction yields exactly one cross-file edge and lint passes — no edit to the contract file

#### Scenario: Property-less constraint rejected
- **WHEN** an error Constraint has no unit property at all
- **THEN** the tier-1 coverage check fails the file — coverage over every constraint is enforced today; a named-but-unfalsifiable label is not a specced error

#### Scenario: Mismatched label predicate rejected
- **WHEN** an error Constraint's unit property predicate asserts a different label string than the constraint's own
- **THEN** the tier-2 check fails the file — exact-label assertion waits for `linter-failure_shape`; today's coverage cannot see the predicate text, so no tier-1 claim is made
