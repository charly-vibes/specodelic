# embedded-guide Delta

## ADDED Requirements

### Requirement: Embedded Format Guide Command

The CLI SHALL provide an `explain` subcommand that serves a distilled,
machine-facing guide to the Specodelic format from content embedded in
the binary (`include_str!`), requiring no repository access, and SHALL
emit it through the standard genesis JSON envelope (human-readable under
`--human`).

#### Scenario: Agent in a consumer repo asks for the format

- **WHEN** `spk explain format` runs in a directory with no
  `specs/specodelic.md` on disk
- **THEN** it exits 0 and the envelope `data.body` contains the
  four-layer table and the lifecycle description

#### Scenario: Topic listing

- **WHEN** `spk explain` runs with no topic argument
- **THEN** it exits 0 and the envelope `data.topics` lists exactly the
  topics enumerated in the Guide Topics requirement

#### Scenario: Unknown topic

- **WHEN** `spk explain nope` runs
- **THEN** it exits non-zero with an envelope failure whose hint lists
  the valid topics

### Requirement: Guide Topics

The guide SHALL expose exactly six topics: `format` (the four layers
and the lifecycle), `ears` (the five EARS patterns), `kinds` (the closed
`kind` value sets for all five schema objects), `references` (the
Reference Typing table), `lifecycle` (the stage gates and checker
ownership), and `lint-rules` (every rule id the binary can emit, each
with a one-line semantics string).

#### Scenario: Closed kind sets rendered from enforced constants

- **WHEN** `spk explain kinds` runs
- **THEN** the rendered constraint/property `kind` sets equal the
  constants the linter enforces, because both render from the same
  `pub const` source in `guide.rs`

#### Scenario: Lint rule catalog is complete

- **WHEN** `spk explain lint-rules` runs
- **THEN** every rule id the linter can emit appears in the catalog
  with a non-empty semantics string (unit-tested by iterating the
  linter's rule table)

### Requirement: Embedded Format Revision Pinning

The binary SHALL declare a `FORMAT_REVISION` constant naming the corpus
revision it mirrors (e.g. `specodelic.md Revision 8`); `spk --version
--json` SHALL include it as `format_revision` and every `explain`
payload SHALL include it.

#### Scenario: Version reports format knowledge

- **WHEN** `spk --version --json` runs
- **THEN** the envelope contains `format_revision`

### Requirement: Guide-Drift Guard

The repo's test suite SHALL fail if `FORMAT_REVISION` is stale relative
to the latest `Revision N` heading in `specs/specodelic.md`, comparing
the trailing integers numerically, so binary knowledge cannot silently
lag the corpus.

#### Scenario: Corpus revision bumped without updating the constant

- **WHEN** `specs/specodelic.md` gains a newer `Revision N` marker than
  `FORMAT_REVISION` and the test suite runs
- **THEN** a unit test fails naming both revisions

#### Scenario: Double-digit revision ordering

- **WHEN** the corpus's latest heading is `Revision 10` and
  `FORMAT_REVISION` names `Revision 9`
- **THEN** the comparison reports the corpus as newer (numeric, not
  lexicographic, comparison)

### Requirement: Scaffold Teaches the Format

`spk new` SHALL generate a template whose each of the four layers
carries an HTML-comment guidance note (valid kinds for that layer's
rows, what a guard may cite, law-case requirements for `kind = "law"`
properties), and the guidance SHALL be deletable without affecting lint
results, because the linter never parses prose.

#### Scenario: Scaffolded file explains its own tables

- **WHEN** `spk new order.cancel` runs
- **THEN** the generated file contains comment guidance under the
  Constraints, Model, and Properties sections

#### Scenario: Guidance does not affect linting

- **WHEN** a fully filled-in spec containing the scaffold guidance
  comments is linted
- **THEN** the guidance produces no findings
