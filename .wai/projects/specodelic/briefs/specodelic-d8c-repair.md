# Continuation brief 2: specodelic-d8c — repair archived layer.md, land debris, close tdd-ro5 run

You are a pi subagent in repo `/var/home/sasha/para/areas/dev/gh/charly/specodelic`.
Ticket `specodelic-d8c` implementation is committed (49ea3c6, 57f82d9, 4415e90).
The change is archived at `openspec/changes/archive/2026-11-11-add-ref-kind-enforcement/`.
`just ci` fails ONLY at `lint-deltas` on
`openspec/changes/archive/2026-11-11-add-ref-kind-enforcement/specs/linter-referential_integrity/layer.md`:
16 findings = 2× `linter.terminal_states_emit` (labels parent-namespaced
`linter.referential_integrity.index_failure`/`.resolution_failure` instead of
file-owned `linter.referential_integrity.layer.*`) + 14× `linter.coverage`
(each constraint lacks an own-file deriving property).

## Do

- `export WAI_PROJECT=specodelic`; `export TMPDIR=/var/tmp/specodelic-xch1`.
- Repair ONLY that `layer.md` sidecar:
  1. Re-namespace the 2 emitted labels to `[[linter.referential_integrity.layer.<label>]]`
     (keep the failure terminals emitting exactly one file-owned effect
     Constraint each — terminal_states_emit).
  2. Author deriving Properties rows for all 14 constraints (kind=unit,
     real generator/predicate cells — mirror the shape of the 8 existing
     archived layer sidecars that lint clean; find them under
     `openspec/changes/archive/*/specs/*/layer.md`). Keep ids unique in-file;
     keep guards/coverage satisfied.
  3. Optionally run `spk feedback bug --dry-run` to capture the suspected
     `spk migrate` namespace-derivation bug and INCLUDE the dry-run output in
     your report (do NOT file upstream, do NOT run bd).
- Then verify: `just ci` exit 0 (the debris is uncommitted — ci runs on the
  working tree, that is expected); `ah check --run-tests` still 0 findings;
  `openspec validate --all --strict`; `just lint-specs` clean.
- If green: tick the archived change's `tasks.md` boxes that are NOW honestly
  satisfied by landed work (the prior report flagged 4.1, 5.1–5.5 unchecked —
  tick each ONLY if its deliverable verifiably exists in the tree; leave any
  that remain false unticked and name them).
- Commit (git add <paths>, never bare git add):
  - `openspec/changes/archive/2026-11-11-add-ref-kind-enforcement/**`
    (layer.md repair + tasks.md ticks)
  - `.espectacular/linter-referential_integrity/*.toml` (all 10)
  - `openspec/specs/linter-referential_integrity/spec.md`
  - `AGENTS.md`
  - `.wai/pipeline-runs/tdd-ro5-2026-11-12-specodelic-d8c-*.yml` + any
    `.wai/projects/specodelic/` artifacts you authored
  - `.beads/issues.jsonl` ONLY if bd changed (it should not)
- Then `wai pipeline current --json` must show run_id starting
  `tdd-ro5-2026-11-12-specodelic-d8c-` at step 9/9 — `wai pipeline next` to
  complete the run, executing its ship-close prompt EXCEPT: no push, no
  `bd close`, no `wai close` (orchestrator owns those).
- Never touch: `.genesis/tools.toml`, `.wai/pipeline-runs/*x0wy*`, `src/**`,
  `tests/**`, the deployed `openspec/specs/**` beyond what is already done.

## Report format (end with this)

## Report
**Commits**
- `<hash>` <message> — one line each
**Gates run**
- each command → result
**Deviations** (or "none")
**Next**
- exact next action for the orchestrator, or "ticket complete"
