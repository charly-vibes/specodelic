## ADDED Requirements

### Requirement: Preservation report classification

`spk preserve --before <file> --after <file> --oracle <corpus.jsonl>` SHALL
evaluate the same oracle corpus against both compiled files with the conform
walk and classify every scenario id exactly once as `preserved`,
`intentionally_changed` (the oracle record declared the change),
`undeclared_change` (verdicts differ and no declaration exists — a finding),
or `unchecked` (either side could not classify the trace).

#### Scenario: Preserved scenarios agree

- **WHEN** a scenario receives the same verdict class from both files
- **THEN** the report classifies it `preserved`

#### Scenario: Undeclared change is a finding

- **WHEN** a scenario's verdict class differs between the two files and no
  declaration file record marks the scenario `expect_change`
- **THEN** the report classifies it `undeclared_change`, emits a finding
  record naming both verdicts, and the run exits 1 per the exit-code
  requirement

#### Scenario: Prose guards leave the scenario unchecked

- **WHEN** either file's covering guard is prose so the walk cannot classify
- **THEN** the scenario is `unchecked` and the report counts it — never
  silently treated as `preserved`

#### Scenario: Identical inputs are pinned trivially preserved

- **WHEN** the before and after paths resolve to the same compiled artifacts
- **THEN** every scenario classifies `preserved` and the report is otherwise
  identical to any other run — no special case

### Requirement: Preservation evidence scope

Every preservation report SHALL carry the evidence-scope statement that the
result is agreement on the supplied corpus, never a proof of behavioral
equality, and each record SHALL name the observed verdicts and the oracle
corpus digest observed for both files.

#### Scenario: Report header carries the scope

- **WHEN** a preservation report is emitted in any output format
- **THEN** its header states agreement-on-corpus scope, not equivalence, and
  both per-file `scope_sha256` values are present

#### Scenario: Corpus mismatch fails labeled

- **WHEN** the two runs observe different oracle corpus digests
- **THEN** the run fails labeled, naming the digest mismatch

### Requirement: Preservation exit-code contract

`spk preserve` SHALL exit 0 when no scenario classified `undeclared_change`,
exit 1 when at least one scenario classified `undeclared_change`, and exit 2
for invocation errors and gate refusals with zero verdict records —
`preserved` and `unchecked` never affect the exit code, mirroring conform's
exit-code discipline.

#### Scenario: Undeclared change fails the run

- **WHEN** any scenario classified `undeclared_change`
- **THEN** the exit code is 1 and the finding records are surfaced with
  counts in both output views

#### Scenario: Unchecked scenarios never fail the run

- **WHEN** every scenario classified `preserved` or `unchecked`
- **THEN** the exit code is 0 and the unchecked count is surfaced in both
  output views

### Requirement: Preservation evidence substance

`spk preserve` SHALL refuse to classify against an oracle corpus that is
empty or that matches zero scenario ids in either file, failing labeled — a
preservation claim resting on no evidence is not a claim.

#### Scenario: Empty corpus refused

- **WHEN** the oracle corpus contains zero scenario records, or none of its
  ids match a scenario in either file
- **THEN** the run fails labeled naming the empty evidence, and no
  classification report is emitted

### Requirement: Preservation artifact gate

`spk preserve` SHALL refuse to run on files that are not lint-clean with
current compiled artifacts, reusing conform's artifact gate unchanged, and
never compiles, never writes artifacts, and never advances a lifecycle
transition. A `--declarations <changes.json>` input, when given, SHALL be
parsed as a preserve-owned record list of `{scenario_id, expect_change}`
with unknown fields refused, never silently ignored — conform's oracle-record
schema is not extended.

#### Scenario: Stale artifacts refused

- **WHEN** either input file's compiled artifacts are absent or stale
- **THEN** the command fails labeled with a remediation hint naming the
  compile stage, and emits no verdict records
