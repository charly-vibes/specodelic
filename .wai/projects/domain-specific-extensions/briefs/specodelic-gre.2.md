# Subagent brief: specodelic-gre.2 — Keep edge projections byte-stable after formatting cleanup

You are a pi subagent working in worktree
`/var/home/sasha/para/areas/dev/gh/charly/specodelic-gre-wt`
(Rust CLI `specodelic`, alias `spk`; branch `gre/edge-projection-tidy` off
origin/main). All commands run there; the orchestrator works in the main repo
and is hands-off: you own the implementation end-to-end. The orchestrator will
verify with real gates — never claim work the diff doesn't contain.

## Orientation (do this first)

- Read, in order: `openspec/changes/add-graph-views/tasks.md` §1.4 (this
  ticket's ONLY task), design.md D2/D3 (the contract you must NOT change),
  and the gre.1 slice already landed (`src/graph.rs::edges_tsv`,
  `--format edges` in `src/main.rs`, the 9 CLI tests in
  `tests/cli/parse_misc.rs`).
- This is a CLEANUP-ONLY ticket: **zero changes to accepted inputs, output
  payloads or exit statuses**. The meter is `just ci` with all existing
  fixtures passing WITHOUT weakening any assertion.

## What to build (task 1.4 TIDY)

- Establish green characterization first: the existing `--format edges` CLI
  fixtures ARE the characterization — run them and confirm green BEFORE any
  edit. If coverage for byte-stability is thin, ADD a characterization test
  pinning current bytes first (separate commit), then refactor (separate
  commit).
- Extract projection formatting into a testable unit in `src/graph.rs`
  (pure function over the `GraphReport` / edge+violation sets — no I/O, no
  CLI state), so tasks 1.5/1.6 (dot/mermaid/wiring) can reuse the
  normalization and column machinery.
- Dead-flag and clippy sweep (`just ci` includes clippy `-D warnings`).
- Any preparatory tidy and subsequent refactoring are SEPARATE COMMITS from
  characterization. Do not bundle a semantic change — there must be none.
- Check task 1.4's checkbox in
  `openspec/changes/add-graph-views/tasks.md` only (nothing else).

## Hard scope guard

- Allowed files: `src/graph.rs`, `src/main.rs`, `tests/cli/parse_misc.rs`,
  `openspec/changes/add-graph-views/tasks.md` (checkbox 1.4 only).
- Never edit: `openspec/specs/`, `.espectacular/`, `pretender.toml`,
  `specs/` (the corpus), `.wai/resources/**`, this template, `src/guide.rs`.
- Respect shrink-only ratchets: run `pretender check` before committing.
- A parallel orchestrator session works `specodelic-68m.4` in the main repo —
  it touches `src/model_check.rs`, `src/orchestrate.rs`, `src/verify.rs`,
  `tests/cli/model_check.rs`, `tests/cli/orchestrate_verify.rs`. Do NOT touch
  those files; if main moves mid-run, `git rebase origin/main` and re-run
  `just ci`.

## Repo facts (boilerplate — applies to every ticket)

- **Commit hygiene** (standing, every commit): before ANY `git commit`, run
  `git status --short` and stage only files YOU authored
  (`git add <paths>`, never bare `git add`); unstage foreign files.
  Attribute the message only to what the diff contains.
- **Output discipline**: `--format edges`'s stdout override is the ONE
  documented exception (D3) — keep it scoped to the flag.
- **beads**: the orchestrator closes the ticket; you do NOT run `bd close`.
- **Do NOT push** — the orchestrator verifies and pushes.
- **Do NOT run `wai close`** — the orchestrator owns session close.
- **ah/espectacular**: read-only over `openspec/`.
- **New source files** need Purpose/Responsibilities/Rationale headers;
  contract-changing edits update Rationale.
- **Non-interactive shells**: use `-f`/`-rf`/`-y` flags; commands may be
  aliased with `-i` and will hang on prompts.
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