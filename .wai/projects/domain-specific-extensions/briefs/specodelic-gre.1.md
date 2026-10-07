# Subagent brief: specodelic-gre.1 — Raw graph edges preserve every node and violation

You are a pi subagent working in worktree
`/var/home/sasha/para/areas/dev/gh/charly/specodelic-gre-wt`
(Rust CLI `specodelic`, alias `spk`; branch `gre/edge-projection` off main).
All commands run there; the orchestrator works in the main repo and is
hands-off: you own the implementation end-to-end. The orchestrator will
verify with real gates — never claim work the diff doesn't contain.

## Orientation (do this first)

- `export WAI_PROJECT=domain-specific-extensions` (run wai from the MAIN repo
  cwd if wai state is needed; the worktree is where you edit/test/commit)
- Read, in order: `openspec/changes/add-graph-views/design.md` (esp. **D2**
  canonical-ids-only and **D3** violations-ride-along + flag-precedence
  exception), `tasks.md` §1 (tasks **1.1, 1.2, 1.3, 1.7** — this ticket's
  slice; do NOT do 1.4/1.5/1.6), `proposal.md` (slice scoping).
- `wai search "graph edges TSV"` — check accumulated patterns before designing.
- Start your tdd-ro5 pipeline:
  `wai pipeline start tdd-ro5 --topic="specodelic-gre.1: raw graph edge projection (--format edges)"`
  Advance ONLY via `wai pipeline next` — never hand-edit the run YAML.

## What to build

Expose `spk graph --format edges`: a deterministic six-column TSV preserving
qualified row IDs, duplicate edge instances and every violation; document
escaping and output precedence.

- **1.1 RED**: CLI tests in `tests/cli/parse_misc.rs` for the projection
  contract: sorted TSV shape (six columns), byte-identical re-runs, endpoints
  are canonical node IDs (intent IDs and qualified row IDs — assert NO
  label-qualified endpoints like `refactor (intent)` using a fixture corpus
  that triggers them), zero-file directory exits 0 with EMPTY output, single
  intent corpus yields a well-formed row set. Run the tests — all new tests
  must FAIL for the intended reason (the flag does not exist yet). Record
  commands and results.
- **1.2 GREEN**: canonical-id normalization (D2) — normalize label-qualified
  nodes at or before projection in `src/graph.rs`; add `--format edges` to the
  `Graph` command in `src/main.rs` emitting the TSV to stdout, overriding
  `--json`/`--human` per D3's flag-precedence rule (documented exception to
  Output::emit — document it in the flag's help). Decide the annotation column
  content — full reason text vs class code (D3 open question) — against real
  output and document it in the flag's help. If full reason text is chosen,
  pin reason strings tab-free (or escaped) so the six-column TSV contract
  holds.
- **1.3 RED→GREEN**: violation annotation rows — fixture corpus with known
  typing violations; assert one annotation row per violation and zero on a
  clean corpus (`violations_survive_projection`, `clean_corpus_no_annotations`).
- **1.7 (raw-projection portion)**: pin two states and one transition in one
  file — raw TSV retains distinct qualified IDs and both from/to edges;
  retain duplicate edge instances (multiplicity, D2).

TSV row shape (D3): recorded edges = `from, edge_kind, to` mapped into six
columns; violation annotation rows = empty source id/kind,
`violation:<edge_kind>` in the field column, target id/kind, annotation
column carrying the finding.

## Hard scope guard

- Allowed files: `src/graph.rs`, `src/main.rs`, `tests/cli/parse_misc.rs`,
  `openspec/changes/add-graph-views/tasks.md` (checkboxes 1.1/1.2/1.3/1.7 only).
- Never edit: `openspec/specs/`, `.espectacular/`, `pretender.toml`,
  `specs/` (the corpus), `.wai/resources/**`, this template, `src/guide.rs`.
- Respect shrink-only ratchets: run `pretender check` before committing; if a
  pinned source file breaches, move new tests to `tests/` rather than raising
  entries.
- The meter is `cargo test --test cli` — but the RED step must run and fail
  for the intended reason before GREEN; a zero-test filtered run is not
  evidence.

## Repo facts (boilerplate — applies to every ticket)

- **Commit hygiene** (standing, every commit): before ANY `git commit`, run
  `git status --short` and stage only files YOU authored
  (`git add <paths>`, never bare `git add`); unstage foreign files.
  Attribute the message only to what the diff contains.
- **Output discipline**: every command emits through
  `genesis::guide::Output::emit`; errors carry a remediation hint. The
  `--format edges` stdout override is the ONE documented exception (D3) —
  scope it to the flag, not the command.
- **beads**: the orchestrator closes the ticket; you do NOT run `bd close`.
- **Do NOT push** — the orchestrator verifies and pushes.
- **Do NOT run `wai close`** — the orchestrator owns session close.
- **ah/espectacular**: read-only over `openspec/`.
- **New source files** need Purpose/Responsibilities/Rationale headers;
  contract-changing edits update Rationale.
- **Non-interactive shells**: use `-f`/`-rf`/`-y` flags; commands may be
  aliased with `-i` and will hang on prompts.
- Test runner during iteration: `just test-smart` (falls back to
  `cargo test`); full `just ci` gate before you report done.

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