---
tags: [pipeline-run:epic-orchestrator-2026-11-13-specodelic-lf3-migrate-specodelic-kinds-properties-run-4, pipeline-step:spawn-subagent]
---

# RO5U review — specodelic-lf3 run 4 (specodelic + kinds property migration)

Scope reviewed: `openspec/specs/specodelic/spec.md`, `openspec/specs/kinds/spec.md`,
`src/lib.rs` (new `fixtures::specs` module), `tests/specodelic_fixtures.rs`,
the regenerated `specodelic/*_props.rs` and `specodelic/*.check.json`.

## High

None. Every migrated row's fragment calls a real `specodelic::*` API and
asserts that row's own claim; the two law rows carry Revision 21 per-case
bodies and the compiler emitted them as separate blocks (31 + 15 passing
blocks over the two files).

## Medium

- **Four rows have no honest seam and stay on `todo_predicate!`** —
  `openspec/specs/specodelic/spec.md` (`silently_shrunk_id_set_rejected`) and
  `openspec/specs/kinds/spec.md` (`constraint_row_extra_field_rejected`,
  `state_row_with_extra_field_rejected`,
  `subkind_addition_without_revision_rejected`).
  *Evidence*: the current lint path parses an extra Constraint column, an extra
  State field and a sixth section without a finding, and no API reads a diff
  between revisions; the earlier probe run confirmed all four shapes pass.
  *Fix*: leave the predicates unchanged (no fragment — the honest placeholder),
  and name the four rows in `tests/specodelic_fixtures.rs`
  (`the_unmigrated_rows_are_the_documented_four`) so the set is on the record.
  **Done in this change.**

## Low

- **`unknown_kind_rejected` proves the closed set through the schema checker**
  (`openspec/specs/kinds/spec.md`, row 1) — the fragment widens
  `acset::schema::canonical()` with an `Actions` object and asserts
  `schema::check` rejects it, rather than asserting the row-level linter rule.
  *Evidence*: the fixture's sixth section is a section-level shape, and the
  kind-level rule fires on a property row's kind cell.
  *Fix*: none required — the claim ("a kind outside the closed set is
  rejected") is decided by the tool's own closed-set checker, and the fragment
  also asserts the canonical schema still checks clean, so it cannot pass by
  accident.

- **The corpus-wide `scope_sha256` moved, so all 22 `specodelic/*.check.json`
  reports changed** (one line each) — expected: the scope digest covers the
  whole corpus, so a corpus change invalidates every report. Committing only
  the two migrated reports would have left the other twenty stale and made
  `just verify-specs` vacuous.

- **The fixture registry was compacted to keep `src/lib.rs` under the
  structural ratchet** (2626 → 2286 lines, `pretender.toml` `file_lines_max`
  is 2300) — the registry entries are now a flat `(name, builder)` table and
  the fixtures' internal property-row generator cell was shortened. The
  generated artifacts are byte-identical after the refactor.

