# Subagent brief: specodelic-gre.9 — Readers can rebuild diagrams and follow embedded render recipes

You are a pi subagent working in worktree
`/var/home/sasha/para/areas/dev/gh/charly/specodelic-gre-wt`
(Rust CLI `specodelic`, alias `spk`; branch `gre/docs-wiring` off
origin/main). All commands run there; the orchestrator works in the main repo
and is hands-off: you own the implementation end-to-end. The orchestrator will
verify with real gates — never claim work the diff doesn't contain.

## Orientation (do this first)

- Read, in order: `openspec/changes/add-graph-views/tasks.md` tasks
  **3.1, 3.2, 3.3, 3.4** (this ticket's tasks), design.md **D5/D6** (docs
  wiring; generated views gitignored), D8 (render recipes), then the landed
  surface you document: `--format edges|dot|mermaid`, `--view wiring`,
  `guide --schema --json`, `scripts/graph_views.py` (states/trace/schema
  subcommands).
- Start your tdd-ro5 pipeline and execute each step's prompt literally:
  `wai pipeline start tdd-ro5 --topic="specodelic-gre.9: docs-graphs wiring and primer (tasks 3.1-3.4)"`
  Advance ONLY via `wai pipeline next`. Use
  `export WAI_PROJECT=domain-specific-extensions`.

## What to build

- **3.1** `just docs-graphs` recipe: regenerate all views into
  `docs/src/views/` (gitignore that dir FIRST — D6); assert `git status`
  clean after a full build (RED first: fails while recipe absent — record
  commands and results). Produce graph and schema envelopes with the same
  binary, pass the schema export to the script, keep intermediate exports
  untracked.
- **3.2** Docs page `docs/src/graph-views.md` consuming the generated
  includes: views for this repo's own corpus plus the revision-labeled
  schema view; one sentence of philosophy — views are never more current or
  more correct than the graph artifact.
- **3.3** Wire `docs-graphs` into the docs build path (`justfile`); do NOT
  add it to `just ci` gates in v1 (rendering is build-time only).
- **3.4** `spk explain graph-views` primer topic (APPENDED at the end of the
  topic list, never renumbered): the view taxonomy, format flags, and
  one-pipe render recipes (`| dot -Tsvg`, `graph-easy` for ASCII terminal,
  mermaid paste targets); docs pages updated. The explain topic lives in
  `src/guide.rs` — keep the edit additive (new topic appended; zero changes
  to existing topic numbering or text).
- Meter: `just docs-graphs && just ci`. Task checkboxes 3.1-3.4 only.

## Hard scope guard

- Allowed files: `justfile`, `.gitignore`, `src/guide.rs`,
  `docs/src/SUMMARY.md`, `docs/src/graph-views.md` (new),
  `tests/ci_wiring.rs`, `openspec/changes/add-graph-views/tasks.md`
  (checkboxes 3.1-3.4 only). `docs/src/views/` is generated output —
  gitignored, never committed.
- Never edit: `openspec/specs/`, `.espectacular/`, `pretender.toml`,
  `specs/` (the corpus), `.wai/resources/**`, this template,
  `scripts/graph_to_dot.jq`, `src/acset/schema.rs`, `Cargo.toml`,
  `Cargo.lock`. No new dependencies. Do NOT add docs-graphs to `just ci`.
- `src/guide.rs` is shared with the parallel 68m epic's landed work — keep
  your diff strictly additive (new topic + its data at the end).
- Respect shrink-only ratchets: run `pretender check` before committing;
  if `src/guide.rs` breaches a pinned threshold, SPLIT the topic data into a
  new module rather than raising the threshold.
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
- `just docs-graphs` → <result>
- `just ci` → <result>
- `ah check` → <result>
- `openspec validate --all --strict` → <result>

**Deviations** (or "none")
- <any scope/plan deviation and why>

**Next**
- <exact next action for the orchestrator, or "ticket complete">