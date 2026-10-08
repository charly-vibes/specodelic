---
id: parse
kind: intent
statement: "WHEN a user runs spk parse on one spec file THE parse capability SHALL emit the full parsed Spec IR as a json envelope, SHALL succeed independently of lint status, and SHALL fail with a labeled error envelope on unparseable input."
---

# structured parse export

The structured layer (Properties rows included) is fully modeled in the lib
crate but unreachable from outside it: `spk compile --json` embeds generated
artifact text and exposes no Properties rows as structure. Fleet consumers
(espectacular's `ah sync`) must not reimplement the markdown-table parser —
they consume the typed IR through the versioned-binary boundary instead.

`spk parse` is the minimal exposure: same parse path as lint and compile, the
`Serialize`-derived `Spec` emitted directly, envelope-conformant, syntax-only.

## Constraints

| id | kind | expr | traces_to |
|----|------|------|-----------|
| process_lifecycle | invariant | `the capability advances through its declared lifecycle states under the repo's change process — each stage transition fires only when its stage gate holds` | [[parse]] |
| C-parse-ir | invariant | spk parse emits the full parsed Spec IR for a well-formed file — intent, constraints rows, states, transitions, properties rows, and structured links — with no structured field omitted | [[parse]] |
| C-parse-lint-independent | invariant | parse succeeds on a file that fails spk lint and the parse envelope embeds no lint status | [[parse]] |
| C-parse-envelope | invariant | parse output conforms to the shared json envelope discipline — ok, envelope_version, cli_version, envelope_kind, data, warnings, hints, meta — with hints suggesting spk lint on the parsed file | [[parse]] |
| C-parse-error | effect | `spec.parse_failure(detail) — the file could not be parsed or located because detail; the error envelope satisfies the fleet error contract (error-kind envelope, labeled, remediation hint present, non-zero exit)` | [[parse]] |
| C-parse-single-file | advisory | parse accepts exactly one file path argument per invocation, without globbing or bulk export | [[parse]] |

## Model

### States

- `reading`
- `parsed`
- `emitted`
- `failed` (emits: `[[parse.C-parse-error]])`

### Transitions

| id | from | to | guard |
|----|------|----|-------|
| t-parse | reading | parsed | [[parse.C-parse-ir]]  |
| t-emit | parsed | emitted | [[parse.C-parse-envelope]]  |
| t-fail | reading | failed | `¬([[parse.C-parse-ir]])`  |

## Properties

| id | kind | derives_from | generator | predicate |
|----|------|--------------|-----------|-----------|
| process_lifecycle_checked | unit | [[parse.process_lifecycle]] | `lifecycle_model_present()` | `check(file) == passed` |
| P-ir | unit | [[parse.C-parse-ir]] | `well_formed_four_layer_file()` | `data.carries(intent, constraints, states, transitions, properties, links) ∧ every_row_present` |
| P-lintindep | unit | [[parse.C-parse-lint-independent]] | a file that parses but fails spk lint | parse exits zero with the full IR and no lint status anywhere in the envelope |
| P-envelope | unit | [[parse.C-parse-envelope]] | parse output inspected field by field | every shared envelope field is present and hints suggest spk lint |
| P-error | unit | [[parse.C-parse-error]] | `unparseable_file()` and `nonexistent_path()` | `envelope.has_label ∧ envelope.has_hint ∧ exit != 0` in both cases |
| P-single | unit | [[parse.C-parse-single-file]] | parse invoked with zero or two or more paths | the invocation is rejected with usage guidance |

## Purpose

Expose the typed Spec IR through the CLI so external fleet tools consume the
structured layer without reimplementing the format parser, keeping the
versioned binary as the single integration boundary.

## ADDED Requirements

### Requirement: Structured Parse Export
`spk parse <file>` SHALL emit the parsed Spec IR — intent, constraints rows, states, transitions, properties rows, and structured links — as a `--json`-conformant envelope, SHALL succeed independently of lint status without embedding it, and SHALL fail with a labeled error envelope and non-zero exit on unparseable or missing input.

#### Scenario: Export full IR
- **GIVEN** a well-formed four-layer spec file with three Properties rows
- **WHEN** `spk parse <file> --json` runs
- **THEN** the envelope is `ok` and `data` carries intent, constraints, states, transitions, all three property rows, and the structured links

#### Scenario: Lint-dirty file still parses
- **GIVEN** a file that parses but fails `spk lint`
- **WHEN** `spk parse <file> --json` runs
- **THEN** it exits zero with the full IR and the envelope embeds no lint status

#### Scenario: Unparseable input errors with label
- **GIVEN** a file that violates the format's parse rules, and separately a nonexistent path
- **WHEN** `spk parse` runs on each
- **THEN** each yields a labeled error envelope with a remediation hint and non-zero exit

#### Scenario: Single file per invocation
- **GIVEN** zero or two or more path arguments
- **WHEN** `spk parse` runs
- **THEN** the invocation is rejected with usage guidance

## Requirements

### Requirement: Structured Parse Export
`spk parse <file>` SHALL emit the parsed Spec IR — intent, constraints rows, states, transitions, properties rows, and structured links — as a `--json`-conformant envelope, SHALL succeed independently of lint status without embedding it, and SHALL fail with a labeled error envelope and non-zero exit on unparseable or missing input.

#### Scenario: Export full IR
- **GIVEN** a well-formed four-layer spec file with three Properties rows
- **WHEN** `spk parse <file> --json` runs
- **THEN** the envelope is `ok` and `data` carries intent, constraints, states, transitions, all three property rows, and the structured links

#### Scenario: Lint-dirty file still parses
- **GIVEN** a file that parses but fails `spk lint`
- **WHEN** `spk parse <file> --json` runs
- **THEN** it exits zero with the full IR and the envelope embeds no lint status

#### Scenario: Unparseable input errors with label
- **GIVEN** a file that violates the format's parse rules, and separately a nonexistent path
- **WHEN** `spk parse` runs on each
- **THEN** each yields a labeled error envelope with a remediation hint and non-zero exit

#### Scenario: Single file per invocation
- **GIVEN** zero or two or more path arguments
- **WHEN** `spk parse` runs
- **THEN** the invocation is rejected with usage guidance