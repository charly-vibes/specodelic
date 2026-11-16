## ADDED Requirements

### Requirement: Merged evidence row keys are unique

The merged audit report SHALL never carry two rows with the same
`scenario_id`: when a FOLDED row (`bind:<owner-id>` or
`pack:<pack_id>:<subject_id>`) collides with a key already present — an oracle
row whose corpus id is spelled exactly like a folded key included — the folded
row SHALL take the first free `~<n>` suffix while the row already present
keeps its id, both rows SHALL survive with their own classes and counts, and
the exit contract SHALL be unchanged.

#### Scenario: Oracle id spelled like a bind row key

- **WHEN** an oracle corpus id equals a folded bind row's key
  (`bind:locked`)
- **THEN** the oracle row keeps `bind:locked` with its oracle class, the
  folded row is emitted as `bind:locked~2` with class `binds_nothing`,
  and both rows are present in the report

#### Scenario: A pack row collision follows the same rule

- **WHEN** a folded pack row's key collides with a key already in the
  report
- **THEN** the pack row takes the first free `~<n>` suffix and the row
  already present keeps its id — no row is dropped or merged

#### Scenario: No duplicate ids survive the merge

- **WHEN** any folded row collides with an existing row key
- **THEN** the merged report carries no duplicate `scenario_id`, so the
  prior-report diff's `scenario_id` lookup always addresses exactly one
  row

### Requirement: Disambiguation is deterministic

Folded-row disambiguation SHALL be a pure function of the corpus rows —
no counters, random state, or map-iteration order — so repeated assembly
over identical inputs yields identical keys and byte-identical
serialization, and a `--before` diff keeps matching every oracle row by
its recorded corpus id.

#### Scenario: Same corpus merges to the same keys

- **WHEN** the same oracle and folded rows are merged twice
- **THEN** the two merged reports carry identical `scenario_id` sequences

#### Scenario: Oracle evidence is never renamed

- **WHEN** a suffix is assigned in a collision
- **THEN** only the folded row is renamed; every oracle row keeps its
  corpus id, so diffing against evidence recorded in a prior report still
  resolves
