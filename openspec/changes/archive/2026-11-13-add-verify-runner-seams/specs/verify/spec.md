## ADDED Requirements

### Requirement: Properties-only scope verdict

`spk verify` SHALL expose a properties-only scope verdict that reports
`properties_failed` exactly when a compiled proptest! block failed, never
for a missing or stale model run, and never `verified` — the `verified`
transition keeps both gates required. The per-file envelope carries the
same distinction: the properties gate's state rides its own field, the
model gate's state another, so a scoped CI gate can read the properties
stage without the model stage hiding it.

#### Scenario: Failed block reports properties_failed

- **WHEN** a compiled proptest! block failed
- **THEN** the properties-only scope verdict reports `properties_failed`,
  with the same status, message and hint the full verdict uses for a
  failed property

#### Scenario: Missing model run never reports properties_failed

- **WHEN** the model-check run report is missing or stale and the
  properties gate passed
- **THEN** the properties-only scope verdict does not report
  `properties_failed`, and the full verdict names the model stage

#### Scenario: Scoped pass is never verified

- **WHEN** every compiled proptest! block passed
- **THEN** the properties-only scope verdict reports `properties_passed`,
  never `verified` — a properties-only pass can never be mistaken for a
  verified file
