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

### Requirement: An evidence row places a claim only by the declared placement rule

`spk check` SHALL place a declared claim only when an evidence row names that
claim directly or names a property whose `derives_from` names that claim, and
SHALL account for every row that places no claim in the coverage section's
residual bucket.

#### Scenario: A row places a claim through the property it derives from

- **WHEN** an evidence row names a property whose `derives_from` names a
  declared invariant claim of the judged file
- **THEN** the claim-coverage section lists that claim, and the row places no
  other claim

#### Scenario: A row that places no claim is not attributed to one

- **WHEN** an evidence row names neither a declared claim nor a property that
  derives from one
- **THEN** the row appears in the section's residual bucket and no claim
  carries it

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

#### Scenario: A row that places no claim is accounted for

- **WHEN** the run produces an evidence row that corresponds to no declared
  claim of the judged file (for example a model-level row)
- **THEN** the coverage section accounts for that row in its residual bucket
  rather than dropping it, using the existing class set

### Requirement: A claim's coverage class is the strongest class among the rows that place it

`spk check` SHALL give a claim exactly one coverage class: the strongest class
among the rows that place it, ordered `contradicted`, then `stale`, then
`current`, then `new`, then `binds_nothing`, then `unchecked`. A claim placed
by no row SHALL carry `unchecked`.

#### Scenario: A claim with several rows takes the strongest class

- **WHEN** a declared claim is placed by a current row and by a contradicted
  row
- **THEN** the coverage section lists that claim once, with class
  `contradicted`

#### Scenario: A vacuous placement is not an unchecked claim

- **WHEN** a declared claim is placed only by a row whose text cites no
  declared id
- **THEN** the coverage section lists that claim with class `binds_nothing`,
  not with class `unchecked`

### Requirement: The coverage section states its scope and leaves the audit report untouched

`spk check` SHALL state that the coverage section's classes describe the
evidence the run's own producers produced, and SHALL materialize the section in
the check envelope without changing the audit report's schema version or field
set.

#### Scenario: The section says which evidence it describes

- **WHEN** the report renders the claim-coverage section
- **THEN** the section states that its classes describe the evidence this run's
  producers produced, so `unchecked` is never read as "no evidence exists"

#### Scenario: The section leaves the audit report untouched

- **WHEN** the report is emitted
- **THEN** the audit report's schema version and field set are unchanged, and
  the coverage section is carried by the check envelope
