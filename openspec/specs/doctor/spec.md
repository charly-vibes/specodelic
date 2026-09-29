---
id: spec
kind: intent
statement: "THE doctor command SHALL classify the workspace mode and warn — never fail — when the binary's embedded format knowledge lags the local corpus."
---

# doctor Specification

## Purpose
Diagnose a workspace's specodelic setup from inside the binary:
classify the workspace as self-hosting or consumer, check the local
corpus against the embedded format knowledge, and report findings over
the standard envelope without failing on informational conditions.

## Constraints

| id                | kind      | expr                                                                                                                                                          | traces_to |
|-------------------|-----------|---------------------------------------------------------------------------------------------------------------------------------------------------------------|-----------|
| mode_detection    | invariant | `workspace classifies as self_hosting when specs/specodelic.md exists and consumer otherwise, with the mode reported in envelope data`                          | [[spec]]  |
| currency_warning  | invariant | `whenever a local corpus exists, the latest Revision N heading is compared numerically against FORMAT_REVISION; a lagging binary warns without failing; unreadable or revision-less corpora skip with an informational note` | [[spec]]  |

## Model

### States
- `probed`
- `classified`
- `currency_checked`

### Transitions

| id       | from        | to               | guard                      |
|----------|-------------|------------------|----------------------------|
| classify | probed      | classified       | [[spec.mode_detection]]    |
| currency | classified  | currency_checked | [[spec.currency_warning]]  |

## Properties

| id          | kind | derives_from              | generator                  | predicate                                     |
|-------------|------|---------------------------|----------------------------|-----------------------------------------------|
| p_mode      | unit | [[spec.mode_detection]]   | `arbitrary_workspace()`    | `mode == expected(ws) ∧ mode ∈ envelope_data`  |
| p_currency  | unit | [[spec.currency_warning]] | `arbitrary_corpus_state()` | `lag ⇒ warning ∧ ¬failure ∧ skip ⇒ note`       |

## Requirements
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
