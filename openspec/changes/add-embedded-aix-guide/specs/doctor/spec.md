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

In consumer mode with a local corpus present, `spk doctor` SHALL parse
the latest `Revision N` marker from the local `specs/specodelic.md` and
SHALL emit a warning — not a failure — when that revision is newer than
the binary's embedded `FORMAT_REVISION`.

#### Scenario: Binary lags local corpus

- **WHEN** `spk doctor` runs in a consumer workspace whose
  `specs/specodelic.md` declares `Revision 8` while the binary embeds
  `Revision 7`
- **THEN** the envelope contains a warning naming both revisions and
  exits 0

#### Scenario: Binary current

- **WHEN** `spk doctor` runs in a consumer workspace whose corpus
  revision equals the embedded `FORMAT_REVISION`
- **THEN** no knowledge-currency warning is emitted
