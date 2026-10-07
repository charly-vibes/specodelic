---
id: spec
kind: intent
statement: "WHEN an expr cell opts into kernel translation, THE toolchain SHALL extract the expression under the closed kernel grammar per cell, SHALL fail labeled on grammar violations, SHALL compile prose expr cells byte-identically to before, and SHALL extract the kernel.binding cell as an opaque claim carrier the toolchain never interprets."
---

# Compile specification after TypeScript emitter

## Purpose
This full snapshot applies only after add-py-fragment-emission is deployed.
Python and TypeScript support is limited to unit Property predicates; invariant and law
fragment restrictions remain. See python-properties and property-generators.

Extend the compile leg's expr-cell handling with kernel opt-in: an
expr cell may carry a kernel expression under the closed grammar,
grammar violations fail labeled (sd1 discipline), prose expr cells
retain their extraction and legacy artifacts remain byte-identical when
generators are unmarked. Explicit generator opt-ins use the new typed
semantics. The `kernel.binding` cell extracts as an
opaque claim carrier so external checkers can claim constraints through
contract-TOML `flags` without specodelic learning their language.

## Constraints

| id                       | kind      | expr                                                                                                                                                                                                                                                              | traces_to |
|--------------------------|-----------|---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|-----------|
| kernel_opt_in_extracted  | invariant | `an expr cell may carry a kernel expression in fragment position per cell — the fragment-position rule (specodelic-sd1) carries over: an occurrence outside fragment position is a mention and never extracts`                                                          | [[spec]]  |
| kernel_grammar_violation_labeled | invariant | `an expr cell whose kernel expression uses a non-member atomic is a labeled extraction failure naming the atomic and the closed set — never treated as prose, never extracted, never silently ignored`                                                               | [[spec]]  |
| prose_back_compat        | invariant | `unmarked legacy generator cells retain emitted bytes; unchanged Rust and prose cells retain extraction; explicit generator opt-ins use typed semantics and are excluded from artifact byte preservation`                               | [[spec]]  |
| kernel_binding_extracted | invariant | `an invariant-kind Constraint's kernel.binding cell extracts as an opaque string, surfaced verbatim and never interpreted; the claim path for external checkers is contract-TOML flags, not a toolchain registry`                                                     | [[spec]]  |

## Model

### States
- `prose_expr`
- `kernel_expr`
- `claimed`

### Transitions

| id        | from        | to         | guard                                                                           |
|-----------|-------------|------------|---------------------------------------------------------------------------------|
| opt_in    | prose_expr  | kernel_expr | `an expr cell carries a kernel expression per [[spec.kernel_opt_in_extracted]]`             |
| reject    | prose_expr  | prose_expr | `a non-member atomic fails labeled per [[spec.kernel_grammar_violation_labeled]], cell unchanged`      |
| claim     | kernel_expr | claimed    | `an external checker claims the constraint per [[spec.kernel_binding_extracted]]`          |

## Properties

| id                        | kind | derives_from                             | generator                              | predicate                                                                  |
|---------------------------|------|------------------------------------------|----------------------------------------|----------------------------------------------------------------------------|
| kernel_expr_extracts      | unit | [[spec.kernel_opt_in_extracted]]         | `expr_cell_with_kernel_expression()`   | `expression extracted under the closed grammar with position rule intact`   |
| nonmember_atomic_rejects  | unit | [[spec.kernel_grammar_violation_labeled]] | `expr_cell_with_nonmember_atomic()`   | `extraction fails labeled, naming the atomic and the closed set`            |
| prose_unchanged           | unit | [[spec.prose_back_compat]]               | `pre_revision_unmarked_generator_fixtures()` | `legacy artifact byte-identical; marked generator migration tested separately`   |
| midspan_mention_ignored   | unit | [[spec.kernel_opt_in_extracted]]         | `cell_with_midspan_kernel_marker()`    | `no expression extracted; cell compiles exactly as before`                  |
| binding_extracts_verbatim | unit | [[spec.kernel_binding_extracted]]        | `constraint_with_arbitrary_binding_text()` | `binding cell contents extracted and surfaced verbatim, never interpreted` |

## MODIFIED Requirements

### Requirement: Kernel expr opt-in
An expr cell SHALL accept a kernel expression in fragment position per
cell under the closed kernel grammar; the fragment-position rule
carries over (occurrences outside fragment position are mentions and
never extract); an expression using a non-member atomic SHALL fail
labeled, naming the atomic and the closed set.

#### Scenario: Opt-in extracts under the closed grammar
- **WHEN** an expr cell carries a kernel expression in fragment position
- **THEN** the expression extracts under the closed grammar

#### Scenario: Non-member atomic rejected labeled
- **WHEN** an expr cell's expression uses an atomic outside the closed set
- **THEN** extraction fails labeled, naming the atomic and the closed set — the cell is never prose and never silently ignored

#### Scenario: Mid-span occurrence is a mention
- **WHEN** a cell mentions kernel syntax mid-span (as the defining rows of this very table do)
- **THEN** nothing extracts and the cell compiles exactly as before

### Requirement: Pure widening
Legacy artifact preservation SHALL apply to unchanged inputs whose generator
cells are unmarked. Unchanged Rust fragments and prose cells SHALL retain
extraction behavior. Explicit **gen:** cells SHALL use the typed generator
contract, even if an older compiler accepted those cells as placeholders;
they are excluded from artifact byte preservation. Malformed or ill-typed
marked generators SHALL fail labeled. Legacy verification acceptance is
governed separately by property-generators.

#### Scenario: Prose expr cell untouched
- **WHEN** an unchanged input has prose expr cells and unmarked legacy generators
- **THEN** its compiled legacy artifacts are byte-identical to their pre-change artifacts

#### Scenario: Nothing valid is invalidated
- **WHEN** a pre-change legacy input with unmarked generators is recompiled without source changes
- **THEN** its legacy artifacts remain byte-identical; this guarantee excludes explicit generator opt-ins

#### Scenario: Marked generator replaces the former placeholder
- **WHEN** a previously accepted generator cell carries **gen:** int(0,1)
- **THEN** compile emits integer strategy values 0 or 1 rather than the former constant name string, and migration guidance requires predicates to use integers and evidence to be regenerated

#### Scenario: No-tag cell untouched
- **WHEN** a Property row's predicate carries no tag and its generator is unmarked (the legacy placeholder path)
- **THEN** its compiled artifact is byte-identical to the pre-change artifact

#### Scenario: Invariant expr cells share the grammar
- **WHEN** an invariant-kind Constraint's expr cell carries a `**py:**` fragment
- **THEN** the same closed tag grammar applies, but compile rejects invariant Python execution as an unsupported position

### Requirement: Binding column extraction
An invariant-kind Constraint's `kernel.binding` cell SHALL extract as
an opaque string surfaced verbatim and never interpreted by the
toolchain; the claim path for external checkers is contract-TOML
`flags`, not a toolchain registry.

#### Scenario: Binding extracts verbatim
- **WHEN** a constraint carries an arbitrary `kernel.binding` text
- **THEN** the contents extract and surface verbatim; no interpretation, no validation of their internals

#### Scenario: Claim path is contract flags
- **WHEN** an external checker claims a claimed constraint
- **THEN** the claim is carried by contract-TOML `flags` binding and the toolchain builds no registry

### Requirement: Closed language-tag fragment opt-in
The fragment opt-in grammar SHALL accept exactly one tag from the closed
set `{rust, py, ts}` in fragment position, SHALL reject an unknown tag
with a labeled extraction failure naming the tag and the closed set, and
SHALL emit supported Python and TypeScript unit Property predicates through
their language emitters while preserving Rust fragment extraction; Rust artifact byte
preservation follows the Pure widening requirement. A fragment in an
unsupported position SHALL fail labeled with remediation.

#### Scenario: Rust opt-in unchanged
- **WHEN** a predicate cell carries a `**rust:**` fragment under the pre-change grammar
- **THEN** the extracted fragment is byte-identical to the pre-change extraction

#### Scenario: Unknown tag rejected labeled
- **WHEN** a predicate cell carries `**go:**` (or any tag outside the closed set)
- **THEN** extraction fails labeled, naming the tag and the closed set — the cell is never prose and never silently ignored

#### Scenario: Tag without emitter gates compile honestly
- **WHEN** a TypeScript fragment is used in an unsupported invariant or law position
- **THEN** compile fails labeled and names the unsupported position rather than implying execution

#### Scenario: Mid-span occurrence is a mention
- **WHEN** a cell mentions `**py:**` mid-span (as the defining rows of this very table do)
- **THEN** nothing extracts and the cell compiles exactly as before

### Requirement: Properties table compiles to proptest blocks
The system SHALL compile each supported Property row to at least one
language-appropriate test block (Rust proptest, Python Hypothesis/pytest, or TypeScript fast-check)
with its declared generator and predicate, and
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

### Requirement: Kernel expr opt-in
An expr cell SHALL accept a kernel expression in fragment position per
cell under the closed kernel grammar; the fragment-position rule
carries over (occurrences outside fragment position are mentions and
never extract); an expression using a non-member atomic SHALL fail
labeled, naming the atomic and the closed set.

#### Scenario: Opt-in extracts under the closed grammar
- **WHEN** an expr cell carries a kernel expression in fragment position
- **THEN** the expression extracts under the closed grammar

#### Scenario: Non-member atomic rejected labeled
- **WHEN** an expr cell's expression uses an atomic outside the closed set
- **THEN** extraction fails labeled, naming the atomic and the closed set — the cell is never prose and never silently ignored

#### Scenario: Mid-span occurrence is a mention
- **WHEN** a cell mentions kernel syntax mid-span (as the defining rows of this very table do)
- **THEN** nothing extracts and the cell compiles exactly as before

### Requirement: Pure widening
Legacy artifact preservation SHALL apply to unchanged inputs whose generator
cells are unmarked. Unchanged Rust fragments and prose cells SHALL retain
extraction behavior. Explicit **gen:** cells SHALL use the typed generator
contract, even if an older compiler accepted those cells as placeholders;
they are excluded from artifact byte preservation. Malformed or ill-typed
marked generators SHALL fail labeled. Legacy verification acceptance is
governed separately by property-generators.

#### Scenario: Prose expr cell untouched
- **WHEN** an unchanged input has prose expr cells and unmarked legacy generators
- **THEN** its compiled legacy artifacts are byte-identical to their pre-change artifacts

#### Scenario: Nothing valid is invalidated
- **WHEN** a pre-change legacy input with unmarked generators is recompiled without source changes
- **THEN** its legacy artifacts remain byte-identical; this guarantee excludes explicit generator opt-ins

#### Scenario: Marked generator replaces the former placeholder
- **WHEN** a previously accepted generator cell carries **gen:** int(0,1)
- **THEN** compile emits integer strategy values 0 or 1 rather than the former constant name string, and migration guidance requires predicates to use integers and evidence to be regenerated

#### Scenario: No-tag cell untouched
- **WHEN** a Property row's predicate carries no tag and its generator is unmarked (the legacy placeholder path)
- **THEN** its compiled artifact is byte-identical to the pre-change artifact

#### Scenario: Invariant expr cells share the grammar
- **WHEN** an invariant-kind Constraint's expr cell carries a `**py:**` fragment
- **THEN** the same closed tag grammar applies, but compile rejects invariant Python execution as an unsupported position

### Requirement: Binding column extraction
An invariant-kind Constraint's `kernel.binding` cell SHALL extract as
an opaque string surfaced verbatim and never interpreted by the
toolchain; the claim path for external checkers is contract-TOML
`flags`, not a toolchain registry.

#### Scenario: Binding extracts verbatim
- **WHEN** a constraint carries an arbitrary `kernel.binding` text
- **THEN** the contents extract and surface verbatim; no interpretation, no validation of their internals

#### Scenario: Claim path is contract flags
- **WHEN** an external checker claims a claimed constraint
- **THEN** the claim is carried by contract-TOML `flags` binding and the toolchain builds no registry

### Requirement: Closed language-tag fragment opt-in
The fragment opt-in grammar SHALL accept exactly one tag from the closed
set `{rust, py, ts}` in fragment position, SHALL reject an unknown tag
with a labeled extraction failure naming the tag and the closed set, and
SHALL emit supported Python and TypeScript unit Property predicates through
their language emitters while preserving Rust fragment extraction; Rust artifact byte
preservation follows the Pure widening requirement. A fragment in an
unsupported position SHALL fail labeled with remediation.

#### Scenario: Rust opt-in unchanged
- **WHEN** a predicate cell carries a `**rust:**` fragment under the pre-change grammar
- **THEN** the extracted fragment is byte-identical to the pre-change extraction

#### Scenario: Unknown tag rejected labeled
- **WHEN** a predicate cell carries `**go:**` (or any tag outside the closed set)
- **THEN** extraction fails labeled, naming the tag and the closed set — the cell is never prose and never silently ignored

#### Scenario: Tag without emitter gates compile honestly
- **WHEN** a TypeScript fragment is used in an unsupported invariant or law position
- **THEN** compile fails labeled and names the unsupported position rather than implying execution

#### Scenario: Mid-span occurrence is a mention
- **WHEN** a cell mentions `**py:**` mid-span (as the defining rows of this very table do)
- **THEN** nothing extracts and the cell compiles exactly as before

### Requirement: Properties table compiles to proptest blocks
The system SHALL compile each supported Property row to at least one
language-appropriate test block (Rust proptest, Python Hypothesis/pytest, or TypeScript fast-check)
with its declared generator and predicate, and
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
