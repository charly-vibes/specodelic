---
tags: [pipeline-run:tdd-ro5-2026-11-12-specodelic-75m-revision-upgrade-tooling, pipeline-step:plan]
---

ORIENT → PLAN (specodelic-75m revision-upgrade tooling).

DESIRED BEHAVIOR
1. Core-owned thin revision registry in src/migrate.rs: const manifest of known format revisions (2..=20, the corpus's own '## Revision N' headings) with date where recorded, a one-line change note, and an OPTIONAL migration step (most revisions are additive → no-op). Revision 18 is the one genuinely breaking corpus-level transform (the id law: 'id: spec' → parent-dir-derived id + [[spec.*]] re-key) and reuses the existing migrate::rekey transform; every other revision is a no-op step.
2. Verb: 'spk migrate --to-revision <N> [--from-revision <R>] [--write] <paths...>' — an upgrade mode on the existing migrate verb (the wrap and --rekey modes keep their exact current behaviour). Upgrade mode is DRY-RUN BY DEFAULT (preview only); --write (alias --force) applies; --write and --dry-run conflict.
3. Source revision: explicit --from-revision, else detected per file (a step's detector says the file still needs it → the file predates that revision; run-level source = min over supplied files). Unknown source/target revision → labeled failure naming the known range + remediation. Target below the corpus's current revision → labeled failure (no downgrade).
4. Steps are shape-guarded and idempotent: a file that needs no step is left BYTE-IDENTICAL (never rewritten → CRLF and final-terminator preserved exactly); only files whose content actually changes are written, and only under the supplied paths (no sidecars, no writes outside the given paths).
5. Output through genesis::guide::Output::emit with remediation hints; refusals exit 2.

OUT OF SCOPE
- No new format Revision, no change to the format spec or the linter rules.
- No rewriting of archived/applied plain deltas; no merge of sidecars into plain deltas (Revision 19 contract preserved).
- No manufactured transformations for additive revisions; no invented/ synthetic old-format shapes.
- The existing wrap path and --rekey contract (refusal set, exit codes, byte-preservation) stay unchanged.

TEST CASES (RED first, tests/**)
- upgrade_fixture_corpus_to_current_revision_lints_clean: a fixture corpus at an old revision (one file with the real pre-Revision-18 'id: spec' shape, one file already carrying its derived id) upgraded to the current revision lints with ZERO findings; the transformed file's id/refs are rewritten; the other file is byte-identical.
- upgrade_leaves_unchanged_files_byte_identical: bytes pinned before/after (CRLF + no-trailing-newline variants included).
- upgrade_dry_run_writes_nothing: default mode leaves the disk bytes untouched and reports the would-be content.
- upgrade_unknown_target_revision_fails_labeled: --to-revision 99 → labeled failure naming the known range + remediation.
- upgrade_unknown_source_revision_fails_labeled: --from-revision 99 → labeled failure naming the known range.
- upgrade_target_below_current_revision_fails_labeled: --to-revision below the corpus's current revision → labeled failure, no writes.
- upgrade_never_touches_paths_outside_the_given_ones: a sibling file outside the supplied path is untouched.

NARROW TEST COMMAND
  TMPDIR=/var/tmp/specodelic-xch1 cargo test --test cli migrate::upgrade
FULL VERIFICATION COMMAND
  TMPDIR=/var/tmp/specodelic-xch1 just ci   (fmt-check, clippy -D warnings, tests, release build)
  + cargo fmt && pretender check && openspec validate add-revision-upgrade --strict

FILES LIKELY TOUCHED
- src/migrate.rs (registry + upgrade engine; new public seam, unit tests in-module)
- src/main.rs (migrate verb wiring only: --to-revision/--from-revision/--write flags + dispatch)
- tests/cli/migrate.rs (new upgrade integration tests)
- openspec/changes/add-revision-upgrade/{proposal.md,design.md,tasks.md,specs/migrate/spec.md,specs/migrate/layer.md}
- .espectacular/migrate/*.toml (new contract TOMLs for the upgrade scenarios, archive commit only)
- AGENTS.md only via 'spk init --force' (catalog-only diff)

