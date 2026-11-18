# merge-blast-radius-intersection

## Invariant

Let `touched_A` be the ids defined in files A changed relative to base, plus ids
the base defined and A deleted (symmetrically for B). Let `radius_X` be the
mixed transitive closure over X's own instance from `touched_X`. If
`radius_A ∩ radius_B ≠ ∅`, the report contains a `blast_radius_intersection`
finding and the verdict is at least `needs_review` (unless a `failed`-class
finding also exists). If the radii are disjoint, no such finding appears.

## Why it matters

This is `merge.semantic_conflict_iff_blast_radius_intersects`: two branches can
touch textually disjoint files and still interfere through the reference graph.
The canonical case is a rename on A and a new reference on B — the reference's
own forward reach lands on the renamed id, so the radii intersect even though no
file overlaps. Without this property, a semantically broken merge reports
clean.

## Check mode

`assert` over generated triples; the interesting state is `needs_review`.

## Public API / seam

`touched_ids` (`src/merge.rs:93-122`), `blast_radius` (`:125-136`),
`acset::query::blast_radius` (`src/acset/query.rs:184-203`). The closure is the
fixpoint of backward ∘ forward, seeded with the filtered touched ids.

## Generator sketch

- Base: two namespaces, `a.*` and `b.*`, with a reference from a `b` row to an
  `a` row.
- A: rename `a.x` → `a.y` (delete old definition, add new) or edit `a.x`'s row.
- B: add a new reference to `a.x` from a `b` row in a file A did not touch.
- Also generate: disjoint edits with no cross-reference (expect no finding);
  edits touching the same file (expect `textual_conflict`/failed, which
  dominates).
- Assert the finding's presence and the named intersection id.

## Predicted at HEAD `c76e2b0c`

**GREEN** for the canonical scenario. `touched_ids` includes the deleted `a.x`
and the new `a.y` on A. On B, the new reference's source row is touched; the
mixed closure follows the row's `traces_to` edge forward to `a.x`, so
`a.x` joins `radius_B`. A's radius seeds `a.x` directly. Intersection non-empty.

The parity test in `merge.rs:420-494` pins the closure's definition against the
pre-migration walk, including the "dependent's forward reach joins the radius"
case.

## What would make it red

- `touched_ids` missing deletions (then A's radius would not include `a.x`).
- The closure following only backward edges (then B's radius would not include
  `a.x`; this is the exact bug the mixed-closure comment warns about).
- A graph instance that drops dangling references, so a reference to a
  now-deleted id contributes no edge on B — but B's instance still defines the
  reference target? In the canonical scenario `a.x` is defined on B (B starts
  from base and only adds a reference), so the edge exists.
- The `needs_review` verdict being overridden by a `failed` finding. The
  property should either use a triple with no failed-class finding, or accept
  `failed` as "at least needs_review".

## Existing instrumentation

Partial: `textually_clean_semantic_conflict_is_flagged`
(`tests/merge_flow.rs:180-219`) and `dangling_rename_across_branches_is_flagged`
(`:146-179`) cover two fixed rename/reference scenarios. No generated graph.

## Assumptions and Open Questions

- [ASSUMPTION] "Touched" is file-level, not row-level. The code documents this;
  the property should use it as the definition, not as a bug.
- Open: is a blast-radius intersection that also has a textual conflict supposed
  to report both findings, or just the dominating one? The code reports both;
  the verdict is `failed`. (resolved by inspection: findings are additive)