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

#### Scenario: Mid-span kernel syntax is a mention
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
set `{rust, py, ts}` in fragment position and SHALL reject an unknown
tag with a labeled extraction failure naming the tag and the closed set.
Supported Python and TypeScript unit Property predicates SHALL emit
through their language emitters; Rust artifact byte preservation follows
the Pure widening requirement. A fragment in an unsupported position
SHALL fail labeled with remediation.

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
Each supported Property row SHALL compile to at least one language-
appropriate test block (Rust proptest, Python Hypothesis/pytest, or
TypeScript fast-check) with its declared generator and predicate, and a
law-kind property to exactly one block per case enumerated in the row's
predicate in machine-findable `**name:**` label form. The identity and
associativity floor is mandatory and lint-enforced before compile; extra
named cases compile as first-class checkable blocks, never prose.

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
