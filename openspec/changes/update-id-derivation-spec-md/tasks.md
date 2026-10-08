# Tasks: update-id-derivation-spec-md

## 1. Linter semantics (TDD)

- [x] 1.1 Failing tests: `spec_md_derives_expected_id_from_parent_dir`,
      `spec_md_with_legacy_id_spec_fires_id_matches_file` (exactly one
      finding), `spec_md_without_parent_dir_keeps_stem_derivation`,
      `spec_md_underscore_dir_is_literal`,
      `real_id_dual_format_files_resolve_cross_file`,
      `modified_delta_with_non_spec_id_fires_dual_format_valid` (revised to
      real-id-clean), `dual_format_file_with_wrong_id_fires_id_matches_file`,
      `same_id_files_do_not_collide_in_reference_resolution` (delta/deployed
      same-id pair), `real_id_files_resolve_corpus_wide_and_typos_still_dangle`
- [x] 1.2 Implement `expected_id_from_path` in `src/lint/rules.rs`; wire
      into `id_matches_file`; drop the `dual_format_valid` id check
- [x] 1.3 Retire `total_refs` self-containment scoping + bare-local arm
      (`src/lint/graph.rs`); same for `lint_graph_shape`, `src/graph.rs`
      `resolve_link`, `src/acset/instance.rs`, `src/lint/observability.rs`
- [x] 1.4 Scope law: fold `isolated_scope_required` into
      `duplicate_corpus_identity` (`src/citation_corpus.rs`); rewrite the
      CLI tests that asserted the retired refusal
- [x] 1.5 `cargo test` green (lib + integration + fuzz)

## 2. Scaffolds

- [x] 2.1 `spk migrate` generates the derived real id (`src/migrate.rs`
      takes the path; scaffold refs re-keyed)
- [x] 2.2 `spk new` default target `openspec/specs/<id>/spec.md`
- [x] 2.3 Naming-law warning text updated in `cmd_migrate`

## 3. Format corpus (Revision 18)

- [x] 3.1 `specs/specodelic.md`: `id_matches_file` row expr + `## Revision 18`
      heading; `FORMAT_REVISION` → 18; guide drift guard passes
- [x] 3.2 `specs/linter-frontmatter.md`: expr + `filename_mismatch` property +
      Notes revision
- [x] 3.3 `specs/model_check.md`: `corpus_identity_scope` row (renamed from
      `dual_format_isolated_scope`) + property rows + Notes
- [x] 3.4 `specs/linter-referential_integrity.md`: resolution-algorithm note
- [x] 3.5 Embedded guide (`src/guide.md`) dual-format + references topics;
      `docs/src/commands.md` migrate/verify/archive passages

## 4. This repo's tree migration

- [x] 4.1 All dual-format `spec.md` under `openspec/specs/` + active
      changes: frontmatter id → parent-dir-derived id; `[[spec.*]]`/`[[spec]]`
      refs re-keyed
- [x] 4.2 Archived deltas: frontmatter stripped (frozen evidence,
      parse-skip; openspec grammar untouched)
- [x] 4.3 Boilerplate lifecycle models anchored: `process_lifecycle`
      invariant + deriving property + cited guards
- [x] 4.4 `just lint-deltas` (28 files, 0 issues), `just lint-specs` (22
      files, 0 issues), `just model-check-specs` (22 files, 0 failed)

## 5. Gates

- [x] 5.1 `openspec validate --all --strict` (25/25)
- [x] 5.2 `just sync-sections` + `sync-sections-test` + `summary-completeness`
      + `lint-doc-examples` + `lint-baseline`
- [x] 5.3 `ah check` (espectacular correspondence)
- [x] 5.4 `just ci` full gate
