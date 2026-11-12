---
id: migrate
kind: intent
statement: "WHEN an openspec delta file is migrated, THE migrate capability SHALL wrap it in place into the Revision 19 pair — leaving the plain delta byte-untouched and creating the sibling layer.md sidecar with generated frontmatter and wired scaffold layers — while refusing any file that is not a plain delta in need of a sidecar, supporting --rekey for id: spec legacy files, and upgrading a corpus across the known format revisions when --to-revision names a target revision."
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
| delta_text_preserved    | invariant | `∀ migrated pair: the given delta file's text is preserved byte-for-byte — the plain delta stays untouched; the wrap creates the sibling layer.md sidecar rather than rewriting the delta (Revision 19: the dual-format wrap and its ## Requirements mirror are retired)` | [[migrate]]  |
| sidecar_generated       | invariant | `the wrap produces the pair — the byte-unchanged plain delta plus the generated sibling layer.md carrying frontmatter (the parent-directory-derived id with a .layer suffix, kind intent, an EARS scaffold statement) and the wired scaffold layers; no mirror exists to maintain`                                                                | [[migrate]]  |
| refuse_already_migrated | invariant | `a file that is not a plain delta in need of a sidecar is refused (exit 2), never rewritten: a file carrying frontmatter (already a specodelic file), a file whose only Requirements heading is a deployed spec's ## Requirements section, and a file carrying no Requirements heading at all` | [[migrate]]  |
| merge_adds_only_missing | invariant | `only missing pieces are inserted when migrating a partially migrated pair — an existing sidecar passes through untouched and no scaffold statement is inserted; only missing sidecar pieces are generated (Revision 19: the mirror is no longer a piece that can be missing)`                             | [[migrate]]  |
| scaffold_lints_clean    | invariant | `the generated sidecar (frontmatter + wired scaffold_* placeholder rows + one-state model) exits spk lint 0 with zero findings as written — the author starts from a green baseline` | [[migrate]]  |
| dry_run_writes_nothing  | invariant | `--dry-run emits the resulting pair content as data and never writes a file; the on-disk bytes are unchanged`                                                               | [[migrate]]  |
| rekey_derives_real_id   | invariant | `--rekey derives the real id by the naming law — the parent directory name for a spec.md file (- ⇔ ., _ literal) — and replaces the frontmatter `id: spec` line with it; every [[spec]] ref re-keys to [[<derived-id>]] and every [[spec. prefix to [[<derived-id>.; every other byte stays untouched` | [[migrate]] |
| rekey_refusals          | invariant | `--rekey on a file without frontmatter is refused (a plain delta takes the wrap path); on a spec.md with no derivable id (bare, or parent dir named `spec`) it is refused; on a file whose id is already not `spec` it is an idempotent no-op with a warning, never rewritten` | [[migrate]] |
| upgrade_registry_core_owned | invariant | `the known format revisions resolve from a core-owned registry — each entry records the revision number, its date where the format corpus records one, a one-line change note, and an optional migration step; a revision whose change required no corpus rewrite carries no step and upgrades as a byte-stable no-op` | [[migrate]] |
| upgrade_steps_applied_in_order | invariant | `the upgrade applies the registered steps for the revisions between the corpus's source revision and the target revision in ascending revision order — each step is shape-guarded, so a file that does not carry that step's pre-shape is not rewritten by it` | [[migrate]] |
| upgrade_unknown_revision_refused | invariant | `an unknown source or target revision is refused (exit 2) with a labeled message naming the known revision range and a remediation hint — a refused upgrade writes no file` | [[migrate]] |
| upgrade_no_downgrade | invariant | `a target revision below the corpus's current revision — the oldest revision still satisfied by a file in the supplied set — is refused (exit 2) with a labeled message naming the current revision and the file that still carries the older shape — a refused upgrade writes no file` | [[migrate]] |
| upgrade_no_op_byte_identical | invariant | `a file that no registered step changes is never written — its bytes, its line endings and its final terminator are preserved exactly` | [[migrate]] |
| upgrade_atomic_no_partial_write | invariant | `the upgrade is all-or-nothing — when any file in the supplied set cannot be upgraded, every file in the set stays unchanged` | [[migrate]] |
| upgrade_scope_confined | invariant | `the upgrade reads and writes only the files under the supplied paths — it creates no file, so no sibling sidecar and no report file is ever written` | [[migrate]] |
| upgrade_conflicting_modes_refused | invariant | `the migrate verb's modes are exclusive — --to-revision together with --rekey, and --write together with --dry-run, are refused (exit 2) with a labeled message; neither flag is ever silently ignored` | [[migrate]] |

## Model

### States

- `unwrapping`
- `wrapping`
- `written`
- `refused`
- `rekeyed`
- `upgrading`
- `upgraded`

### Transitions

| id           | from       | to        | guard                                    |
|--------------|------------|-----------|-------------------------------------------|
| begin_wrap   | unwrapping | wrapping  | [[migrate.delta_text_preserved]]  |
| write_result | wrapping   | written   | [[migrate.sidecar_generated]]  |
| dry_run_pass | wrapping   | wrapping  | [[migrate.dry_run_writes_nothing]]  |
| refuse       | unwrapping | refused   | [[migrate.refuse_already_migrated]]  |
| merge_only   | wrapping   | written   | [[migrate.merge_adds_only_missing]]  |
| verify_green | written    | written   | [[migrate.scaffold_lints_clean]]  |
| rekey_result | unwrapping | rekeyed   | [[migrate.rekey_derives_real_id]]  |
| rekey_refuse | unwrapping | refused   | [[migrate.rekey_refusals]]  |
| upgrade_begin | unwrapping | upgrading | [[migrate.upgrade_registry_core_owned]]  |
| upgrade_applied | upgrading | upgraded | [[migrate.upgrade_steps_applied_in_order]]  |
| upgrade_refuse | unwrapping | refused | [[migrate.upgrade_unknown_revision_refused]]  |

## Properties

| id                        | kind | derives_from                     | generator                        | predicate                                        |
|---------------------------|------|----------------------------------|----------------------------------|--------------------------------------------------|
| process_lifecycle_checked | unit | [[migrate.process_lifecycle]] | `lifecycle_model_present()` | `check(file) == passed` |
| p_delta_text_preserved    | unit | [[migrate.delta_text_preserved]]    | `delta_with(frontmatter: any)`   | `migrate(file).delta_body == original.delta_body` |
| p_sidecar_generated       | unit | [[migrate.sidecar_generated]]   | `delta_with(sections: any)`      | `result.sidecar present with .layer-derived frontmatter ∧ result.delta byte-unchanged`         |
| p_refuse_already_migrated | unit | [[migrate.refuse_already_migrated]] | `file_with_frontmatter_or_non_delta()`             | `migrate(file) == refused ∧ disk_unchanged`       |
| p_merge_adds_only_missing | unit | [[migrate.merge_adds_only_missing]] | `pair_with(partial_sidecar: any)`| `existing_pieces(result) == existing_pieces(original)` |
| p_scaffold_lints_clean    | unit | [[migrate.scaffold_lints_clean]]    | `any_plain_delta()`              | `lint(migrate(file).sidecar) == zero_findings`            |
| p_dry_run_writes_nothing  | unit | [[migrate.dry_run_writes_nothing]]  | `any_plain_delta()`              | `dry_run(file).disk_bytes == original.disk_bytes` |
| p_rekey_derives_real_id   | unit | [[migrate.rekey_derives_real_id]] | `legacy_id_spec_dual_format_file()` | `rekey(file).id == parent_dir_derived ∧ refs re-keyed ∧ rest byte-identical` |
| p_rekey_refusals          | unit | [[migrate.rekey_refusals]] | `plain_delta_or_bare_spec_md()` | `rekey(file) == refused ∧ disk_unchanged ∧ second_run_is_noop` |
| p_upgrade_registry_core_owned | unit | [[migrate.upgrade_registry_core_owned]] | `known_revision(n)` | `∀ n ∈ registry: n.number is known ∧ (n.step absent ∨ n.step is shape-guarded)` |
| p_upgrade_steps_applied_in_order | unit | [[migrate.upgrade_steps_applied_in_order]] | `upgrade(corpus, target)` | `applied_steps == the registry steps between the source revision and the target revision ∧ ascending` |
| p_upgrade_unknown_revision_refused | unit | [[migrate.upgrade_unknown_revision_refused]] | `unknown_revision_request()` | `upgrade(corpus, target) == refused ∧ disk_unchanged` |
| p_upgrade_no_downgrade | unit | [[migrate.upgrade_no_downgrade]] | `target_below_current_revision()` | `upgrade(corpus, target) == refused ∧ disk_unchanged` |
| p_upgrade_no_op_byte_identical | unit | [[migrate.upgrade_no_op_byte_identical]] | `file_needing_no_step()` | `upgrade(file).bytes == original.bytes ∧ line_endings_preserved ∧ terminator_preserved` |
| p_upgrade_atomic_no_partial_write | unit | [[migrate.upgrade_atomic_no_partial_write]] | `corpus_with_one_unupgradable_file()` | `upgrade(corpus) == refused ∧ every file byte-unchanged` |
| p_upgrade_scope_confined | unit | [[migrate.upgrade_scope_confined]] | `corpus_under_supplied_paths()` | `written_paths ⊆ supplied_paths ∧ no file created` |
| p_upgrade_conflicting_modes_refused | unit | [[migrate.upgrade_conflicting_modes_refused]] | `conflicting_mode_request()` | `upgrade(corpus, target) == refused ∧ disk_unchanged` |

## Requirements

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

### Requirement: Rekey an id:spec dual-format file to its real id
The system SHALL support `--rekey` on the migrate command: a file
carrying frontmatter with `id: spec` SHALL be rewritten in place with
the real id derived by the naming law — the parent directory name for a
`spec.md` file (`-` ⇔ `.`, `_` literal) — every `[[spec]]` reference
re-keyed to `[[<derived-id>]]` and every `[[spec.` prefix re-keyed to
`[[<derived-id>.`, and every other byte of the file left untouched.

#### Scenario: Legacy dual-format file rekeys
- **WHEN** `spk migrate openspec/specs/ge-cli/spec.md --rekey` runs on a file declaring `id: spec` with `[[spec.c1]]` refs
- **THEN** the file declares `id: ge.cli`, its refs read `[[ge.cli]]` / `[[ge.cli.c1]]`, and everything else is byte-identical

#### Scenario: Dry-run rekey writes nothing
- **WHEN** `spk migrate <file> --rekey --dry-run` runs
- **THEN** the on-disk bytes are unchanged and the envelope data carries the would-be content

### Requirement: Rekey refusals keep the sweep idempotent
A file without frontmatter SHALL be refused (it is a plain delta — the
wrap path's job); a bare `spec.md` with no parent directory SHALL be
refused (no derivable id); a file whose id is already not `spec` SHALL
be a no-op with a warning, never rewritten, so a directory-wide rekey
sweep is idempotent.

#### Scenario: Refusals never rewrite
- **WHEN** `--rekey` runs on a plain delta (no frontmatter) or a bare `spec.md` with no parent directory
- **THEN** the command fails (exit 2) with a labeled refusal naming the right path (the wrap path, or an unguessable id) and the file is unchanged

#### Scenario: Rekey sweep is idempotent
- **WHEN** `--rekey` runs twice, or on a file already carrying its real id
- **THEN** the second run is a success with a no-op warning and the file is unchanged

### Requirement: Dry-run writes nothing
The system SHALL support `--dry-run`, emitting the resulting content as
data without writing the file.

#### Scenario: Dry-run leaves disk untouched
- **WHEN** `spk migrate --dry-run delta.md` runs
- **THEN** the on-disk bytes are unchanged and the envelope data carries the would-be content

### Requirement: Upgrade a corpus across the known format revisions
The system SHALL support `spk migrate --to-revision <N> [--from-revision
<R>] [--write] <paths...>`: the upgrade SHALL resolve the known format
revisions from the core-owned registry, SHALL apply the registered
migration steps for the revisions between the corpus's source revision
and the target revision in ascending revision order, and SHALL report
the outcome as envelope data. The upgrade SHALL NOT rewrite a file that
no registered step changes.

#### Scenario: Old-revision corpus upgrades to the target revision
- **WHEN** `spk migrate --to-revision 20 --write <corpus>` runs on a corpus whose capability file still declares the pre-Revision-18 `id: spec` shape
- **THEN** that file declares its parent-directory-derived id, its `[[spec…]]` refs read `[[<derived-id>…]]`, the corpus lints with zero findings, and the files that needed no step are byte-identical

#### Scenario: The source revision is declared or detected
- **WHEN** the upgrade runs over a corpus holding one file that still declares the pre-Revision-18 `id: spec` shape
- **THEN** the corpus's source revision resolves to the oldest revision the supplied files still satisfy, or to the revision declared by `--from-revision`, and the envelope data names the resolved source and target revisions

#### Scenario: Dry-run upgrade writes nothing
- **WHEN** `spk migrate --to-revision 20 <corpus>` runs without `--write`
- **THEN** the on-disk bytes are unchanged and the envelope data carries the would-be content of every file the upgrade would change

#### Scenario: The upgrade never touches files outside the supplied paths
- **WHEN** the upgrade runs over one directory of a corpus
- **THEN** no file outside that directory changes and no file is created beside the upgraded ones

#### Scenario: Conflicting modes are refused
- **WHEN** the upgrade is asked for together with `--rekey` (a different mode of the verb), or with both `--write` and `--dry-run`
- **THEN** the command fails (exit 2) with a labeled refusal, neither flag is silently ignored, and no file is changed

### Requirement: The revision registry is core-owned and thin
The system SHALL resolve the known format revisions from a core-owned
registry that records, for each known revision, its number, its date
(where the format corpus records one), a one-line change note, and an
OPTIONAL migration step — a revision whose change required no corpus
rewrite SHALL carry no step and SHALL upgrade as a byte-stable no-op.

#### Scenario: Known revisions resolve from the registry
- **WHEN** the upgrade resolves a target revision
- **THEN** the registry names the known revision range and carries a date and a change note for every known revision it records

### Requirement: Refusals label the unknown or impossible revision
An unknown source or target revision SHALL be refused (exit 2) with a
labeled message naming the known revision range and a remediation hint;
a target revision below the corpus's current revision SHALL be refused
(exit 2) with a labeled message naming the current revision and the
file that still carries the older shape. No file SHALL be written by a
refused upgrade.

#### Scenario: Unknown target revision is refused
- **WHEN** the upgrade is asked for a target revision outside the known range
- **THEN** the command fails (exit 2) with a labeled refusal naming the known revision range and a remediation hint, and no file is changed

#### Scenario: Unknown source revision is refused
- **WHEN** the upgrade is given a `--from-revision` outside the known range
- **THEN** the command fails (exit 2) with a labeled refusal naming the known revision range and a remediation hint, and no file is changed

#### Scenario: Target below the current revision is refused
- **WHEN** the upgrade is asked for a target revision below the corpus's current revision
- **THEN** the command fails (exit 2) with a labeled refusal naming the current revision and the file that still carries the older shape, and no file is changed

### Requirement: The upgrade is atomic, byte-stable and scope-confined
The upgrade SHALL write only the files whose content a registered step
changed, SHALL leave every other file byte-identical (its line endings
and final terminator preserved), SHALL create no file, and SHALL write
nothing at all when any file in the supplied set cannot be upgraded.

#### Scenario: Unchanged files stay byte-identical
- **WHEN** the upgrade runs over a corpus whose files need no step
- **THEN** every file's bytes are unchanged — CRLF line endings and the absence of a final terminator are preserved exactly

#### Scenario: A failed upgrade writes nothing
- **WHEN** the upgrade runs over a set in which one file cannot be upgraded
- **THEN** the command fails (exit 2) and every file in the set is unchanged
