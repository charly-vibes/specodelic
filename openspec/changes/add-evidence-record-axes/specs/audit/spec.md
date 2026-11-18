## ADDED Requirements

### Requirement: Evidence record axes are documented with implementation status

The audit capability SHALL carry a normative evidence-record table in
`docs/src/graded-evidence.md` enumerating every closed-set value of the
evidence vocabularies — conform's verdicts and audit's lifecycle classes —
with each row naming its axis, its code source, and a status of `implemented`
or `reserved` plus a named trigger.

#### Scenario: Every value appears exactly once

- **WHEN** a reader opens the evidence-record table
- **THEN** every `Verdict` and `AuditClass` value has exactly one row, and no value is listed twice

#### Scenario: A reserved row names its producer

- **WHEN** a row's status is `reserved`
- **THEN** the row carries a `trigger:` naming the change or ticket that will give the value a producer, or the gate fails naming the row

### Requirement: Evidence vocabulary drift is gated

A guard (`scripts/check_evidence_axes.py`, wired into the `ci` recipe) SHALL
compare the documented sets and the verdict-to-class mapping against
`src/conform.rs` and `src/audit.rs`, and SHALL fail naming the divergent value
when the two disagree in either direction, when a documented mapping differs
from `AuditClass::of`, or when a `reserved` row names a value already present
in a parsed closed set.

#### Scenario: A code value with no documented row fails the gate

- **WHEN** `Verdict` or `AuditClass` declares a value the table does not list
- **THEN** the guard exits non-zero naming the undocumented value and its code source

#### Scenario: A documented `implemented` row with no code value fails the gate

- **WHEN** the table lists a value marked `implemented` that neither enum declares
- **THEN** the guard exits non-zero naming the stale row

#### Scenario: A documented mapping that disagrees with the code fails the gate

- **WHEN** the table's verdict-to-class mapping differs from the corresponding `AuditClass::of` arm
- **THEN** the guard exits non-zero naming the verdict, the documented class, and the code's class

#### Scenario: A reserved value must not appear in a parsed closed set

- **WHEN** a row marked `reserved` names a value present in `Verdict` or `AuditClass`
- **THEN** the guard exits non-zero — either the row's status or the value's code presence is wrong

#### Scenario: Unparseable input fails closed

- **WHEN** the guard cannot extract the enum blocks or the table from their files
- **THEN** it exits non-zero with a remediation hint naming the file and the expected shape, emitting no success line

#### Scenario: A synchronized tree passes

- **WHEN** every parsed code value has exactly one documented row, every `implemented` row has a code value, the mappings agree, and every `reserved` row carries a trigger
- **THEN** the guard exits zero and emits the counts it compared