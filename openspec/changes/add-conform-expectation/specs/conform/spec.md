## ADDED Requirements

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