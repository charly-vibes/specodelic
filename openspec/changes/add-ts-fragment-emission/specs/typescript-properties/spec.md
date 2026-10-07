---
id: spec
kind: intent
statement: "THE toolchain SHALL execute TypeScript unit properties using the shared generator and evidence contracts."
---

# TypeScript properties

## Purpose
THE toolchain SHALL execute TypeScript unit properties using the shared generator and evidence contracts.

## Constraints

| id | kind | expr | traces_to |
|----|------|------|-----------|
| domains | invariant | TypeScript strategies preserve the established typed generator domains | [[spec]] |
| complete_results | invariant | installed project tools report every expected block without downloads | [[spec]] |
| bounded_shrinking | invariant | execution is bounded and counterexample shrinking is honestly attributed | [[spec]] |

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
| domains_checked | unit | [[spec.domains]] | cross_language_domain_fixtures() | bounds and Unicode scalar semantics agree |
| complete_results_checked | unit | [[spec.complete_results]] | missing_tool_or_partial_result() | missing and partial execution fail labeled |
| bounded_shrinking_checked | unit | [[spec.bounded_shrinking]] | typescript_timeout_and_failure() | timeout blocks and unshrunk examples are labeled |

## ADDED Requirements

### Requirement: TypeScript properties preserve generator domains
TypeScript unit predicates SHALL compile to deterministic fast-check test
artifacts using the shared generator IR. Integer and collection bounds,
Unicode scalar string lengths, named definitions, and explicit singleton
domains SHALL match the established Rust/Python domains. Predicate inputs
SHALL map Int/String/List(T) to number/string/Array<T> recursively, with
identical choice branch types after named expansion and no coercion.
Ill-typed choices SHALL fail before emission, even inside empty lists. Unsupported
fragment positions SHALL fail labeled rather than silently compile.

#### Scenario: Unicode and integers agree across emitters
- **WHEN** shared fixtures include non-BMP strings and signed 32-bit endpoints
- **THEN** emitted TypeScript strategies obey the same declared domains without UTF-16 length substitution

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

## Requirements

### Requirement: TypeScript properties preserve generator domains
TypeScript unit predicates SHALL compile to deterministic fast-check test
artifacts using the shared generator IR. Integer and collection bounds,
Unicode scalar string lengths, named definitions, and explicit singleton
domains SHALL match the established Rust/Python domains. Predicate inputs
SHALL map Int/String/List(T) to number/string/Array<T> recursively, with
identical choice branch types after named expansion and no coercion.
Ill-typed choices SHALL fail before emission, even inside empty lists. Unsupported
fragment positions SHALL fail labeled rather than silently compile.

#### Scenario: Unicode and integers agree across emitters
- **WHEN** shared fixtures include non-BMP strings and signed 32-bit endpoints
- **THEN** emitted TypeScript strategies obey the same declared domains without UTF-16 length substitution

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
