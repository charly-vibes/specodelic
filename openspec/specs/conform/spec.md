---
id: conform
kind: intent
statement: "WHEN an oracle scenario record optionally declares `expect: forbidden` and conform classifies its trace, THE conform capability SHALL echo the declaration and its met or unmet outcome in the verdict record, SHALL account met expectations as non-gating agreement-tier evidence while keeping the mechanical verdict, SHALL account unmet expectations in the hard forbidden bucket so the run exits 1, SHALL preserve unchanged semantics for undeclared records, and SHALL account prose-only advisory records separately so they never gate."
---

# conform Specification

## Purpose

Check a spec against reality instead of against itself: classify traces
recorded from an external oracle (a legacy system, curated cases, a
reference implementation) against what the spec declares and can execute.
conform is the external-evidence leg the pipeline lacks — every other verb
evaluates the spec's own claims. Its discipline is the negative space: a
missing transition is underspecification, never prohibition; prose is never
silently checked; agreement on a finite corpus is never behavioral
equality. conform is read-only over specs and oracles alike — it evaluates,
it never generates traces, implementations, or verdicts beyond its
declared scope.

## Constraints

| id                       | kind      | expr                                                                                                                                                                                                                                                            | traces_to       |
|--------------------------|-----------|-----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|-----------------|
| closed_expectation_schema | invariant | `the scenario schema accepts an optional expect drawn only from {forbidden}; an out-of-set value or an unknown field is a corpus_parse refusal naming the field, the closed set, and the one-JSON-object-per-line remediation — never a silent "no expectation"` | [[conform]] |
| expectation_echoed       | invariant | `every verdict record echoes the declared expectation in expect and its qualification in expectation_met, both null when no declaration exists`                                                                                                                  | [[conform]] |
| agreement_tier           | invariant | `a met expectation is counted in agreement_counts keyed by the verdict spelling, excluded from verdict_counts and from the forbidden + unsupported exit predicate, exits 0, and renders as the oracle agreeing`                                                  | [[conform]] |
| unmet_gates              | invariant | `an unmet expectation keeps the mechanical verdict, leads its reason with the expectation_unmet marker, counts in the hard forbidden bucket, and exits 1`                                                                                                          | [[conform]] |
| absence_preserved        | invariant | `records without expect keep their previous verdict, reason, count placement, gating, and rendering, differing only by the two new null fields`                                                                                                                  | [[conform]] |
| advisory_non_gating      | invariant | `undeclared prose-only covering evidence keeps the closed unknown verdict, is marked advisory: true, counted in advisory_counts, excluded from verdict_counts and the exit predicate, and named non-gating in the human view`                                       | [[conform]] |

## Model

### States
- `gated`
- `classified`
- `tiered`
- `reported`

### Transitions

| id       | from       | to         | guard                                                                          |
|----------|------------|------------|--------------------------------------------------------------------------------|
| classify | gated      | classified | [[conform.closed_expectation_schema]] ∧ [[conform.expectation_echoed]] |
| tier     | classified | tiered     | [[conform.agreement_tier]] ∧ [[conform.unmet_gates]]                |
| report   | tiered     | reported   | [[conform.absence_preserved]] ∧ [[conform.advisory_non_gating]]    |

## Properties

| id         | kind | derives_from                                  | generator                     | predicate                                                                                                              |
|------------|------|-----------------------------------------------|-------------------------------|------------------------------------------------------------------------------------------------------------------------|
| p_schema   | unit | [[conform.closed_expectation_schema]]   | `arbitrary_scenario_corpus()` | `invalid expect or unknown field ⇒ corpus_parse refusal naming field, closed set, and remediation; forbidden ⇒ parses`  |
| p_echo     | unit | [[conform.expectation_echoed]]          | `arbitrary_declared_scenario()` | `record.expect equals the declaration ∧ record.expectation_met equals the verdict match`                              |
| p_agreement | unit | [[conform.agreement_tier]]             | `met_forbidden_expectation()` | `agreement_counts counts the verdict ∧ verdict_counts excludes it ∧ exit 0`                                            |
| p_unmet    | unit | [[conform.unmet_gates]]                 | `unmet_forbidden_expectation()` | `mechanical verdict retained ∧ reason leads expectation_unmet ∧ hard forbidden count grows ∧ exit 1`                  |
| p_absence  | unit | [[conform.absence_preserved]]           | `arbitrary_undeclared_scenario()` | `expect null ∧ expectation_met null ∧ prior verdict, reason, count placement, and gating unchanged`                    |
| p_advisory | unit | [[conform.advisory_non_gating]]         | `prose_only_covering_claim()` | `advisory true ∧ verdict unknown ∧ advisory_counts counted ∧ verdict_counts excluded ∧ exit 0`                          |

## Requirements

### Requirement: Oracle trace classification
The system SHALL provide `spk conform <spec-path> --oracle <scenarios.jsonl>`
classifying each trace in the corpus with exactly one verdict from the
closed taxonomy {permitted, forbidden, underspecified, unknown,
unsupported}, each verdict record carrying its reason and the ids of the
claims evaluated, deriving only from structural Model facts and executable
claims — never from prose, never from a constraint's name, and never from
the existence of a deriving Property.

#### Scenario: Executable claim contradicts a trace
- **WHEN** a corpus trace's observed transition violates an executable invariant claim (`**rust:**` fragment or kernel atom) over the compiled model
- **THEN** the trace is classified `forbidden` and the verdict record names the contradicted claim id in its reason

#### Scenario: Prose-only claim covers a trace
- **WHEN** the only claim covering a trace has a prose-only expr (no kernel, `**rust:**`, or citation marker)
- **THEN** the trace's verdict is `unknown` with a reason naming the prose claim — never `permitted` or `forbidden`

#### Scenario: No declared claim covers a trace
- **WHEN** a trace references a transition or observation no declared Model element or claim covers
- **THEN** the verdict is `underspecified` in the default open-world mode

### Requirement: Closed-world gate
The system SHALL separate the two prohibition evidence classes: a trace
that violates a declared, executable claim SHALL be `forbidden` in any
invocation mode, and the closed-world declaration SHALL be recorded in
every verdict record and the report header.

#### Scenario: Contradiction forbidden in any mode
- **WHEN** a trace violates an executable claim and the run was invoked without `--closed-world`
- **THEN** the verdict is `forbidden` and its record carries `closed_world: false` with the contradicted claim id in the reason

### Requirement: Uncovered traces follow the closed-world declaration
A trace no declared Model element or claim covers SHALL be `forbidden`
only when the invocation explicitly declared closed-world mode via
`--closed-world`; in the default open-world mode, absence of coverage
SHALL yield `underspecified`, never `forbidden`.

#### Scenario: Uncovered trace forbidden under the declaration
- **WHEN** a trace no declared claim or Model element covers and the run was invoked with `--closed-world`
- **THEN** the verdict is `forbidden` and its record carries `closed_world: true` with the exhaustiveness declaration named in the reason

#### Scenario: Uncovered trace underspecified without the declaration
- **WHEN** the same uncovered trace occurs in a run without `--closed-world`
- **THEN** the verdict is `underspecified`, never `forbidden`

### Requirement: Verdict report schema and evidence scope
The system SHALL emit the verdict report through the standard envelope
conventions (JSON default for pipes, `--human` for TTYs), carrying a
conform-local `report_schema_version`, per-trace verdict records (scenario
id, verdict, reason, evaluated claim ids, closed_world flag), and an
`evidence_scope` field stating that the report establishes agreement on
the supplied corpus and not behavioral equality.

#### Scenario: Evidence scope present in both views
- **WHEN** the report is rendered as JSON and as `--human`
- **THEN** both views carry the same `evidence_scope` statement — no view can omit it

### Requirement: Scope digest and deterministic re-runs
The report SHALL carry a scope digest binding the parsed structured
content and the consumed scenario corpus bytes. Re-running conform over
unchanged inputs SHALL produce byte-identical reports — verdict records
sorted by scenario id, no timestamps, no iteration-order leakage — and a
corpus with zero traces SHALL yield a valid report with zero records and
the evidence_scope intact.

#### Scenario: Scope digest binds inputs
- **WHEN** two runs consume identical spec content but scenario corpora differing by one byte
- **THEN** the two reports carry different scope digests; identical inputs yield identical digests

#### Scenario: Re-runs are byte-identical
- **WHEN** conform runs twice over unchanged spec and corpus inputs
- **THEN** both reports are byte-identical — records sorted by scenario id, no timestamps

#### Scenario: Empty corpus yields a valid empty report
- **WHEN** the `--oracle` file contains zero trace records
- **THEN** conform exits 0 with a valid report carrying zero verdict records and the evidence_scope intact

### Requirement: Exit-code contract
The system SHALL map exit codes to verdict classes: exit 0 when no verdict
in the report is `forbidden` or `unsupported`; exit 1 when any verdict is
`forbidden` or `unsupported`; `unknown` and `underspecified` verdicts
SHALL NOT affect the exit code and SHALL be surfaced with counts in both
report views. Invocation errors and gate refusals SHALL remain exit 2.

#### Scenario: Violation-class verdict fails the run
- **WHEN** the report contains at least one `forbidden` or `unsupported` verdict
- **THEN** conform exits 1

#### Scenario: Coverage gaps do not fail the run
- **WHEN** the report contains only `permitted`, `unknown`, and `underspecified` verdicts
- **THEN** conform exits 0 with the unknown and underspecified counts visible in both views

#### Scenario: Invocation error stays exit 2
- **WHEN** the spec path does not exist or the corpus is refused by the artifact gate
- **THEN** conform exits 2 without emitting verdict records

### Requirement: Artifact gate
The system SHALL refuse to run conform over a spec file that does not
parse, does not lint clean, or whose compiled artifacts are not current,
with a remediation hint naming the failing stage (`run spk lint` / `run
spk compile`), and SHALL NOT emit verdict records from such an invocation.

#### Scenario: Stale artifacts refused
- **WHEN** the spec file's structured content changed after its last compile
- **THEN** conform refuses with a hint naming compile, and no verdict records are emitted

#### Scenario: Lint-dirty file refused
- **WHEN** the target file carries lint findings
- **THEN** conform refuses with a hint naming lint, and no verdict records are emitted

### Requirement: Outside the artifact lifecycle
The system SHALL keep conform outside the artifact lifecycle: a conform run
SHALL NOT advance the spec artifact's lifecycle stage, SHALL NOT gate any
lifecycle transition, and SHALL NOT be part of `orchestrate` runs.

#### Scenario: Orchestrate never runs conform
- **WHEN** `spk orchestrate` runs over a spec file with conform available
- **THEN** the pipeline is lint → compile → model_check → verify as specified and conform is not invoked

#### Scenario: Conform does not advance lifecycle
- **WHEN** conform completes over a `model_checked` artifact
- **THEN** the artifact's lifecycle stage is unchanged after the run

### Requirement: Scenario expectation declaration
The oracle scenario corpus schema SHALL accept an optional, string-valued
`expect` field drawn from the closed set {`forbidden`} on every
`ScenarioTrace` record, SHALL keep `deny_unknown_fields`, and SHALL refuse a
value outside the closed set or an unknown field as a `corpus_parse` schema
error naming the offending field, the accepted closed set, and the
one-JSON-object-per-line remediation hint — never silently treating an
unrecognized expectation as "no expectation".

#### Scenario: Expected forbidden declaration parses
- **GIVEN** a scenario record with `"expect": "forbidden"`
- **WHEN** the oracle corpus is parsed
- **THEN** the record parses with its `id`, `setup`, `trace`, and the
  declaration preserved, and the declaration is available to
  classification

#### Scenario: Invalid expectation value refused
- **GIVEN** a scenario record whose `expect` value is not in the closed set
  (for example `"banana"`)
- **WHEN** the oracle corpus is parsed
- **THEN** parsing fails with a `corpus_parse` schema error that names the
  `expect` field, names the accepted closed set (`forbidden`) through the
  serde unknown-variant path, and carries the one-JSON-object-per-line
  remediation hint — never a silently ignored declaration

#### Scenario: Unknown field still refused
- **GIVEN** a scenario record carrying a field the schema does not declare
  (for example `"expected": "confirmed"`)
- **WHEN** the oracle corpus is parsed
- **THEN** parsing fails with a `corpus_parse` refusal carrying the
  remediation hint — adding `expect` does not open the record schema

### Requirement: Expectation echo and agreement tier
Every `VerdictRecord` SHALL echo the scenario's declared expectation in
`expect` and the classification outcome in `expectation_met`, both `null`
when none was declared. A verdict that meets its declaration SHALL count in
`agreement_counts` rather than `verdict_counts`, SHALL NOT enter the
`forbidden + unsupported` exit predicate, SHALL exit 0, and SHALL render as
the oracle agreeing; a record without `expect` SHALL keep its previous
verdict, reason, counts, gating, and rendering.

#### Scenario: Expected forbidden agrees with the oracle
- **GIVEN** a corpus whose scenario declares `expect: forbidden` and whose
  trace the compiled Model forbids
- **WHEN** conform classifies the trace and builds the report
- **THEN** the record carries `verdict: forbidden`, `expect: forbidden`,
  and `expectation_met: true`; the verdict is counted in
  `agreement_counts` and absent from `verdict_counts`; the run exits 0;
  and the human view states that the oracle agreed

#### Scenario: Unexpected forbidden with no declaration fails the run
- **GIVEN** a corpus whose scenario declares no `expect` and whose trace
  contradicts an executable claim
- **WHEN** conform classifies the trace and builds the report
- **THEN** the record keeps `verdict: forbidden`, is counted in the hard
  `verdict_counts`, the run exits 1, and no agreement tier is recorded

#### Scenario: Absent declaration leaves the record unchanged
- **GIVEN** a scenario record with no `expect` field
- **WHEN** conform classifies it and builds the report
- **THEN** its verdict, reason, count placement, gating, and human
  rendering are those of the pre-expectation behavior, `expect` and
  `expectation_met` are `null`, and its previously serialized fields are
  unchanged apart from the two new `null` fields

### Requirement: Unmet expectations are hard disagreements
When a scenario declares an expectation and the classified verdict does
not match it, the record SHALL retain its mechanical verdict and SHALL
carry a reason that leads with the `expectation_unmet:` marker naming the
declared expectation and the observed verdict. The report SHALL count the
record in the hard `forbidden` bucket of `verdict_counts`, so the
`forbidden + unsupported` exit predicate fires and the run exits 1 —
never a silent pass and never a rewritten verdict.

#### Scenario: Declared prohibition the Model permits fails the run
- **GIVEN** a trace the compiled Model permits but whose scenario declares
  `expect: forbidden`
- **WHEN** conform classifies the trace and builds the report
- **THEN** the record keeps `verdict: permitted` with
  `expectation_met: false`, its reason leads with `expectation_unmet:` and
  names the declared `forbidden` expectation, the report counts one hard
  `forbidden`, and the run exits 1

### Requirement: Advisory prose-only evidence tier
A verdict record that declares no expectation and whose covering evidence
is a prose-only claim SHALL set `advisory` to true, SHALL keep the closed
verdict `unknown` with the `prose_claim_not_interpretable` reason naming the
covering claim, and SHALL be counted in `advisory_counts` instead of
`verdict_counts`; it SHALL NOT enter the `forbidden + unsupported` exit
predicate, and the human view SHALL name the tier as non-gating and not a
model judgment.

#### Scenario: Prose-only advisory evidence is non-gating
- **GIVEN** a corpus trace that declares no expectation and whose only
  covering claim has a prose-only expr
- **WHEN** conform classifies the trace and builds the report
- **THEN** the record carries `verdict: unknown`, `advisory: true`, and a
  `prose_claim_not_interpretable` reason naming the claim; the record is
  counted in `advisory_counts` and absent from `verdict_counts`; the run
  exits 0; and the human view names the advisory tier as non-gating
