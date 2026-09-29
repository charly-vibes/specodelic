# Change: Add observability contracts — `observes` references, unobserved-effect reporting, derived external boundaries

## Why

The format can *declare* contracts (`extension_point`/`satisfies`, Revision 7)
and *declare* outputs (`emits` → effect Constraints), but has no vocabulary
for **what must be observable**: a spec can model a system with zero declared
observables and lint clean, and nothing distinguishes an edge that crosses a
component boundary from one that stays internal. The result is that systems
specced in the format are not transparent from the start — observability and
boundary structure are reconstructed after the fact instead of being
first-class, checkable facts.

Design was reviewed under a Rule-of-5 pass (converged at Stage 4); three
corrections are folded in: no authored boundary tag (derived instead), the
`observes` field is Constraint-sourced only, and the "zero new extraction"
claim was corrected to extraction-free boundary *queries* only.

## What Changes

- **Format (additive, via sanctioned Revisions):**
  - New Reference Typing row: `observes` — one-directional outbound reference
    from a Constraint row to a Constraint with `kind == effect`, on any file
    (cross-file allowed). Mirrors the `satisfies` precedent (`specodelic.md`
    Revision 7). → `specs/specodelic.md` Revision 8, `specs/kinds.md`
    Revision 5.
  - `observes` edges are **acyclic-exempt**: they do not join the acyclic
    invariant's edge set (a claim, not a dependency — same reasoning that
    keeps `satisfies` out). Stated explicitly, not left implicit. →
    `specs/graph.md` + `specs/linter-graph_shape.md` note.
- **Tool — linter:** new check `linter.observability`: every effect
  Constraint is either the target of ≥1 `observes` edge or is reported as an
  **advisory** finding naming the unobserved row. No waiver machinery in v1;
  never gates a lifecycle transition. Gating (advisory → invariant) is
  deliberately deferred until after dogfooding.
- **Tool — graph:** derived external-boundary classification: a file is an
  external boundary iff it hosts ≥1 `extension_point` Constraint. Pure
  derivation over existing edges — **no authored tagging mechanism** (no
  frontmatter field, no new column), per `graph_is_derived_not_authored`'s
  philosophy.
- **Dogfooding:** one corpus/USAGE worked example carrying real `emits` +
  `observes` rows, so the advisory-vs-gating decision is made against real
  friction, not in the abstract.

Not in scope: a sixth schema kind; verifying actual conformers of
`extension_point` contracts (explicitly rejected by Revision 7); waiver
machinery (deferred; `linter-external_completeness.md`'s covered/waived
pattern is the candidate when the question reopens).

## Impact

- Affected specs (openspec capabilities): **new capability `observability`**
  (delta under `specs/observability/` in this change).
- Affected format corpus: `specs/specodelic.md` (Revision 8), `specs/kinds.md`
  (Revision 5), `specs/graph.md`, new `specs/linter-observability.md`,
  `specs/USAGE.md` (worked example), `specs/STATUS.md` (status table row).
- Affected code: `src/spec.rs` (parse optional `observes` column),
  `src/lint.rs` (`linter.observability`), `src/graph.rs` (`observes` edge in
  `total_extraction`; external-boundary classification), `tests/cli.rs`
  fixtures.
- Backward compatible: nothing existing becomes invalid — the check is
  vacuous until the first effect row appears (no corpus file uses `emits`
  today, per `kinds.md` Revision 3 Notes).
- Note: `kinds.md`'s `constraint_row_shape` states fields as exactly
  `{id, kind, expr, traces_to}` while `satisfies` already adds an optional
  column (USAGE §2.6); Revision 5 must reconcile this — possibly one of the
  known `specodelic-qc8` coverage gaps. Not weakened to land this change.