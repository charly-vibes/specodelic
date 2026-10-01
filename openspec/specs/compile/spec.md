---
id: spec
kind: intent
statement: "WHEN a law-kind property row is checked or compiled, THE toolchain SHALL read the row's required cases only from machine-findable `**name:**` labels in the row's own predicate, with the identity and associativity floor mandatory and every extra named case a first-class declaration."
---

# compile Specification

## Purpose
Make law-row named cases machine-checkable instead of free predicate
prose: the case labels the compile gate already parses become the
format's only case enumeration, the `law_requires_cases` floor is
lint-enforced before compile, and extra named cases compile as
first-class checkable blocks.

## Constraints

| id                     | kind      | expr                                                                                                                                                                  | traces_to |
|------------------------|-----------|-----------------------------------------------------------------------------------------------------------------------------------------------------------------------|-----------|
| cases_machine_form     | invariant | `every law-kind property row enumerates its required cases as **name:** labels in the row's own predicate — a prose mention of a case name is not an enumeration`     | [[spec]]  |
| floor_mandatory        | invariant | `every law-kind property row's label set includes identity and associativity — the law_requires_cases floor, a minimum rather than a ceiling`                          | [[spec]]  |
| extra_cases_first_class| invariant | `every case label beyond the floor compiles to its own proptest block — a checkable declaration carrying the case's own assertion, never prose`                        | [[spec]]  |
| unlabeled_rejected     | invariant | `the linter rejects a law-kind row whose predicate lacks the floor labels, naming each missing case in the finding`                                                    | [[spec]]  |
| compile_reads_labels   | invariant | `compile expands a law row to exactly one block per label-enumerated case; the unlabeled fallback to the floor survives only as defense-in-depth for input that skipped the compile precondition` | [[spec]]  |

## Model

### States

- `prose_only`
- `labeled`
- `enforced`

### Transitions

| id      | from       | to       | guard                                                              |
|---------|------------|----------|--------------------------------------------------------------------|
| ratify  | prose_only | labeled  | [[spec.cases_machine_form]]                                        |
| gate    | labeled    | enforced | [[spec.floor_mandatory]] ∧ [[spec.unlabeled_rejected]]             |
| compile | labeled    | enforced | [[spec.compile_reads_labels]] ∧ [[spec.extra_cases_first_class]]   |

## Properties

| id                   | kind | derives_from                | generator                                        | predicate                                                              |
|----------------------|------|-----------------------------|--------------------------------------------------|-------------------------------------------------------------------------|
| floor_missing_fails  | unit | [[spec.unlabeled_rejected]] | `law_property_missing("identity")`               | `check(file) == failed` — the finding names the missing floor case      |
| prose_mention_fails  | unit | [[spec.cases_machine_form]] | `law_predicate_mentioning_cases_without_labels()`| `check(file) == failed` — the exact shape free prose used to slip through |
| extra_case_compiles  | unit | [[spec.extra_cases_first_class]] | `law_property_with_extra_case("commutativity")` | `compile(file) emits one block per enumerated case`                    |
| fallback_never_masks | unit | [[spec.compile_reads_labels]] | `law row compiled without the lint precondition` | `blocks fall back to the floor — never a silent pass on unlabeled input` |
| floor_is_minimum     | unit | [[spec.floor_mandatory]]    | `law_property_with_floor_and_extra_case("commutativity")` | `check(file) == passed` — the floor never caps the case list      |

## MODIFIED Requirements

### Requirement: Properties table compiles to proptest blocks
The system SHALL compile each Property row to at least one `proptest!`
block — generator as input strategy, predicate as assertion body — and
a law-kind property to exactly one block per case enumerated in the
row's predicate in machine-findable `**name:**` label form. The
identity and associativity floor (`law_requires_cases`) is mandatory
and lint-enforced before compile (the compile precondition gate);
extra named cases compile as first-class checkable blocks, never prose.

#### Scenario: Law expands to enumerated cases
- **WHEN** a law-kind property row's predicate carries `**identity:**`,
  `**associativity:**`, and `**commutativity:**` labels
- **THEN** exactly three blocks are emitted for that row, one per
  enumerated case

#### Scenario: Unlabeled floor fails the precondition
- **WHEN** a law-kind property row's predicate states its cases in
  prose only (no `**name:**` labels)
- **THEN** the compile precondition gate reports the `linter.law_cases`
  finding naming the missing floor cases
- **AND** no artifact set is reported as compiled

## Requirements

### Requirement: Properties table compiles to proptest blocks
The system SHALL compile each Property row to at least one `proptest!`
block — generator as input strategy, predicate as assertion body — and
a law-kind property to exactly one block per case enumerated in the
row's predicate in machine-findable `**name:**` label form. The
identity and associativity floor (`law_requires_cases`) is mandatory
and lint-enforced before compile (the compile precondition gate);
extra named cases compile as first-class checkable blocks, never prose.

#### Scenario: Law expands to enumerated cases
- **WHEN** a law-kind property row's predicate carries `**identity:**`,
  `**associativity:**`, and `**commutativity:**` labels
- **THEN** exactly three blocks are emitted for that row, one per
  enumerated case

#### Scenario: Unlabeled floor fails the precondition
- **WHEN** a law-kind property row's predicate states its cases in
  prose only (no `**name:**` labels)
- **THEN** the compile precondition gate reports the `linter.law_cases`
  finding naming the missing floor cases
- **AND** no artifact set is reported as compiled