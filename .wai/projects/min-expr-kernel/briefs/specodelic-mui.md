# Subagent brief: specodelic-mui — every opted-in kernel claim appears in command results

You are a pi subagent in repo `/var/home/sasha/para/areas/dev/gh/charly/specodelic`
(Rust CLI `specodelic`, alias `spk`). An orchestrator has claimed ticket
`specodelic-mui` and is hands-off: you own the implementation end-to-end.
The orchestrator will verify with real gates — never claim work the diff
doesn't contain.

## Orientation (do this first)

- `export WAI_PROJECT=min-expr-kernel`
- Read, in order: `openspec/changes/add-min-expr-kernel/design.md` (esp. D1
  grounding, D2 agreement, D9 citation-resolution rows),
  `tasks.md` §"Corrective integration amendment" (task 3.7 — yours),
  `proposal.md` (slice scoping), then the delta specs this ticket extends.
- `wai search "kernel"` and `wai search "model-check"` — check accumulated
  patterns before designing.
- Your ticket maps to a tdd-ro5 run — start it and execute each step's
  prompt literally:
  `wai pipeline start tdd-ro5 --topic="specodelic-mui: §3.7 — kernel claims across the complete invocation corpus"`
  Advance ONLY via `wai pipeline next` — never hand-edit the run YAML under
  `.wai/pipeline-runs/`.
- Baseline facts: hb4 (§3.6, citation resolution over the invocation corpus,
  commit aa414e9) is landed — §3.7 builds directly on its scope-threading
  work. Phase 4's agreement property (36n) is in `just ci`.

## What to build

add-min-expr-kernel **§3.7 (RED→GREEN)** — run compile → model-check on
kernel-only and mixed Rust/kernel fixtures (two states versus cardinality
999). Every opted-in `kernel:` claim appears exactly once in command output;
evaluate over the complete explicit corpus. Reversed file order yields
identical statuses; empty input is an invocation error. Thread scope through
CLI and orchestrate; incapable backends return labeled unsupported results.
**Test actual command output (CLI assertions), not a direct KernelEnv-only
harness.**

Ticket meter (hard gate): the two-state cardinality-999 fixture is a
`counterexample` in both CLI JSON output and the persisted report, alongside
passing Rust claims in the same run. At least 5 fixtures cover: mixed
input, kernel-only input, reversed file order, empty input, incapable
backend. Zero omitted claims or fabricated state traces. The RED test must
be observed failing for the intended reason before GREEN — record commands
and results; a zero-test filtered run is not evidence.

## Hard scope guard

- Allowed files: `src/model_check.rs`, `src/commands/model_check.rs`,
  `src/orchestrate.rs`, `src/kernel.rs`, `src/acset/instance.rs`,
  `tests/cli/model_check.rs`, new files under `tests/` (e.g.
  `tests/kernel_corpus.rs`), `openspec/changes/add-min-expr-kernel/tasks.md`
  (only the §3.7 checkbox).
- Never edit: `openspec/specs/`, `.espectacular/`, `pretender.toml`,
  `specs/` (the corpus), `.wai/resources/**`, this template,
  `openspec/changes/add-min-expr-kernel/{proposal,design}.md`.
- Respect shrink-only ratchets: run `pretender check` before committing; if a
  pinned source file breaches, move new tests to `tests/` rather than raising
  entries. `src/model_check.rs` is pinned at 2300 lines (2284 as of
  2026-10-06) — new model-check tests go in `tests/` integration files, NOT
  inline in `src/model_check.rs`.
- Agent test runner during iteration: `just test-smart` (falls back to plain
  `cargo test`); the `just ci` gate stays the orchestrator's.

## Repo facts (boilerplate — applies to every ticket)

- **Commit hygiene** (standing, every commit): before ANY `git commit`, run
  `git status --short` and stage only files YOU authored
  (`git add <paths>`, never bare `git add`); unstage foreign files.
  Attribute the message only to what the diff contains.
- **Output discipline**: every command emits through
  `genesis::guide::Output::emit`; errors carry a remediation hint.
- **beads**: issues live in `.beads/issues.jsonl`; commit the export after
  `bd close` (`bd export` gotcha: run it or the close is invisible to git).
- **Do NOT push** — the orchestrator verifies and pushes.
- **Do NOT run `wai close`** — the orchestrator owns session close.
- **ah/espectacular**: read-only over `openspec/`; contract TOMLs for
  change-phase scenarios are orphans until archive — author them in the
  archive commit, never mid-change.
- **New source files** need Purpose/Responsibilities/Rationale headers
  (file-headers skill convention).
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
