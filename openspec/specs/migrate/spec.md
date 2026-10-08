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
| rekey_derives_real_id   | invariant | `--rekey derives the real id by the naming law — the parent directory name for a spec.md file (- ⇔ ., _ literal) — and replaces the frontmatter `id: spec` line with it; every [[spec]] ref re-keys to [[<derived-id>]] and every [[spec. prefix to [[<derived-id>.; every other byte stays untouched` | [[migrate]] |
| rekey_refusals          | invariant | `--rekey on a file without frontmatter is refused (a plain delta takes the wrap path); on a spec.md with no derivable id (bare, or parent dir named `spec`) it is refused; on a file whose id is already not `spec` it is an idempotent no-op with a warning, never rewritten` | [[migrate]] |

## Model

### States

- `unwrapping`
- `wrapping`
- `written`
- `refused`
- `rekeyed`

### Transitions

| id           | from       | to        | guard                                    |
|--------------|------------|-----------|-------------------------------------------|
| begin_wrap   | unwrapping | wrapping  | [[migrate.delta_text_preserved]]  |
| write_result | wrapping   | written   | [[migrate.mirror_byte_identical]]  |
| dry_run_pass | wrapping   | wrapping  | [[migrate.dry_run_writes_nothing]]  |
| refuse       | unwrapping | refused   | [[migrate.refuse_already_migrated]]  |
| merge_only   | wrapping   | written   | [[migrate.merge_adds_only_missing]]  |
| verify_green | written    | written   | [[migrate.scaffold_lints_clean]]  |
| rekey_result | unwrapping | rekeyed   | [[migrate.rekey_derives_real_id]]  |
| rekey_refuse | unwrapping | refused   | [[migrate.rekey_refusals]]  |

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
| p_rekey_derives_real_id   | unit | [[migrate.rekey_derives_real_id]] | `legacy_id_spec_dual_format_file()` | `rekey(file).id == parent_dir_derived ∧ refs re-keyed ∧ rest byte-identical` |
| p_rekey_refusals          | unit | [[migrate.rekey_refusals]] | `plain_delta_or_bare_spec_md()` | `rekey(file) == refused ∧ disk_unchanged ∧ second_run_is_noop` |

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


### Requirement: Rekey an id:spec dual-format file to its real id
The system SHALL support `--rekey` on the migrate command: a file
carrying frontmatter with `id: spec` SHALL be rewritten in place with
the real id derived by the naming law — the parent directory name for a
`spec.md` file (`-` ⇔ `.`, `_` literal) — every `[[spec]]` reference
re-keyed to `[[<derived-id>]]` and every `[[spec.` prefix re-keyed to
`[[<derived-id>.`, and every other byte of the file left untouched. A
file without frontmatter SHALL be refused (it is a plain delta — the
wrap path's job); a bare `spec.md` with no parent directory SHALL be
refused (no derivable id); a file whose id is already not `spec` SHALL
be a no-op with a warning, never rewritten, so a directory-wide rekey
sweep is idempotent.

#### Scenario: Legacy dual-format file rekeys
- **WHEN** `spk migrate openspec/specs/ge-cli/spec.md --rekey` runs on a file declaring `id: spec` with `[[spec.c1]]` refs
- **THEN** the file declares `id: ge.cli`, its refs read `[[ge.cli]]` / `[[ge.cli.c1]]`, and everything else is byte-identical

#### Scenario: Refusals never rewrite
- **WHEN** `--rekey` runs on a plain delta (no frontmatter) or a bare `spec.md` with no parent directory
- **THEN** the command fails (exit 2) with a labeled refusal naming the right path (the wrap path, or an unguessable id) and the file is unchanged

#### Scenario: Rekey sweep is idempotent
- **WHEN** `--rekey` runs twice, or on a file already carrying its real id
- **THEN** the second run is a success with a no-op warning and the file is unchanged

#### Scenario: Dry-run rekey writes nothing
- **WHEN** `spk migrate <file> --rekey --dry-run` runs
- **THEN** the on-disk bytes are unchanged and the envelope data carries the would-be content

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


### Requirement: Rekey an id:spec dual-format file to its real id
The system SHALL support `--rekey` on the migrate command: a file
carrying frontmatter with `id: spec` SHALL be rewritten in place with
the real id derived by the naming law — the parent directory name for a
`spec.md` file (`-` ⇔ `.`, `_` literal) — every `[[spec]]` reference
re-keyed to `[[<derived-id>]]` and every `[[spec.` prefix re-keyed to
`[[<derived-id>.`, and every other byte of the file left untouched. A
file without frontmatter SHALL be refused (it is a plain delta — the
wrap path's job); a bare `spec.md` with no parent directory SHALL be
refused (no derivable id); a file whose id is already not `spec` SHALL
be a no-op with a warning, never rewritten, so a directory-wide rekey
sweep is idempotent.

#### Scenario: Legacy dual-format file rekeys
- **WHEN** `spk migrate openspec/specs/ge-cli/spec.md --rekey` runs on a file declaring `id: spec` with `[[spec.c1]]` refs
- **THEN** the file declares `id: ge.cli`, its refs read `[[ge.cli]]` / `[[ge.cli.c1]]`, and everything else is byte-identical

#### Scenario: Refusals never rewrite
- **WHEN** `--rekey` runs on a plain delta (no frontmatter) or a bare `spec.md` with no parent directory
- **THEN** the command fails (exit 2) with a labeled refusal naming the right path (the wrap path, or an unguessable id) and the file is unchanged

#### Scenario: Rekey sweep is idempotent
- **WHEN** `--rekey` runs twice, or on a file already carrying its real id
- **THEN** the second run is a success with a no-op warning and the file is unchanged

#### Scenario: Dry-run rekey writes nothing
- **WHEN** `spk migrate <file> --rekey --dry-run` runs
- **THEN** the on-disk bytes are unchanged and the envelope data carries the would-be content

### Requirement: Dry-run writes nothing
The system SHALL support `--dry-run`, emitting the resulting content as
data without writing the file.

#### Scenario: Dry-run leaves disk untouched
- **WHEN** `spk migrate --dry-run delta.md` runs
- **THEN** the on-disk bytes are unchanged and the envelope data carries the would-be content
