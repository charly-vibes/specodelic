## ADDED Requirements

### Requirement: Shared fragment definitions

A spec file MAY carry an optional `## Fragments` section of def-lines
(`def <name>:` followed by a fenced rust body); any fragment cell in the same
file MAY reference a definition by name, and compile SHALL inline each
referenced definition's body at every use site. The linter treats the
section as opaque prose — it carries no frontmatter, tables, or typed
references. Referencing, hygiene, and termination behaviors are specified in
the scenarios.

#### Scenario: Helper inlines at each use site

- **WHEN** a file defines one helper and two property fragments reference it
- **THEN** each emitted block carries the helper's body inlined, and the
  helper definition itself is emitted once per use site — not referenced by
  a generated indirection

#### Scenario: Unknown reference fails labeled

- **WHEN** a fragment references a name no `## Fragments` row defines
- **THEN** compile fails labeled, naming the unresolved reference

#### Scenario: Name collision fails labeled

- **WHEN** a definition's name collides with a generated symbol
- **THEN** compile fails labeled, naming both symbols — never a silent shadow

#### Scenario: Helper cycle fails labeled

- **WHEN** definitions reference each other in a cycle (`def a:` uses `b`,
  `def b:` uses `a`)
- **THEN** compile fails labeled naming the cycle — inlining terminates by
  construction

#### Scenario: Transitive reference inlines fully

- **WHEN** a fragment references a helper that references another helper
- **THEN** both bodies are inlined at the use site

#### Scenario: Unused definition warns

- **WHEN** a definition is referenced by zero fragment cells
- **THEN** compile emits a dead-helper warning on the warnings channel and
  still succeeds

#### Scenario: Absent section preserves bytes

- **WHEN** a file has no `## Fragments` section
- **THEN** its compiled artifacts are byte-identical to the pre-change
  compiler's output for that file
