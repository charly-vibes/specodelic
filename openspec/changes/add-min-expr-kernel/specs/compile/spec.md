---
id: spec
kind: intent
statement: "WHEN an expr cell opts into kernel translation, THE toolchain SHALL extract the expression under the closed kernel grammar per cell, SHALL fail labeled on grammar violations, SHALL compile prose expr cells byte-identically to before, and SHALL extract the kernel.binding cell as an opaque claim carrier the toolchain never interprets."
---

# compile Specification

## Purpose
Extend the compile leg's expr-cell handling with kernel opt-in: an
expr cell may carry a kernel expression under the closed grammar,
grammar violations fail labeled (sd1 discipline), prose expr cells
compile byte-identically (pure widening — nothing valid before this
Revision is invalidated), and the `kernel.binding` cell extracts as an
opaque claim carrier so external checkers can claim constraints through
contract-TOML `flags` without specodelic learning their language.

## Constraints

| id                       | kind      | expr                                                                                                                                                                                                                                                              | traces_to |
|--------------------------|-----------|---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|-----------|
| kernel_opt_in_extracted  | invariant | `an expr cell may carry a kernel expression in fragment position per cell — the fragment-position rule (specodelic-sd1) carries over: an occurrence outside fragment position is a mention and never extracts`                                                          | [[spec]]  |
| kernel_grammar_violation_labeled | invariant | `an expr cell whose kernel expression uses a non-member atomic is a labeled extraction failure naming the atomic and the closed set — never treated as prose, never extracted, never silently ignored`                                                               | [[spec]]  |
| prose_back_compat        | invariant | `every expr cell that compiled before this Revision compiles to the identical artifact after it — the widening is pure: nothing valid before this Revision is invalidated, and a cell without kernel content compiles exactly as before`                               | [[spec]]  |
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
| prose_unchanged           | unit | [[spec.prose_back_compat]]               | `pre_revision_prose_fixtures()`        | `compiled artifact identical to the pre-Revision artifact, byte-for-byte`   |
| midspan_mention_ignored   | unit | [[spec.kernel_opt_in_extracted]]         | `cell_with_midspan_kernel_marker()`    | `no expression extracted; cell compiles exactly as before`                  |
| binding_extracts_verbatim | unit | [[spec.kernel_binding_extracted]]        | `constraint_with_arbitrary_binding_text()` | `binding cell contents extracted and surfaced verbatim, never interpreted` |

## ADDED Requirements

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
The compile change SHALL invalidate nothing that was valid before it:
every expr cell that compiled before this Revision compiles to the
identical artifact after it, and cells without kernel content compile
exactly as before.

#### Scenario: Prose expr cell untouched
- **WHEN** an expr cell carries no kernel content (the pre-Revision path)
- **THEN** its compiled artifact is byte-identical to the pre-Revision artifact

#### Scenario: Nothing valid is invalidated
- **WHEN** any pre-Revision artifact is recompiled under the widened grammar
- **THEN** the artifact is identical to its pre-Revision compilation

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
The compile change SHALL invalidate nothing that was valid before it:
every expr cell that compiled before this Revision compiles to the
identical artifact after it, and cells without kernel content compile
exactly as before.

#### Scenario: Prose expr cell untouched
- **WHEN** an expr cell carries no kernel content (the pre-Revision path)
- **THEN** its compiled artifact is byte-identical to the pre-Revision artifact

#### Scenario: Nothing valid is invalidated
- **WHEN** any pre-Revision artifact is recompiled under the widened grammar
- **THEN** the artifact is identical to its pre-Revision compilation

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
