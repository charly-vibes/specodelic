---
id: migrate
kind: intent
statement: "WHEN an openspec delta file is migrated, THE migrate capability SHALL wrap it in place into the dual-format four-layer skeleton — preserving the delta text byte-for-byte, mirroring the ADDED Requirements section into a byte-identical Requirements mirror, and refusing any file that already carries the mirror."
---

# migrate Specification

## Purpose
Give adopters a tool surface for the dual-format migration recipe: wrap an
existing openspec delta in place — frontmatter, wired scaffold layers, and
the byte-identical Requirements mirror — instead of hand-editing (sed
one-liners on the mirror were the top onboarding cost, gh#6 item 1). The
scaffold is a marked skeleton, never derived prose; finishing it is the
author's job.

## Constraints

| id                      | kind      | expr                                                                                                                                                                    | traces_to |
|-------------------------|-----------|-------------------------------------------------------------------------------------------------------------------------------------------------------------------------|-----------|
| process_lifecycle | invariant | `the capability advances through its declared lifecycle states under the repo's change process — each stage transition fires only when its stage gate holds` | [[migrate]] |
| delta_text_preserved    | invariant | `∀ migrated file: the ADDED Requirements section body and any pre-existing frontmatter appear in the result byte-for-byte — no rewrite, no reflow, no dropped heading` | [[migrate]]  |
| mirror_byte_identical   | invariant | `the generated Requirements mirror section body == the ADDED Requirements section body (byte-exact slice)`                                                                | [[migrate]]  |
| refuse_already_migrated | invariant | `a file carrying both frontmatter and a Requirements mirror is refused (exit 2), never rewritten; a file with neither section, or with a mirror but no ADDED, is refused as not-a-delta` | [[migrate]]  |
| merge_adds_only_missing | invariant | `only missing pieces are inserted — generated frontmatter, missing layer skeletons, missing mirror; existing sections pass through untouched`                             | [[migrate]]  |
| scaffold_lints_clean    | invariant | `the generated scaffold (frontmatter + wired scaffold_* placeholder rows + one-state model) exits spk lint 0 with zero findings as written — the author starts from a green baseline` | [[migrate]]  |
| dry_run_writes_nothing  | invariant | `--dry-run emits the resulting content as data and never writes the file; the on-disk bytes are unchanged`                                                               | [[migrate]]  |

## Model

### States

- `unwrapping`
- `wrapping`
- `written`
- `refused`

### Transitions

| id           | from       | to        | guard                                    |
|--------------|------------|-----------|-------------------------------------------|
| begin_wrap   | unwrapping | wrapping  | [[migrate.delta_text_preserved]]  |
| write_result | wrapping   | written   | [[migrate.mirror_byte_identical]]  |
| dry_run_pass | wrapping   | wrapping  | [[migrate.dry_run_writes_nothing]]  |
| refuse       | unwrapping | refused   | [[migrate.refuse_already_migrated]]  |
| merge_only   | wrapping   | written   | [[migrate.merge_adds_only_missing]]  |
| verify_green | written    | written   | [[migrate.scaffold_lints_clean]]  |

## Properties

| id                        | kind | derives_from                     | generator                        | predicate                                        |
|---------------------------|------|----------------------------------|----------------------------------|--------------------------------------------------|
| process_lifecycle_checked | unit | [[migrate.process_lifecycle]] | `lifecycle_model_present()` | `check(file) == passed` |
| p_delta_text_preserved    | unit | [[migrate.delta_text_preserved]]    | `delta_with(frontmatter: any)`   | `migrate(file).added_body == original.added_body` |
| p_mirror_byte_identical   | unit | [[migrate.mirror_byte_identical]]   | `delta_with(sections: any)`      | `result.mirror_body == result.added_body`         |
| p_refuse_already_migrated | unit | [[migrate.refuse_already_migrated]] | `dual_format_file()`             | `migrate(file) == refused ∧ disk_unchanged`       |
| p_merge_adds_only_missing | unit | [[migrate.merge_adds_only_missing]] | `delta_with(partial_layers: any)`| `existing_sections(result) == existing_sections(original)` |
| p_scaffold_lints_clean    | unit | [[migrate.scaffold_lints_clean]]    | `any_plain_delta()`              | `lint(migrate(file)) == zero_findings`            |
| p_dry_run_writes_nothing  | unit | [[migrate.dry_run_writes_nothing]]  | `any_plain_delta()`              | `dry_run(file).disk_bytes == original.disk_bytes` |

## ADDED Requirements

### Requirement: In-place wrap of a plain delta
The system SHALL rewrite the given delta file in place — inserting generated
frontmatter (`id: spec`, kind `intent`, an EARS scaffold statement), the
wired scaffold layers, and the Requirements mirror — while preserving the
ADDED Requirements text byte-for-byte.

#### Scenario: Plain delta becomes dual-format
- **WHEN** `spk migrate delta.md` runs on a file with an ADDED Requirements section and no frontmatter
- **THEN** the file gains frontmatter, scaffold layers, and the mirror, and the ADDED body is unchanged

#### Scenario: Scaffold lints clean
- **WHEN** the migration completes
- **THEN** `spk lint <file>` exits 0 with zero findings

### Requirement: Merge adds only missing pieces
The system SHALL keep every already-present section (frontmatter, layers)
verbatim when migrating a partially migrated file, inserting only the
missing frontmatter, layers, or mirror.

#### Scenario: Frontmatter kept verbatim
- **WHEN** the file already carries frontmatter
- **THEN** it appears unchanged in the result and no scaffold statement is inserted

### Requirement: Refuse already migrated or non-delta files
The system SHALL refuse (exit 2) any file that already carries both
frontmatter and a Requirements mirror, any file whose only Requirements
heading is the mirror (a plain spec), and any file carrying neither
section.

#### Scenario: Second run is a refusal
- **WHEN** `spk migrate` runs on an already migrated file
- **THEN** the command fails with an already-migrated message and the file is unchanged

### Requirement: Dry-run writes nothing
The system SHALL support `--dry-run`, emitting the resulting content as
data without writing the file.

#### Scenario: Dry-run leaves disk untouched
- **WHEN** `spk migrate --dry-run delta.md` runs
- **THEN** the on-disk bytes are unchanged and the envelope data carries the would-be content

## Requirements

### Requirement: In-place wrap of a plain delta
The system SHALL rewrite the given delta file in place — inserting generated
frontmatter (`id: spec`, kind `intent`, an EARS scaffold statement), the
wired scaffold layers, and the Requirements mirror — while preserving the
ADDED Requirements text byte-for-byte.

#### Scenario: Plain delta becomes dual-format
- **WHEN** `spk migrate delta.md` runs on a file with an ADDED Requirements section and no frontmatter
- **THEN** the file gains frontmatter, scaffold layers, and the mirror, and the ADDED body is unchanged

#### Scenario: Scaffold lints clean
- **WHEN** the migration completes
- **THEN** `spk lint <file>` exits 0 with zero findings

### Requirement: Merge adds only missing pieces
The system SHALL keep every already-present section (frontmatter, layers)
verbatim when migrating a partially migrated file, inserting only the
missing frontmatter, layers, or mirror.

#### Scenario: Frontmatter kept verbatim
- **WHEN** the file already carries frontmatter
- **THEN** it appears unchanged in the result and no scaffold statement is inserted

### Requirement: Refuse already migrated or non-delta files
The system SHALL refuse (exit 2) any file that already carries both
frontmatter and a Requirements mirror, any file whose only Requirements
heading is the mirror (a plain spec), and any file carrying neither
section.

#### Scenario: Second run is a refusal
- **WHEN** `spk migrate` runs on an already migrated file
- **THEN** the command fails with an already-migrated message and the file is unchanged

### Requirement: Dry-run writes nothing
The system SHALL support `--dry-run`, emitting the resulting content as
data without writing the file.

#### Scenario: Dry-run leaves disk untouched
- **WHEN** `spk migrate --dry-run delta.md` runs
- **THEN** the on-disk bytes are unchanged and the envelope data carries the would-be content
