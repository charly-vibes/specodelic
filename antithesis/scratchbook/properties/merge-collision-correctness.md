# merge-collision-correctness

## Invariant

For every `(base, a, b)` and every id defined on both tips:

- if the base did not define the id, the report contains `id_collision`;
- if the base defined it and both branch texts differ from the base text, the
  report contains `id_collision` (including when the two edits agree with each
  other);
- if the base defined it and at most one branch changed its text, there is **no**
  `id_collision` finding for it.

"Text" is the raw file text containing the definition, as stored in `Tip.ids`
(keyed by qualified id).

## Why it matters

`merge.no_new_id_collision` is the primary hazard a textual 3-way merge cannot
see: two branches mint the same id independently and git merges both cleanly.
A false negative silently produces a corpus with duplicate ids; a false
positive blocks every normal merge that touches an inherited id on one side.

## Check mode

`assert`, per generated id.

## Public API / seam

`merge::collision_findings` (`src/merge.rs:191-221`), fed by `Tip.ids` built in
`load_tip` (`:54-91`). `Tip.ids` maps `intent.id` or `intent.row` to
`(rel path, raw text)`.

## Generator sketch

- Base with 1–3 ids across 1–2 files.
- For each id, independently choose branch behavior:
  - unchanged in both;
  - edited in A only;
  - edited in B only;
  - edited in both to the same text;
  - edited in both differently;
  - deleted in A, edited in B (modify/delete; collision should not fire but a
    textual conflict should).
- Add fresh ids independently on each branch (some shared, some not).
- Assert the collision presence/absence partition above.

## Predicted at HEAD `c76e2b0c`

**GREEN.** The code computes `newly_minted = base_text.is_none()` and
`both_edited = a_text != base && b_text != base` and flags exactly their
disjunction. The "edited identically on both branches" case is flagged, which
matches the constraint text (the id was not *identically defined in the
ancestor*).

## What would make it red

- Comparing parsed definitions instead of raw file text: a change elsewhere in
  the same file would count as an edit of every id in it. The current
  file-text comparison is deliberately coarser; the property must state which
  semantics it checks.
- Missing row ids from `Tip.ids` (they are qualified with the intent id).
- A collision on a *deleted* id (deletion is not a definition; the code
  correctly has no entry).
- Base parsing failure: a broken base file yields no ids, so everything on both
  tips looks newly minted and is flagged. The property generator should keep
  the base parseable.

## Existing instrumentation

Partial. `id_collision_across_branches_is_flagged` and the clean-merge test in
`tests/merge_flow.rs:104-145` cover two fixed cases. No generated partition.

## Assumptions and Open Questions

- [ASSUMPTION] File-level text is the intended granularity for "changed
  definition". The code's comment says "content differs, or the file is new"
  (`src/merge.rs:90-92`).
- Open: should two branches making the *same* edit be a collision or a
  no-op merge? The code flags it; the constraint text supports that, but a user
  may expect the identical result to merge cleanly. (needs human input)

## Investigation Log

- Examined: `collision_findings` (`src/merge.rs:191-221`), `Tip.ids`
  construction (`:54-91`), and the two fixed merge-flow tests.
- Found: the predicate is exactly `newly_minted || both_edited`, where
  `both_edited` compares raw file text to base. The constraint text's
  "identically in the common ancestor" supports flagging identical dual edits.
- Not done: no generated partition; the user-facing expectation is a product
  question, not a code question.