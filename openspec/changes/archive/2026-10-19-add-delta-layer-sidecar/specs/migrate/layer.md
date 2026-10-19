---
id: migrate.layer
kind: intent
statement: "WHEN an openspec delta file is migrated, THE migrate capability SHALL wrap it in place into the Revision 19 pair — leaving the plain delta byte-untouched and creating the sibling layer.md sidecar with generated frontmatter and wired scaffold layers — while refusing any file that is not a plain delta in need of a sidecar and supporting --rekey for id: spec legacy files."
---

## Constraints

| id                      | kind      | expr                                                                                                                                                                    | traces_to |
|-------------------------|-----------|-------------------------------------------------------------------------------------------------------------------------------------------------------------------------|-----------|
| process_lifecycle | invariant | `the capability advances through its declared lifecycle states under the repo's change process — each stage transition fires only when its stage gate holds` | [[migrate.layer]] |
| delta_text_preserved    | invariant | `∀ migrated pair: the given delta file's text is preserved byte-for-byte — the plain delta stays untouched; the wrap creates the sibling layer.md sidecar rather than rewriting the delta (Revision 19: the dual-format wrap and its ## Requirements mirror are retired)` | [[migrate.layer]]  |
| sidecar_generated       | invariant | `the wrap produces the pair — the byte-unchanged plain delta plus the generated sibling layer.md carrying frontmatter (the parent-directory-derived id with a .layer suffix, kind intent, an EARS scaffold statement) and the wired scaffold layers; no mirror exists to maintain`                                                                | [[migrate.layer]]  |
| refuse_already_migrated | invariant | `a file that is not a plain delta in need of a sidecar is refused (exit 2), never rewritten: a file carrying frontmatter (already a specodelic file), a file whose only Requirements heading is a deployed spec's ## Requirements section, and a file carrying no Requirements heading at all` | [[migrate.layer]]  |
| merge_adds_only_missing | invariant | `only missing pieces are inserted when migrating a partially migrated pair — an existing sidecar passes through untouched and no scaffold statement is inserted; only missing sidecar pieces are generated (Revision 19: the mirror is no longer a piece that can be missing)`                             | [[migrate.layer]]  |
| scaffold_lints_clean    | invariant | `the generated sidecar (frontmatter + wired scaffold_* placeholder rows + one-state model) exits spk lint 0 with zero findings as written — the author starts from a green baseline` | [[migrate.layer]]  |
| dry_run_writes_nothing  | invariant | `--dry-run emits the resulting pair content as data and never writes a file; the on-disk bytes are unchanged`                                                               | [[migrate.layer]]  |
| rekey_derives_real_id   | invariant | `--rekey derives the real id by the naming law — the parent directory name for a spec.md file (- ⇔ ., _ literal) — and replaces the frontmatter `id: spec` line with it; every [[spec]] ref re-keys to [[<derived-id>]] and every [[spec. prefix to [[<derived-id>.; every other byte stays untouched` | [[migrate.layer]] |
| rekey_refusals          | invariant | `--rekey on a file without frontmatter is refused (a plain delta takes the wrap path); on a spec.md with no derivable id (bare, or parent dir named `spec`) it is refused; on a file whose id is already not `spec` it is an idempotent no-op with a warning, never rewritten` | [[migrate.layer]] |

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
| begin_wrap   | unwrapping | wrapping  | [[migrate.layer.delta_text_preserved]]  |
| write_result | wrapping   | written   | [[migrate.layer.sidecar_generated]]  |
| dry_run_pass | wrapping   | wrapping  | [[migrate.layer.dry_run_writes_nothing]]  |
| refuse       | unwrapping | refused   | [[migrate.layer.refuse_already_migrated]]  |
| merge_only   | wrapping   | written   | [[migrate.layer.merge_adds_only_missing]]  |
| verify_green | written    | written   | [[migrate.layer.scaffold_lints_clean]]  |
| rekey_result | unwrapping | rekeyed   | [[migrate.layer.rekey_derives_real_id]]  |
| rekey_refuse | unwrapping | refused   | [[migrate.layer.rekey_refusals]]  |

## Properties

| id                        | kind | derives_from                     | generator                        | predicate                                        |
|---------------------------|------|----------------------------------|----------------------------------|--------------------------------------------------|
| process_lifecycle_checked | unit | [[migrate.layer.process_lifecycle]] | `lifecycle_model_present()` | `check(file) == passed` |
| p_delta_text_preserved    | unit | [[migrate.layer.delta_text_preserved]]    | `delta_with(frontmatter: any)`   | `migrate(file).delta_body == original.delta_body` |
| p_sidecar_generated       | unit | [[migrate.layer.sidecar_generated]]   | `delta_with(sections: any)`      | `result.sidecar present with .layer-derived frontmatter ∧ result.delta byte-unchanged`         |
| p_refuse_already_migrated | unit | [[migrate.layer.refuse_already_migrated]] | `file_with_frontmatter_or_non_delta()`             | `migrate(file) == refused ∧ disk_unchanged`       |
| p_merge_adds_only_missing | unit | [[migrate.layer.merge_adds_only_missing]] | `pair_with(partial_sidecar: any)`| `existing_pieces(result) == existing_pieces(original)` |
| p_scaffold_lints_clean    | unit | [[migrate.layer.scaffold_lints_clean]]    | `any_plain_delta()`              | `lint(migrate(file).sidecar) == zero_findings`            |
| p_dry_run_writes_nothing  | unit | [[migrate.layer.dry_run_writes_nothing]]  | `any_plain_delta()`              | `dry_run(file).disk_bytes == original.disk_bytes` |
| p_rekey_derives_real_id   | unit | [[migrate.layer.rekey_derives_real_id]] | `legacy_id_spec_dual_format_file()` | `rekey(file).id == parent_dir_derived ∧ refs re-keyed ∧ rest byte-identical` |
| p_rekey_refusals          | unit | [[migrate.layer.rekey_refusals]] | `plain_delta_or_bare_spec_md()` | `rekey(file) == refused ∧ disk_unchanged ∧ second_run_is_noop` |
