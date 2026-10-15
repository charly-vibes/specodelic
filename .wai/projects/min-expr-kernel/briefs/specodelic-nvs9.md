# Subagent brief: specodelic-nvs9 — migrate wraps MODIFIED-only deltas (gh-16)

You are a pi subagent in repo `/var/home/sasha/para/areas/dev/gh/charly/specodelic`
(Rust CLI `specodelic`, alias `spk`). An orchestrator has claimed ticket
`specodelic-nvs9` and is hands-off: you own the implementation end-to-end.
The orchestrator will verify with real gates — never claim work the diff
doesn't contain.

## Orientation (do this first)

- `export WAI_PROJECT=min-expr-kernel`
- `bd show specodelic-nvs9` — the ticket carries a grounded Meter note
  (added 2026-10-15): current behavior is `MigrateError::NoDeltaSection` at
  `src/migrate.rs:57` ("no ## ADDED Requirements section found — nothing to
  wrap") — verified live; only deltas carrying `## ADDED Requirements` wrap.
- `wai search "migrate"` — accumulated patterns.
- Start your tdd-ro5 run and execute each step literally:
  `wai pipeline start tdd-ro5 --topic="specodelic-nvs9: migrate wraps MODIFIED-only deltas (gh-16)"`
  Advance ONLY via `wai pipeline next` — never hand-edit run YAMLs.

## What to build

`spk migrate` currently refuses openspec deltas that have no
`## ADDED Requirements` section — MODIFIED-only (and REMOVED-only) deltas
fail with `NoDeltaSection`. Upstream reporter (gh-16, charly-vibes/dont
adoption): 8 MODIFIED-only deltas were left plain, so `spk lint openspec`
keeps skipping them (no frontmatter).

Extend migrate to wrap deltas that carry a `## MODIFIED Requirements`
section (with or without an ADDED section), applying the same treatment:
in-place YAML frontmatter + scaffold layers + byte-identical `## Requirements`
mirror per the linter.requirement_drift aggregation rule (the mirror holds
each requirement's text once — MODIFIED body in its modified form, per the
54v mixed-delta implementation — read `src/migrate.rs` lines ~170-260 for
the existing mixed-delta machinery and reuse it).

Design notes to respect:
- A MODIFIED-only delta has no ADDED body to mirror verbatim — the mirror
  for MODIFIED-only deltas is the MODIFIED body (that is what
  requirement_drift compares for delta sections present in the file).
- Reuse the existing per-requirement mirror aggregation; do not fork a
  second mirror builder.
- Keep `MigrateError::NoDeltaSection` for files with NEITHER ADDED nor
  MODIFIED sections (plain specs still refuse via `NotADelta` when they
  have `## Requirements` already). Update the error message to name both
  accepted sections.
- REMOVED-only deltas: openspec REMOVED deltas delete requirements — a
  mirror cannot hold text for requirements the capability no longer has.
  If extending to REMOVED is non-trivial, do NOT half-land it: keep
  REMOVED refused with a clear labeled message and note the follow-up in
  your report.
- TDD: red test first in `tests/cli/migrate.rs` (MODIFIED-only delta
  migrate → lint exit 0, mirror holds the MODIFIED text), then implement,
  then tidy.

## Hard scope guard

- Allowed files: `src/migrate.rs`, `tests/cli/migrate.rs`
- Never edit: `openspec/specs/`, `.espectacular/`, `pretender.toml`,
  `specs/` (the corpus), `.wai/resources/**`, this template.
- Respect shrink-only ratchets: run `pretender check` before committing; if a
  pinned source file breaches, move new tests to `tests/` rather than raising
  entries. `src/model_check.rs` is pinned (see `pretender.toml`) — new
  model-check tests go in `tests/` integration files.

## Repo facts (boilerplate — applies to every ticket)

- **Commit hygiene** (standing, every commit): before ANY `git commit`, run
  `git status --short` and stage only files YOU authored
  (`git add <paths>`, never bare `git add`); unstage foreign files.
  Attribute the message only to what the diff contains.
- **Output discipline**: every command emits through
  `genesis::guide::Output::emit`; errors carry a remediation hint.
- **beads**: issues live in `.beads/issues.jsonl`; commit the export after
  `bd close` (`bd export` gotcha: run it or the close is invisible to git).
- **Do NOT push** — the orchestrator verifies and pushes.
- **Do NOT run `wai close`** — the orchestrator owns session close.
- **ah/espectacular**: read-only over `openspec/`; contract TOMLs for
  change-phase scenarios are orphans until archive — author them in the
  archive commit, never mid-change.
- **New source files** need Purpose/Responsibilities/Rationale headers
  (file-headers skill convention).
- **Non-interactive shells**: use `-f`/`-rf`/`-y` flags; commands may be
  aliased with `-i` and will hang on prompts.
- **Set TMPDIR** (`export TMPDIR=/var/tmp/<your-session-slug>` created
  fresh) before running test gates — /tmp quota exhaustion masquerades as
  hundreds of test failures.

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
