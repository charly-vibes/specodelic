# Subagent brief: specodelic-68m.6 — Keep assurance guidance consistent after documentation cleanup

You are a pi subagent in repo `/var/home/sasha/para/areas/dev/gh/charly/specodelic`
(Rust CLI `specodelic`, alias `spk`). An orchestrator has claimed ticket
`specodelic-68m.6` and is hands-off: you own the implementation end-to-end.
The orchestrator will verify with real gates — never claim work the diff
doesn't contain.

## Orientation (do this first)

- `export WAI_PROJECT=min-expr-kernel`
- Read, in order: the openspec change's `design.md` (esp. **D5 — Explain
  assurance levels**), `tasks.md` §3, `proposal.md`, then the docs the
  ticket touches.
- `wai search "<topic>"` — check accumulated patterns before designing.
- This is a **TIDY (cleanup-only) ticket**. There is no tdd-ro5 run for it;
  do NOT start one. Do NOT advance any pipeline — the orchestrator owns the
  epic-orchestrator run.
- A parallel session is working graph-view tickets (`src/graph.rs`,
  `src/main.rs`, `tests/cli/parse_misc.rs`) — never stage or commit its
  files.

## What to build

tasks.md task **3.3**: "TIDY: separate documentation cleanup commit removes
stale restatements."

Concretely:

1. Establish green characterization first: the docs-consistency check
   landed in 68m.5 (version literals vs Cargo metadata + capability-status
   assertions) passes at HEAD, and `just ci` is green. Record commands and
   results.
2. Remove stale repeated assurance claims across the docs — restatements of
   the verification/assurance semantics that duplicate or contradict the
   canonical statement (post-68m.5 canonical content). Retain canonical
   references: where a doc needs to mention assurance, link/point to the
   canonical statement instead of restating it.
3. Preserve the D5 capability table and the application-test-execution
   distinction — cleanup removes *stale restatements*, not the substance.
4. **Zero behavior change**: docs-only edits; all checks (docs consistency,
   corpus lint, tests) must behave identically.
5. Keep the change in a single separate commit (TIDY commit, distinct from
   any feature work).
6. After landing, update ONLY the `3.3` checkbox in
   `openspec/changes/define-verification-claim-gates/tasks.md` (leave every
   other checkbox untouched).

## Hard scope guard

- Allowed files: `README.md`, `docs/src/status.md`, `src/guide.rs` (and its
  `include_str!` content file `src/guide.md`), `specs/STATUS.md`,
  `openspec/changes/define-verification-claim-gates/tasks.md` (the 3.3
  checkbox only).
- Never edit: `openspec/specs/`, `.espectacular/`, `pretender.toml`,
  `specs/**` except `specs/STATUS.md`, `.wai/resources/**`, this template,
  `src/graph.rs`, `src/main.rs`, `tests/cli/parse_misc.rs`,
  `openspec/changes/add-graph-views/**` (parallel ticket's files).
- If a stale restatement lives in a file outside the allowed list, STOP and
  report it instead of expanding scope.

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
