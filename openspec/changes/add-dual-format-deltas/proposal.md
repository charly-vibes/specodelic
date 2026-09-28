# Change: Dual-format deltas — author openspec requirements as specodelic files

## Why

Two spec systems currently describe overlapping reality in two grammars:
openspec (engineering change workflow) and specodelic (domain spec
format). Every openspec change must hand-author requirement text that is
a mechanical restatement of specodelic rows (see `add-compile-functor`'s
delta, self-labeled "mirrors specs/compile.md"), and `openspec archive`
regenerates `openspec/specs/<cap>/spec.md` from its own parse — dropping
the specodelic layer and creating a second formalized source of truth
that `spk lint` cannot see. A spike
(`openspec/changes/archive/2026-09-28-spike-dual-format/`) proved one
file can satisfy both parsers strictly; this change makes that the
repo's protocol.

## What Changes

- **Policy**: every change delta file is a *dual-format file* — YAML
  frontmatter + `## Constraints`/`## Model`/`## Properties` tables
  (specodelic) alongside `## ADDED Requirements` + `## Purpose` +
  `## Requirements` (openspec). Authored once, validated by both parsers,
  linted by `spk lint` throughout the lifecycle.
- **Naming law acceptance**: openspec hard-requires the delta filename
  `spec.md`, so dual-format files declare `id: spec`. Legal — `unique_id`
  is per-file (verified with two coexisting `id: spec` files). Deltas stay
  self-contained: wiki-refs resolve only within the file; domain
  semantics are referenced by prose path (`specs/compile.md`), never
  `[[wiki-link]]`.
- **Archive protocol**: `openspec archive <id> --skip-specs` (the
  archiver's regeneration destroys frontmatter and tables — verified),
  then copy the dual-format file verbatim from
  `changes/archive/<date-id>/specs/<cap>/spec.md` to
  `openspec/specs/<cap>/spec.md`. Engineering truth remains
  specodelic-lintable after archive.
- **Modeled change lifecycle**: each delta's `## Model` section encodes
  the change's own lifecycle (`proposed → approved → implemented →
  archived`) with transitions guarded by the file's own constraints —
  the orchestrator pattern of `specs/orchestrate.md` applied to the
  engineering workflow.
- **CI gates**: `just ci` gains `openspec validate --strict` over every
  active change and `spk lint` over all dual-format delta files; a
  section-sync check (ADDED vs Requirements text identical) runs in CI —
  the one known wart of the pattern, checked rather than hand-policed.
- **Pilot migration**: the `add-compile-functor` delta converts to dual
  format as the adoption test.

## Impact

- Affected specs: new OpenSpec capability `spec-integration` (this
  change's own delta, written in dual format). Domain corpus
  (`specs/*.md`) untouched — the spec-of-record separation holds:
  openspec owns engineering-workflow truth, specodelic format is the
  *encoding* of that truth.
- Affected code: `justfile` (new recipes + `ci` wiring); no `src/`
  changes (the pattern relies on verified parser behavior, not new lint
  rules — a lint-rule for dual-format files is a possible follow-up,
  see design.md open questions).
- Affected process: `openspec/project.md` "keep both; don't conflate"
  wording is replaced by the dual-format protocol.
- Dependency: `specodelic-6pi` (non-recursive directory lint silently
  lints 0 files) should be fixed or the CI recipe must lint deltas
  file-by-file.