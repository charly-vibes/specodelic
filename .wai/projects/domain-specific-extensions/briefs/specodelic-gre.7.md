# Subagent brief: specodelic-gre.7 — Traceability diagrams show file dependencies and distinct-source fan-in

You are a pi subagent working in worktree
`/var/home/sasha/para/areas/dev/gh/charly/specodelic-gre-wt`
(Rust CLI `specodelic`, alias `spk`; branch `gre/traceability-view` off
origin/main). All commands run there; the orchestrator works in the main repo
and is hands-off: you own the implementation end-to-end. The orchestrator will
verify with real gates — never claim work the diff doesn't contain.

## Orientation (do this first)

- Read, in order: `openspec/changes/add-graph-views/tasks.md` task **2.5**
  (this ticket's ONLY task), design.md **D2** (fan-in counts DISTINCT
  targets/sources; multiplicity), D3, D8. Then study the landed script
  modules: `scripts/graph_views.py` (thin CLI entry),
  `scripts/view_common.py`, `scripts/state_view.py` + cases modules
  (gre.6's states view — reuse its structure: scope gate, fixture corpus
  loading, consumer-test split), and `scripts/schema_view.py`.
- Start your tdd-ro5 pipeline and execute each step's prompt literally:
  `wai pipeline start tdd-ro5 --topic="specodelic-gre.7: file-level traceability view (task 2.5)"`
  Advance ONLY via `wai pipeline next`. Use
  `export WAI_PROJECT=domain-specific-extensions`.

## What to build (task 2.5 RED→GREEN)

The file-level traceability view, from graph artifacts only (0 markdown
re-walks; scope validated before output, same `out_of_scope_refused` legs as
the states view):
- Collapse ONLY file-level projections to owning intents (gre.1's raw TSV
  keeps row-level IDs; this view collapses).
- Exactly 1 node per intent in fixtures (including a single-intent corpus).
- File dependencies as edges; fan-in annotation counts DISTINCT source
  intents — repeated edges from one source increase fan-in by 0 beyond its
  first contribution (D2 multiplicity rule; the raw TSV's duplicate
  instances are intentionally collapsed HERE, view-level).
- 0 silently omitted violation annotations — violations stay annotated
  (they are not recorded edges; follow gre.6's precedent for how the states
  view handles lint/violation visibility and stay consistent).
- Distinct-source fan-in ("distinct-source fan-in" per the ticket title).
- RED first: tests fail for the intended reason (view absent) — record
  commands and results. Task checkbox 2.5 only in tasks.md.
- Meter: `python3 -m unittest discover -s scripts -p test_graph_views.py`
  (and keep `just ci` green).

## Hard scope guard

- Allowed files: `scripts/graph_views.py`, `scripts/view_common.py`,
  `scripts/test_graph_views.py`, new `scripts/traceability_view*.py` case
  modules (ratchet-driven split, same pattern as gre.6), new fixture corpus
  files under `tests/fixtures/` if needed (reuse existing
  `zero_file`/`single_intent`/`typing_violations`/`lint_dirty` where
  possible), `tests/cli/graph_views.rs`,
  `openspec/changes/add-graph-views/tasks.md` (checkbox 2.5 only).
- Never edit: `openspec/specs/`, `.espectacular/`, `pretender.toml`,
  `specs/` (the corpus), `.wai/resources/**`, this template,
  `scripts/graph_to_dot.jq`, `src/guide.rs`, `src/acset/schema.rs`,
  `src/graph.rs`, `src/main.rs`, `Cargo.toml`, `Cargo.lock`. No new
  dependencies. A Rust-side change must NOT be needed for this view — if
  you believe one is, STOP and retag DESIGN rather than guessing.
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