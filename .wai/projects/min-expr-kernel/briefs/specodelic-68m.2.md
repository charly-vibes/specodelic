# Subagent brief: specodelic-68m.2 — Keep aggregate verdicts consistent across report views

You are a pi subagent in repo `/var/home/sasha/para/areas/dev/gh/charly/specodelic`
(Rust CLI `specodelic`, alias `spk`). An orchestrator has claimed ticket
`specodelic-68m.2` and is hands-off: you own the implementation end-to-end.
The orchestrator will verify with real gates — never claim work the diff
doesn't contain.

## Orientation (do this first)

- `export WAI_PROJECT=min-expr-kernel`
- Read, in order: the openspec change's `design.md` (esp. decision rows this
  ticket depends on), `tasks.md` (this ticket's section), `proposal.md` (slice
  scoping), then the delta specs this ticket extends.
- `wai search "<topic>"` — check accumulated patterns before designing.
- This is a **TIDY (cleanup-only) ticket**. There is no tdd-ro5 run for it;
  do NOT start one. Do NOT advance any pipeline — the orchestrator owns the
  epic-orchestrator run.

## What to build

tasks.md task **1.3**: "TIDY: separate ticket/commit consolidates aggregate
rendering only after the command fixtures pass unchanged."

Design row **D4 — One aggregation implementation** is the target end-state:
model-check, verify, and orchestrate share claim classification and aggregate
rules — one aggregation implementation, not three divergent renderers.

Concretely:

1. Establish green characterization first: confirm the command fixtures from
   specodelic-68m.1 (7 fixtures across model_check / verify / orchestrate)
   pass at HEAD. Record the exact commands and results.
2. Consolidate the aggregate-verdict rendering/reporting paths so
   model-check, verify, and orchestrate surface aggregate verdicts through
   the shared implementation (same rules, same rendering helpers) instead of
   duplicated per-command logic.
3. **Zero behavior change**: accepted inputs, output payloads, and exit
   statuses must be byte-identical for all existing fixtures. If consolidation
   would change any payload, STOP — that is a semantic change; retag DESIGN
   and report back instead of guessing.
4. Keep the change in a single separate commit (TIDY commit, distinct from
   any feature work).
5. After landing, update ONLY the `1.3` checkbox in
   `openspec/changes/define-verification-claim-gates/tasks.md` (leave every
   other checkbox untouched).

## Hard scope guard

- Allowed files: `src/model_check.rs`, `src/commands/model_check.rs`,
  `src/orchestrate.rs`, `src/verify.rs`, `src/human.rs`,
  `tests/cli/model_check.rs`, `tests/cli/orchestrate_verify.rs`,
  `openspec/changes/define-verification-claim-gates/tasks.md` (the 1.3
  checkbox only).
- Never edit: `openspec/specs/`, `.espectacular/`, `pretender.toml`,
  `specs/` (the corpus), `.wai/resources/**`, this template.
- Respect shrink-only ratchets: run `pretender check` before committing; if a
  pinned source file breaches, move new tests to `tests/` rather than raising
  entries. `src/model_check.rs` is pinned (see `pretender.toml`) — new
  model-check tests go in `tests/` integration files.
- If consolidation requires touching files outside the allowed list, STOP and
  report — do not expand scope.

## Repo facts (boilerplate — applies to every ticket)

- **Commit hygiene** (standing, every commit): before ANY `git commit`, run
  `git status --short` and stage only files YOU authored
  (`git add <paths>`, never bare `git add`); unstage foreign files.
  Attribute the message only to what the diff contains.
- **Output discipline**: every command emits through
  `genesis::guide::Output::emit`; errors carry a remediation hint.
- **beads**: issues live in `.beads/issues.jsonl`; commit the export after
  `bd close` (`bd export` gotcha: run it or the close is invisible to git).
  — **you may NOT close this ticket**; the orchestrator owns close.
- **Do NOT push** — the orchestrator verifies and pushes.
- **Do NOT run `wai close`** — the orchestrator owns session close.
- **ah/espectacular**: read-only over `openspec/`; contract TOMLs for
  change-phase scenarios are orphans until archive — author them in the
  archive commit, never mid-change.
- **New source files** need Purpose/Responsibilities/Rationale headers
  (file-headers skill convention). If a consolidation extracts a new module,
  it needs a header and stays inside the allowed scope.
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
