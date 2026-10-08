# Subagent brief: specodelic-bxk — Kernel capabilities retain complete contracts when archived

You are a pi subagent in repo `/var/home/sasha/para/areas/dev/gh/charly/specodelic`
(Rust CLI `specodelic`, alias `spk`). An orchestrator has claimed ticket
`specodelic-bxk` and is hands-off: you own the implementation end-to-end.
The orchestrator will verify with real gates — never claim work the diff
doesn't contain.

## Orientation (do this first)

- `export WAI_PROJECT=min-expr-kernel`
- Read, in order: the openspec change's `design.md`, `tasks.md` §7 (tasks
  7.1–7.4), `proposal.md`, `specs/AGENTS.md` (revision discipline), the
  dual-format archive recipe in
  `openspec/decisions/2026-10-01-espectacular-adoption.md` and the spike
  findings under `openspec/changes/archive/2026-09-28-spike-dual-format/`.
- `wai search "<topic>"` — check accumulated patterns before designing.
- This ticket **maps to a tdd-ro5 run** for its RED→GREEN parts, but its
  heart is chores + archive. Use judgment: run tdd-ro5 only if a genuine
  RED→GREEN cycle exists; otherwise document the chore steps as wai
  research artifacts. Do NOT advance the epic-orchestrator pipeline.
- **Known stale-TOML stash**: `/tmp/txo-contracts/` holds 5 contract TOMLs
  authored for slice-1 scenarios (specodelic-txo). Verify they are present
  and current before relying on them — the /tmp quota was exhausted earlier
  today, so re-check contents against the current scenarios and re-author
  if stale. Prefer fresh authoring over blind reuse.
- A parallel session is working gre.6 (`scripts/graph_views.py`,
  `scripts/test_graph_views.py`, `tests/cli/parse_misc.rs`). **You both
  touch parse_misc.rs** — see conflict protocol below.

## What to build

tasks.md tasks **7.1–7.4**:

1. **7.1 CHORE — FORMAT_REVISION bump**: bump the corpus revision under the
   format's Revision rules: `FORMAT_REVISION` const in `src/guide.rs:20` +
   ALL `'specodelic.md Revision N'` literals (tests/cli.rs ×several:
   `--version --json`, doctor format_revision ×2, init AGENTS.md block,
   skew advisory; `tests/cli/parse_misc.rs:787`; any others — grep). New
   Constraints introduced by the change need deriving Properties (the
   coverage lint blocks compile corpus-wide otherwise). Regenerate compiled
   corpus artifacts via `just ci`.
2. **7.2 GREEN — per-scenario contract TOMLs**: author genuine contract
   TOMLs for EVERY scenario added by this change, bound to the real cargo
   tests (use the stashed TOMLs as exemplars; re-author fresh where
   scenarios evolved). `.espectacular/` is writable ONLY for new contract
   TOMLs — never touch its managed config.
3. **7.3 GREEN — `delta_self_contained` verification** per capability in
   the change's specs (the dual-format layer: frontmatter + Constraints/
   Model/Properties tables must survive without the change wrapper).
4. **7.4 TIDY — archive**: run `openspec validate add-min-expr-kernel
   --strict` (green), `just ci` (green), `ah check --changes
   add-min-expr-kernel` (and executed checks when supported), then archive
   ONLY via the sanctioned **dual-format recipe**:
   `openspec archive add-min-expr-kernel --skip-specs` + verbatim `cp` of
   the specodelic-format spec files into `openspec/specs/` (default archive
   DESTROYS the frontmatter/Constraints/Model/Properties tables — the cp
   step is mandatory; follow the spike findings exactly). Update
   `openspec/changes/add-min-expr-kernel/tasks.md` checkboxes 7.1–7.4 only.
5. Hard gate: 0 stale revision literals and 0 missing deployed scenario
   bindings. Meter: `just ci && ah check && openspec validate
   add-min-expr-kernel --strict` before archive; after archive: `just ci &&
   ah check && openspec validate --all --strict` with 0 orphans.
6. Preserve every retained requirement identity through the archive
   (all-requirements-dry review before deleting the change dir).
7. **No push, no release** — orchestrator-owned.

## File-conflict protocol (parse_misc.rs vs parallel gre.6)

- Before EACH commit: `git status --short`; stage ONLY your hunks/files.
- If `tests/cli/parse_misc.rs` shows concurrent modification (foreign
  staged/unstaged hunks or unexpected upstream commits), run `git pull
  --rebase` first; if rebase conflicts in parse_misc.rs, STOP and report
  instead of hand-merging.
- Keep parse_misc.rs edits minimal and append-oriented where possible.

## Hard scope guard

- Allowed files: `src/guide.rs`, `tests/cli.rs` (revision-literal fixtures),
  `tests/cli/model_check.rs`, `tests/cli/parse_misc.rs`, `.espectacular/`
  (new contract TOMLs only), `openspec/changes/add-min-expr-kernel/**`,
  `openspec/specs/**` (ONLY via the dual-format archive cp — verbatim copies,
  no hand edits), `docs/src/SUMMARY.md` (kernel capability entry if missing),
  regenerated `specodelic/` artifacts.
- Never edit: `.espectacular/config.toml` and managed blocks,
  `.wai/resources/**`, this template, `src/graph.rs`, `src/main.rs`,
  `scripts/graph_views.py`, `scripts/test_graph_views.py`,
  `openspec/changes/add-graph-views/**` (parallel ticket's files).
- Respect shrink-only ratchets: run `pretender check` before committing.
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
- **ah/espectacular**: the ONLY sanctioned write to `openspec/specs/` is the
  dual-format archive cp in this ticket.
- **New source files** need Purpose/Responsibilities/Rationale headers.
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