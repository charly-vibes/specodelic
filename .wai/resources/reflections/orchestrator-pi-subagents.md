# Orchestrator pattern: pi subagents + wai pipeline (tdd-ro5)

Established 2026-10-06 during specodelic-txo (slice 1 of add-min-expr-kernel).

## Loop

The session lead **orchestrates only**: tracks bd/openspec/wai state, verifies
gates, pushes. Each ticket's pipeline steps execute in a **pi subagent**
(`pi -p "<brief>"` spawned from the repo cwd).

1. Orchestrator: claim ticket (`bd update <id> --claim`), gate/verify openspec
   change, land any HITL gates first.
2. Orchestrator: drive pipeline steps that need fresh in-context work OR run
   them inline when trivial; commit green before spawning (subagents start
   from a clean tree).
3. Spawn subagent with a written brief (scope guard: exact file list; repo
   facts: ratchets, bd export gotcha, commit hygiene, no push, no `wai close`).
4. Verify subagent report: `git log`, gates (`just test`, `ah check`,
   `openspec validate`), beads state. Then push + close.

## Session/usage recording (non-negotiable)

- **Never** spawn subagents with `--no-session` — usage/cost records are lost.
- Always pass `-n "subagent:<ticket>:<steps>"` so sessions are discoverable:
  `grep -l "subagent:<ticket>" ~/.pi/agent/sessions/<cwd-key>/*.jsonl`.
- Sessions persist to `~/.pi/agent/sessions/--var-home-...-<repo>--/`.

## Conventions that bit once

- wai pipelines referencing oracle `tests-pass` need
  `.wai/resources/oracles/tests-pass.sh` (exit 0 = `just test` green).
- wai `add`/`pipeline` commands need `WAI_PROJECT=<name>` when the workspace
  has multiple projects.
- ah binds contract TOMLs to **deployed** scenarios only: change-phase TOMLs
  are `orphan-toml` (gate red without overlay, `no-toml` with overlay). Author
  TOMLs in the archive commit; stash WIP TOMLs outside `.espectacular/`.
- pretender `file_lines` ratchet is shrink-only: when a source file breaches,
  move new tests to `tests/` rather than raising entries.
- `src/model_check.rs` pinned at 2300 lines (2284 as of 2026-10-06); new
  model-check tests go to `tests/citation_algebra.rs` or a new integration file.