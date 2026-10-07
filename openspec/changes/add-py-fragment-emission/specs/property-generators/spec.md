---
id: spec
kind: intent
statement: "THE compiler SHALL generate typed bounded values from explicitly declared generator data."
---

# Property generators

## Purpose
THE compiler SHALL generate typed bounded values from explicitly declared generator data.

## Constraints

| id | kind | expr | traces_to |
|----|------|------|-----------|
| typed_values | invariant | explicit generator grammar maps to the same value domains in each emitter | [[spec]] |
| registry_freshness | invariant | named definitions are explicit data and their changes invalidate evidence | [[spec]] |
| adequate_inputs | invariant | legacy constant placeholders cannot satisfy the property gate | [[spec]] |

## Model

### States
- proposed
- approved
- implemented
- deployed

### Transitions

| id | from | to | guard |
|----|------|----|-------|
| approve | proposed | approved | maintainer approves the proposal |
| implement | approved | implemented | every specified behavior has passing evidence |
| deploy | implemented | deployed | validation passes and the dual-format archive preserves all requirements |

## Properties

| id | kind | derives_from | generator | predicate |
|----|------|--------------|-----------|-----------|
| typed_values_checked | unit | [[spec.typed_values]] | generator_domain_fixtures() | integer and collection boundaries agree across Rust and Python |
| registry_freshness_checked | unit | [[spec.registry_freshness]] | registry_missing_cycle_or_edit() | bad definitions fail and edited definitions make reports stale |
| adequate_inputs_checked | unit | [[spec.adequate_inputs]] | legacy_singleton_and_parameterless_cases() | only explicit singleton or parameterless inputs can pass |

## ADDED Requirements

### Requirement: Explicit bounded generator vocabulary
Generator cells SHALL opt in at fragment position with **gen:** and the
closed grammar int(lo,hi), string(min,max), list(G,min,max), one_of(G,G,...),
or named(identifier). After named expansion, types SHALL be Int, String,
or recursively List(T). A choice SHALL require identical branch types;
no coercion or heterogeneous values are allowed, including inside lists.
Empty lists SHALL retain their generator's element type. Type mismatches
SHALL fail labeled before emitting any artifacts. Predicate v0 SHALL use
i32, String, Vec<T> in Rust; int, str, list[T] in Python; and number, string,
Array<T> in TypeScript, with the same recursively declared domains.
Integer endpoints SHALL be inclusive signed 32-bit
values; lengths SHALL be inclusive integers from 0 through 1024; strings
SHALL contain Unicode scalar values. Invalid bounds, arity, names, or syntax
SHALL fail labeled. Unmarked legacy cells SHALL retain emitted bytes.

#### Scenario: Real integer inputs replace name strings
- **WHEN** a unit Property uses a marked int(-2,2) generator
- **THEN** its Rust and Python strategies yield integers in the inclusive range rather than the generator name

#### Scenario: Choice types are checked recursively
- **WHEN** a choice mixes integer and string branches directly, through named aliases, or as list element types
- **THEN** compile fails before artifact emission with the property ID, conflicting types and a remediation hint, even when list bounds force empty lists

#### Scenario: Homogeneous choices preserve their predicate type
- **WHEN** a choice contains only integer branches or only lists of integers, including named aliases
- **THEN** its predicate receives the corresponding integer or list-of-integer type in every supported emitter

#### Scenario: Malformed opt-in fails before emission
- **WHEN** a marked generator has reversed bounds or unknown syntax
- **THEN** compile fails with a generator label and remediation hint without reporting a compiled artifact set

### Requirement: Named definitions are explicit and fresh
Named generators SHALL resolve only through [generators] data definitions
in .specodelic/generators.toml at the selected project root. Missing names,
cycles, or expansions beyond 32 levels SHALL fail labeled. Consumed
registry definitions SHALL participate in artifact and verification freshness.

#### Scenario: Registry changes invalidate generated evidence
- **WHEN** a named generator definition changes after compilation
- **THEN** verify refuses the old generated evidence until recompilation and rerun

### Requirement: Placeholder sampling cannot satisfy the property gate
Generated property execution SHALL distinguish legacy constant-name
placeholders from explicit typed generators and parameterless unit tests.
A legacy placeholder SHALL block properties_pass with migration guidance;
an intentional singleton domain SHALL remain an adequate declared domain.
No successful run SHALL be described as exhaustive input coverage.

#### Scenario: Placeholder and singleton are different
- **WHEN** a passing predicate is run with a legacy word() placeholder and separately with marked int(1,1)
- **THEN** the placeholder run is blocked as inadequate and the explicit singleton may pass

#### Scenario: Parameterless unit tests remain usable
- **WHEN** an executable unit predicate has no generator inputs
- **THEN** its executed result contributes normally to the property gate

## Requirements

### Requirement: Explicit bounded generator vocabulary
Generator cells SHALL opt in at fragment position with **gen:** and the
closed grammar int(lo,hi), string(min,max), list(G,min,max), one_of(G,G,...),
or named(identifier). After named expansion, types SHALL be Int, String,
or recursively List(T). A choice SHALL require identical branch types;
no coercion or heterogeneous values are allowed, including inside lists.
Empty lists SHALL retain their generator's element type. Type mismatches
SHALL fail labeled before emitting any artifacts. Predicate v0 SHALL use
i32, String, Vec<T> in Rust; int, str, list[T] in Python; and number, string,
Array<T> in TypeScript, with the same recursively declared domains.
Integer endpoints SHALL be inclusive signed 32-bit
values; lengths SHALL be inclusive integers from 0 through 1024; strings
SHALL contain Unicode scalar values. Invalid bounds, arity, names, or syntax
SHALL fail labeled. Unmarked legacy cells SHALL retain emitted bytes.

#### Scenario: Real integer inputs replace name strings
- **WHEN** a unit Property uses a marked int(-2,2) generator
- **THEN** its Rust and Python strategies yield integers in the inclusive range rather than the generator name

#### Scenario: Choice types are checked recursively
- **WHEN** a choice mixes integer and string branches directly, through named aliases, or as list element types
- **THEN** compile fails before artifact emission with the property ID, conflicting types and a remediation hint, even when list bounds force empty lists

#### Scenario: Homogeneous choices preserve their predicate type
- **WHEN** a choice contains only integer branches or only lists of integers, including named aliases
- **THEN** its predicate receives the corresponding integer or list-of-integer type in every supported emitter

#### Scenario: Malformed opt-in fails before emission
- **WHEN** a marked generator has reversed bounds or unknown syntax
- **THEN** compile fails with a generator label and remediation hint without reporting a compiled artifact set

### Requirement: Named definitions are explicit and fresh
Named generators SHALL resolve only through [generators] data definitions
in .specodelic/generators.toml at the selected project root. Missing names,
cycles, or expansions beyond 32 levels SHALL fail labeled. Consumed
registry definitions SHALL participate in artifact and verification freshness.

#### Scenario: Registry changes invalidate generated evidence
- **WHEN** a named generator definition changes after compilation
- **THEN** verify refuses the old generated evidence until recompilation and rerun

### Requirement: Placeholder sampling cannot satisfy the property gate
Generated property execution SHALL distinguish legacy constant-name
placeholders from explicit typed generators and parameterless unit tests.
A legacy placeholder SHALL block properties_pass with migration guidance;
an intentional singleton domain SHALL remain an adequate declared domain.
No successful run SHALL be described as exhaustive input coverage.

#### Scenario: Placeholder and singleton are different
- **WHEN** a passing predicate is run with a legacy word() placeholder and separately with marked int(1,1)
- **THEN** the placeholder run is blocked as inadequate and the explicit singleton may pass

#### Scenario: Parameterless unit tests remain usable
- **WHEN** an executable unit predicate has no generator inputs
- **THEN** its executed result contributes normally to the property gate
