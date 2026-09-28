# Delta spec: compile capability (mirrors specs/compile.md)

## ADDED Requirements

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
