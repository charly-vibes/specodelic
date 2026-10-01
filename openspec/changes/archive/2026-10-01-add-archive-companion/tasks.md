# Tasks: add-archive-companion

## 1. Module (TDD)

- [x] 1.1 RED: `resolve_archive_dir` — newest `*-<id>` match under
  `openspec/changes/archive/`; none → labeled error (change id named +
  `openspec list` hint). Test: two matching dirs → greatest wins (D6).
- [x] 1.2 GREEN: implement `resolve_archive_dir` (recipe semantics:
  `find … -name "*-<id>" | sort | tail -1` equivalent).
- [x] 1.3 RED: dual-format verification — first line `---` AND a
  `## Constraints` header present; frontmatterless or table-less delta
  → refusal listing the delta + migration-recipe hint, no partial copy
  (D3). Tests for both failure shapes and the pass shape.
- [x] 1.4 GREEN: implement `verify_dual_format`.
- [x] 1.5 RED: `restore` — copy each verified delta verbatim to
  `openspec/specs/<cap>/spec.md` (mkdir -p); empty delta set → success
  with empty restored list (D8); byte-identity pinned in test.
- [x] 1.6 GREEN: implement `restore` (+ `RestoreOutcome` data).
- [x] 1.7 RED: `run` orchestration — active change → invoke openspec
  (`archive <id> --skip-specs --yes`) via injectable runner seam;
  already-archived → skip invocation (D5); openspec failure/missing →
  labeled error with hint (D2).
- [x] 1.8 GREEN: implement `run` with `ArchiveRunner` seam (closure or
  trait), so tests never shell out.
- [x] 1.9 RED: `--dry-run` — full resolution + verification, no
  openspec invocation, no writes, `dry_run: true` in payload (D7).
- [x] 1.10 GREEN: implement dry-run path.

## 2. CLI + integration

- [x] 2.1 Wire `ArchiveCompanion { change_id, dry_run }` arm in
  `src/main.rs` dispatch; envelope `{change_id, archived, archive_dir,
  restored, dry_run}`; failure envelopes carry the hint (repo
  convention: ok:true + envelope_kind:"error").
- [x] 2.2 Integration test (assert_cmd): dry-run over a temp fixture
  repo tree writes nothing; refusal case exits non-zero without
  touching deployed specs.

## 3. Recipe + docs

- [x] 3.1 `just archive-change` delegates to `spk archive-companion`
  (interface unchanged); recipe keeps the exit-status semantics.
- [x] 3.2 `docs/src/commands.md` gains the archive-companion section
  (ingestion-gate lint applies); README command list mention.

## 4. Gates + close

- [x] 4.1 `just ci` green (fmt, clippy -D warnings, tests, release
  build) + `just lint-specs` + openspec strict + graph 0 dangling.
- [x] 4.2 `spk archive-companion add-archive-companion` — dogfood: the
  change archives itself through its own tool; then CHANGELOG entry.
- [x] 4.3 Close specodelic-fzo, push, GH#7 answered with the shipped
  command reference.
