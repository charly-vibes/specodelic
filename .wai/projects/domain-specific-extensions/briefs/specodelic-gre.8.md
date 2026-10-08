# Subagent brief: specodelic-gre.8 — Keep derived diagrams unchanged when prose or renderer structure changes

You are a pi subagent working in worktree
`/var/home/sasha/para/areas/dev/gh/charly/specodelic-gre-wt`
(Rust CLI `specodelic`, alias `spk`; branch `gre/views-stable-tidy` off
origin/main). All commands run there; the orchestrator works in the main repo
and is hands-off: you own the implementation end-to-end. The orchestrator will
verify with real gates — never claim work the diff doesn't contain.

## Orientation (do this first)

- Read, in order: `openspec/changes/add-graph-views/tasks.md` task **2.7**
  (this ticket's ONLY task), design.md **D2/D3/D8**. Then study the landed
  script modules: `scripts/graph_views.py` (thin CLI entry),
  `scripts/view_common.py`, `scripts/schema_view.py` / `state_view.py` /
  `traceability_view.py` + their cases modules, and
  `scripts/test_graph_views.py` (gre.5/6/7's work).
- This is a CLEANUP-ONLY ticket: **zero changes to accepted inputs, output
  payloads or exit statuses**. The meter is
  `python3 -m unittest discover -s scripts -p test_graph_views.py` with all
  fixtures passing WITHOUT weakening assertions; `just ci` must stay green.
- Start your tdd-ro5 pipeline and execute each step's prompt literally:
  `wai pipeline start tdd-ro5 --topic="specodelic-gre.8: derived diagrams stable under prose perturbation (task 2.7)"`
  Advance ONLY via `wai pipeline next`. Use
  `export WAI_PROJECT=domain-specific-extensions`.

## What to build (task 2.7 TIDY, two legs)

1. **Characterization first (RED for the missing test)**: a
   `views_from_artifact_only` test — perturb the PROSE blocks (frontmatter
   statement, prose under headings, anything outside structured tables) of a
   checked-in fixture corpus and assert every view output is byte-identical.
   Write the test, run it, observe it pass on current code (it pins existing
   correct behavior) — if it FAILS, that is a real bug: stop and retag
   DESIGN rather than "fixing" the view to match prose. If no checked-in
   fixture fits, add a prose-perturbed copy of an existing fixture corpus
   under `tests/fixtures/` (do NOT modify existing fixtures in place).
2. **TIDY (separate commit)**: shared rendering helpers — the three views
   (schema/state/traceability) duplicate scope-gating, envelope loading,
   TSV parsing, fan-in/distinct-counting, Mermaid element emission and
   no-output labeling. Extract genuinely shared pieces into
   `view_common.py` (or new `render_common.py` if cleaner), leaving each
   view's semantics untouched. Keep the ratchet discipline (file ≤300
   lines, fn ≤59 — split rather than grow).
- Task checkbox 2.7 only in tasks.md.

## Hard scope guard

- Allowed files: `scripts/graph_views.py`, `scripts/view_common.py`,
  `scripts/schema_view.py`, `scripts/state_view.py`,
  `scripts/traceability_view.py`, their cases modules,
  `scripts/test_graph_views.py`, new fixture files under `tests/fixtures/`,
  `openspec/changes/add-graph-views/tasks.md` (checkbox 2.7 only).
- Never edit: `openspec/specs/`, `.espectacular/`, `pretender.toml`,
  `specs/` (the corpus), existing fixtures (in place), `.wai/resources/**`,
  this template, `scripts/graph_to_dot.jq`, anything under `src/`,
  `Cargo.toml`, `Cargo.lock`. No new dependencies.
- Respect shrink-only ratchets; run `pretender check` before committing.
- Parallel orchestrator sessions work other epics in the main repo. If main
  moves mid-run, `git rebase origin/main` and re-run the gates.

## Repo facts (boilerplate — applies to every ticket)

- **Commit hygiene** (standing, every commit): before ANY `git commit`, run
  `git status --short` and stage only files YOU authored
  (`git add <paths>`, never bare `git add`); unstage foreign files.
  Attribute the message only to what the diff contains.
- **beads**: the orchestrator closes the ticket; you do NOT run `bd close`.
- **Do NOT push** — the orchestrator verifies and pushes.
- **Do NOT run `wai close`** — the orchestrator owns session close.
- **ah/espectacular**: read-only over `openspec/`.
- **New source files** need Purpose/Responsibilities/Rationale headers;
  contract-changing edits update Rationale.
- **Non-interactive shells**: use `-f`/`-rf`/`-y` flags.
- /tmp quota gotcha: if tests fail with `QuotaExceeded`, clear stale
  `/tmp/specodelic-*` scratch caches.

## Report format (end with this — the orchestrator verifies against it)

## Report

**Commits**
- `<hash>` <message> — <what it does, one line>

**Gates run**
- `just ci` → <result>
- `python3 -m unittest discover -s scripts -p test_graph_views.py` → <result>
- `ah check` → <result>
- `openspec validate --all --strict` → <result>

**Deviations** (or "none")
- <any scope/plan deviation and why>

**Next**
- <exact next action for the orchestrator, or "ticket complete">