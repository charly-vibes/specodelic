# merge-modify-delete-flagged

## Invariant

For a base file `p`:

- if one branch deletes `p` and the other leaves it byte-identical to base, the
  union tree omits `p` and no conflict finding names it;
- if one branch deletes `p` and the other edits it, the report contains a
  `textual_conflict` finding naming `p`, the verdict is `failed`, and the edited
  text remains in the union tree passed to relint;
- the behavior is symmetric in the branch argument order.

## Why it matters

A modify/delete is a real semantic conflict that git also marks. Silently
keeping the deletion loses the other branch's work; silently keeping the edit
pretends the deletion never happened. The merge tool is a check, not an
applicator, so it must at least refuse a clean verdict and show the edited tree
it linted.

## Check mode

`assert` over generated triples in both argument orders.

## Public API / seam

`merge::union_tree` (`src/merge.rs:310-380`), specifically the symmetric
deletion loop at `:342-377`; verdict precedence at `:170-182`.

## Generator sketch

- Base with one file `p.md` defining id `p`; optionally other files.
- Branch variants: delete `p.md`; leave unchanged; edit a row in `p.md`; edit a
  different file.
- For each delete/edit pairing, run both argument orders.
- Assert the finding/omission partition, and that the merged text used for
  relint is the edited text when a conflict fires. (The union tree is not
  public; infer it from the relint findings or replicate the logic in the
  test's oracle.)

## Predicted at HEAD `c76e2b0c`

**GREEN.** Fixed tests exist for both directions
(`tests/merge_flow.rs:220-266`). The code's second loop computes
`edited` by comparing the surviving branch's text to base, flags
`textual_conflict` when true, and otherwise removes the path.

## What would make it red

- Treating a delete as "B-only addition" in the first loop (it does not; the
  second loop handles base files).
- Comparing the *deleted* branch's text (absent) and defaulting to "not edited",
  which would drop the conflict finding.
- Removing the path unconditionally after flagging, so relint never sees the
  edited version. The comment says the edited version stays; a regression here
  would not change the verdict (the conflict already forces `failed`) but would
  change which relint findings are reported.
- Asymmetry in the `in_a`/`edited_tip` selection. The current selection is
  symmetric; the property checks both orders.

## Existing instrumentation

Fixed subprocess tests for both directions. No generated pair.

## Assumptions and Open Questions

- [ASSUMPTION] "Edited" is file-text inequality with base, the same granularity
  as `collision_findings`.
- Open: should the union tree be exposed so callers can inspect the version
  kept on conflict? (needs human input; overlaps `merge-relint-gates-verdict`)

## Investigation Log

- Examined: `union_tree`'s deletion loop (`src/merge.rs:342-377`) and both
  modify/delete fixed tests (`tests/merge_flow.rs:220-266`).
- Found: the surviving edited text is kept for relint and a `textual_conflict`
  is pushed; the selection of `edited_tip` is symmetric in the two argument
  orders.
- Not done: no generated pair; no direct observation of the union tree (not
  public).