# Subagent brief: specodelic-gre.6 — State diagrams preserve distinct states and show guarded transitions

You are a pi subagent working in worktree
`/var/home/sasha/para/areas/dev/gh/charly/specodelic-gre-wt`
(Rust CLI `specodelic`, alias `spk`; branch `gre/state-diagrams` off
origin/main). All commands run there; the orchestrator works in the main repo
and is hands-off: you own the implementation end-to-end. The orchestrator will
verify with real gates — never claim work the diff doesn't contain.

## Orientation (do this first)

- Read, in order: `openspec/changes/add-graph-views/tasks.md` tasks
  **1.7 (rendering portion), 2.3 (corpus/scope portion), 2.4** (this
  ticket's tasks), design.md **D8** (visual grammar), D2 (multiplicity:
  fan-in counts DISTINCT targets; raw TSV retains duplicate instances),
  D3 (violations ride along). Then study the landed pieces you build on:
  `src/graph.rs` (`GraphReport` — edges, violations, fan_in; the raw TSV
  already retains both from/to transition edges per gre.1's 1.7 pin) and
  `scripts/graph_views.py` + `scripts/test_graph_views.py` (gre.5's schema
  view: scope validation, `schema_export_invalid`, fixture-corpus test
  pattern — reuse its structure and helpers).
- Start your tdd-ro5 pipeline and execute each step's prompt literally:
  `wai pipeline start tdd-ro5 --topic="specodelic-gre.6: per-file state diagrams (tasks 1.7-render, 2.3-corpus, 2.4)"`
  Advance ONLY via `wai pipeline next`. Use
  `export WAI_PROJECT=domain-specific-extensions`.

## What to build

- **2.4 GREEN — per-file state-machine view** (the Python transform's first
  graph-artifact-driven view): transition edges grouped by owning file;
  guards annotated (dashed, per D8); files without transitions skipped
  cleanly; fan-in counts distinct targets per D2's multiplicity rule.
- **1.7 rendering portion**: the state-machine rendering retains the two
  states of a two-state one-transition fixture as exactly 2 distinct states
  (the raw TSV side of 1.7 already landed in gre.1 — you own only the
  render side). Only file-level views collapse to an owning intent — the
  state view must NOT collapse.
- **2.3 corpus/scope portion**: check in the three fixture corpora the task
  names — empty (zero spec files = intentless: script asserts
  `out_of_scope_refused`, exits non-zero with a remediation hint),
  single-intent (`single_intent_sane`), violation-bearing — plus a
  lint-dirty fixture for the other `out_of_scope_refused` leg. Assert:
  parse of the TSV contract, valid Mermaid output, violations rendered as
  annotated elements. (The schema-leg of 2.3 landed in gre.5 — do not
  duplicate it; tick only your legs.)
- **0 markdown re-walks**: the script consumes `spk graph --json` /
  `--format edges` / `guide --schema --json` artifacts ONLY — never parses
  corpus markdown. Validate script scope (the out_of_scope_refused legs)
  BEFORE output.
- Task checkboxes: 2.4 fully; 1.7 render leg + 2.3 corpus legs (annotate
  the partial ticks like gre.5 did for 2.3).
- Meter: `cargo test --test cli` AND
  `python3 -m unittest discover -s scripts -p test_graph_views.py`.

## Hard scope guard

- Allowed files: `scripts/graph_views.py`, `scripts/test_graph_views.py`,
  checked-in fixture corpora under `tests/fixtures/` (new dirs/files for the
  empty / single-intent / violation-bearing / lint-dirty corpora),
  `tests/cli/parse_misc.rs`, `openspec/changes/add-graph-views/tasks.md`.
  Touch `src/graph.rs` / `src/main.rs` ONLY if a Rust-side change is
  strictly required (e.g. a missing graph JSON field) — prefer not to; if
  you must, keep it additive and note it in the report.
- Never edit: `openspec/specs/`, `.espectacular/`, `pretender.toml`,
  `specs/` (the corpus), `.wai/resources/**`, this template,
  `scripts/graph_to_dot.jq`, `src/guide.rs`, `src/acset/schema.rs`,
  `Cargo.toml`, `Cargo.lock`. No new dependencies.
- Respect shrink-only ratchets: `tests/cli/parse_misc.rs` is at
  ~2283/2300 lines — add as little as possible there (fixture corpora are
  files, not inline strings; prefer `tests/fixtures/`). Run
  `pretender check` before committing.
- Parallel orchestrator sessions work other epics in the main repo. Do NOT
  touch their files; if main moves mid-run, `git rebase origin/main` and
  re-run the gates.

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
  `/tmp/specodelic-*` scratch caches — do not report infra failures as code
  failures without checking.

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