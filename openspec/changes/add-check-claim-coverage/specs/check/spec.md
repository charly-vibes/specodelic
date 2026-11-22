## ADDED Requirements

### Requirement: Judged claims are named by their file-qualified claim id

`spk check` SHALL name every judged claim by its file-qualified claim id —
`<file id>.<claim id>`, the form the corpus already guarantees unique
corpus-wide — and SHALL never abbreviate a claim to its bare id in a report
row, nor match a claim by name, slug, filename, or similarity.

#### Scenario: Two files declaring the same claim id stay distinguishable

- **WHEN** `spk check demo.md` judges a file whose claim id also appears in
  another capability file
- **THEN** the report's rows carry the file-qualified form, each row resolves
  to the claim of the file it names, and neither claim borrows the other's id

#### Scenario: A claim is never matched by name or slug

- **WHEN** a claim id, file name, or sidecar-style slug resembles a claim
  without being it
- **THEN** no row is attributed to that claim, and the resemblance produces no
  row of any kind

### Requirement: Claim coverage is listed and never gates

`spk check` SHALL list every declared invariant claim of the judged file in
the report's claim-coverage section, each carrying exactly one audit class
from audit's existing closed set, and SHALL count the classes in every output
view. A claim with no evidence row SHALL carry `unchecked` — the class the
vocabulary already reserves for knowledge absence. The section SHALL NOT
affect the exit code: an unchecked claim is not a contradiction.

#### Scenario: A claim with no evidence is listed as unchecked

- **WHEN** a declared invariant claim of the judged file has no evidence row
- **THEN** the coverage section lists that claim with class `unchecked`, the
  count is surfaced, and no row claims evidence on its behalf

#### Scenario: An unchecked claim does not fail the run

- **WHEN** a run's rows are all current and one declared claim is unchecked
- **THEN** the exit code is the one the evidence rows alone determine, and the
  unchecked-claim count is reported next to the other counts

#### Scenario: A contradicted claim is listed as contradicted

- **WHEN** a declared invariant claim's evidence row is contradicted
- **THEN** the coverage section lists that claim with class `contradicted`,
  and the run's exit code is unchanged by the section itself

#### Scenario: The section adds no vocabulary value

- **WHEN** the change lands
- **THEN** `scripts/check_evidence_axes.py` passes with no new value row: the
  coverage section reuses audit's existing closed set unchanged

#### Scenario: The section is countable and stable

- **WHEN** the same inputs are judged twice
- **THEN** the coverage section lists the same claims in the same order with
  the same classes — the report remains byte-identical
