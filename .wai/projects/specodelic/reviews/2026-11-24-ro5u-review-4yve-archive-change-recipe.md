# RO5U review — 4yve archive-change recipe

Date: 2026-11-24 · Scope: `justfile` (archive-change recipe),
`scripts/archive_change_id.sh`, `tests/archive_change_recipe.rs`

## Stage 1 — DRAFT (shape, scope)

Recipe-only fix, no CLI or spec edits: one normalization script, one recipe
change, one drift-guard test file. Mirrors the repo's existing
`tests/ci_wiring.rs` drift-guard pattern. Scope is right.

## Stage 2 — CORRECTNESS

- Prefix stripped exactly once (`${id#id=}`); a bare id passes through.
- Empty id exits 2 with a remediation hint; `set -euo pipefail` with `${1-}`
  is safe under `set -u`.
- `scripts/archive_change_id.sh` is mode 755 in the index (`git ls-files -s`
  → `100755`); the end-to-end test executes it directly, so a lost exec bit
  fails the suite instead of production.

## Stage 3 — CLARITY

The recipe comment states WHY the prefix is stripped and cites
`specodelic-4yve`; the script header documents usage and the exit code.

## Stage 4 — EDGE CASES

- Change id containing shell metacharacters — pre-existing exposure, unchanged
  by this fix (L1).
- A change id literally starting with `id=` — none exists in
  `openspec/changes/`.
- Empty id — covered by two tests (script level and recipe level).

## Findings

**CRITICAL:** none.
**HIGH:** none.

**MEDIUM**

- **M1** `tests/archive_change_recipe.rs:36-40` — the recipe-body extractor
  matched the first line whose trimmed text merely *starts with*
  `archive-change`; a future sibling recipe (`archive-change-dry-run:`) would
  be picked up and the header assertion would fail confusingly.
  *Fix:* match the exact header `archive-change id:` and fail with an
  explicit message. **→ fixed**
- **M2** `justfile:286` — the script was invoked repo-relatively, which only
  works because `just` runs recipes from the justfile's directory.
  *Fix:* resolve the script through `{{justfile_directory()}}`.
  **→ fixed**

**LOW**

- **L1** `justfile:286-287` — `just` substitutes `{{id}}` unescaped, so an id
  containing quotes/backticks breaks the shell line. Pre-existing in the
  previous recipe (identical quoting), not introduced here.
  *Fix:* deferred — expanding the fix's blast radius for an id shape no
  corpus change id has is not warranted; noted in the quality ledger.
- **L2** `scripts/archive_change_id.sh:14` — the hint named only the bare
  form. *Fix:* mention that the `id=<change-id>` form is accepted too.
  **→ fixed**

## Convergence

New CRITICAL: 0 · Total new: 4 · New vs previous stage: +33% ·
Estimated false positive rate: ~0% (each finding re-read at its cited line) ·
Status: CONTINUE → M1/M2/L2 applied, narrow test re-run → 7/7 green.
