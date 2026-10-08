---
id: compile
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
| process_lifecycle | invariant | `the capability advances through its declared lifecycle states under the repo's change process — each stage transition fires only when its stage gate holds` | [[compile]] |
| kernel_opt_in_extracted  | invariant | `an expr cell may carry a kernel expression in fragment position per cell — the fragment-position rule (specodelic-sd1) carries over: an occurrence outside fragment position is a mention and never extracts`                                                          | [[compile]]  |
| kernel_grammar_violation_labeled | invariant | `an expr cell whose kernel expression uses a non-member atomic is a labeled extraction failure naming the atomic and the closed set — never treated as prose, never extracted, never silently ignored`                                                               | [[compile]]  |
| prose_back_compat        | invariant | `every expr cell that compiled before this Revision compiles to the identical artifact after it — the widening is pure: nothing valid before this Revision is invalidated, and a cell without kernel content compiles exactly as before`                               | [[compile]]  |
| kernel_binding_extracted | invariant | `an invariant-kind Constraint's kernel.binding cell extracts as an opaque string, surfaced verbatim and never interpreted; the claim path for external checkers is contract-TOML flags, not a toolchain registry`                                                     | [[compile]]  |

## Model

### States
- `prose_expr`
- `kernel_expr`
- `claimed`

### Transitions

| id        | from        | to         | guard                                                                           |
|-----------|-------------|------------|---------------------------------------------------------------------------------|
| opt_in    | prose_expr  | kernel_expr | `an expr cell carries a kernel expression per [[compile.kernel_opt_in_extracted]]`  |
| reject    | prose_expr  | prose_expr | `a non-member atomic fails labeled per [[compile.kernel_grammar_violation_labeled]], cell unchanged`  |
| claim     | kernel_expr | claimed    | `an external checker claims the constraint per [[compile.kernel_binding_extracted]]`  |

## Properties

| id                        | kind | derives_from                             | generator                              | predicate                                                                  |
|---------------------------|------|------------------------------------------|----------------------------------------|----------------------------------------------------------------------------|
| process_lifecycle_checked | unit | [[compile.process_lifecycle]] | `lifecycle_model_present()` | `check(file) == passed` |
| kernel_expr_extracts      | unit | [[compile.kernel_opt_in_extracted]]         | `expr_cell_with_kernel_expression()`   | `expression extracted under the closed grammar with position rule intact`   |
| nonmember_atomic_rejects  | unit | [[compile.kernel_grammar_violation_labeled]] | `expr_cell_with_nonmember_atomic()`   | `extraction fails labeled, naming the atomic and the closed set`            |
| prose_unchanged           | unit | [[compile.prose_back_compat]]               | `pre_revision_prose_fixtures()`        | `compiled artifact identical to the pre-Revision artifact, byte-for-byte`   |
| midspan_mention_ignored   | unit | [[compile.kernel_opt_in_extracted]]         | `cell_with_midspan_kernel_marker()`    | `no expression extracted; cell compiles exactly as before`                  |
| binding_extracts_verbatim | unit | [[compile.kernel_binding_extracted]]        | `constraint_with_arbitrary_binding_text()` | `binding cell contents extracted and surfaced verbatim, never interpreted` |

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

#### Scenario: Kernel mid-span occurrence is a mention
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

The grammar change SHALL invalidate nothing that was valid before it:
every cell that extracted under the pre-change `**rust:**` grammar
extracts identically after, and cells without tags compile exactly as
before.

#### Scenario: No-tag cell untouched
- **WHEN** a Property row's predicate carries no tag (the `todo_predicate!` placeholder path)
- **THEN** its compiled artifact is byte-identical to the pre-change artifact

#### Scenario: Invariant expr cells share the grammar
- **WHEN** an invariant-kind Constraint's expr cell carries a `**py:**` fragment
- **THEN** the same tag grammar and failure modes apply as for predicate cells

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
SHALL fail labeled — with a remediation hint naming the missing emitter
and its follow-up change — when a `py` or `ts` fragment reaches compile.

#### Scenario: Rust opt-in unchanged
- **WHEN** a predicate cell carries a `**rust:**` fragment under the pre-change grammar
- **THEN** the extracted fragment is byte-identical to the pre-change extraction

#### Scenario: Unknown tag rejected labeled
- **WHEN** a predicate cell carries `**go:**` (or any tag outside the closed set)
- **THEN** extraction fails labeled, naming the tag and the closed set — the cell is never prose and never silently ignored

#### Scenario: Tag without emitter gates compile honestly
- **WHEN** a fragment tagged `**py:**` reaches compile before the py emitter lands
- **THEN** compile fails labeled and the remediation hint names the py-emitter follow-up

#### Scenario: Mid-span occurrence is a mention
- **WHEN** a cell mentions `**py:**` mid-span (as the defining rows of this very table do)
- **THEN** nothing extracts and the cell compiles exactly as before

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

#### Scenario: Kernel mid-span occurrence is a mention
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

The grammar change SHALL invalidate nothing that was valid before it:
every cell that extracted under the pre-change `**rust:**` grammar
extracts identically after, and cells without tags compile exactly as
before.

#### Scenario: No-tag cell untouched
- **WHEN** a Property row's predicate carries no tag (the `todo_predicate!` placeholder path)
- **THEN** its compiled artifact is byte-identical to the pre-change artifact

#### Scenario: Invariant expr cells share the grammar
- **WHEN** an invariant-kind Constraint's expr cell carries a `**py:**` fragment
- **THEN** the same tag grammar and failure modes apply as for predicate cells

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
SHALL fail labeled — with a remediation hint naming the missing emitter
and its follow-up change — when a `py` or `ts` fragment reaches compile.

#### Scenario: Rust opt-in unchanged
- **WHEN** a predicate cell carries a `**rust:**` fragment under the pre-change grammar
- **THEN** the extracted fragment is byte-identical to the pre-change extraction

#### Scenario: Unknown tag rejected labeled
- **WHEN** a predicate cell carries `**go:**` (or any tag outside the closed set)
- **THEN** extraction fails labeled, naming the tag and the closed set — the cell is never prose and never silently ignored

#### Scenario: Tag without emitter gates compile honestly
- **WHEN** a fragment tagged `**py:**` reaches compile before the py emitter lands
- **THEN** compile fails labeled and the remediation hint names the py-emitter follow-up

#### Scenario: Mid-span occurrence is a mention
- **WHEN** a cell mentions `**py:**` mid-span (as the defining rows of this very table do)
- **THEN** nothing extracts and the cell compiles exactly as before

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
