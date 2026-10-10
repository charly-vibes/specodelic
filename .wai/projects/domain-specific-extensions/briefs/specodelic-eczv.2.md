# Subagent brief: specodelic-eczv.2 — conform phase 2: report schema and scope digest

You are a pi subagent in repo `/var/home/sasha/para/areas/dev/gh/charly/specodelic`
(Rust CLI `specodelic`, alias `spk`). An orchestrator has claimed ticket
`specodelic-eczv.2` and is hands-off: you own the implementation end-to-end.
The orchestrator will verify with real gates — never claim work the diff
doesn't contain.

## Orientation (do this first)

- `export WAI_PROJECT=domain-specific-extensions`
- Read, in order: `openspec/changes/add-conform/design.md` (decisions D4, D6
  — report schema, evidence_scope as data, digest binding), `tasks.md`
  phase 2 (tasks 2.1–2.4), `proposal.md` (slice scoping), and the delta spec
  under `openspec/changes/add-conform/specs/`.
- Phase 1 already landed: `src/conform.rs` (verdict engine, `conform::reason`
  module) + `tests/conform_classification.rs`. Read both — you consume the
  phase-1 verdict types.
- `wai search "conform report"` and `wai search "scope_sha256"` — check
  accumulated patterns before designing.
- `src/model_check.rs` is pretender-pinned — you may EXTEND the scope-digest
  computation path per task 2.3 but must keep model_check's own digest
  contract byte-identical; new model-check tests go in `tests/` integration
  files, never inside `src/model_check.rs`.
- Start and execute the tdd-ro5 pipeline literally:
  `wai pipeline start tdd-ro5 --topic="specodelic-eczv.2: conform phase 2 report schema and scope digest"`
  Advance ONLY via `wai pipeline next` — never hand-edit the run YAML under
  `.wai/pipeline-runs/`.

## What to build

Persisted conform report in `src/conform.rs` + `tests/conform_report.rs`:

- `report_schema_version` — conform-LOCAL value, distinct from model_check's
  `claim_schema_version`.
- Per-trace records: scenario id, verdict, reason, evaluated claim ids,
  closed_world flag.
- Fixed `evidence_scope` statement (agreement-on-the-supplied-corpus; NEVER
  behavioral equality) present in BOTH the JSON envelope and `--human` views
  (D6 — data, not docs).
- `scope_sha256` binding the parsed structured content AND the consumed
  scenario corpus bytes: identical spec content with a one-byte-different
  corpus → different digest; identical inputs → identical digests across
  runs and path reordering.
- Records sorted by scenario id, no timestamps → two runs over identical
  inputs are byte-identical.
- Emit through `genesis::guide::Output::emit` (envelope default for pipes,
  `--human` for TTYs). Note: phase 2 is still library/emit-layer work — the
  `spk conform` CLI command itself is phase 4; expose the report through a
  testable API surface, reusing genesis emit primitives.

TDD discipline per tasks.md phase 2 (each test observed RED before GREEN):

- 2.1 **RED**: `tests/conform_report.rs` — `report_schema_roundtrip`,
  `evidence_scope_present_in_both_views`, `rerun_byte_identical`.
- 2.2 **RED**: `digest_binds_scenarios` (one-byte corpus difference flips
  the digest; identical inputs agree across runs and path reordering).
- 2.3 **GREEN**: implement the report in `src/conform.rs` emitting through
  `genesis::guide::Output::emit`; extend model_check's scope-digest
  computation to bind scenario corpus bytes for this run only — model_check's
  own digest contract unchanged (its existing tests must pass untouched).
- 2.4 **TIDY**: shared digest helper extracted IF AND ONLY IF the model_check
  digest contract stays byte-identical; otherwise keep conform-local. Separate
  tidy commit.

Check off tasks 2.1–2.4 in `openspec/changes/add-conform/tasks.md`
(checkbox edits only).

## Hard scope guard

- Allowed files: `src/conform.rs`, `tests/conform_report.rs` (new),
  minimal digest extension in the model_check path ONLY where task 2.3
  requires (no contract change), `openspec/changes/add-conform/tasks.md`
  (task 2.x checkoffs only).
- Never edit: `openspec/specs/`, `.espectacular/`, `pretender.toml`,
  `specs/` (the corpus), `.wai/resources/**`, this template,
  `src/model_check.rs` pinned-line count (respect shrink-only ratchet —
  if a pinned source file breaches, move new tests to `tests/`).
- Run `just pretender-check` before committing; do NOT raise pinned entries.

## Repo facts (boilerplate — applies to every ticket)

- **TMPDIR gotcha**: `export TMPDIR=/var/tmp/specodelic-eczv` (directory
  already created) before any cargo test run — /tmp quota exhaustion
  masquerades as hundreds of test failures.
- **Known pre-existing issue** (specodelic-9b1h, not yours to fix):
  `cargo clippy --all-targets -- -D warnings` fails in foreign files
  `src/lint/mod.rs:1605` and `src/migrate.rs:377`; the repo's own gate
  (`just lint`) is clean. Verify your NEW code is clippy-clean via
  `just lint`; ignore the two known foreign errors.
- **Commit hygiene** (standing, every commit): before ANY `git commit`, run
  `git status --short` and stage only files YOU authored
  (`git add <paths>`, never bare `git add`); unstage foreign files.
  Attribute the message only to what the diff contains.
- **Output discipline**: every command emits through
  `genesis::guide::Output::emit`; errors carry a remediation hint.
- **beads**: issues live in `.beads/issues.jsonl`; the orchestrator handles
  close/export — you do not touch bd.
- **Do NOT push** — the orchestrator verifies and pushes.
- **Do NOT run `wai close`** — the orchestrator owns session close.
- **ah/espectacular**: read-only over `openspec/`; contract TOMLs for
  change-phase scenarios are orphans until archive — do NOT author any.
- **Non-interactive shells**: use `-f`/`-rf`/`-y` flags; commands may be
  aliased with `-i` and will hang on prompts.

## Report format (end with this — the orchestrator verifies against it)

## Report

**Commits**
- `<hash>` <message> — <what it does, one line>

**Gates run**
- `just test` → <result>
- `ah check` → <result>
- `openspec validate --all --strict` → <result>

**Deviations** (or "none")
- <any scope/plan deviation and why>

**Next**
- <exact next action for the orchestrator, or "ticket complete">
