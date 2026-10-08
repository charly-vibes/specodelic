# Tasks: add-migrate-rekey

## 1. Rekey transform (TDD)

- [x] 1.1 Failing tests (`src/migrate.rs` tests + `tests/cli/migrate.rs`):
      `rekey_rewrites_id_and_refs` (spec.md under `<cap>/` → derived id,
      `[[spec]]`→`[[<id>]]`, `[[spec.x]]`→`[[<id>.x]]`, rest byte-identical);
      `rekey_refuses_plain_delta` (no frontmatter); `rekey_refuses_bare_spec_md`
      (no derivable id); `rekey_idempotent_second_run_is_noop` (id ≠ spec,
      warning, disk unchanged); `rekey_non_spec_id_is_warning_noop`;
      `rekey_preserves_non_wiki_spec_occurrences` (prose `spec.` words,
      `[[spec…` partial-brace text untouched); `rekey_dry_run_writes_nothing`
- [x] 1.2 Implement `pub fn rekey(text, path) -> Result<MigrateOutcome, MigrateError>`
      in `src/migrate.rs` (reuse `expected_id_from_path` + the
      `generated_id` naming-law derivation)
- [x] 1.3 CLI: `--rekey` flag on `Migrate` (`src/main.rs`); dispatch +
      envelope fields (`rekeyed: true`, distinct next-step hint) in
      `cmd_migrate`

## 2. Spec delta + corpus

- [x] 2.1 Delta for `migrate`: ADDED requirement (rekey semantics +
      scenarios), constraint rows (`rekey_derives_real_id`,
      `rekey_refs_rekeyed`, `rekey_refusals`, `rekey_idempotent`),
      `rekeying` state + transitions
- [x] 2.2 Dual-format archive per the sanctioned recipe when the change
      completes (deploy the delta verbatim)
- [x] 2.3 Docs: `docs/src/commands.md` migrate section + release-notes
      migration recipe (0.6.0 → 0.7.0)

## 3. Gates

- [x] 3.1 `cargo test` green; `just ci` green; `ah check --run-tests`
      (contract TOMLs for the new scenarios, bound to the new test flags)
- [x] 3.2 Dogfood: run `--rekey` against this repo's openspec tree —
      zero files should need it (already migrated in mcy); the sweep
      must be a warning no-op everywhere
