# Subagent brief: specodelic-eczv.5 — conform phase 5: lifecycle independence and dogfood

You are a pi subagent in repo `/var/home/sasha/para/areas/dev/gh/charly/specodelic`
(Rust CLI `specodelic`, alias `spk`). An orchestrator has claimed ticket
`specodelic-eczv.5` and is hands-off: you own the implementation end-to-end.
The orchestrator will verify with real gates — never claim work the diff
doesn't contain.

## Orientation (do this first)

- `export WAI_PROJECT=domain-specific-extensions`
- Read, in order: `openspec/changes/add-conform/design.md` (D5, D6 — the
  evidence_scope wording is at design.md line ~89-94:
  "agreement on the supplied corpus; not a proof of behavioral equality"),
  `tasks.md` phase 5 (tasks 5.1–5.5), `proposal.md`, and the delta spec
  under `openspec/changes/add-conform/specs/`.
- Phases 1–4 already landed: `src/conform.rs` (engine, report, gate),
  `spk conform` CLI (exit-code contract), `tests/conform_classification.rs`,
  `tests/conform_report.rs`, `tests/conform_gate.rs`, `tests/cli/conform.rs`.
- `wai search "conform"` and `wai search "orchestrate stages"` — check
  accumulated patterns.
- Start and execute the tdd-ro5 pipeline literally:
  `wai pipeline start tdd-ro5 --topic="specodelic-eczv.5: conform phase 5 lifecycle and dogfood"`
  Advance ONLY via `wai pipeline next` — never hand-edit the run YAML.

## What to build

- **5.1 RED** `tests/conform_lifecycle.rs`: `outside_lifecycle` —
  `spk orchestrate` over a spec with conform available never invokes
  conform; pipeline stages unchanged; a conform run over a `model_checked`
  artifact leaves the stage unchanged.
- **5.2 GREEN**: assert orchestrate's stage list; NO orchestrate code change
  expected (constraint is conformance, not behavior).
- **5.3 RED→GREEN**: read-only dogfood — run conform against a FIXTURE COPY
  of a `specs/` corpus file + fixture oracle corpus; assert spec files and
  git worktree byte-identical after the run (`consumes_never_writes`).
  NEVER write to the real `specs/` corpus — fixture copies only.
- **5.4 GREEN**: dual-format gates — `spk lint` the delta clean,
  `just sync-sections` passes (ADDED/Requirements mirror identical),
  `spk lint openspec/specs` unaffected.
- **5.5 TIDY**: `just ci` full gate (see known pre-existing clippy --all-targets
  failure below — it is NOT yours; record it as pre-existing with the filed
  ticket id specodelic-9b1h); docs book pipeline section gains the conform
  page stating the evidence scope with D6 wording VERBATIM
  ("agreement on the supplied corpus; not a proof of behavioral equality").
  Find the docs book under `docs/` (mdbook) — check how other pipeline
  commands have pages and follow that pattern.

Check off tasks 5.1–5.5 in `openspec/changes/add-conform/tasks.md`
(checkbox edits only).

## Hard scope guard

- Allowed files: `tests/conform_lifecycle.rs` (new),
  `docs/` book pipeline section (new/updated conform page),
  `openspec/changes/add-conform/tasks.md` (task 5.x checkoffs only).
- Never edit: `openspec/specs/`, `.espectacular/`, `pretender.toml`,
  `specs/` (the REAL corpus — fixture copies live under a temp/fixture
  dir), `.wai/resources/**`, this template, `src/model_check.rs`,
  `src/orchestrate.rs`, `src/main.rs`, `src/conform.rs` (phase 5 should
  need NO source changes — it is a verification + docs phase).
- Respect shrink-only ratchets: run `just pretender-check` before
  committing.

## Repo facts (boilerplate — applies to every ticket)

- **TMPDIR gotcha**: `export TMPDIR=/var/tmp/specodelic-eczv` (directory
  already created) before any cargo test run.
- **Known pre-existing issue** (specodelic-9b1h, not yours to fix):
  `cargo clippy --all-targets -- -D warnings` fails in foreign files
  `src/lint/mod.rs:1605` and `src/migrate.rs:377`; `just lint` is clean.
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
- `just ci` → <result>
- `ah check` → <result>
- `openspec validate --all --strict` → <result>

**Deviations** (or "none")
- <any scope/plan deviation and why>

**Next**
- <exact next action for the orchestrator, or "ticket complete">
