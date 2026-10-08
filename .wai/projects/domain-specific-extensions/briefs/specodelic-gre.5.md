# Subagent brief: specodelic-gre.5 — Schema diagrams render the canonical typing rules without duplication

You are a pi subagent working in worktree
`/var/home/sasha/para/areas/dev/gh/charly/specodelic-gre-wt`
(Rust CLI `specodelic`, alias `spk`; branch `gre/schema-diagrams` off
origin/main). All commands run there; the orchestrator works in the main repo
and is hands-off: you own the implementation end-to-end. The orchestrator will
verify with real gates — never claim work the diff doesn't contain.

## Orientation (do this first)

- Read, in order: `openspec/changes/add-graph-views/tasks.md` tasks
  **2.1, 2.2, 2.3 (schema portions), 2.6** (this ticket's tasks),
  design.md **D4** (schema view derives from the acset `Schema` value;
  `guide::REFERENCE_TYPING` const stays OUT of the derivation path;
  revision label from `guide::FORMAT_REVISION`), D8, and the proposal's
  schema-view slice.
- Start your tdd-ro5 pipeline and execute each step's prompt literally:
  `wai pipeline start tdd-ro5 --topic="specodelic-gre.5: guide schema JSON and schema diagrams (tasks 2.1-2.3)"`
  Advance ONLY via `wai pipeline next`. Use
  `export WAI_PROJECT=domain-specific-extensions` (registry ambiguity in the
  worktree).

## What to build

- **2.1 RED**: CLI fixtures pin `guide --json` to kinds, row shapes and
  format_revision, and `guide --schema --json` to D4's versioned schema
  payload. Assert canonical objects/morphisms, refinements, source rules,
  endo flags and sorted deterministic data; no guide typing constants. Both
  selectors currently absent — run and observe failure first; record
  commands and results.
- **2.2 GREEN**: implement the guide JSON command and schema selector,
  emitting through genesis Output::emit. Share a Schema-to-data exporter
  between `canonical()` production input and constructed test values;
  leave reference typing out of the ordinary guide value-set payload.
- **2.3 (schema portions) RED**: script tests (`scripts/test_graph_views.py`)
  for the schema view fed serialized output from two valid constructed
  schemas differing in ONE morphism; assert the corresponding 1-edge
  diagram difference and the revision label. Missing fields, unknown
  version, unsuccessful envelope, duplicate identities and dangling
  endpoints fail `schema_export_invalid` before output writes (≥5 malformed
  cases).
- **2.6 GREEN**: implement the schema view consuming ONLY the schema export
  from 2.2, including format revision and refinement labels. New
  `scripts/graph_views.py` + `scripts/test_graph_views.py` (both new paths;
  pytest/unittest per the meter). Acceptance: two valid schema exports
  differing by 1 morphism produce the matching 1-edge diagram difference;
  versioned sorted data includes every object/morphism/refinement/source
  rule/endo flag; ≥5 malformed-export cases fail `schema_export_invalid`
  before output; 0 Rust-source parsing and NO duplicate guide typing table.
- Task checkboxes 2.1/2.2/2.3/2.6 only in tasks.md (2.3 only its schema
  portion — leave the corpus-fixture legs unticked for gre.6/gre.7).

## Hard scope guard

- Allowed files: `src/main.rs`, `src/guide.rs`, `src/acset/schema.rs`,
  `scripts/graph_views.py` (new), `scripts/test_graph_views.py` (new),
  `tests/cli/parse_misc.rs`, `openspec/changes/add-graph-views/tasks.md`
  (checkboxes 2.1/2.2/2.3/2.6 only).
- Never edit: `openspec/specs/`, `.espectacular/`, `pretender.toml`,
  `specs/` (the corpus), `.wai/resources/**`, this template,
  `scripts/graph_to_dot.jq`, `Cargo.toml`, `Cargo.lock`.
- No new dependencies. Rust-source parsing of the typing table is FORBIDDEN
  (D4: the Schema value is the single source; the lint gate
  `schema_matches_typing_table` already polices it).
- Respect shrink-only ratchets: run `pretender check` before committing; if
  `tests/cli/parse_misc.rs` approaches its 2300-line cap, move new byte-pins
  to unit tests over pure functions in `src/` (gre.4 precedent) — never
  raise the threshold.
- Parallel orchestrator sessions work the 68m epic (specs/, docs/src/,
  .espectacular/, README.md) and another epic in the main repo. Do NOT touch
  those files; if main moves mid-run, `git rebase origin/main` and re-run
  `just ci`.

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
- Test runner during iteration: `just test-smart`; full `just ci` PLUS
  `python3 -m unittest discover -s scripts -p test_graph_views.py` (the
  meter) before you report done.

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