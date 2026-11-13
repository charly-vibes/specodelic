---
reviews: 2026-11-13-green-the-row-shape-rule-linter-row-shape-closed.md
tags: [pipeline-run:epic-orchestrator-2026-11-13-specodelic-3632-lint-revision-seams-for-the-four-blocked-lf3-rows, pipeline-step:spawn-subagent]
---

# RO5U review — specodelic-3632 (lint + revision seams for the four blocked rows)

Scope reviewed: the working-tree diff (src/lint/**, src/spec.rs, src/packs.rs,
src/fixtures/specs.rs, tests/**, openspec/specs/{kinds,specodelic}/spec.md, justfile).
Rule-of-5 stages; no Critical or High findings.

## Findings

### MEDIUM — src/lint/mod.rs: `pub mod row_shape;` widened the crate's public API
- Evidence: the module was declared `pub` although only the revision seam is
  called from outside the crate (the property fragments call
  `specodelic::lint::revision::check_pair`); the row-shape entry point
  `lint_row_shape` is `pub(crate)`.
- Fix: demote to `mod row_shape;` — the crate's public surface stays the
  revision seam only. **Fixed.**

### LOW — src/lint/revision.rs: duplicated "the current revision carries a new
`## Revision N` heading" computation
- Evidence: `check_pair` recomputed the predicate for the Reference Typing
  fields, while `diff_id_sets` computed the same predicate for the other
  governed id-sets.
- Fix: extract `has_new_revision_heading(prior, current)` and use it in both
  places. **Fixed.**

### LOW — src/lint/row_shape.rs: `allowed_fields`' doc comment ended with a
dangling empty doc line. **Fixed.**

## Stage notes

- Stage 1 (shape/scope): the two seams live in their own modules; the rule is
  wired into the schema-shape family, which the corpus's Checker Ownership
  table now assigns the four `kinds.*_row_shape` constraints to.
- Stage 2 (correctness): the per-kind base-field table is checked against the
  Schema-as-data's reference columns in a unit test (a Constraint row may
  carry traces_to/supersedes/satisfies/observes/uses, never guard); the
  revision seam's removal / silent-growth / declared-growth cases each have a
  unit test.
- Stage 3 (clarity): every new file carries a Purpose/Responsibilities/Rationale
  header; the corpus Notes block states which checker now decides the four
  constraints.
- Stage 4 (edge cases): a State bullet with no tail, a bare tail, an
  `emits:` tail and a named tail are each exercised; a growth under a new
  Revision heading is allowed while the same growth without one is a finding;
  the conformance matrix's fixtures were corrected to the corpus's own
  Reference Typing (`observes` is a Constraint field) rather than weakening
  the closed row shape.
- Stage 5 (polish): `cargo fmt` clean, `cargo clippy -- -D warnings` clean,
  the full suite, `just ci`, `just lint-specs` and `ah check --run-tests
  --all-tests` green.

Verdict: pass after the three fixes above.
