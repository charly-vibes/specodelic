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

### Requirement: Model section compiles to TLA+ module
The system SHALL compile the Model section to one TLA+ module: each
State becomes a value in the state variable's range, each Transition
becomes one disjunct of the `Next` action guarded by its guard field,
and a State with an `emits` field becomes one entry in an `Output`
function from that state value to the effect-kind Constraint's `expr` —
absent for states with no `emits`, never a default or null entry.

#### Scenario: Disjunct count matches transitions
- **WHEN** a Model section with n transitions is compiled
- **THEN** the emitted module's `Next` action contains exactly n disjuncts

#### Scenario: Output covers emitting states only
- **WHEN** a Model has e states with `emits` and s states without
- **THEN** the emitted `Output` function's domain is exactly the e emitting states

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
The system SHALL either produce all three artifacts or report, for
exactly one of them, which stage failed and why — never a silent
partial result.

#### Scenario: Failure is labeled, not partial
- **WHEN** a compilation stage fails on an otherwise lint-clean file
- **THEN** the envelope failure names the failed stage and artifact
- **AND** no partial artifact set is reported as compiled

### Requirement: Compile preserves ids
The system SHALL make every id present in the source file appear,
unchanged, in at least one compiled artifact — no id silently dropped.

#### Scenario: Ids in ⊆ ids out
- **WHEN** a lint-clean spec file is compiled
- **THEN** the set of source ids is a subset of the ids appearing across the three artifacts

### Requirement: Compile round-trips without semantic drift
The system SHALL be idempotent under its own round trip: re-parsing the
compiled TOML artifact and re-emitting it yields output identical to the
original compile.

#### Scenario: Round-trip stable
- **WHEN** a lint-clean spec file is compiled, its TOML re-parsed, and re-emitted
- **THEN** the re-emitted artifacts are byte-identical to the first emission
