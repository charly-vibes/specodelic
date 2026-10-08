# hooks Specification

## Purpose
Let `spk hooks install` wire `spk lint openspec` as a pre-commit
command so dual-format drift fails before the commit lands, while the
repo's existing hook ownership (beads' `core.hooksPath`, chained to
lefthook) keeps flowing untouched.

## Constraints

| id                   | kind      | expr                                                                                                                              | traces_to |
|----------------------|-----------|------------------------------------------------------------------------------------------------------------------------------------|-----------|
| chain_preserved      | invariant | `wiring is purely additive: existing commands, entries, and their order in the lefthook config are never dropped, reordered, or wrapped` | [[hooks]]  |
| no_foreign_writes    | invariant | `no install or uninstall path ever writes .git/hooks hook files, .beads/hooks hook files, or the core.hooksPath config`              | [[hooks]]  |
| honest_anchor        | invariant | `a config whose stage or commands structure cannot be anchored is refused without modification, with a labeled error and a hint`     | [[hooks]]  |
| idempotent_lifecycle | invariant | `install on a wired repo and uninstall on an unwired repo are successful no-ops that leave the config byte-identical`                | [[hooks]]  |

## Model

### States
- `unwired`
- `wired`
- `refused`

### Transitions

| id                | from     | to       | guard                                                      |
|-------------------|----------|----------|-------------------------------------------------------------|
| install_ok        | unwired  | wired    | `lefthook config exists and the stage is anchorable`         |
| reinstall_ok      | wired    | wired    | `the managed block is already present`                       |
| refuse            | unwired  | refused  | `no lefthook config, unsupported framework, or no anchor`    |
| uninstall_ok      | wired    | unwired  | `the managed block markers are present`                      |
| uninstall_notwired| unwired  | unwired  | `no managed block present — reported, not an error`          |

## Properties

| id                    | kind | derives_from                   | generator                                    | predicate                                                              |
|-----------------------|------|--------------------------------|----------------------------------------------|-------------------------------------------------------------------------|
| additive_wiring       | unit | [[hooks.chain_preserved]]       | `fixture config with an existing pre-commit commands mapping` | `existing entries unchanged and lefthook parses the config`            |
| idempotent_install    | unit | [[hooks.idempotent_lifecycle]]  | `install run twice on a fixture`             | `config byte-identical after the second install`                        |
| foreign_refused       | unit | [[hooks.no_foreign_writes]]     | `fixture with a bd-owned pre-commit hook`    | `.git/hooks and .beads/hooks hook files byte-identical after install`   |
| clean_uninstall       | unit | [[hooks.idempotent_lifecycle]]  | `wired fixture, then uninstall`              | `rest of the config byte-identical and the gate entry gone`             |
| unsupported_framework | unit | [[hooks.honest_anchor]]         | `fixture with a husky-sigil pre-commit`      | `install errors labeling the framework and the tree is untouched`       |

## ADDED Requirements

### Requirement: spk hooks install wires the gate additively into the lefthook config
The system SHALL provide `spk hooks install` that injects a
marker-guarded managed block wiring the command `spk lint openspec`
into the `pre-commit:` stage of the repository's lefthook config
(`lefthook.yml` or `lefthook.yaml`), idempotently — and when the stage
already has a `commands:` mapping, the entry SHALL be inserted inside
that mapping rather than as a second `commands:` key. Indentation
SHALL be inferred from the stage's own child entries (2 spaces for an
empty stage), never assumed. The install SHALL run the wired gate
command once as a dry-run and report its outcome over the envelope —
warning, never failing, when the gate would fail immediately.

#### Scenario: entry added inside an existing commands mapping
- **WHEN** the repo's `lefthook.yml` has a `pre-commit:` stage whose
  section already contains a `commands:` mapping with other entries
- **AND** `spk hooks install` runs
- **THEN** the gate entry is inserted inside the existing
  `commands:` mapping as a marker-guarded block
- **AND** the entry's indentation matches the stage's existing child
  entries
- **AND** the pre-existing entries and their order are unchanged
- **AND** lefthook still parses the config (no duplicate
  `commands:` key is created)

#### Scenario: full wrapper injected when the stage has no commands
- **WHEN** the repo's lefthook config has no `pre-commit:` section, or
  the section has no `commands:` mapping
- **AND** `spk hooks install` runs
- **THEN** the full `commands:` wrapper containing the marker-guarded
  gate entry is injected into the stage, formatted at the stage's own
  children indentation

#### Scenario: install is idempotent
- **WHEN** `spk hooks install` runs on a repo whose config already
  contains the managed block
- **THEN** the command succeeds reporting the already-wired outcome
  and the config is byte-identical

#### Scenario: gate that would fail immediately is a warning, not a trap
- **WHEN** `spk hooks install` runs in a repo whose openspec tree does
  not yet pass the gate (for example, deltas not yet migrated to dual
  format)
- **THEN** the install succeeds and the envelope reports the dry-run
  outcome on the warnings channel
- **AND** the warning carries the failure summary and an
  `spk hooks uninstall` escape hint

### Requirement: hook wiring never claims foreign ownership
The system SHALL never claim `core.hooksPath`, never write hook files
under `.git/hooks/` or `.beads/hooks/`, and never modify a hook file
owned by another tool — `spk hooks install` wires only the lefthook
config, so an existing chain such as beads' `.beads/hooks` shims →
lefthook keeps flowing unchanged.

#### Scenario: beads-owned hook chain is left untouched
- **WHEN** `spk hooks install` runs in a repo where
  `core.hooksPath` points at beads-owned hook shims that chain to
  lefthook
- **THEN** the hook files under `.beads/hooks/` and `.git/hooks/` are
  byte-identical after the run
- **AND** the wiring landed only in the lefthook config

### Requirement: unsupported framework or missing config is a labeled error with a hint
The system SHALL refuse to wire, with a labeled error and a
remediation hint and no file modification, when the repository has no
lefthook config (never creating one), when the detected hook framework
is unsupported (husky, prek), or when the target stage or its
`commands:` key cannot be anchored (quoted, indented, unrecognized
structure, or unascertainable children indentation).

#### Scenario: no lefthook config
- **WHEN** `spk hooks install` runs in a repo with neither
  `lefthook.yml` nor `lefthook.yaml`
- **THEN** the command fails with an error naming the missing config
  and a hint to create one first — no config is created

#### Scenario: unsupported framework
- **WHEN** `spk hooks install` runs in a repo whose pre-commit hook
  carries the husky sigil and no lefthook config exists
- **THEN** the command fails with an error naming the detected
  framework and a manual-wiring hint, and no file is modified

#### Scenario: unanchorable structure
- **WHEN** the lefthook config's `commands:` key inside the target
  stage is quoted or otherwise not anchorable, or the stage's children
  indentation cannot be ascertained
- **THEN** the command fails with an anchor error and the config file
  is unchanged

### Requirement: spk hooks uninstall strips only the managed block
The system SHALL provide `spk hooks uninstall` that removes exactly
the lines between the managed-block markers (inclusive) from the
lefthook config — leaving the rest of the file byte-identical — and
SHALL report a successful no-op (not an error) when no block is
present. A stage section that install itself appended is left behind
as an empty stage section rather than removed.

#### Scenario: block removed, rest intact
- **WHEN** `spk hooks uninstall` runs on a wired repo
- **THEN** the gate entry and its markers are removed
- **AND** every other byte of the config is unchanged
- **AND** a stage section that install itself appended remains as an
  empty stage section (residue is documented, not hidden)

#### Scenario: unwired repo is a no-op
- **WHEN** `spk hooks uninstall` runs on a repo with no managed block
- **THEN** the command succeeds reporting the not-wired outcome and
  the config is unchanged

## Requirements
### Requirement: spk hooks install wires the gate additively into the lefthook config
The system SHALL provide `spk hooks install` that injects a
marker-guarded managed block wiring the command `spk lint openspec`
into the `pre-commit:` stage of the repository's lefthook config
(`lefthook.yml` or `lefthook.yaml`), idempotently — and when the stage
already has a `commands:` mapping, the entry SHALL be inserted inside
that mapping rather than as a second `commands:` key. Indentation
SHALL be inferred from the stage's own child entries (2 spaces for an
empty stage), never assumed. The install SHALL run the wired gate
command once as a dry-run and report its outcome over the envelope —
warning, never failing, when the gate would fail immediately.

#### Scenario: entry added inside an existing commands mapping
- **WHEN** the repo's `lefthook.yml` has a `pre-commit:` stage whose
  section already contains a `commands:` mapping with other entries
- **AND** `spk hooks install` runs
- **THEN** the gate entry is inserted inside the existing
  `commands:` mapping as a marker-guarded block
- **AND** the entry's indentation matches the stage's existing child
  entries
- **AND** the pre-existing entries and their order are unchanged
- **AND** lefthook still parses the config (no duplicate
  `commands:` key is created)

#### Scenario: full wrapper injected when the stage has no commands
- **WHEN** the repo's lefthook config has no `pre-commit:` section, or
  the section has no `commands:` mapping
- **AND** `spk hooks install` runs
- **THEN** the full `commands:` wrapper containing the marker-guarded
  gate entry is injected into the stage, formatted at the stage's own
  children indentation

#### Scenario: install is idempotent
- **WHEN** `spk hooks install` runs on a repo whose config already
  contains the managed block
- **THEN** the command succeeds reporting the already-wired outcome
  and the config is byte-identical

#### Scenario: gate that would fail immediately is a warning, not a trap
- **WHEN** `spk hooks install` runs in a repo whose openspec tree does
  not yet pass the gate (for example, deltas not yet migrated to dual
  format)
- **THEN** the install succeeds and the envelope reports the dry-run
  outcome on the warnings channel
- **AND** the warning carries the failure summary and an
  `spk hooks uninstall` escape hint

### Requirement: hook wiring never claims foreign ownership
The system SHALL never claim `core.hooksPath`, never write hook files
under `.git/hooks/` or `.beads/hooks/`, and never modify a hook file
owned by another tool — `spk hooks install` wires only the lefthook
config, so an existing chain such as beads' `.beads/hooks` shims →
lefthook keeps flowing unchanged.

#### Scenario: beads-owned hook chain is left untouched
- **WHEN** `spk hooks install` runs in a repo where
  `core.hooksPath` points at beads-owned hook shims that chain to
  lefthook
- **THEN** the hook files under `.beads/hooks/` and `.git/hooks/` are
  byte-identical after the run
- **AND** the wiring landed only in the lefthook config

### Requirement: unsupported framework or missing config is a labeled error with a hint
The system SHALL refuse to wire, with a labeled error and a
remediation hint and no file modification, when the repository has no
lefthook config (never creating one), when the detected hook framework
is unsupported (husky, prek), or when the target stage or its
`commands:` key cannot be anchored (quoted, indented, unrecognized
structure, or unascertainable children indentation).

#### Scenario: no lefthook config
- **WHEN** `spk hooks install` runs in a repo with neither
  `lefthook.yml` nor `lefthook.yaml`
- **THEN** the command fails with an error naming the missing config
  and a hint to create one first — no config is created

#### Scenario: unsupported framework
- **WHEN** `spk hooks install` runs in a repo whose pre-commit hook
  carries the husky sigil and no lefthook config exists
- **THEN** the command fails with an error naming the detected
  framework and a manual-wiring hint, and no file is modified

#### Scenario: unanchorable structure
- **WHEN** the lefthook config's `commands:` key inside the target
  stage is quoted or otherwise not anchorable, or the stage's children
  indentation cannot be ascertained
- **THEN** the command fails with an anchor error and the config file
  is unchanged

### Requirement: spk hooks uninstall strips only the managed block
The system SHALL provide `spk hooks uninstall` that removes exactly
the lines between the managed-block markers (inclusive) from the
lefthook config — leaving the rest of the file byte-identical — and
SHALL report a successful no-op (not an error) when no block is
present. A stage section that install itself appended is left behind
as an empty stage section rather than removed.

#### Scenario: block removed, rest intact
- **WHEN** `spk hooks uninstall` runs on a wired repo
- **THEN** the gate entry and its markers are removed
- **AND** every other byte of the config is unchanged
- **AND** a stage section that install itself appended remains as an
  empty stage section (residue is documented, not hidden)

#### Scenario: unwired repo is a no-op
- **WHEN** `spk hooks uninstall` runs on a repo with no managed block
- **THEN** the command succeeds reporting the not-wired outcome and
  the config is unchanged