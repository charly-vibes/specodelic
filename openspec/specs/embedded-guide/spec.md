---
id: spec
kind: intent
statement: "THE CLI SHALL serve an embedded, repository-free guide to the format through spk explain and keep that knowledge pinned to a corpus revision."
---

# embedded-guide Specification

## Purpose
Make the format's knowledge travel with the binary: `spk explain`
serves a distilled machine-facing guide embedded via `include_str!`,
the embedded knowledge is pinned to a corpus revision and guarded
against drift, and `spk new` scaffolds files that teach their own
tables.

## Constraints

| id               | kind      | expr                                                                                                                                              | traces_to |
|------------------|-----------|----------------------------------------------------------------------------------------------------------------------------------------------------|-----------|
| explain_topics   | invariant | `spk explain serves exactly six topics — format, ears, kinds, references, lifecycle, lint-rules — over the standard envelope; unknown topics fail with a hint listing valid ones` | [[spec]]  |
| revision_pinned  | invariant | `FORMAT_REVISION names the mirrored corpus revision and appears in spk --version --json and every explain payload`                                  | [[spec]]  |
| drift_guard      | invariant | `the test suite fails when FORMAT_REVISION lags the corpus's latest Revision N heading, compared numerically`                                       | [[spec]]  |
| scaffold_teaches | invariant | `spk new generates per-layer guidance comments that the linter ignores`                                                                             | [[spec]]  |

## Model

### States
- `queried`
- `resolved`
- `rendered`

### Transitions

| id       | from     | to       | guard                    |
|----------|----------|----------|--------------------------|
| resolve  | queried  | resolved | [[spec.explain_topics]]  |
| render   | resolved | rendered | [[spec.revision_pinned]] |

## Properties

| id         | kind | derives_from             | generator             | predicate                                   |
|------------|------|--------------------------|-----------------------|---------------------------------------------|
| p_topics   | unit | [[spec.explain_topics]]  | `arbitrary_topic()`   | `known ⇒ body_nonempty ∧ unknown ⇒ hint`     |
| p_revision | unit | [[spec.revision_pinned]] | `arbitrary_version()` | `payload.contains(format_revision)`          |
| p_drift    | unit | [[spec.drift_guard]]     | `stale_constant()`    | `numeric(latest) > numeric(FORMAT_REVISION) ⇒ test_fails` |
| p_scaffold | unit | [[spec.scaffold_teaches]] | `arbitrary_new_file()` | `guidance_present ∧ lint(f) == zero_issues` |

## Requirements
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
