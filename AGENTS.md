# AGENTS.md

Standing instructions for any agent working in this repo.

## What this repo is

**specodelic** — a Rust CLI (`ddl`) for the Specodelic specification
format (formerly `spec-format`). The format corpus lives in `specs/`:
every file there is a markdown spec written in the format it describes.
`specs/specodelic.md` is the core; `specs/STATUS.md` §1 is the primer.

- Format questions → `specs/specodelic.md` (single source of truth).
- Checker semantics → the matching `specs/linter-*.md` file.
- Tool semantics → `specs/compile.md`, `verify.md`, `rename.md`,
  `merge.md`, `graph.md`, `refactor.md`, `orchestrate.md`,
  `model_check.md`.
- Repo workflow (file naming, revision discipline) → `specs/AGENTS.md`.

## Conventions

- **Genesis baseline**: shared CLI/AIX infrastructure comes from the
  `genesis-vibes` crate — envelope output (`Output`/`Envelope`), CLI
  helpers (completions, `--version --json`), verbosity/format flags.
  Domain logic stays here; only cross-cutting pieces go upstream.
- **Binary name is `ddl`**, crate name is `specodelic`.
- **Output discipline**: every command emits through
  `genesis::guide::Output::emit` — JSON envelope by default for pipes,
  human-readable for TTYs. Errors must carry a remediation hint.
- **Rename provenance**: the corpus was renamed from `spec-format` to
  `specodelic` on import (2026-09-28). `specs/USAGE.md`'s title records
  it. Don't reintroduce `spec-format` references.
- **Issue tracking**: `bd` (beads) in no-db mode — `.beads/issues.jsonl`
  is the source of truth, tracked in git. No Dolt database.
- **Quality gates**: `just ci` (fmt-check, clippy `-D warnings`, tests,
  release build) must pass before pushing.
- **Dogfooding**: `just lint-specs` lints the corpus with the tool
  itself. Known coverage gaps are tracked as beads issues, not ignored.

## File naming (from the format — non-negotiable)

A spec file's frontmatter `id` equals its filename stem with `-` ⇔ `.`:
`linter-graph_shape.md` ⇔ `id: linter.graph_shape`. `_` is literal in
both. Non-spec files (no frontmatter) are exempt: `AGENTS.md`,
`STATUS.md`, `USAGE.md`, `CHANGELOG.md`, `theory.md` (in `specs/`), and
this file.