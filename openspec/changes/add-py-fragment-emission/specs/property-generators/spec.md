# Property generators

## Purpose
THE compiler SHALL generate typed bounded values from explicitly declared generator data.

## ADDED Requirements

### Requirement: Generator opt-in is an explicit bounded vocabulary
Generator cells SHALL opt in at fragment position with **gen:** and the
closed grammar int(lo,hi), string(min,max), list(G,min,max), one_of(G,G,...),
or named(identifier). Predicate v0 SHALL use i32, String, Vec<T> in Rust;
int, str, list[T] in Python; and number, string, Array<T> in TypeScript,
with the same recursively declared domains.

#### Scenario: Real integer inputs replace name strings
- **WHEN** a unit Property uses a marked int(-2,2) generator
- **THEN** its Rust and Python strategies yield integers in the inclusive range rather than the generator name

### Requirement: Choice typing is closed and homogeneous
A choice SHALL require identical branch types; no coercion or
heterogeneous values are allowed, including inside lists. Empty lists
SHALL retain their generator's element type. Type mismatches SHALL fail
labeled before emitting any artifacts.

#### Scenario: Choice types are checked recursively
- **WHEN** a choice mixes integer and string branches directly, through named aliases, or as list element types
- **THEN** compile fails before artifact emission with the property ID, conflicting types and a remediation hint, even when list bounds force empty lists

#### Scenario: Homogeneous choices preserve their predicate type
- **WHEN** a choice contains only integer branches or only lists of integers, including named aliases
- **THEN** its predicate receives the corresponding integer or list-of-integer type in every supported emitter

### Requirement: Generator bounds and failures are closed
Integer endpoints SHALL be inclusive signed 32-bit values; lengths SHALL
be inclusive integers from 0 through 1024; strings SHALL contain Unicode
scalar values. Invalid bounds, arity, names, or syntax SHALL fail
labeled. Unmarked legacy cells SHALL retain emitted bytes.

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
