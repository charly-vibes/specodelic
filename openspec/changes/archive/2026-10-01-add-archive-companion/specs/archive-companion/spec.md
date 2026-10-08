# archive-companion Specification

## Purpose
Make the dual-format corpus survive the openspec archive round-trip at
the tool level: `openspec archive` regenerates deployed specs from
parsed deltas and drops frontmatter + the Constraints/Model/Properties
tables; the companion performs the archive with spec application skipped
and deploys the archived deltas verbatim, so the deployed spec and its
archived delta are byte-identical dual-format files. Fail-closed on any
delta lacking the layer — the tool must never be the thing that deploys
a stripped spec.

## Constraints

| id                      | kind      | expr                                                                                                                                                                                                                                                                                   | traces_to |
|-------------------------|-----------|------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|-----------|
| layer_preserved         | invariant | `after the command completes, every deployed openspec/specs/<cap>/spec.md touched by the change is byte-identical to its archived delta under openspec/changes/archive/*-<id>/specs/<cap>/spec.md — frontmatter and all three tables included`                                              | [[archive.companion]]  |
| openspec_not_forked     | invariant | `archive semantics come from the openspec CLI itself (invoked with spec application skipped); the companion never re-implements delta merging, requirement parsing, or directory moves`                                                                                                     | [[archive.companion]]  |
| fail_closed_unverifiable | invariant | `a delta lacking the dual-format layer (no frontmatter opener or no Constraints section) is refused before any copy, the whole command fails listing every refused delta, and no deployed spec is written`                                                                                  | [[archive.companion]]  |
| idempotent_rerun        | invariant | `re-running the command for an already-archived change skips the openspec invocation and re-executes the restore; the result is byte-identical to a first run`                                                                                                                              | [[archive.companion]]  |
| newest_archive_wins     | invariant | `when multiple archive directories match the change id, the lexicographically greatest name is used and is named in the envelope — ambiguity is observable, never silent`                                                                                                                    | [[archive.companion]]  |
| dry_run_never_mutates   | invariant | `--dry-run resolves the restore plan (archive directory, delta list, dual-format verification) and emits the envelope without invoking openspec and without writing any file`                                                                                                                | [[archive.companion]]  |
| empty_restore_valid     | invariant | `a change with no spec deltas completes successfully with an empty restored list — nothing to restore is a stated outcome, not an error`                                                                                                                                                     | [[archive.companion]]  |
| delta_self_contained    | invariant | `the verbatim deployment presumes self-contained dual-format deltas per the corpus convention; a partial delta would narrow the deployed spec — the companion does not merge, so the convention (every delta restates the capability's full requirement set) is load-bearing`                  | [[archive.companion]]  |

## Model

### States
- `resolved`
- `archived`
- `restored`
- `refused`

### Transitions

| id       | from      | to        | guard                                                                                                                 |
|----------|-----------|-----------|-------------------------------------------------------------------------------------------------------------------------|
| locate   | resolved  | archived  | [[archive.companion.openspec_not_forked]]                                                                                             |
| deploy   | archived  | restored  | [[archive.companion.layer_preserved]] ∧ [[archive.companion.fail_closed_unverifiable]] ∧ [[archive.companion.delta_self_contained]]                              |
| reject   | archived  | refused   | `¬[[archive.companion.fail_closed_unverifiable]]`                                                                                      |

## Properties

| id                        | kind | derives_from                       | generator                                    | predicate                                                                                      |
|---------------------------|------|-------------------------------------|----------------------------------------------|-------------------------------------------------------------------------------------------------|
| deployed_byte_identical   | unit | [[archive.companion.layer_preserved]]            | `change_archived_then_compared()`            | `each deployed spec equals its archived delta byte-for-byte`                                     |
| no_bespoke_merge          | unit | [[archive.companion.openspec_not_forked]]        | `module_inspected_for_merge_logic()`         | `the module contains no delta-merge code path; only the external invocation and cp remain`        |
| plain_delta_refused       | unit | [[archive.companion.fail_closed_unverifiable]]   | `archive_dir_with_frontmatterless_delta()`   | `command fails, lists the delta, writes nothing`                                                  |
| rerun_byte_identical      | unit | [[archive.companion.idempotent_rerun]]           | `companion_run_twice()`                      | `second run's envelope matches the first's restored set; deployed files unchanged`                |
| ambiguous_dir_reported    | unit | [[archive.companion.newest_archive_wins]]        | `two_archive_dirs_same_change_id()`          | `lexicographically greatest dir used and named in the envelope`                                   |
| dry_run_writes_nothing    | unit | [[archive.companion.dry_run_never_mutates]]      | `dry_run_over_populated_archive()`           | `envelope emitted; no file mtimes changed; openspec not invoked`                                  |
| docs_only_change_succeeds | unit | [[archive.companion.empty_restore_valid]]        | `archived_change_without_spec_deltas()`      | `exit 0, restored: []`                                                                            |
| missing_openspec_labeled  | unit | [[archive.companion.openspec_not_forked]]        | `path_without_openspec_binary()`             | `labeled error with an install/PATH remediation hint`                                             |
| partial_delta_narrows     | unit | [[archive.companion.delta_self_contained]]       | `archive_dir_with_partial_delta()`           | `deployed spec retains exactly the requirements the delta restates — narrowing is deployable and detectable only by the convention, so self-containment is load-bearing` |

## ADDED Requirements

### Requirement: Archive with dual-format preservation
The system SHALL provide `spk archive-companion <CHANGE_ID>` that
archives the named openspec change via the openspec CLI with spec
application skipped, then deploys each archived `specs/<cap>/spec.md`
delta verbatim to `openspec/specs/<cap>/spec.md`, leaving every deployed
spec byte-identical to its archived delta.

#### Scenario: Dual-format delta survives archiving
- **WHEN** an active change carries a dual-format delta for capability `c`
- **THEN** after `spk archive-companion <id>`, `openspec/specs/c/spec.md`
  is byte-identical to the archived delta and retains frontmatter and
  all three tables

#### Scenario: Already-archived change re-runs cleanly
- **WHEN** the change directory no longer exists but its archive does
- **THEN** the openspec invocation is skipped and the restore re-runs to
  a byte-identical result

### Requirement: Fail closed on stripped deltas
The system SHALL refuse to deploy any archived delta lacking the
dual-format layer — no frontmatter opener or no `## Constraints`
section — failing the whole command with a listing of every refused
delta and a remediation hint naming the `spk explain dual-format`
migration recipe, and writing no deployed spec.

#### Scenario: Frontmatterless delta is refused
- **WHEN** an archived delta is plain openspec grammar
- **THEN** the command fails, the delta is listed, and no
  `openspec/specs/<cap>/spec.md` is modified

### Requirement: Observable resolution and preview
The system SHALL emit an envelope naming the archive directory used,
whether the openspec invocation ran, and the restored paths — and SHALL
support `--dry-run` producing that same resolution without invoking
openspec or writing any file.

#### Scenario: Dry-run reports the plan
- **WHEN** `spk archive-companion <id> --dry-run` runs over a
  resolvable change
- **THEN** the envelope lists the planned restores and no file is
  written and no openspec invocation occurs


## Requirements

### Requirement: Archive with dual-format preservation
The system SHALL provide `spk archive-companion <CHANGE_ID>` that
archives the named openspec change via the openspec CLI with spec
application skipped, then deploys each archived `specs/<cap>/spec.md`
delta verbatim to `openspec/specs/<cap>/spec.md`, leaving every deployed
spec byte-identical to its archived delta.

#### Scenario: Dual-format delta survives archiving
- **WHEN** an active change carries a dual-format delta for capability `c`
- **THEN** after `spk archive-companion <id>`, `openspec/specs/c/spec.md`
  is byte-identical to the archived delta and retains frontmatter and
  all three tables

#### Scenario: Already-archived change re-runs cleanly
- **WHEN** the change directory no longer exists but its archive does
- **THEN** the openspec invocation is skipped and the restore re-runs to
  a byte-identical result

### Requirement: Fail closed on stripped deltas
The system SHALL refuse to deploy any archived delta lacking the
dual-format layer — no frontmatter opener or no `## Constraints`
section — failing the whole command with a listing of every refused
delta and a remediation hint naming the `spk explain dual-format`
migration recipe, and writing no deployed spec.

#### Scenario: Frontmatterless delta is refused
- **WHEN** an archived delta is plain openspec grammar
- **THEN** the command fails, the delta is listed, and no
  `openspec/specs/<cap>/spec.md` is modified

### Requirement: Observable resolution and preview
The system SHALL emit an envelope naming the archive directory used,
whether the openspec invocation ran, and the restored paths — and SHALL
support `--dry-run` producing that same resolution without invoking
openspec or writing any file.

#### Scenario: Dry-run reports the plan
- **WHEN** `spk archive-companion <id> --dry-run` runs over a
  resolvable change
- **THEN** the envelope lists the planned restores and no file is
  written and no openspec invocation occurs

