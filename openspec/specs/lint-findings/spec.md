# lint-findings Specification

## Purpose
TBD - created by archiving change add-embedded-aix-guide. Update Purpose after archive.
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

