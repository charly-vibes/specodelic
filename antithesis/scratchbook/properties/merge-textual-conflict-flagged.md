# merge-textual-conflict-flagged

## Invariant

If a path `p` exists in the base, both tips, and both tip texts differ from the
base text and from each other, the report contains a `textual_conflict` finding
naming `p` and the verdict is `failed`.

## Why it matters

Git marks this case, but the merge tool is also usable on in-memory trees and
must not claim a clean result on a file it cannot merge. The finding preserves
the "textual conflict precedes semantic review" ordering: a human resolves the
text first, then reruns the check.

## Check mode

`assert` over generated triples.

## Public API / seam

`merge::textual_conflict_findings` (`src/merge.rs:223-247`). Conditions:
`tip_a.texts[p] != base`, `tip_b.texts[p] != base`, `a_text != b_text`.

## Generator sketch

- Base with one or more paths.
- A and B each edit the same path differently; vary whether one edit equals the
  other (expect no finding when equal and both differ from base? The code
  requires `a_text != b_text`, so identical edits are not flagged).
- Add a case where only one branch edits (expect no finding).
- Add a case where the path is absent from base (add/add) and assert the
  current **no-finding** behavior is documented as the `merge-verdict-symmetry`
  gap, not as correct.

## Predicted at HEAD `c76e2b0c`

**GREEN** for the stated invariant. The condition is implemented directly.

Caveat: two branches making the *same* edit to a file are not flagged (both
differ from base but `a_text == b_text`). That matches "git would merge them
cleanly", but it can still be a semantic collision if the edit mints an id; the
id-collision property catches that separately.

## What would make it red

- Removing the `a_text != b_text` condition so identical edits are flagged.
- Using parsed-row differences instead of file text, which would miss
  whitespace-only conflicts.
- `textual_conflict` not being in the failed set (verdict regression).
- Path normalization: relative paths must match across tips. The merge API
  takes relative paths; a generator that uses different path spellings for the
  same file would create false negatives, but that is a harness bug.

## Existing instrumentation

Indirect: modify/delete tests produce `textual_conflict` findings; the
same-path both-edited case has no dedicated test.

## Assumptions and Open Questions

- [ASSUMPTION] File-level text comparison is the intended granularity.
- Open: should add/add be folded into this finding kind (git treats it as a
  conflict)? See `merge-verdict-symmetry`. (needs human input)

## Investigation Log

- Examined: `textual_conflict_findings` (`src/merge.rs:223-247`) and the
  modify/delete tests that also produce this finding.
- Found: the predicate requires a base entry for the path; add/add has none and
  is skipped. Identical edits on both branches are also skipped.
- Not done: no dedicated same-path both-edited test; add/add is deferred to the
  symmetry property.