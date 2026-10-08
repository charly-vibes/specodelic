---
id: lint.findings
kind: intent
statement: "THE linter SHALL make every finding self-describing by carrying its stable rule id and a one-line rule semantics string in both JSON and human output."
---

# lint-findings Specification

## Purpose
Turn each lint finding into its own documentation: a stable
`linter.<name>` rule id plus a one-line semantics string, present in
the JSON envelope and the human-readable output, so agents and humans
can act on a finding without repository access.

## Constraints

| id               | kind      | expr                                                                                                                      | traces_to |
|------------------|-----------|----------------------------------------------------------------------------------------------------------------------------|-----------|
| process_lifecycle | invariant | `the capability advances through its declared lifecycle states under the repo's change process — each stage transition fires only when its stage gate holds` | [[lint.findings]] |
| self_describing  | invariant | `every finding carries rule_id linter.<name> and a non-empty one-line rule_semantics sourced from the rule table, in JSON and human output` | [[lint.findings]]  |

## Model

### States
- `found`
- `emitted`

### Transitions

| id     | from  | to       | guard                     |
|--------|-------|----------|---------------------------|
| emit   | found | emitted  | [[lint.findings.self_describing]]  |

## Properties

| id       | kind | derives_from              | generator             | predicate                                   |
|----------|------|---------------------------|-----------------------|---------------------------------------------|
| process_lifecycle_checked | unit | [[lint.findings.process_lifecycle]] | `lifecycle_model_present()` | `check(file) == passed` |
| p_finding | unit | [[lint.findings.self_describing]] | `arbitrary_finding()` | `finding.rule_id stable ∧ semantics non-empty in both output modes` |

## Requirements
### Requirement: Self-Describing Lint Findings

Every lint finding SHALL carry a `rule_id` (the stable
`linter.<name>` identifier) and a one-line `rule_semantics` string
stating what the rule requires, in both the JSON envelope and the
human-readable output, so the finding is documentation for itself.
Skipped-file notices (e.g. non-spec markdown without frontmatter) are
emitted on the envelope's `warnings` channel and are out of scope for
this requirement.

#### Scenario: Agent reads a finding without repo access

- **WHEN** `spk lint bad-spec.md --format json` runs on a file whose
  `statement` matches no EARS pattern
- **THEN** the finding's `rule_id` is `linter.ears_syntax` and
  `rule_semantics` is a non-empty one-line description of the EARS
  requirement

#### Scenario: Human output shows the rule id

- **WHEN** `spk lint bad-spec.md --human` runs on an invalid file
- **THEN** each printed finding includes its `rule_id`
