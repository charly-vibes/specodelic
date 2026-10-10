# Subagent brief: specodelic-eczv.4 — conform phase 4: CLI wiring and exit-code contract

You are a pi subagent in repo `/var/home/sasha/para/areas/dev/gh/charly/specodelic`
(Rust CLI `specodelic`, alias `spk`). An orchestrator has claimed ticket
`specodelic-eczv.4` and is hands-off: you own the implementation end-to-end.
The orchestrator will verify with real gates — never claim work the diff
doesn't contain.

## Orientation (do this first)

- `export WAI_PROJECT=domain-specific-extensions`
- Read, in order: `openspec/changes/add-conform/proposal.md` (What Changes),
  `design.md` (D2, D6), `tasks.md` phase 4 (tasks 4.1–4.3), and the delta
  spec under `openspec/changes/add-conform/specs/`.
- Phases 1–3 already landed: `src/conform.rs` exposes the verdict engine,
  report (`build_report`, `emit_report`), gate, and corpus validation.
  Phase 4 wires them to the CLI.
- Study existing `Commands` enum + exit-code conventions in `src/main.rs`
  and existing CLI tests in `tests/cli/` (e.g. how other commands assert
  envelopes and exit codes). Reuse genesis CLI helpers.
- `wai search "conform"` and `wai search "exit code"` — check accumulated
  patterns.
- Start and execute the tdd-ro5 pipeline literally:
  `wai pipeline start tdd-ro5 --topic="specodelic-eczv.4: conform phase 4 CLI wiring"`
  Advance ONLY via `wai pipeline next` — never hand-edit the run YAML.

## What to build

- `spk conform <file> --oracle scenarios.jsonl` as a `Commands` variant in
  `src/main.rs` (help text stating READ-ONLY evaluation).
- `--closed-world` accepted and recorded in verdict records + report header.
- Exit-code contract: 0 = no `forbidden`/`unsupported` verdict; 1 = any
  `forbidden`/`unsupported`; `unknown`/`underspecified` surfaced as counts
  but never failing; 2 = invocation error / gate refusal with ZERO verdict
  records emitted.
- `open_world_never_forbidden` asserted at CLI level.
- Completions regenerate (follow repo's completions workflow — check how
  other commands' completions are generated/tracked).
- `spk explain` UNTOUCHED — no new topic this change (proposal contract).

TDD discipline per tasks.md phase 4:

- 4.1 **RED**: CLI test in `tests/cli/` — report envelope produced;
  `--closed-world` recorded; open-world runs never contain `forbidden`
  (`open_world_never_forbidden`). Observe RED before implementation.
- 4.2 **GREEN**: add the `Conform` command; wire the exit-code contract.
- 4.3 **TIDY**: completions regenerate; confirm `spk explain` untouched.
  Separate tidy commit if it touches different files than 4.2.

Check off tasks 4.1–4.3 in `openspec/changes/add-conform/tasks.md`
(checkbox edits only).

## Hard scope guard

- Allowed files: `src/main.rs`, `tests/cli/` (new conform test file),
  completions output (however the repo tracks it),
  `openspec/changes/add-conform/tasks.md` (task 4.x checkoffs only),
  `src/conform.rs` ONLY if the CLI shim needs a tiny helper there.
- Never edit: `openspec/specs/`, `.espectacular/`, `pretender.toml`,
  `specs/` (the corpus), `.wai/resources/**`, this template,
  `src/model_check.rs`, explain topics.
- Respect shrink-only ratchets: run `just pretender-check` before
  committing; `src/main.rs` may be pinned — if it breaches, move test
  logic to `tests/cli/` rather than raising entries.

## Repo facts (boilerplate — applies to every ticket)

- **TMPDIR gotcha**: `export TMPDIR=/var/tmp/specodelic-eczv` (directory
  already created) before any cargo test run.
- **Known pre-existing issue** (specodelic-9b1h, not yours to fix):
  `cargo clippy --all-targets -- -D warnings` fails in foreign files
  `src/lint/mod.rs:1605` and `src/migrate.rs:377`; `just lint` is clean.
  Verify your NEW code via `just lint`.
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
