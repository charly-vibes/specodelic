# Subagent brief: specodelic-eczv.3 — conform phase 3: input gate (artifacts, corpus validation)

You are a pi subagent in repo `/var/home/sasha/para/areas/dev/gh/charly/specodelic`
(Rust CLI `specodelic`, alias `spk`). An orchestrator has claimed ticket
`specodelic-eczv.3` and is hands-off: you own the implementation end-to-end.
The orchestrator will verify with real gates — never claim work the diff
doesn't contain.

## Orientation (do this first)

- `export WAI_PROJECT=domain-specific-extensions`
- Read, in order: `openspec/changes/add-conform/design.md` (decisions D5, D3
  — input gate, corpus validation, mechanical name identity), `tasks.md`
  phase 3 (tasks 3.1–3.4), `proposal.md`, and the delta spec under
  `openspec/changes/add-conform/specs/`.
- Phases 1–2 already landed: `src/conform.rs` (verdict engine + report
  schema: `build_report`, `scope_digest`, `human_view`/`emit_report`),
  `tests/conform_classification.rs`, `tests/conform_report.rs`. Read both.
- Study how `orchestrate` applies the artifact currency check between
  stages (find it in `src/orchestrate.rs` or adjacent) — the gate must REUSE
  it (D5), not re-implement.
- `wai search "conform"` and `wai search "artifact currency"` — check
  accumulated patterns before designing.
- Start and execute the tdd-ro5 pipeline literally:
  `wai pipeline start tdd-ro5 --topic="specodelic-eczv.3: conform phase 3 input gate"`
  Advance ONLY via `wai pipeline next` — never hand-edit the run YAML.

## What to build

Command-path input gate + corpus validation in `src/conform.rs` +
`tests/conform_gate.rs` (new). NO CLI command registration (phase 4 wires
`spk conform`; build the gate as a testable library function the CLI will
call).

- Input gate (D5): target file must parse and lint clean and its compiled
  artifacts be current (reuse orchestrate's artifact currency check);
  refusals carry remediation hints naming `spk lint` / `spk compile` and
  emit ZERO verdict records (`stale_artifacts_refused`,
  `lint_dirty_refused`).
- Scenario corpus validation (D3): malformed JSONL lines, missing `id`,
  duplicate ids, unknown fields — all refused with remediation hints, never
  silently ignored.
- Zero-line corpus: valid report with zero records, `evidence_scope`
  intact, exit-0-equivalent (Ok result) — never an error.
- orchestrate's own currency-check behavior must keep passing untouched.

TDD discipline per tasks.md phase 3 (each test observed RED before GREEN):

- 3.1 **RED**: `tests/conform_gate.rs` — `stale_artifacts_refused`,
  `lint_dirty_refused`, both with zero verdict records.
- 3.2 **GREEN**: gate in the command path reusing orchestrate's currency
  check; refusal carries the remediation hint per the error contract.
- 3.3 **RED→GREEN**: the four corpus-validation refusals.
- 3.4 **RED→GREEN**: `empty corpus` scenario — valid report, zero records,
  evidence_scope intact.
- TIDY: separate tidy commit if warranted; check off tasks 3.1–3.4 in
  `openspec/changes/add-conform/tasks.md` (checkbox edits only).

## Hard scope guard

- Allowed files: `src/conform.rs`, `tests/conform_gate.rs` (new),
  `openspec/changes/add-conform/tasks.md` (task 3.x checkoffs only).
  If reusing orchestrate's currency check requires a visibility tweak
  (e.g. `pub(crate)`), that single-line change in the orchestrate path is
  allowed — note it in your report.
- Never edit: `openspec/specs/`, `.espectacular/`, `pretender.toml`,
  `specs/` (the corpus), `.wai/resources/**`, this template,
  `src/model_check.rs`, `src/orchestrate.rs` beyond the visibility tweak.
- Respect shrink-only ratchets: run `just pretender-check` before
  committing; never raise pinned entries.

## Repo facts (boilerplate — applies to every ticket)

- **TMPDIR gotcha**: `export TMPDIR=/var/tmp/specodelic-eczv` (directory
  already created) before any cargo test run.
- **Known pre-existing issue** (specodelic-9b1h, not yours to fix):
  `cargo clippy --all-targets -- -D warnings` fails in foreign files
  `src/lint/mod.rs:1605` and `src/migrate.rs:377`; the repo's own gate
  (`just lint`) is clean. Verify your NEW code via `just lint`.
- **Commit hygiene** (standing, every commit): before ANY `git commit`, run
  `git status --short` and stage only files YOU authored
  (`git add <paths>`, never bare `git add`); unstage foreign files.
  Attribute the message only to what the diff contains.
- **Output discipline**: every command emits through
  `genesis::guide::Output::emit`; errors carry a remediation hint.
- **beads**: the orchestrator handles close/export — you do not touch bd.
- **Do NOT push** — the orchestrator verifies and pushes.
- **Do NOT run `wai close`** — the orchestrator owns session close.
- **ah/espectacular**: read-only over `openspec/`; no contract TOMLs
  mid-change.
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
