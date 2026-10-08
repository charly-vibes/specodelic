# Subagent brief: specodelic-5c4 — Keep migrated corpus claims lint-clean without new exemptions

You are a pi subagent in repo `/var/home/sasha/para/areas/dev/gh/charly/specodelic`
(Rust CLI `specodelic`, alias `spk`). An orchestrator has claimed ticket
`specodelic-5c4` and is hands-off: you own the implementation end-to-end.
The orchestrator will verify with real gates — never claim work the diff
doesn't contain.

## Orientation (do this first)

- `export WAI_PROJECT=min-expr-kernel`
- Read, in order: the openspec change's `design.md`, `tasks.md` §6 (task
  6.3), `proposal.md`, then the migrated corpus files.
- `wai search "<topic>"` — check accumulated patterns before designing.
- This is a **TIDY (cleanup-only) ticket**. There is no tdd-ro5 run for it;
  do NOT start one. Do NOT advance any pipeline — the orchestrator owns the
  epic-orchestrator run.
- A parallel session is working graph-view tickets (`src/graph.rs`,
  `src/main.rs`, `tests/cli/parse_misc.rs`) — never stage or commit its
  files.

## What to build

tasks.md task **6.3**: "**TIDY**: no new advisory class introduced; if a
migration candidate seems to need one, STOP and record why." Plus: review
and tidy the migrated corpus wording; preserve every executable claim and
evidence fixture.

Concretely:

1. Establish green characterization first: `just lint-specs &&
   just lint-doc-examples && cargo test --test cli` at HEAD (record
   results; lint baseline must show 0 accepted additions).
2. Review the migrated kernel wording in `specs/USAGE.md` (commits
   `aa21795`, `ddf80a1`) and `specs/specodelic.md` (commit `0fd474c`):
   tighten prose around the new `**kernel:**` rows, confirm each kernel row
   still carries its coverage property row, confirm the eligibility
   decisions recorded in 7yk (cells left informal) read coherently.
3. Record explicitly WHY no new advisory class is required (the
   zero-exemptions lint result is the evidence) — put this in the commit
   message and the report.
4. **Zero behavior change**: all extracted fixtures keep passing with
   identical statuses; lint results identical (0 findings, 0 new
   exemptions).
5. Keep the change in a single separate commit (TIDY commit).
6. After landing, update ONLY the `6.3` checkbox in
   `openspec/changes/add-min-expr-kernel/tasks.md`.
7. Known RO5U low findings from 7yk — `src/lint/graph.rs:523` stale prose
   quote and `specs/linter-model_shape.md:22` prose restatement — are
   OUTSIDE this ticket's file list. Do NOT edit them; note them in your
   report as candidates for the bxk/68m.7 revision chores.

## Hard scope guard

- Allowed files: `specs/USAGE.md`, `specs/specodelic.md`,
  `openspec/changes/add-min-expr-kernel/tasks.md` (the 6.3 checkbox only).
- Never edit: `openspec/specs/`, `.espectacular/`, `pretender.toml`,
  `specs/**` except the two allowed corpus files, `.wai/resources/**`, this
  template, `src/**` (except none — src is out of scope entirely),
  `tests/**`, `scripts/**`.
- If the design requires touching files outside the allowed list, STOP and
  report — do not expand scope.

## Repo facts (boilerplate — applies to every ticket)

- **Commit hygiene** (standing, every commit): before ANY `git commit`, run
  `git status --short` and stage only files YOU authored
  (`git add <paths>`, never bare `git add`); unstage foreign files.
- **Output discipline**: every command emits through
  `genesis::guide::Output::emit`; errors carry a remediation hint.
- **beads**: issues live in `.beads/issues.jsonl`. — **you may NOT close
  this ticket**; the orchestrator owns close.
- **Do NOT push** — the orchestrator verifies and pushes.
- **Do NOT run `wai close`** — the orchestrator owns session close.
- **ah/espectacular**: read-only over `openspec/`; contract TOMLs for
  change-phase scenarios are orphans until archive — author them in the
  archive commit, never mid-change.
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