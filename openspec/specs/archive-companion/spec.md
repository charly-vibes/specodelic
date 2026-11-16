---
id: archive.companion
kind: intent
statement: "WHEN an openspec change is archived, THE archive-companion capability SHALL invoke openspec's own archiver with default spec application under a wall-clock bound, SHALL refuse any delta whose layer is unrestorable before the destructive archive, SHALL re-attach each archived sidecar's dual-format layer onto the applied deployed spec so every deployed spec ends up frontmatter-plus-tables again while the archived delta stays plain, SHALL derive the deployed head prose from the archived delta's Purpose exactly when the deployed Purpose is unwritten, and SHALL report every rollback it could not complete."
---

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
|--------------------------|-----------|-----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|-----------|
| process_lifecycle | invariant | `the capability advances through its declared lifecycle states under the repo's change process — each stage transition fires only when its stage gate holds` | [[archive.companion]] |
| layer_preserved         | invariant | `after the command completes, every deployed openspec/specs/<cap>/spec.md touched by the change is the layer-preserving re-assembly of the archived pair — openspec's applied ## Requirements section plus the archived sidecar's frontmatter and Constraints/Model/Properties tables — while the archived delta stays byte-plain and the sidecar stays archived beside it`                                              | [[archive.companion]]  |
| openspec_not_forked     | invariant | `archive semantics come from the openspec CLI itself (invoked with DEFAULT spec application — plain deltas are openspec's native merge); the companion never re-implements delta merging, requirement parsing, or directory moves — it only re-attaches the layer afterwards`                                     | [[archive.companion]]  |
| fail_closed_unverifiable | invariant | `a delta whose layer is unrestorable — no sibling layer.md sidecar, or a sidecar that fails parse or lacks one of the three layer tables — is refused BEFORE the destructive openspec archive; the whole command fails listing every refused delta with the spk migrate wrap remediation hint, and nothing moves and nothing is written`                                                                                  | [[archive.companion]]  |
| attach_shape_asserted   | invariant | `the assembled deployed spec is asserted pre-write: no delta headers, frontmatter plus all three tables present — a shape openspec 1.x strict rejects is refused and reverted; when the run invoked openspec the change dir is moved back out of the archive; the deployed-corpus lint runs post-write and reverts on findings, labeled with a remediation hint`                  | [[archive.companion]]  |
| converged_sidecarless_recognized | invariant | `a sidecar-less ARCHIVED delta whose deployed target already carries the dual-format layer is recognized as converged and skipped, never refused — Stage-1-era archives stay idempotent under rerun; anything else without a sidecar is refused`                  | [[archive.companion]]  |
| idempotent_rerun        | invariant | `re-running the command for an already-archived change skips the openspec invocation and re-executes the re-attachment from the newest archive; the result converges to identical deployed bytes`                                                                                                                              | [[archive.companion]]  |
| newest_archive_wins     | invariant | `when multiple archive directories match the change id, the lexicographically greatest name is used and is named in the envelope — ambiguity is observable, never silent`                                                                                                                    | [[archive.companion]]  |
| dry_run_never_mutates   | invariant | `--dry-run resolves the restore plan (archive directory, delta list, sidecar verification) and emits the envelope without invoking openspec and without writing any file`                                                                                                                | [[archive.companion]]  |
| empty_restore_valid     | invariant | `a change with no spec deltas completes successfully with an empty restored list — nothing to restore is a stated outcome, not an error`                                                                                                                                                     | [[archive.companion]]  |
| archiver_bounded        | invariant | `the openspec archive invocation runs under a wall-clock bound — 600s by default, SPK_ARCHIVE_TIMEOUT_SECS override read at invocation time with a zero, negative, or unparsable value falling back to the default — and expiry kills the archiver's process group, never reaches the attach stage, writes no deployed spec, and fails labeled naming the command openspec archive <id> --yes and the expired bound as timed out after <N>s` | [[archive.companion]]  |
| rollback_reported       | invariant | `a revert that cannot restore its saved bytes is reported in the attach-violation detail as rollback failed: <path>: <io error>, appended after the primary violation text — never silently discarded, never displacing the primary violation — while a revert that restores every entry appends nothing` | [[archive.companion]]  |
| head_prose_derived      | invariant | `the deployed head prose comes from the deployed body except when its Purpose body is empty or opens with the archiver placeholder TBD - created by archiving — then the archived delta's Purpose section supplies it, and a head with no Purpose section adopts that section as the whole head; authored deployed prose is never displaced and a delta without Purpose changes nothing` | [[archive.companion]]  |

## Model

### States

- `resolved`
- `archived`
- `restored`
- `refused`
- `archiver_timed_out`

### Transitions

| id       | from      | to        | guard                                                                                                                 |
|----------|-----------|-----------|-------------------------------------------------------------------------------------------------------------------------|
| locate   | resolved  | archived  | [[archive.companion.openspec_not_forked]]  |
| expire   | resolved  | archiver_timed_out | `¬[[archive.companion.archiver_bounded]]`  |
| deploy   | archived  | restored  | [[archive.companion.layer_preserved]] ∧ [[archive.companion.fail_closed_unverifiable]] ∧ [[archive.companion.attach_shape_asserted]] ∧ [[archive.companion.head_prose_derived]]  |
| reject   | archived  | refused   | `¬[[archive.companion.fail_closed_unverifiable]]`  |

## Properties

| id                        | kind | derives_from                       | generator                                    | predicate                                                                                      |
|---------------------------|------|-------------------------------------|----------------------------------------------|-------------------------------------------------------------------------------------------------|
| process_lifecycle_checked | unit | [[archive.companion.process_lifecycle]] | `lifecycle_model_present()` | `check(file) == passed` |
| deployed_assembly_matches | unit | [[archive.companion.layer_preserved]]            | `change_archived_then_compared()`            | `each deployed spec carries the applied Requirements section plus the sidecar's frontmatter and tables; the archived delta stays plain`                                     |
| no_bespoke_merge          | unit | [[archive.companion.openspec_not_forked]]        | `module_inspected_for_merge_logic()`         | `the module contains no delta-merge code path; only the external invocation and the layer re-attachment remain`        |
| plain_delta_refused       | unit | [[archive.companion.fail_closed_unverifiable]]   | `active_change_with_sidecarless_delta()`   | `command fails before the openspec invocation, lists the delta, writes nothing`                                                  |
| shape_assertion_enforced  | unit | [[archive.companion.attach_shape_asserted]] | `assembly_with_delta_header()` | `write refused and reverted; the run fails naming the capability; an openspec-invoking run moves the change dir back` |
| converged_rerun_skipped   | unit | [[archive.companion.converged_sidecarless_recognized]] | `sidecarless_archive_with_deployed_layer()` | `rerun skips the pair as converged, not refused` |
| rerun_byte_identical      | unit | [[archive.companion.idempotent_rerun]]           | `companion_run_twice()`                      | `second run's envelope matches the first's restored set; deployed files converge to identical bytes`                |
| ambiguous_dir_reported    | unit | [[archive.companion.newest_archive_wins]]        | `two_archive_dirs_same_change_id()`          | `lexicographically greatest dir used and named in the envelope`                                   |
| dry_run_writes_nothing    | unit | [[archive.companion.dry_run_never_mutates]]      | `dry_run_over_populated_archive()`           | `envelope emitted; no file mtimes changed; openspec not invoked`                                  |
| docs_only_change_succeeds | unit | [[archive.companion.empty_restore_valid]]        | `archived_change_without_spec_deltas()`      | `exit 0, restored: []`                                                                            |
| missing_openspec_labeled  | unit | [[archive.companion.openspec_not_forked]]        | `path_without_openspec_binary()`             | `labeled error with an install/PATH remediation hint`                                             |
| archiver_bound_enforced   | unit | [[archive.companion.archiver_bounded]]           | `archiver_that_never_exits(SPK_ARCHIVE_TIMEOUT_SECS: 1)` | `the run returns as a labeled timeout naming openspec archive <id> --yes and timed out after 1s ∧ no deployed spec is written` |
| bound_fallback_default    | unit | [[archive.companion.archiver_bounded]]           | `SPK_ARCHIVE_TIMEOUT_SECS ∈ {unset, 0, negative, unparsable}` | `the effective bound == the 600s default — never zero, never an unbounded wait` |
| rollback_failure_surfaced | unit | [[archive.companion.rollback_reported]]          | `revert_with(unrestorable_saved_bytes)`      | `the attach-violation detail carries rollback failed: <path>: <io error> after the primary violation text ∧ the run still fails as that violation` |
| placeholder_repaired_on_rerun | unit | [[archive.companion.head_prose_derived]]     | `archived_delta_with_purpose_over(placeholder_head)` | `the deployed Purpose == the delta's Purpose section ∧ the archiver placeholder survives nowhere` |
| authored_head_preserved   | unit | [[archive.companion.head_prose_derived]]         | `archived_delta_with_purpose_over(authored_head)` | `the deployed head is byte-identical to the pre-rerun head ∧ the delta's Purpose prose is absent` |

## Requirements

### Requirement: Archive with dual-format preservation
The system SHALL provide `spk archive-companion <CHANGE_ID>` that
archives the named openspec change via the openspec CLI with spec
application ON (plain deltas are openspec's native merge — Revision 19),
then re-attaches the dual-format layer from the change's archived
`layer.md` sidecar, so every deployed `openspec/specs/<cap>/spec.md`
ends up frontmatter-plus-tables again: the archived delta stays plain
and the deployed spec is the layer-preserving re-assembly of the two.

#### Scenario: Dual-format delta survives archiving
- **WHEN** an active change carries a plain delta plus a `layer.md`
  sidecar for capability `c`
- **THEN** after `spk archive-companion <id>`, the archived delta remains
  a plain openspec file, the sidecar is archived beside it, and
  `openspec/specs/c/spec.md` is the dual-format re-assembly of the two

#### Scenario: Plain delta deploys a dual-format spec
- **WHEN** an active change carries a plain delta plus a `layer.md`
  sidecar for capability `c`
- **THEN** after `spk archive-companion <id>`,
  `openspec/specs/c/spec.md` carries the applied `## Requirements`
  section plus the sidecar's frontmatter and Constraints/Model/Properties
  tables, and the archived delta remains a plain openspec file

#### Scenario: Already-archived change re-runs cleanly
- **WHEN** the change directory no longer exists but its archive does
- **THEN** the openspec invocation is skipped and the layer re-attachment
  re-runs to a byte-identical deployed result

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

### Requirement: Fail closed on sidecar-less deltas
The system SHALL refuse to archive any change whose delta lacks a sidecar
capable of restoring the deployed dual-format layer — no `layer.md`
beside the delta, or a sidecar that fails parse or `spk lint` — failing
the whole command before the destructive openspec archive runs, with a
listing of every refused delta and a remediation hint naming the `spk
migrate` wrap path, and writing no deployed spec.

#### Scenario: Sidecar-less delta is refused
- **WHEN** an active change carries a delta with no sibling `layer.md`
- **THEN** the command fails before `openspec archive` is invoked, the
  delta is listed, and no `openspec/specs/<cap>/spec.md` is modified

#### Scenario: Frontmatterless delta is refused
- **WHEN** an active change carries a delta with no sibling `layer.md`
- **THEN** the command fails, the delta is listed, and no
  `openspec/specs/<cap>/spec.md` is modified

#### Scenario: Unlintable sidecar is refused
- **WHEN** an active change carries a `layer.md` sidecar that fails
  `spk lint`
- **THEN** the command fails before `openspec archive` is invoked, the
  delta is listed with the lint findings, and no deployed spec is written

#### Scenario: Post-attach shape assertion
- **WHEN** the layer re-attachment would write a deployed spec carrying a
  delta header, or lacking frontmatter or any of the three tables
- **THEN** the write is refused and reverted and the command fails naming
  the capability — the deployed tree never gains a shape 1.x strict
  rejects

#### Scenario: New capability deploys from scratch
- **WHEN** a change introduces capability `c` with no existing deployed
  `openspec/specs/c/spec.md`
- **THEN** the companion assembles the first deployed spec from the
  delta's applied requirements plus the sidecar's frontmatter and tables
  — same shape assertion, same fail-closed refusal

### Requirement: Bounded archiver invocation

The companion SHALL run `openspec archive <CHANGE_ID> --yes` under a
wall-clock bound — 600 seconds by default (`DEFAULT_ARCHIVE_TIMEOUT_SECS`),
overridable through `SPK_ARCHIVE_TIMEOUT_SECS`, whose zero, negative, or
unparsable value falls back to the default — and on expiry SHALL kill the
archiver's whole process group, SHALL fail with a labeled timeout naming the
command and the expired bound, and SHALL write no deployed spec, draining
both child streams concurrently.

#### Scenario: Hung archiver is killed at the bound

- **WHEN** the invoked archiver does not exit within the effective bound
- **THEN** the companion kills it, the run fails labeled with
  `timed out after <N>s` while naming `openspec archive <id> --yes`, and no
  deployed spec is written

### Requirement: Rollback failure reporting

When the post-attach assertion or the deployed-corpus lint fails and the
companion reverts its writes, every revert that could not restore its saved
bytes SHALL be reported in the attach-violation detail as `rollback failed:
<path>: <io error>`, appended after the primary violation text — never
silently discarded and never displacing the primary violation. A revert that
restores every saved entry appends nothing, and the run still fails as the
same attach violation.

#### Scenario: An unrestorable target is reported

- **WHEN** a revert cannot restore a saved file's prior bytes, or cannot
  remove a file the run itself created
- **THEN** the failure detail carries `rollback failed: <path>: <io error>` for
  that path after the primary violation text, and the run exits as the attach
  violation

### Requirement: Deployed head prose derivation

On attach, the deployed spec's head prose SHALL come from the deployed body,
except when its `## Purpose` body is empty or opens with openspec's archiver
placeholder `TBD - created by archiving` — then the archived delta's
`## Purpose` section SHALL supply it, and a head with no `## Purpose` section
adopts the delta's section as its whole head. Authored deployed prose SHALL
never be displaced, and a delta carrying no Purpose prose SHALL leave the
deployed head byte-for-byte unchanged.

#### Scenario: Placeholder replaced on rerun

- **WHEN** the deployed head's Purpose is empty or starts with
  `TBD - created by archiving`, and the archived delta carries Purpose prose
- **THEN** the deployed head carries the delta's `## Purpose` section and the
  archiver placeholder appears nowhere in the deployed spec

#### Scenario: Real deployed prose is preserved

- **WHEN** the deployed head's Purpose is authored prose and the archived delta
  also carries Purpose prose
- **THEN** the deployed Purpose is byte-identical to the pre-rerun text and the
  delta's prose is not written

#### Scenario: A delta without Purpose changes nothing

- **WHEN** the archived delta carries no `## Purpose` prose, or an empty one
- **THEN** the deployed head is unchanged even when its Purpose is the
  archiver placeholder
