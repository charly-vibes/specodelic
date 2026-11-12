# Continuation brief: specodelic-d8c — finish tdd-ro5 run (steps 8-9)

You are a pi subagent in repo `/var/home/sasha/para/areas/dev/gh/charly/specodelic`.
A prior subagent session implementing ticket `specodelic-d8c` was killed by an
orchestrator timeout at tdd-ro5 step 8/9 (quality-ledger). All implementation
is ALREADY COMMITTED (49ea3c6 scaffold, 57f82d9 the lint rule, 4415e90 scenario
names). The archive was already applied via the sanctioned recipe: the change
now lives at `openspec/changes/archive/2026-11-11-add-ref-kind-enforcement/`,
the delta is deployed to `openspec/specs/linter-referential_integrity/spec.md`,
and `AGENTS.md` carries the refreshed rule catalog. The archive move, the
deployed spec, AGENTS.md, and 7 new `.espectacular/linter-referential_integrity/*.toml`
contract files are UNCOMMITTED debris you must land.

## Do

- `export WAI_PROJECT=specodelic`; `export TMPDIR=/var/tmp/specodelic-xch1`
  (mkdir -p it).
- `wai pipeline current --json` FIRST — it must show run_id starting
  `tdd-ro5-2026-11-12-specodelic-d8c-` (step 8/9). If it shows ANY other run,
  STOP and report; do not advance someone else's run.
- `wai pipeline next` and execute each remaining step's prompt literally
  (quality-ledger, ship-close) until the run completes. Never hand-edit run
  YAMLs.
- Ship-close commits — stage ONLY these paths (git add <paths>, never bare):
  - `.espectacular/linter-referential_integrity/*.toml` (the 7 new files)
  - `openspec/changes/add-ref-kind-enforcement` (deletions) and
    `openspec/changes/archive/2026-11-11-add-ref-kind-enforcement` (additions)
  - `openspec/specs/linter-referential_integrity/spec.md`
  - `AGENTS.md`
  - `.wai/pipeline-runs/tdd-ro5-2026-11-12-specodelic-d8c-*.yml` and any
    `.wai/projects/specodelic/` artifacts YOU authored this session
- Do NOT touch: `.genesis/tools.toml` (unexplained foreign change — leave
  uncommitted), `.wai/pipeline-runs/*2026-11-11-x0wy*` (orchestrator's),
  `src/**`, `tests/**` (already committed, do not re-touch).
- Do NOT push. Do NOT run `bd close`/`bd update`. Do NOT run `wai close`.
- Gates before your report: `just ci` exit 0; `ah check --run-tests`;
  `openspec validate --all --strict` clean; `just lint-specs` clean.
  If any gate fails, STOP and report — do not fix implementation code
  (that would exceed this continuation's scope); name the failure precisely.

## Report format (end with this)

## Report
**Commits**
- `<hash>` <message> — one line each
**Gates run**
- each command → result
**Deviations** (or "none")
**Next**
- exact next action for the orchestrator, or "ticket complete"
