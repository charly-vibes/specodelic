## ADDED Requirements

### Requirement: Shared fragment definitions

A spec file MAY carry an optional `## Fragments` section whose rows define
named helper bodies (`def <name>:` + a fenced rust body); any fragment cell
in the same file MAY reference a definition by name, and compile SHALL inline
each referenced definition's body at every use site, applying the fragment
hygiene rule to helper bodies exactly as to fragment bodies.

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

#### Scenario: Unused definition warns

- **WHEN** a definition is referenced by zero fragment cells
- **THEN** compile emits a dead-helper warning on the warnings channel and
  still succeeds

#### Scenario: Absent section preserves bytes

- **WHEN** a file has no `## Fragments` section
- **THEN** its compiled artifacts are byte-identical to the pre-change
  compiler's output for that file
