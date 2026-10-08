# Subagent brief: specodelic-gre.3 — Users can pipe graph projections into DOT and Mermaid renderers

You are a pi subagent working in worktree
`/var/home/sasha/para/areas/dev/gh/charly/specodelic-gre-wt`
(Rust CLI `specodelic`, alias `spk`; branch `gre/dot-mermaid` off
origin/main). All commands run there; the orchestrator works in the main repo
and is hands-off: you own the implementation end-to-end. The orchestrator will
verify with real gates — never claim work the diff doesn't contain.

## Orientation (do this first)

- Read, in order: `openspec/changes/add-graph-views/tasks.md` task **1.5**
  (this ticket's ONLY task), design.md **D8** (native text projections,
  visual grammar, zero crates, byte-stable, no `--render`) and **D2/D3**
  (canonical ids, violation annotation rows), then gre.1/gre.2's landed
  machinery in `src/graph.rs` (`edge_projection`, `ProjectionRow`,
  `canonical_id`, `escape_tab_free`, `render_tsv_row`) — reuse it.
- Start your tdd-ro5 pipeline and execute each step's prompt literally:
  `wai pipeline start tdd-ro5 --topic="specodelic-gre.3: native dot/mermaid projections (task 1.5)"`
  Advance ONLY via `wai pipeline next` — never hand-edit the run YAML.
  The run file lives in `~/.wai/pipeline-runs/`.
- `wai search "dot mermaid projection"` first.

## What to build (task 1.5 RED→GREEN)

`--format dot` and `--format mermaid` on the `Graph` command emit
plain-text graph projections (zero external crates; byte-stable re-runs):
- Visual grammar (D8): solid = state machine, dashed = guards, bold =
  `emits`, dotted = traceability, red dashed = dangling/violations.
- Parity fixture: the retired `scripts/graph_to_dot.jq` output pinned as the
  expected dot shape (the script still exists in `scripts/` — use its output
  shape, do NOT modify or delete it).
- Violations render as dashed/annotated elements (D3) — never silently clean.
- Same flag-precedence rule as `--format edges` (raw text to stdout,
  overriding `--json`/`--human`; documented in the flag's help).
- Reuse the pure projection core from gre.2; if reuse requires extracting a
  bit more, keep it in `src/graph.rs` and note it in the report.
- RED first: tests must fail for the intended reason (flags absent) — record
  commands and results. Task checkbox 1.5 only in tasks.md.

## Hard scope guard

- Allowed files: `src/graph.rs`, `src/main.rs`, `tests/cli/parse_misc.rs`,
  `openspec/changes/add-graph-views/tasks.md` (checkbox 1.5 only).
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