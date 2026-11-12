## ADDED Requirements

### Requirement: Reference Typing enforcement

`spk lint` SHALL reject a corpus when any resolved `[[link]]` in a
reference field defined by the format's Reference Typing table targets a
row whose kind the table forbids, reading the allowed targets from the
Schema-as-data table (never hardcoded per field name), and each finding
SHALL carry the `linter.ref_kind_compatible` rule id, the rule's
one-line semantics, and a remediation hint.

#### Scenario: Guard citing a non-invariant constraint fires

- **WHEN** a transition's guard cites an `advisory` or `effect` constraint
- **THEN** lint exits 1 with a `linter.ref_kind_compatible` finding naming
  the guard target and the invariant-or-State requirement

#### Scenario: Emits targeting a non-effect constraint fires

- **WHEN** a state's `emits` cell cites a constraint whose kind is not
  `effect`
- **THEN** lint exits 1 with a `linter.ref_kind_compatible` finding

#### Scenario: Constraint traces rejected

- **WHEN** a constraint's `traces_to` cell cites a row that is not an Intent
- **THEN** lint exits 1 with a `linter.ref_kind_compatible` finding

#### Scenario: Derives-from at a non-constraint target fires

- **WHEN** a `unit` property's `derives_from` cell cites a row that is not
  a constraint (a law may derive from the law-Property it restates)
- **THEN** lint exits 1 with a `linter.ref_kind_compatible` finding

#### Scenario: Supersedes across kinds rejected

- **WHEN** a `supersedes` cell cites a row of a different kind than the row
  carrying it
- **THEN** lint exits 1 with a `linter.ref_kind_compatible` finding

#### Scenario: Satisfies and observes at wrong-kind contracts fire

- **WHEN** a `satisfies` cell cites a non-`extension_point` constraint, or
  an `observes` cell cites a non-`effect` constraint
- **THEN** lint exits 1 with a `linter.ref_kind_compatible` finding naming
  the requirement

#### Scenario: Uses targeting a non-profile file fires

- **WHEN** a `uses` cell cites a file whose frontmatter kind is not
  `profile`
- **THEN** lint exits 1 with a `linter.ref_kind_compatible` finding

#### Scenario: Kind-compatible references lint clean

- **WHEN** every typed cell cites a kind-compatible target (guard →
  invariant constraint or State, emits → effect constraint, derives_from →
  constraint or own-law restatement, satisfies → extension_point,
  observes → effect, uses → profile file, member-arm targets typed as
  their row)
- **THEN** lint reports no `ref_kind_compatible` findings

#### Scenario: Dangling target is not double-reported

- **WHEN** a typed cell cites an id that resolves to no row or file
- **THEN** exactly the `linter.total_refs` finding fires — no duplicate
  typing finding for the same dangling target

#### Scenario: Corpus stays green under enforcement

- **WHEN** the linter runs over its own corpus
- **THEN** zero `ref_kind_compatible` findings fire (dogfooding gate)
