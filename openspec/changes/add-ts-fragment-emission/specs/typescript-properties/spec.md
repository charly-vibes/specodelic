# TypeScript properties

## Purpose
THE toolchain SHALL execute TypeScript unit properties using the shared generator and evidence contracts.

## ADDED Requirements

### Requirement: TypeScript properties preserve generator domains
TypeScript unit predicates SHALL compile to deterministic fast-check test
artifacts using the shared generator IR. Integer and collection bounds,
Unicode scalar string lengths, named definitions, and explicit singleton
domains SHALL match the established Rust/Python domains. Predicate inputs
SHALL map Int/String/List(T) to number/string/Array<T> recursively, with
no coercion.

#### Scenario: Unicode and integers agree across emitters
- **WHEN** shared fixtures include non-BMP strings and signed 32-bit endpoints
- **THEN** emitted TypeScript strategies obey the same declared domains without UTF-16 length substitution

### Requirement: TypeScript choice typing and unsupported positions fail closed
Choice branch types SHALL be identical after named expansion. Ill-typed
choices SHALL fail before emission, even inside empty lists. Unsupported
fragment positions SHALL fail labeled rather than silently compile.

#### Scenario: Choice typing stays shared
- **WHEN** the shared conformance fixtures contain homogeneous choices and mixed choices hidden in aliases or nested lists
- **THEN** TypeScript accepts the homogeneous cases with their declared types and rejects the mixed cases before emission just as Rust and Python do

### Requirement: Project tooling runs every expected block
The adapter SHALL use project-local installed TypeScript test dependencies,
SHALL not download missing packages, and SHALL provide structured results
for every expected block. A missing, skipped, failed, stale, or inadequate
block SHALL prevent properties_pass even when other languages pass.

#### Scenario: Missing tooling does not trigger installation
- **WHEN** the selected project lacks Vitest or fast-check
- **THEN** verify fails with a dependency hint and does not fetch packages

#### Scenario: Mixed-language success requires every block
- **WHEN** Rust and Python pass but a TypeScript property fails
- **THEN** verify exits 1 and identifies the failed TypeScript property

### Requirement: Shrinking and execution bounds are explicit
The TypeScript runner SHALL enforce its wall-clock budget and terminate its
process tree on timeout. Failed properties SHALL report native fast-check
counterexample metadata and SHALL label unfinished shrinking as unshrunk.

#### Scenario: A timeout cannot masquerade as a property result
- **WHEN** a generated TypeScript test exceeds its time budget
- **THEN** verify reports timeout, terminates descendant execution, and does not report verified
