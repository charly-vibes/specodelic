---
id: spec
kind: intent
statement: "WHEN a compiled artifact carries guard citations or kernel expressions, THE model-check backend SHALL evaluate guard citations under the citation algebra with an explicit three-valued status, SHALL keep executable guard fragments rejected of record, and SHALL persist run reports naming backend, bound, and per-invariant status with unknown persisted honestly."
---

# model-check Specification

## Purpose
Upgrade guard citations from inert TLA+ annotations to kernel-evaluated
claims — slice 1, the one-green-where-today-is-red increment — while
keeping the fragment_guard_rejected decision of record intact
(executable guard fragments stay rejected; the deferral's
data-carrying-state-space condition is not met by instance-grounded
expressions). Run reports persist backend, bound, and per-invariant
three-valued status, with the backend-agreement property running as a
CI property test over shared fixtures until the py backend lands.

## Constraints

| id                        | kind      | expr                                                                                                                                                                                                                                                                        | traces_to |
|---------------------------|-----------|-------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|-----------|
| guard_citation_evaluated  | invariant | `an invariant-kind Constraint's guard citations resolve against compiled properties and compose under the citation algebra — conjunction and negation with an explicit three-valued status — the evaluated citation is an annotation upgrade and never a sole verdict unless executed` | [[spec]]  |
| fragment_guard_rejected_stands | invariant | `a Transition's guard cell carrying a rust: fragment still fails labeled — the deferral of record (deferred to a future Revision alongside a data-carrying state space) is not met by instance-grounded expressions, so the decision of record stands unchanged`                  | [[spec]]  |
| run_report_status         | invariant | `a model-check run persists backend, bound, and per-invariant status — verified, counterexample, or unknown — unknown is persisted honestly and never coerced, and a run whose invariants were not executed remains exploration_only and explicitly not clean`                      | [[spec]]  |
| backend_agreement_ci      | invariant | `the same fixture corpus evaluates through every deployed backend with identical three-valued status per cell, asserted by a CI property test — the rust-only interim asserts rust status equals the fixture's expected status, and the cross-backend assertion activates with the py backend without being disabled` | [[spec]]  |

## Model

### States
- `citation_inert`
- `citation_evaluated`
- `agreement_active`

### Transitions

| id        | from              | to               | guard                                                                              |
|-----------|-------------------|------------------|------------------------------------------------------------------------------------|
| upgrade   | citation_inert    | citation_evaluated | `citation algebra lands per [[spec.guard_citation_evaluated]]: citations resolve and compose with three-valued status`  |
| activate  | citation_evaluated | agreement_active | `the py backend lands (l8l): the cross-backend assertion activates per [[spec.backend_agreement_ci]]`      |

## Properties

| id                        | kind | derives_from                              | generator                              | predicate                                                                  |
|---------------------------|------|-------------------------------------------|----------------------------------------|-----------------------------------------------------------------------------|
| citation_composes         | unit | [[spec.guard_citation_evaluated]]         | `artifact_with_composite_citation()`   | `citation resolves and composes with an explicit three-valued status`        |
| undischargable_is_unknown | unit | [[spec.guard_citation_evaluated]]         | `artifact_with_undischargable_citation()` | `citation reports unknown, never pass`                                   |
| guard_fragment_still_rejected | unit | [[spec.fragment_guard_rejected_stands]] | `transition_row_with_fragment_guard()` | `compile fails labeled (fragment_extraction stage), decision of record intact` |
| report_names_status       | unit | [[spec.run_report_status]]                | `run_with_mixed_invariant_status()`    | `report persists backend, bound, and per-invariant status with unknown honest` |
| fixture_status_agrees     | unit | [[spec.backend_agreement_ci]]             | `shared_fixture_corpus()`              | `every deployed backend yields identical status per cell on shared fixtures` |

## ADDED Requirements

### Requirement: Guard citation semantics evaluated
Guard citations in invariant-kind Constraints SHALL resolve against
compiled properties and compose under the citation algebra (conjunction
and negation) with an explicit three-valued status; the evaluated
citation remains an annotation upgrade and never a sole verdict unless
executed. Executable guard fragments SHALL remain rejected of record —
the `fragment_guard_rejected` deferral's condition (a data-carrying
state space) is not met by instance-grounded expressions.

#### Scenario: Citation algebra evaluates
- **WHEN** an artifact's invariant Constraint cites `[[a]] ∧ [[b]]` or a negated citation
- **THEN** the citation resolves against compiled properties and composes with an explicit three-valued status

#### Scenario: Undischargable citation is honest
- **WHEN** a citation cannot be discharged over the artifact's instances
- **THEN** it reports unknown, never pass

#### Scenario: Executable guard fragments stay rejected
- **WHEN** a Transition's guard cell carries a `rust:` fragment
- **THEN** compile fails labeled (fragment_extraction stage) — the decision of record stands unchanged

### Requirement: Backend agreement property
The same fixture corpus SHALL evaluate through every deployed backend
with identical three-valued status per cell, asserted by a CI property
test: the rust-only interim asserts rust status equals the fixture's
expected status, and the cross-backend assertion activates with the py
backend without being disabled or commented out.

#### Scenario: Rust-only interim holds
- **WHEN** the shared fixture corpus evaluates through the rust backend
- **THEN** every cell's status equals the fixture's expected status

#### Scenario: Cross-backend agreement activates with the py backend
- **WHEN** the py backend lands (l8l) and the shared fixtures evaluate through both backends
- **THEN** the CI property asserts identical status per cell across backends

### Requirement: Status-bearing run reports
A model-check run SHALL persist backend, bound, and per-invariant
status — `verified`, `counterexample`, or `unknown` — with `unknown`
persisted honestly and never coerced; a run whose invariants were not
executed remains `exploration_only` and is explicitly not clean.

#### Scenario: Report names its status
- **WHEN** a run evaluates invariants with mixed outcomes
- **THEN** the report persists backend, bound, and per-invariant status, including honest unknown

#### Scenario: Unexecuted invariants are not clean
- **WHEN** a run compiles invariants but executes none
- **THEN** the outcome is exploration_only and explicitly not clean

## Requirements

### Requirement: Guard citation semantics evaluated
Guard citations in invariant-kind Constraints SHALL resolve against
compiled properties and compose under the citation algebra (conjunction
and negation) with an explicit three-valued status; the evaluated
citation remains an annotation upgrade and never a sole verdict unless
executed. Executable guard fragments SHALL remain rejected of record —
the `fragment_guard_rejected` deferral's condition (a data-carrying
state space) is not met by instance-grounded expressions.

#### Scenario: Citation algebra evaluates
- **WHEN** an artifact's invariant Constraint cites `[[a]] ∧ [[b]]` or a negated citation
- **THEN** the citation resolves against compiled properties and composes with an explicit three-valued status

#### Scenario: Undischargable citation is honest
- **WHEN** a citation cannot be discharged over the artifact's instances
- **THEN** it reports unknown, never pass

#### Scenario: Executable guard fragments stay rejected
- **WHEN** a Transition's guard cell carries a `rust:` fragment
- **THEN** compile fails labeled (fragment_extraction stage) — the decision of record stands unchanged

### Requirement: Backend agreement property
The same fixture corpus SHALL evaluate through every deployed backend
with identical three-valued status per cell, asserted by a CI property
test: the rust-only interim asserts rust status equals the fixture's
expected status, and the cross-backend assertion activates with the py
backend without being disabled or commented out.

#### Scenario: Rust-only interim holds
- **WHEN** the shared fixture corpus evaluates through the rust backend
- **THEN** every cell's status equals the fixture's expected status

#### Scenario: Cross-backend agreement activates with the py backend
- **WHEN** the py backend lands (l8l) and the shared fixtures evaluate through both backends
- **THEN** the CI property asserts identical status per cell across backends

### Requirement: Status-bearing run reports
A model-check run SHALL persist backend, bound, and per-invariant
status — `verified`, `counterexample`, or `unknown` — with `unknown`
persisted honestly and never coerced; a run whose invariants were not
executed remains `exploration_only` and is explicitly not clean.

#### Scenario: Report names its status
- **WHEN** a run evaluates invariants with mixed outcomes
- **THEN** the report persists backend, bound, and per-invariant status, including honest unknown

#### Scenario: Unexecuted invariants are not clean
- **WHEN** a run compiles invariants but executes none
- **THEN** the outcome is exploration_only and explicitly not clean
