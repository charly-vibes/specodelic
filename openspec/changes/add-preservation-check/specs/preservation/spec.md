## ADDED Requirements

### Requirement: Preservation report classification

`spk preserve --before <dir> --after <dir> --oracle <corpus.jsonl>` SHALL
evaluate the same oracle corpus against both compiled files with the conform
walk and classify every scenario id exactly once as `preserved`,
`intentionally_changed` (the oracle record declared the change),
`undeclared_change` (verdicts differ and no declaration exists — a finding),
or `unchecked` (either side could not classify the trace).

#### Scenario: Preserved scenarios agree

- **WHEN** a scenario receives the same verdict class from both files
- **THEN** the report classifies it `preserved`

#### Scenario: Undeclared change is a finding

- **WHEN** a scenario's verdict class differs between the two files and its
  oracle record declares no change
- **THEN** the report classifies it `undeclared_change`, the run is a finding
  on the warnings-plus-findings channel, and the record names both verdicts

#### Scenario: Prose guards leave the scenario unchecked

- **WHEN** either file's covering guard is prose so the walk cannot classify
- **THEN** the scenario is `unchecked` and the report counts it — never
  silently treated as `preserved`

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

### Requirement: Preservation artifact gate

`spk preserve` SHALL refuse to run on files that are not lint-clean with
current compiled artifacts, reusing conform's artifact gate unchanged, and
never compiles, never writes artifacts, and never advances a lifecycle
transition.

#### Scenario: Stale artifacts refused

- **WHEN** either input file's compiled artifacts are absent or stale
- **THEN** the command fails labeled with a remediation hint naming the
  compile stage, and emits no verdict records
