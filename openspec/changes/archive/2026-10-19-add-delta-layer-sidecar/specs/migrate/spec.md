## MODIFIED Requirements

### Requirement: In-place wrap of a plain delta
The system SHALL rewrite the given delta file pair in place — leaving the
plain delta untouched and creating the sibling `layer.md` sidecar with
generated frontmatter (the parent-directory-derived id with a `.layer`
suffix, kind `intent`, an EARS scaffold statement) and the wired scaffold
layers — while preserving the delta text byte-for-byte (Revision 19: the
dual-format wrap and its `## Requirements` mirror are retired).

#### Scenario: Plain delta becomes dual-format
- **WHEN** `spk migrate delta.md` runs on a file with an ADDED Requirements
  section and no frontmatter
- **THEN** the delta file is byte-unchanged, a sibling `layer.md` is
  created with frontmatter and scaffold layers (the retired dual wrap is
  gone — the delta itself stays plain), and `spk lint` over the pair
  exits 0 with zero findings

#### Scenario: Plain delta gains a sidecar
- **WHEN** the wrap completes on a plain delta
- **THEN** the change directory holds the pair — the byte-unchanged delta
  plus the generated sidecar — and no mirror exists to maintain

#### Scenario: Scaffold lints clean
- **WHEN** the migration completes
- **THEN** `spk lint <layer.md>` exits 0 with zero findings as written —
  the author starts from a green baseline

### Requirement: Merge adds only missing pieces
The system SHALL keep every already-present piece verbatim when migrating
a partially migrated pair — an existing sidecar passes through untouched
and no scaffold statement is inserted — generating only the missing
sidecar pieces (Revision 19: the mirror is no longer a piece that can be
missing).

#### Scenario: Frontmatter kept verbatim
- **WHEN** the pair already carries a `layer.md` with frontmatter
- **THEN** it appears unchanged in the result and no scaffold statement is
  inserted

#### Scenario: Existing sidecar kept verbatim
- **WHEN** the merge runs over a partially migrated pair
- **THEN** every already-present piece (delta text, sidecar tables) passes
  through verbatim and only missing sidecar pieces are generated

### Requirement: Refuse already migrated or non-delta files
The system SHALL refuse (exit 2) any file that is not a plain delta in
need of a sidecar — a file carrying frontmatter (already a specodelic
file), a file whose only Requirements heading is a deployed spec's
`## Requirements` section, and a file carrying no Requirements heading at
all.

#### Scenario: Frontmatter-carrying file is a refusal
- **WHEN** `spk migrate` runs on a file that already carries frontmatter
- **THEN** the command fails with an already-migrated message and no file
  is changed

#### Scenario: Second run is a refusal
- **WHEN** `spk migrate` runs twice over the same change directory
- **THEN** the second run fails with an already-migrated message and both
  files are unchanged
