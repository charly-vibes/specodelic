---
id: conform.layer
kind: intent
statement: "WHEN an oracle scenario record optionally declares `expect: forbidden` and conform classifies its trace, THE conform capability SHALL echo the declaration and its met or unmet outcome in the verdict record, SHALL account met expectations as non-gating agreement-tier evidence while keeping the mechanical verdict, SHALL account unmet expectations in the hard forbidden bucket so the run exits 1, SHALL preserve unchanged semantics for undeclared records, and SHALL account prose-only advisory records separately so they never gate."
---

# conform — dual-format layer (sidecar)

## Purpose
Let an oracle declare its own claim about a trace without disturbing the
closed verdict taxonomy: expectations are echoed and qualified in each
record, met expectations read as agreement instead of disagreement, unmet
expectations stay honest disagreements that gate, and prose-only evidence
is tiered as advisory so it never reads as a hard model judgment.

## Constraints

| id                       | kind      | expr                                                                                                                                                                                                                                                            | traces_to       |
|--------------------------|-----------|-----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|-----------------|
| closed_expectation_schema | invariant | `the scenario schema accepts an optional expect drawn only from {forbidden}; an out-of-set value or an unknown field is a corpus_parse refusal naming the field, the closed set, and the one-JSON-object-per-line remediation — never a silent "no expectation"` | [[conform.layer]] |
| expectation_echoed       | invariant | `every verdict record echoes the declared expectation in expect and its qualification in expectation_met, both null when no declaration exists`                                                                                                                  | [[conform.layer]] |
| agreement_tier           | invariant | `a met expectation is counted in agreement_counts keyed by the verdict spelling, excluded from verdict_counts and from the forbidden + unsupported exit predicate, exits 0, and renders as the oracle agreeing`                                                  | [[conform.layer]] |
| unmet_gates              | invariant | `an unmet expectation keeps the mechanical verdict, leads its reason with the expectation_unmet marker, counts in the hard forbidden bucket, and exits 1`                                                                                                          | [[conform.layer]] |
| absence_preserved        | invariant | `records without expect keep their previous verdict, reason, count placement, gating, and rendering, differing only by the two new null fields`                                                                                                                  | [[conform.layer]] |
| advisory_non_gating      | invariant | `undeclared prose-only covering evidence keeps the closed unknown verdict, is marked advisory: true, counted in advisory_counts, excluded from verdict_counts and the exit predicate, and named non-gating in the human view`                                       | [[conform.layer]] |

## Model

### States
- `gated`
- `classified`
- `tiered`
- `reported`

### Transitions

| id       | from       | to         | guard                                                                          |
|----------|------------|------------|--------------------------------------------------------------------------------|
| classify | gated      | classified | [[conform.layer.closed_expectation_schema]] ∧ [[conform.layer.expectation_echoed]] |
| tier     | classified | tiered     | [[conform.layer.agreement_tier]] ∧ [[conform.layer.unmet_gates]]                |
| report   | tiered     | reported   | [[conform.layer.absence_preserved]] ∧ [[conform.layer.advisory_non_gating]]    |

## Properties

| id         | kind | derives_from                                  | generator                     | predicate                                                                                                              |
|------------|------|-----------------------------------------------|-------------------------------|------------------------------------------------------------------------------------------------------------------------|
| p_schema   | unit | [[conform.layer.closed_expectation_schema]]   | `arbitrary_scenario_corpus()` | `invalid expect or unknown field ⇒ corpus_parse refusal naming field, closed set, and remediation; forbidden ⇒ parses`  |
| p_echo     | unit | [[conform.layer.expectation_echoed]]          | `arbitrary_declared_scenario()` | `record.expect equals the declaration ∧ record.expectation_met equals the verdict match`                              |
| p_agreement | unit | [[conform.layer.agreement_tier]]             | `met_forbidden_expectation()` | `agreement_counts counts the verdict ∧ verdict_counts excludes it ∧ exit 0`                                            |
| p_unmet    | unit | [[conform.layer.unmet_gates]]                 | `unmet_forbidden_expectation()` | `mechanical verdict retained ∧ reason leads expectation_unmet ∧ hard forbidden count grows ∧ exit 1`                  |
| p_absence  | unit | [[conform.layer.absence_preserved]]           | `arbitrary_undeclared_scenario()` | `expect null ∧ expectation_met null ∧ prior verdict, reason, count placement, and gating unchanged`                    |
| p_advisory | unit | [[conform.layer.advisory_non_gating]]         | `prose_only_covering_claim()` | `advisory true ∧ verdict unknown ∧ advisory_counts counted ∧ verdict_counts excluded ∧ exit 0`                          |

## Requirements
### Requirement: Layer sidecar contract

This sidecar is the dual-format layer for the `conform` capability and
SHALL be deployed verbatim by the archive companion; the plain delta in
`spec.md` beside it is its openspec face.

#### Scenario: Archive deploys the layer

- **WHEN** `just archive-change id=add-conform-expectation` runs after the
  change is implemented
- **THEN** the capability deploys with this layer attached fail-closed