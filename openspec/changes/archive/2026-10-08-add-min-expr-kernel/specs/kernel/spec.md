# kernel Specification

## Purpose
Give the corpus's data-dependent cells — the equational and
bounded-quantified ones that are prose-only today — defined, decidable
semantics: a closed kernel grammar over the finite instances `I(k)`
supplied by the deployed acset substrate, a Kleene three-valued status
chain that keeps `unknown` honest, per-atomic grounding in proven
machinery so kernel semantics never outruns what is proven, a widening
law that gates future atomics (including pack-defined predicates) on
decidability, and an opaque binding column that lets external checkers
claim constraints without specodelic learning their language.

## Constraints

| id                      | kind      | expr                                                                                                                                                                                                                                                                          | traces_to |
|-------------------------|-----------|-------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|-----------|
| kernel_grammar_closed   | invariant | `a kernel expression's atomics come from the closed set — equality, comparisons, bounded universal and existential quantification over I(k), conjunction, negation, and the reference-typed atomics resolves, unique, acyclic, reachable — an expression using an atomic outside the closed set is a labeled failure naming the atomic and the closed set, never prose and never silently ignored` | [[kernel]]  |
| kernel_decidable        | invariant | `every kernel expression is decidable over the finite instances I(k) — evaluation is total, terminates, and yields exactly one of verified, counterexample, unknown — an addition to the closed set that fails decidability over finite instances cannot register`               | [[kernel]]  |
| kernel_status_three_valued | invariant | `every kernel evaluation reports an explicit three-valued status — verified, counterexample, unknown — unknown is honest (the claim could not be discharged) and propagates under Kleene rules; no evaluation path coerces unknown to pass or to counterexample`                | [[kernel]]  |
| kernel_grounded_in_machinery | invariant | `each v0 atomic's semantics is grounded in existing proven machinery — acyclic and reachable on graph traversal, unique and resolves on acset traversal, equality, comparisons, and bounded quantifiers on model_check bounded evaluation — an atomic without a grounding entry cannot ship` | [[kernel]]  |
| kernel_binding_opaque   | invariant | `the per-constraint kernel.binding cell is an opaque claim carrier — the toolchain extracts and surfaces its contents verbatim and never interprets them; external checkers claim constraints through contract-TOML flags without the format learning their language; no registry is built` | [[kernel]]  |

## Model

### States
- `expr_absent`
- `expr_evaluated`
- `expr_unknown`

### Transitions

| id       | from          | to            | guard                                                                              |
|----------|---------------|---------------|------------------------------------------------------------------------------------|
| declare  | expr_absent   | expr_evaluated | `an expr cell opts into kernel translation per [[kernel.kernel_grammar_closed]]`               |
| discharge | expr_evaluated | expr_unknown | `an evaluation cannot discharge its claim per [[kernel.kernel_status_three_valued]]`  |
| widen    | expr_evaluated | expr_evaluated | `a new atomic registers per [[kernel.kernel_decidable]] with a grounding entry`        |

## Properties

| id                       | kind | derives_from                          | generator                             | predicate                                                                  |
|--------------------------|------|---------------------------------------|---------------------------------------|----------------------------------------------------------------------------|
| unknown_propagates       | unit | [[kernel.kernel_status_three_valued]]   | `expression_with_undischargable_subclaim()` | `composite status is unknown and no coercion to pass or counterexample occurs` |
| acyclic_decides          | unit | [[kernel.kernel_grounded_in_machinery]] | `instance_with_cycle_fixtures()`      | `acyclic atomic returns counterexample on a cyclic instance, verified on an acyclic one` |
| quantifier_bounded       | unit | [[kernel.kernel_decidable]]             | `large_finite_instance()`             | `bounded quantifier evaluation terminates with exactly one status`          |
| unknown_atomic_fails     | unit | [[kernel.kernel_grammar_closed]]        | `expression_with_nonmember_atomic()`  | `grammar rejects labeled, naming the atomic and the closed set`             |
| binding_never_interpreted | unit | [[kernel.kernel_binding_opaque]]        | `constraint_with_arbitrary_binding_text()` | `binding cell contents surface verbatim; no parse, no interpretation`  |

## ADDED Requirements

### Requirement: Decidable bounded kernel grammar
The kernel SHALL accept expressions built from the closed atomic set —
equality, comparisons, bounded ∀/∃ over the finite instances `I(k)`,
∧/¬, and the reference-typed atomics `resolves`, `unique`, `acyclic`,
`reachable` — SHALL reject an expression using an atomic outside the
closed set with a labeled failure naming the atomic and the closed set,
and SHALL evaluate every accepted expression totally over finite
instances.

#### Scenario: Closed atomic set accepts and rejects
- **WHEN** an expr cell carries a kernel expression over the closed atomic set
- **THEN** it parses and evaluates to exactly one status

- **WHEN** an expr cell carries an expression using a non-member atomic
- **THEN** the grammar rejects labeled, naming the atomic and the closed set — never prose, never silently ignored

#### Scenario: Evaluation is total and terminating
- **WHEN** a kernel expression evaluates over any finite instance `I(k)`
- **THEN** evaluation terminates with exactly one of verified, counterexample, unknown

### Requirement: Three-valued evaluation status
Every kernel evaluation SHALL report an explicit three-valued status —
`verified`, `counterexample`, `unknown` — where `unknown` is honest
(the claim could not be discharged) and propagates under Kleene rules;
no evaluation path SHALL coerce `unknown` to `pass` or to
`counterexample`.

#### Scenario: Honest unknown propagates
- **WHEN** a composite claim contains an undischargable subclaim
- **THEN** the composite status is unknown and no coercion to pass or counterexample occurs

#### Scenario: Verified and counterexample remain first-class
- **WHEN** a claim is discharged or refuted over the finite instances
- **THEN** the status is verified or counterexample respectively, persisted in run output

### Requirement: Semantics grounded in proven machinery
Each v0 atomic SHALL name its grounding machinery — `acyclic` and
`reachable` on graph traversal, `unique` and `resolves` on acset
traversal, equality, comparisons, and bounded quantifiers on
model_check bounded evaluation — and an atomic without a grounding
entry SHALL NOT ship, so kernel semantics never outruns proven
machinery.

#### Scenario: Per-atomic grounding holds
- **WHEN** the v0 atomics evaluate against fixtures derived from their grounding machinery
- **THEN** each atomic's result agrees with the machinery's result on the same instance

#### Scenario: Ungrounded atomic cannot ship
- **WHEN** a proposed atomic has no grounding entry
- **THEN** it is not part of the closed set and its use fails labeled

### Requirement: Widening law with a decidability gate
A new atomic SHALL register only if it is decidable over finite
instances, and pack-defined predicates SHALL register under the same
gate; a predicate that fails the gate stays pack-side, checked by
contract-TOML runners rather than by the kernel.

#### Scenario: Decidable addition widens purely
- **WHEN** a new decidable atomic registers with a grounding entry
- **THEN** every expression valid before the registration evaluates identically after it

#### Scenario: Undecidable predicate stays pack-side
- **WHEN** a pack defines a predicate that is not decidable over finite instances
- **THEN** it does not register in the closed set and remains checked by contract-TOML runners

### Requirement: Opaque binding column
The per-constraint `kernel.binding` cell SHALL be an opaque claim
carrier — extracted and surfaced verbatim, never interpreted by the
toolchain — and external checkers SHALL claim constraints through
contract-TOML `flags` without the format learning their language; no
registry SHALL be built.

#### Scenario: Binding surfaces verbatim
- **WHEN** a constraint carries an arbitrary `kernel.binding` text
- **THEN** the contents surface verbatim; no parse, no interpretation, no validation of their internals

#### Scenario: External checkers claim through flags
- **WHEN** an external checker claims a constraint
- **THEN** the claim is carried by contract-TOML `flags` binding (node id, `-k`, or `[[tests.shell]]`) and the format learns nothing of the checker's language

## Requirements

### Requirement: Decidable bounded kernel grammar
The kernel SHALL accept expressions built from the closed atomic set —
equality, comparisons, bounded ∀/∃ over the finite instances `I(k)`,
∧/¬, and the reference-typed atomics `resolves`, `unique`, `acyclic`,
`reachable` — SHALL reject an expression using an atomic outside the
closed set with a labeled failure naming the atomic and the closed set,
and SHALL evaluate every accepted expression totally over finite
instances.

#### Scenario: Closed atomic set accepts and rejects
- **WHEN** an expr cell carries a kernel expression over the closed atomic set
- **THEN** it parses and evaluates to exactly one status

- **WHEN** an expr cell carries an expression using a non-member atomic
- **THEN** the grammar rejects labeled, naming the atomic and the closed set — never prose, never silently ignored

#### Scenario: Evaluation is total and terminating
- **WHEN** a kernel expression evaluates over any finite instance `I(k)`
- **THEN** evaluation terminates with exactly one of verified, counterexample, unknown

### Requirement: Three-valued evaluation status
Every kernel evaluation SHALL report an explicit three-valued status —
`verified`, `counterexample`, `unknown` — where `unknown` is honest
(the claim could not be discharged) and propagates under Kleene rules;
no evaluation path SHALL coerce `unknown` to `pass` or to
`counterexample`.

#### Scenario: Honest unknown propagates
- **WHEN** a composite claim contains an undischargable subclaim
- **THEN** the composite status is unknown and no coercion to pass or counterexample occurs

#### Scenario: Verified and counterexample remain first-class
- **WHEN** a claim is discharged or refuted over the finite instances
- **THEN** the status is verified or counterexample respectively, persisted in run output

### Requirement: Semantics grounded in proven machinery
Each v0 atomic SHALL name its grounding machinery — `acyclic` and
`reachable` on graph traversal, `unique` and `resolves` on acset
traversal, equality, comparisons, and bounded quantifiers on
model_check bounded evaluation — and an atomic without a grounding
entry SHALL NOT ship, so kernel semantics never outruns proven
machinery.

#### Scenario: Per-atomic grounding holds
- **WHEN** the v0 atomics evaluate against fixtures derived from their grounding machinery
- **THEN** each atomic's result agrees with the machinery's result on the same instance

#### Scenario: Ungrounded atomic cannot ship
- **WHEN** a proposed atomic has no grounding entry
- **THEN** it is not part of the closed set and its use fails labeled

### Requirement: Widening law with a decidability gate
A new atomic SHALL register only if it is decidable over finite
instances, and pack-defined predicates SHALL register under the same
gate; a predicate that fails the gate stays pack-side, checked by
contract-TOML runners rather than by the kernel.

#### Scenario: Decidable addition widens purely
- **WHEN** a new decidable atomic registers with a grounding entry
- **THEN** every expression valid before the registration evaluates identically after it

#### Scenario: Undecidable predicate stays pack-side
- **WHEN** a pack defines a predicate that is not decidable over finite instances
- **THEN** it does not register in the closed set and remains checked by contract-TOML runners

### Requirement: Opaque binding column
The per-constraint `kernel.binding` cell SHALL be an opaque claim
carrier — extracted and surfaced verbatim, never interpreted by the
toolchain — and external checkers SHALL claim constraints through
contract-TOML `flags` without the format learning their language; no
registry SHALL be built.

#### Scenario: Binding surfaces verbatim
- **WHEN** a constraint carries an arbitrary `kernel.binding` text
- **THEN** the contents surface verbatim; no parse, no interpretation, no validation of their internals

#### Scenario: External checkers claim through flags
- **WHEN** an external checker claims a constraint
- **THEN** the claim is carried by contract-TOML `flags` binding (node id, `-k`, or `[[tests.shell]]`) and the format learns nothing of the checker's language
