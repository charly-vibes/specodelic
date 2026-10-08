# Subagent brief: specodelic-gre.4 — Wiring diagrams expose declared producer-consumer connections

You are a pi subagent working in worktree
`/var/home/sasha/para/areas/dev/gh/charly/specodelic-gre-wt`
(Rust CLI `specodelic`, alias `spk`; branch `gre/wiring-view` off
origin/main). All commands run there; the orchestrator works in the main repo
and is hands-off: you own the implementation end-to-end. The orchestrator will
verify with real gates — never claim work the diff doesn't contain.

## Orientation (do this first)

- Read, in order: `openspec/changes/add-graph-views/tasks.md` task **1.6**
  (this ticket's ONLY task), design.md **D8**, the wiring decision record
  `openspec/research/2026-10-01-wiring-view-decision/decision.md`, then the
  landed projection machinery in `src/graph.rs` (`edge_projection`,
  `ProjectionRow`, `render_dot`, `render_mermaid`) and the `GraphFormat`
  enum / `cmd_graph_projection` dispatcher in `src/main.rs`.
- Start your tdd-ro5 pipeline and execute each step's prompt literally:
  `wai pipeline start tdd-ro5 --topic="specodelic-gre.4: wiring view (task 1.6)"`
  Advance ONLY via `wai pipeline next`. Use
  `wai --project domain-specific-extensions` or `export WAI_PROJECT=domain-specific-extensions`
  (registry ambiguity in the worktree).

## What to build (task 1.6 RED→GREEN)

`--view wiring`: file-level producer→consumer projection of
`constraints.satisfies` edges (specodelic-5qj decision):
- Project `constraints.satisfies` to FILE level (owning intent/file of each
  endpoint), exposing the declared inter-file producer→consumer edges.
- Self-loops dropped.
- Corpora with zero satisfies edges emit a labeled `no_wiring` view, never a
  silently clean diagram.
- Works across the output formats already shipped (`--format edges|dot|mermaid`
  must all honor `--view wiring` per the flag design in the proposal;
  if the proposal pins a narrower interaction, follow the proposal and
  document it in the flag help — do not guess).
- RED first: tests must fail for the intended reason (flag absent) — record
  commands and results. Task checkbox 1.6 only in tasks.md.

## Hard scope guard

- Allowed files: `src/graph.rs`, `src/main.rs`, `tests/cli/parse_misc.rs`,
  `openspec/changes/add-graph-views/tasks.md` (checkbox 1.6 only).
- Never edit: `openspec/specs/`, `.espectacular/`, `pretender.toml`,
  `specs/` (the corpus), `.wai/resources/**`, this template, `src/guide.rs`,
  `scripts/graph_to_dot.jq`, `scripts/graph_views.py`.
- No new dependencies — Cargo.toml and Cargo.lock must not change.
- Respect shrink-only ratchets: run `pretender check` before committing.
- A parallel orchestrator session works the 68m epic in the main repo
  (`src/model_check.rs`, `src/orchestrate.rs`, `src/verify.rs`,
  `src/human.rs`, `src/guide.rs`, `tests/cli/model_check.rs`,
  `tests/cli/orchestrate_verify.rs`). Do NOT touch those files; if main
  moves mid-run, `git rebase origin/main` and re-run `just ci`.

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
- Test runner during iteration: `just test-smart`; full `just ci` gate
  before you report done.

## Report format (end with this — the orchestrator verifies against it)

## Report

**Commits**
- `<hash>` <message> — <what it does, one line>

**Gates run**
- `just ci` → <result>
- `ah check` → <result>
- `openspec validate --all --strict` → <result>

**Deviations** (or "none")
- <any scope/plan deviation and why>

**Next**
- <exact next action for the orchestrator, or "ticket complete">