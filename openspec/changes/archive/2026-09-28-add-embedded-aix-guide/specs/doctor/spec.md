# doctor Delta

## ADDED Requirements

### Requirement: Workspace Mode Detection

`spk doctor` SHALL classify the workspace as `self_hosting` when
`specs/specodelic.md` exists, and `consumer` otherwise, and SHALL
report the detected mode in its envelope data.

#### Scenario: Self-hosting corpus

- **WHEN** `spk doctor` runs in a workspace containing
  `specs/specodelic.md`
- **THEN** the envelope reports mode `self_hosting` and keeps today's
  checks (specs directory, core format spec, beads)

#### Scenario: Consumer workspace without the core spec

- **WHEN** `spk doctor` runs in a workspace with a `specs/` directory
  but no `specs/specodelic.md`
- **THEN** the envelope reports mode `consumer`, the check for the core
  spec does not fail the command, and the envelope reports the embedded
  guide's `format_revision` as ok

#### Scenario: Empty workspace

- **WHEN** `spk doctor` runs in a directory with no `specs/` at all
- **THEN** the envelope reports mode `consumer` with an informational
  note suggesting `spk new` to start a corpus, and exits 0

### Requirement: Doctor Reports Knowledge Currency

`spk doctor` SHALL — in any mode, whenever a local corpus is present
(vendored consumer corpora included) — extract
the latest `Revision N` heading (numerically largest trailing integer)
from the local `specs/specodelic.md` and SHALL emit a warning — not a
failure — when that revision is newer than the binary's embedded
`FORMAT_REVISION`. A corpus with no `Revision` heading skips the
currency check with an informational note.

#### Scenario: Corpus file is unreadable

- **WHEN** the local `specs/specodelic.md` exists but cannot be read
  (permissions, non-UTF-8 content)
- **THEN** the currency check is skipped with an informational warning
  naming the read error, and the command neither fails nor claims the
  corpus is current

#### Scenario: Binary lags local corpus

- **WHEN** `spk doctor` runs in a workspace whose vendored
  `specs/specodelic.md` declares `Revision 8` while the binary embeds
  `Revision 7`
- **THEN** the envelope contains a warning naming both revisions and
  exits 0

#### Scenario: Binary current

- **WHEN** `spk doctor` runs in a workspace whose corpus
  revision equals the embedded `FORMAT_REVISION`
- **THEN** no knowledge-currency warning is emitted

#### Scenario: Corpus carries no revision headings

- **WHEN** `spk doctor` runs in a workspace whose
  `specs/specodelic.md` contains no `Revision N` heading
- **THEN** the currency check is skipped with an informational note and
  the command neither warns nor fails
