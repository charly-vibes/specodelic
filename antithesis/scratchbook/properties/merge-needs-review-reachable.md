# merge-needs-review-reachable

## Invariant

At least one generated `(base, a, b)` triple produces `verdict == "needs_review"`
with a `blast_radius_intersection` or `rename_replay` finding (and no
failed-class finding, so the verdict is actually `needs_review`).

## Why it matters

A merge property suite that only ever generates `merged` or `failed` cases
never exercises the middle verdict. `needs_review` is the tool's distinctive
output — the "textually clean but semantically entangled" case — and the
`rename_replay` finding is a flag-only path that does not itself rewrite
anything. Without this generator guard, the blast-radius and replay properties
can pass without ever reaching their target state.

## Check mode

`sometimes` — generator non-vacuity, not product liveness.

## Public API / seam

`merge::run` (`src/merge.rs:140`), `blast_radius_findings` (`:249-267`),
`rename_replay_findings` (`:269-308`), verdict precedence (`:170-182`).

## Generator sketch

Two independent legs:

1. **Blast radius:** base defines `a.x` and `b.r` with `b.r` tracing to `a.x`;
   A edits/renames `a.x`; B edits `b.r` in a file A did not touch. Touched rows
   are in disjoint files but their closures intersect at `a.x`.
2. **Rename replay:** A removes `a.x` (rename to `a.y`); B's file still links to
   `a.x`. A touched `a.x`, B still references it → `rename_replay` finding.

Avoid same-file edits (which would produce `textual_conflict`/failed) and avoid
duplicate ids (which would produce `id_collision`/failed).

## Predicted at HEAD `c76e2b0c`

**GREEN** with the generators above. The canonical scenarios are exercised by
fixed tests (`tests/merge_flow.rs:146-219`) and the closure parity test, so the
state is reachable.

**RED** if the generator only copies files or makes disjoint edits with no
cross-references: then every verdict is `merged`, and the `sometimes` leg
exposes the vacuity.

## What would make it red

- The blast-radius leg uses same-file edits, so `textual_conflict` forces
  `failed` and `needs_review` never appears.
- The rename-replay leg renames an id that B does not reference.
- A code change that always escalates `needs_review` to `failed` (then the
  verdict assertion fails) or always demotes it to `merged` (a real regression).
- A code change that reports `rename_replay` but sets `merged` because the
  verdict precedence no longer treats non-empty findings as `needs_review`.

## Existing instrumentation

Missing as an explicit guard. The fixed tests prove two scenarios; no generated
check ensures the distribution reaches the middle verdict.

## Assumptions and Open Questions

- [ASSUMPTION] `failed` findings dominate `needs_review` by design; the
  generator must avoid them to assert the middle verdict.
- None that change the invariant.