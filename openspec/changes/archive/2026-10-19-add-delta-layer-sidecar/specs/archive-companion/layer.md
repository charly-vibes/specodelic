---
id: archive.companion.layer
kind: intent
statement: "WHEN an openspec change is archived, THE archive-companion capability SHALL invoke openspec's own archiver with default spec application, SHALL refuse any delta whose layer is unrestorable before the destructive archive, and SHALL re-attach each archived sidecar's dual-format layer onto the applied deployed spec so every deployed spec ends up frontmatter-plus-tables again while the archived delta stays plain."
---

## Constraints

| id                      | kind      | expr                                                                                                                                                                                                                                                                                   | traces_to |
|--------------------------|-----------|-----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|-----------|
| process_lifecycle | invariant | `the capability advances through its declared lifecycle states under the repo's change process — each stage transition fires only when its stage gate holds` | [[archive.companion.layer]] |
| layer_preserved         | invariant | `after the command completes, every deployed openspec/specs/<cap>/spec.md touched by the change is the layer-preserving re-assembly of the archived pair — openspec's applied ## Requirements section plus the archived sidecar's frontmatter and Constraints/Model/Properties tables — while the archived delta stays byte-plain and the sidecar stays archived beside it`                                              | [[archive.companion.layer]]  |
| openspec_not_forked     | invariant | `archive semantics come from the openspec CLI itself (invoked with DEFAULT spec application — plain deltas are openspec's native merge); the companion never re-implements delta merging, requirement parsing, or directory moves — it only re-attaches the layer afterwards`                                     | [[archive.companion.layer]]  |
| fail_closed_unverifiable | invariant | `a delta whose layer is unrestorable — no sibling layer.md sidecar, or a sidecar that fails parse or lacks one of the three layer tables — is refused BEFORE the destructive openspec archive; the whole command fails listing every refused delta with the spk migrate wrap remediation hint, and nothing moves and nothing is written`                                                                                  | [[archive.companion.layer]]  |
| attach_shape_asserted   | invariant | `the assembled deployed spec is asserted pre-write: no delta headers, frontmatter plus all three tables present — a shape openspec 1.x strict rejects is refused and reverted; when the run invoked openspec the change dir is moved back out of the archive; the deployed-corpus lint runs post-write and reverts on findings, labeled with a remediation hint`                  | [[archive.companion.layer]]  |
| converged_sidecarless_recognized | invariant | `a sidecar-less ARCHIVED delta whose deployed target already carries the dual-format layer is recognized as converged and skipped, never refused — Stage-1-era archives stay idempotent under rerun; anything else without a sidecar is refused`                  | [[archive.companion.layer]]  |
| idempotent_rerun        | invariant | `re-running the command for an already-archived change skips the openspec invocation and re-executes the re-attachment from the newest archive; the result converges to identical deployed bytes`                                                                                                                              | [[archive.companion.layer]]  |
| newest_archive_wins     | invariant | `when multiple archive directories match the change id, the lexicographically greatest name is used and is named in the envelope — ambiguity is observable, never silent`                                                                                                                    | [[archive.companion.layer]]  |
| dry_run_never_mutates   | invariant | `--dry-run resolves the restore plan (archive directory, delta list, sidecar verification) and emits the envelope without invoking openspec and without writing any file`                                                                                                                | [[archive.companion.layer]]  |
| empty_restore_valid     | invariant | `a change with no spec deltas completes successfully with an empty restored list — nothing to restore is a stated outcome, not an error`                                                                                                                                                     | [[archive.companion.layer]]  |

## Model

### States

- `resolved`
- `archived`
- `restored`
- `refused`

### Transitions

| id       | from      | to        | guard                                                                                                                 |
|----------|-----------|-----------|-------------------------------------------------------------------------------------------------------------------------|
| locate   | resolved  | archived  | [[archive.companion.layer.openspec_not_forked]]  |
| deploy   | archived  | restored  | [[archive.companion.layer.layer_preserved]] ∧ [[archive.companion.layer.fail_closed_unverifiable]] ∧ [[archive.companion.layer.attach_shape_asserted]]  |
| reject   | archived  | refused   | `¬[[archive.companion.layer.fail_closed_unverifiable]]`  |

## Properties

| id                        | kind | derives_from                       | generator                                    | predicate                                                                                      |
|---------------------------|------|-------------------------------------|----------------------------------------------|-------------------------------------------------------------------------------------------------|
| process_lifecycle_checked | unit | [[archive.companion.layer.process_lifecycle]] | `lifecycle_model_present()` | `check(file) == passed` |
| deployed_assembly_matches | unit | [[archive.companion.layer.layer_preserved]]            | `change_archived_then_compared()`            | `each deployed spec carries the applied Requirements section plus the sidecar's frontmatter and tables; the archived delta stays plain`                                     |
| no_bespoke_merge          | unit | [[archive.companion.layer.openspec_not_forked]]        | `module_inspected_for_merge_logic()`         | `the module contains no delta-merge code path; only the external invocation and the layer re-attachment remain`        |
| plain_delta_refused       | unit | [[archive.companion.layer.fail_closed_unverifiable]]   | `active_change_with_sidecarless_delta()`   | `command fails before the openspec invocation, lists the delta, writes nothing`                                                  |
| shape_assertion_enforced  | unit | [[archive.companion.layer.attach_shape_asserted]] | `assembly_with_delta_header()` | `write refused and reverted; the run fails naming the capability; an openspec-invoking run moves the change dir back` |
| converged_rerun_skipped   | unit | [[archive.companion.layer.converged_sidecarless_recognized]] | `sidecarless_archive_with_deployed_layer()` | `rerun skips the pair as converged, not refused` |
| rerun_byte_identical      | unit | [[archive.companion.layer.idempotent_rerun]]           | `companion_run_twice()`                      | `second run's envelope matches the first's restored set; deployed files converge to identical bytes`                |
| ambiguous_dir_reported    | unit | [[archive.companion.layer.newest_archive_wins]]        | `two_archive_dirs_same_change_id()`          | `lexicographically greatest dir used and named in the envelope`                                   |
| dry_run_writes_nothing    | unit | [[archive.companion.layer.dry_run_never_mutates]]      | `dry_run_over_populated_archive()`           | `envelope emitted; no file mtimes changed; openspec not invoked`                                  |
| docs_only_change_succeeds | unit | [[archive.companion.layer.empty_restore_valid]]        | `archived_change_without_spec_deltas()`      | `exit 0, restored: []`                                                                            |
| missing_openspec_labeled  | unit | [[archive.companion.layer.openspec_not_forked]]        | `path_without_openspec_binary()`             | `labeled error with an install/PATH remediation hint`                                             |
