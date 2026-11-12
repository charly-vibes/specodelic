# Tasks: add-revision-upgrade

## 1. The revision registry (TDD)

- [x] 1.1 RED: failing integration tests (`tests/cli/migrate_upgrade.rs`):
      `upgrade_old_corpus_upgrades_to_target_revision` (a pre-Revision-18
      `id: spec` corpus file is rewritten to its derived id; the corpus
      lints with zero findings; a file needing no step stays
      byte-identical); `upgrade_dry_run_writes_nothing`;
      `upgrade_never_touches_files_outside_supplied_paths`;
      `upgrade_unknown_target_revision_is_refused`;
      `upgrade_unknown_source_revision_is_refused`;
      `upgrade_target_below_current_revision_is_refused`;
      `upgrade_unchanged_files_are_byte_identical` (CRLF + no final
      terminator); `upgrade_failed_upgrade_writes_nothing`
- [x] 1.2 RED: failing unit tests in `src/migrate.rs` — the registry
      knows the revision range and each entry's date/note; a file's
      revision is detected from its shape; the planner applies the
      registered steps in ascending revision order
- [x] 1.3 GREEN: `pub const REVISIONS: &[RevisionEntry]` in
      `src/migrate.rs` — revision number, date (where the corpus records
      one), one-line change note, optional migration step; Revision 18 is
      the only entry with a step
- [x] 1.4 GREEN: the pure upgrade seam — revision detection per file,
      the corpus source revision (declared or detected), the step
      application (shape-guarded, EOL-aware), and the plan (per-file
      new content, changed flags, refusals)

## 2. The CLI arm

- [x] 2.1 `src/main.rs`: the migrate verb's flags — `--to-revision`,
      `--from-revision`, `--write` (alias `--force`); the positional
      becomes a path list; `--write` + `--dry-run` is refused
- [x] 2.2 `src/main.rs`: the upgrade arm — collect the supplied paths,
      plan, write only the changed files, and emit the envelope through
      `genesis::guide::Output` (refusals exit 2 with a remediation hint)
- [x] 2.3 The wrap and `--rekey` paths keep their behavior: exactly one
      file each (more is refused, exit 2), same refusals, same exit
      codes, same envelope fields

## 3. Spec, contracts, gates

- [x] 3.1 Delta for `migrate` (ADDED requirements + scenarios) and the
      full post-change layer sidecar (`specs/migrate/layer.md`) —
      `openspec validate add-revision-upgrade --strict`
- [x] 3.2 Contract TOMLs for the new scenarios in `.espectacular/migrate/`,
      bound to the new test flags (`ah check`)
- [x] 3.3 Gates: `cargo fmt`, `cargo clippy -D warnings`, `cargo test`,
      `just ci`, `pretender check`, `spk lint` over the corpus
- [x] 3.4 Archive through the sanctioned recipe: `just archive-change
      add-revision-upgrade`, then `spk init --force` (the `AGENTS.md`
      diff stays catalog-only)
