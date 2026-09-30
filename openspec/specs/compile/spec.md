---
id: spec
kind: intent
statement: "THE compile step SHALL turn a lint-clean spec file into its full artifact set (TOML constraints, proptest sources, TLA+ module) without loss or silent partial results."
---

# compile Specification

## Purpose
Compile a lint-clean specodelic file into the artifact set downstream
consumers run against: a TOML projection of the Constraints table,
proptest scaffolding from the Properties table, and a TLA+ module from
the Model section — total, id-preserving, and byte-stable.

## Constraints

| id                    | kind      | expr                                                                                                                        | traces_to |
|-----------------------|-----------|-----------------------------------------------------------------------------------------------------------------------------|-----------|
| precondition_gate     | invariant | `compile runs only on a file passing lint with zero issues; refusal is labeled precondition_satisfied with a remediation hint` | [[spec]]  |
| constraints_to_toml   | invariant | `every Constraint row compiles to one TOML entry {id, kind, expr, traces_to} field-for-field, losslessly`                      | [[spec]]  |
| model_to_ir           | invariant | `the Model section extracts to a backend-neutral IR — states, guarded transitions, and an emits mapping covering emitting states only` | [[spec]]  |
| properties_to_proptest | invariant | `each Property row compiles to at least one proptest block; law-kind rows expand to one block per required case`               | [[spec]]  |
| compile_total         | invariant | `compile produces the full artifact set due under the backend decision or reports exactly which stage failed — never a silent partial result` | [[spec]]  |
| preserves_ids         | invariant | `every source id appears unchanged in at least one compiled artifact or the ModelIR`                                          | [[spec]]  |
| round_trip_stable     | invariant | `re-parsing the compiled TOML and re-emitting it yields output byte-identical to the original compile`                         | [[spec]]  |
| model_to_tla          | invariant | `one TLA+ module per compiled file, always: one Next disjunct per transition, Output defined on emitting states only, spec text carried verbatim in comments` | [[spec]]  |
| compilation_failure   | effect    | `spec.compilation_failure(stage, detail) — the compile failed in stage because detail; the envelope failure names the failed stage and artifact (single_labeled_failure)` | [[spec]]  |

## Model

### States
- `received`
- `precondition_ok`
- `emitted`
- `failed` (emits: `[[spec.compilation_failure]]`)

### Transitions

| id            | from           | to             | guard                         |
|---------------|----------------|----------------|-------------------------------|
| lint_gate     | received       | precondition_ok | [[spec.precondition_gate]]   |
| emit_all      | precondition_ok | emitted       | [[spec.compile_total]]       |
| label_failure | precondition_ok | failed        | [[spec.compile_total]]       |

## Properties

| id             | kind | derives_from             | generator                  | predicate                                   |
|----------------|------|--------------------------|----------------------------|---------------------------------------------|
| p_precondition | unit | [[spec.precondition_gate]] | `arbitrary_spec_file()`   | `lint(f) == passed ∨ exit_names_precondition` |
| p_toml         | unit | [[spec.constraints_to_toml]] | `arbitrary_constraint_row()` | `reparse(emit(row)) == row`             |
| p_ir           | unit | [[spec.model_to_ir]]     | `arbitrary_model_section()` | `ir.transitions.len == n ∧ emits.domain == emitting_states` |
| p_proptest     | unit | [[spec.properties_to_proptest]] | `arbitrary_property_row()` | `blocks_for(row) >= 1 ∧ law_cases_expanded` |
| p_total        | unit | [[spec.compile_total]]   | `failing_stage()`          | `report.names(stage) ∧ ¬partial_reported`   |
| p_ids          | unit | [[spec.preserves_ids]]   | `arbitrary_lint_clean_file()` | `source_ids ⊆ artifact_ids ∪ model_ir_ids` |
| p_roundtrip    | unit | [[spec.round_trip_stable]] | `arbitrary_lint_clean_file()` | `reemit(reparse(toml)) == toml`          |
| p_tla          | unit | [[spec.model_to_tla]]    | `arbitrary_model_section()` | `disjuncts == n ∧ output.domain == emitting_states` |
| p_compilation_failure | unit | [[spec.compilation_failure]] | `failing_stage()`  | `error_label == "spec.compilation_failure"` — renaming the label touches the error Constraint, this property, and its note together |

## Requirements
### Requirement: Compile precondition gate
The system SHALL run `compile` only on a spec file that passes lint with
zero issues (linted and covered), refusing otherwise with a labeled
failure and a remediation hint.

#### Scenario: Lint-dirty file refused
- **WHEN** `spk compile <file>` is invoked on a file with a coverage-rule finding
- **THEN** the command exits non-zero
- **AND** the envelope failure names `precondition_satisfied` as the failed stage

#### Scenario: Lint-clean corpus compiles
- **WHEN** `spk compile specs --json` is invoked on the repo's own corpus
- **THEN** the command exits 0 and every file reports three artifacts

### Requirement: Constraints table compiles to TOML
The system SHALL compile every Constraint row to one TOML table entry
`{id, kind, expr, traces_to}` field-for-field, with no lossy
transformation.

#### Scenario: TOML round-trips
- **WHEN** an arbitrary Constraint row is compiled and the emitted TOML is re-parsed
- **THEN** the parsed entry equals the source row's four fields

### Requirement: Model section extracts to backend-neutral IR
The system SHALL extract the Model section into a backend-neutral
intermediate representation — states, guarded transitions, and the
`emits`→effect-Constraint mapping — from which a model-checker module
(TLA+ or Alloy, per beads `specodelic-mp1` row 6) can be emitted without
re-parsing the markdown.

#### Scenario: IR captures guarded transitions
- **WHEN** a Model section with n transitions is extracted
- **THEN** the IR contains exactly n transitions, each with its from/to states and guard

#### Scenario: IR captures emitting states only
- **WHEN** a Model has e states with `emits` and s states without
- **THEN** the IR's emits mapping has exactly e entries — no entry for the rest, never a default or null

### Requirement: Properties table compiles to proptest blocks
The system SHALL compile each Property row to at least one `proptest!`
block — generator as input strategy, predicate as assertion body — and a
law-kind property to one block per required case (identity,
associativity, …).

#### Scenario: One block per property row
- **WHEN** an arbitrary Property row is compiled
- **THEN** the emitted proptest source contains at least one block for that row

#### Scenario: Law expands to required cases
- **WHEN** a law-kind property row requires identity and associativity
- **THEN** exactly two blocks are emitted for that row

### Requirement: Compile is total
The system SHALL either produce all artifacts due under the current
backend decision (TOML + proptest sources; plus the model module once
`mp1` row 6 lands) or report, for exactly one of them, which stage
failed and why — never a silent partial result.

#### Scenario: Failure is labeled, not partial
- **WHEN** a compilation stage fails on an otherwise lint-clean file
- **THEN** the envelope failure names the failed stage and artifact
- **AND** no partial artifact set is reported as compiled

### Requirement: Compile preserves ids
The system SHALL make every id present in the source file appear,
unchanged, in at least one compiled artifact — no id silently dropped.
Ids whose only home is the model artifact (states, transitions) are
preserved in the `ModelIR` until the backend lands.

#### Scenario: Ids in ⊆ ids out
- **WHEN** a lint-clean spec file is compiled
- **THEN** the set of source ids is a subset of the ids appearing across the emitted artifacts and the model IR

### Requirement: Compile round-trips without semantic drift
The system SHALL be idempotent under its own round trip: re-parsing the
compiled TOML artifact and re-emitting it yields output identical to the
original compile.

#### Scenario: Round-trip stable
- **WHEN** a lint-clean spec file is compiled, its TOML re-parsed, and re-emitted
- **THEN** the re-emitted artifacts are byte-identical to the first emission

### Requirement: Model section compiles to a TLA+ module
The system SHALL emit one TLA+ module per compiled spec file — always,
regardless of which model_check backend later runs against the model:
each State becomes a value in the module's state variable's range, each
Transition becomes one disjunct of the `Next` action, and a State with an
`emits` field additionally becomes one entry in an `Output` function from
that state value to the effect-Constraint's `expr` — absent for a state
with no `emits`, never a default or null entry. Spec-side guard and
emits text is not TLA+; it SHALL be carried verbatim in comments while
the emitted disjuncts are valid TLA+ over the state variable.

#### Scenario: Disjunct count matches transitions
- **WHEN** a Model section with n transitions is compiled
- **THEN** the emitted module's `Next` action contains exactly n disjuncts

#### Scenario: Output covers emitting states only
- **WHEN** a Model has e states with `emits` and s states without
- **THEN** the emitted `Output` function's domain is exactly the e emitting states
- **AND** no entry exists for the rest — never a default or null

#### Scenario: Module always emitted
- **WHEN** a lint-clean spec file is compiled
- **THEN** the command writes `<stem>.tla` alongside the TOML and proptest artifacts
- **AND** recompiling yields byte-identical module text
