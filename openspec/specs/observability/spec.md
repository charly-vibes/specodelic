---
id: spec
kind: intent
statement: "THE observability capability SHALL let Constraint rows declare observability of effect Constraints via a typed one-directional observes reference, SHALL report every unobserved effect as an advisory finding, and SHALL derive external-boundary classification from published extension_point rows alone."
---

# observability Specification

## Purpose
Make declared observability and external-boundary structure checkable
facts from the start of a system's life: a spec author declares which
behaviors must be observable via `observes` references, the linter reports
every declared effect that nothing observes (advisory in v1, never a
silent pass or a fabricated gate), and `spk graph` derives which files are
external boundaries from published `extension_point` contracts alone —
no hand-maintained tags that can drift from the edges.

## Constraints

| id                        | kind      | expr                                                                                                                                                                                                                                                                       | traces_to |
|---------------------------|-----------|-----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|-----------|
| observes_typing           | invariant | `every observes reference is a one-directional outbound pointer from a Constraint row to a Constraint row with kind == effect (intra-file or cross-file) — same shape as the satisfies precedent, per the Reference Typing row for observes in specs/specodelic.md`                                                                                                                                                                          | [[spec]]  |
| acyclic_edge_set_stable   | invariant | `the acyclic invariant's edge set (specs/linter-graph_shape.md) grows only under a new Revision of that file; observes is deliberately absent from it — an observation claim is not a dependency, so mutual observation across files is well-formed`                                                                                                                         | [[spec]]  |
| unobserved_effect_reported | invariant | `within the lint invocation's file set (corpus-wide lint via just lint-specs is the canonical run), every effect Constraint is either the target of ≥1 observes edge or is reported as an advisory warning on the lint output's warnings channel — exit 0, never counted as a lint failure (design D5) — naming the unobserved row; no waiver machinery exists in v1 (specs/linter-external_completeness.md's covered/waived pattern is the candidate when the need is demonstrated)` | [[spec]]  |
| no_gate_change            | invariant | `whether linter.observability ever gates a lifecycle transition (specs/orchestrate.md) is decided only after dogfooding produces real friction evidence — until then it is advisory by severity and the stage guards are untouched`                                                                                                                                        | [[spec]]  |
| boundary_derived          | invariant | `external-boundary classification is derived from published extension_point Constraints (a file hosting ≥1 is an external boundary); no authored boundary tag exists anywhere — hand-maintained tags drift from edges, against specs/graph.md's graph_is_derived_not_authored philosophy`   | [[spec]]  |

## Model

### States
- `proposed`
- `approved`
- `implemented`
- `archived`

### Transitions

| id         | from       | to          | guard                                                                                      |
|------------|------------|-------------|--------------------------------------------------------------------------------------------|
| approve    | proposed   | approved    | `proposal reviewed and approved by the maintainer`                                          |
| implement  | approved   | implemented | `all tasks.md items complete; spk lint and openspec validate --strict pass over the change` |
| archive    | implemented| archived    | `just archive-change id=add-observability-contracts ran with the dual-format recipe`        |

## Properties

| id                             | kind | derives_from                     | generator                                              | predicate                                                              |
|--------------------------------|------|----------------------------------|----------------------------------------------------------------------------------------------|
| observes_wrong_target_rejected | unit | [[spec.observes_typing]]         | `observes_field_pointing_at_an_invariant_constraint()`  | `check(file) == failed` — same shape as emits_cannot_target_non_effect |
| observes_cross_file_resolves   | unit | [[spec.observes_typing]]         | `observes_from_consumer_file_to_published_effect()`     | `extraction yields exactly one cross-file edge and lint passes`        |
| observes_cycle_not_flagged     | unit | [[spec.acyclic_edge_set_stable]] | `two_files_mutually_observing_each_others_effects()`    | `acyclic check passes — no cycle finding emitted`                      |
| unobserved_effect_finding      | unit | [[spec.unobserved_effect_reported]] | `effect_with_zero_observes_edges()`                  | `advisory finding linter.observability emitted naming the row id`      |
| observed_effect_silent         | unit | [[spec.unobserved_effect_reported]] | `effect_with_at_least_one_observes_edge()`           | `no linter.observability finding emitted`                              |
| gates_untouched                | unit | [[spec.no_gate_change]]          | `corpus_with_unobserved_effects_through_full_pipeline()` | `linted/compiled/model_checked/verified outcomes identical to before` |
| boundary_from_extension_points | unit | [[spec.boundary_derived]]        | `file_hosting_extension_point_row()`                   | `graph classifies the file as an external boundary`                    |
| plain_file_not_boundary        | unit | [[spec.boundary_derived]]        | `file_with_no_extension_point_rows()`                  | `graph does not classify the file as an external boundary`             |
| stale_boundary_impossible      | unit | [[spec.boundary_derived]]        | `file_whose_extension_points_are_all_removed()`        | `boundary classification disappears — no authored tag left to clean`   |

## ADDED Requirements

### Requirement: Typed one-directional observes reference
The system SHALL accept an optional `observes` reference on Constraint rows, typed as a one-directional outbound pointer from any Constraint to a Constraint with kind == `effect` on any file — mirroring the `satisfies` precedent — and SHALL reject any `observes` pointing at a non-effect row.

#### Scenario: Cross-file observation resolves
- **WHEN** a Constraint row in file A carries `observes` pointing at an effect Constraint in file B
- **THEN** graph extraction yields exactly one cross-file edge and `spk lint` reports no typing violation

#### Scenario: Wrong target kind rejected
- **WHEN** a Constraint row carries `observes` pointing at a Constraint with kind == `invariant`
- **THEN** `spk lint` fails the file with a remediation hint naming the target's actual kind

#### Scenario: Mutual observation is not a cycle violation
- **WHEN** two files' Constraint rows each `observes` an effect in the other file
- **THEN** the acyclic check passes — `observes` is a claim, not a dependency edge

### Requirement: Unobserved effects are reported, never silent
The system SHALL report, with rule id `linter.observability`, on the lint output's warnings channel (advisory severity — exit 0, never counted as a lint failure; design D5), every effect Constraint within the lint invocation's file set that is the target of no `observes` edge (corpus-wide lint is the canonical run) — and SHALL NOT gate any lifecycle transition on this finding in v1.

#### Scenario: Effect with no observer reported
- **WHEN** a linted spec declares an effect Constraint that no `observes` reference targets
- **THEN** the lint output carries an advisory `linter.observability` warning on the warnings channel, naming the unobserved row id, with exit 0

#### Scenario: Observed effect produces no finding
- **WHEN** every effect Constraint in the corpus is targeted by at least one `observes` edge
- **THEN** no `linter.observability` finding is emitted and the lifecycle gate outcomes are unchanged

### Requirement: External boundaries are derived, not tagged
The system SHALL classify a file as an external boundary in `spk graph` output iff it hosts at least one `extension_point` Constraint, deriving the classification from existing reference structure — and SHALL NOT introduce any authored boundary tag.

#### Scenario: Boundary derived from published contract
- **WHEN** `spk graph --json` runs on a corpus where file B hosts an `extension_point` Constraint
- **THEN** B is classified as an external boundary without any authored tag

#### Scenario: No tag drift
- **WHEN** a file's `extension_point` rows are all removed in a revision
- **THEN** the file's external-boundary classification disappears — there is no stale tag to clean up

## Requirements
### Requirement: Typed one-directional observes reference
The system SHALL accept an optional `observes` reference on Constraint rows, typed as a one-directional outbound pointer from any Constraint to a Constraint with kind == `effect` on any file — mirroring the `satisfies` precedent — and SHALL reject any `observes` pointing at a non-effect row.

#### Scenario: Cross-file observation resolves
- **WHEN** a Constraint row in file A carries `observes` pointing at an effect Constraint in file B
- **THEN** graph extraction yields exactly one cross-file edge and `spk lint` reports no typing violation

#### Scenario: Wrong target kind rejected
- **WHEN** a Constraint row carries `observes` pointing at a Constraint with kind == `invariant`
- **THEN** `spk lint` fails the file with a remediation hint naming the target's actual kind

#### Scenario: Mutual observation is not a cycle violation
- **WHEN** two files' Constraint rows each `observes` an effect in the other file
- **THEN** the acyclic check passes — `observes` is a claim, not a dependency edge

### Requirement: Unobserved effects are reported, never silent
The system SHALL report, with rule id `linter.observability`, on the lint output's warnings channel (advisory severity — exit 0, never counted as a lint failure; design D5), every effect Constraint within the lint invocation's file set that is the target of no `observes` edge (corpus-wide lint is the canonical run) — and SHALL NOT gate any lifecycle transition on this finding in v1.

#### Scenario: Effect with no observer reported
- **WHEN** a linted spec declares an effect Constraint that no `observes` reference targets
- **THEN** the lint output carries an advisory `linter.observability` warning on the warnings channel, naming the unobserved row id, with exit 0

#### Scenario: Observed effect produces no finding
- **WHEN** every effect Constraint in the corpus is targeted by at least one `observes` edge
- **THEN** no `linter.observability` finding is emitted and the lifecycle gate outcomes are unchanged

### Requirement: External boundaries are derived, not tagged
The system SHALL classify a file as an external boundary in `spk graph` output iff it hosts at least one `extension_point` Constraint, deriving the classification from existing reference structure — and SHALL NOT introduce any authored boundary tag.

#### Scenario: Boundary derived from published contract
- **WHEN** `spk graph --json` runs on a corpus where file B hosts an `extension_point` Constraint
- **THEN** B is classified as an external boundary without any authored tag

#### Scenario: No tag drift
- **WHEN** a file's `extension_point` rows are all removed in a revision
- **THEN** the file's external-boundary classification disappears — there is no stale tag to clean up
