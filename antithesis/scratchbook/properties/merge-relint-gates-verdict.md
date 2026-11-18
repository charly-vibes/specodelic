# merge-relint-gates-verdict

## Invariant

If the union tree fails to parse, fails `lint::lint_corpus`, or has a dangling
structured reference, the report contains at least one `relint_failure` finding
and `verdict == "failed"`, regardless of any `blast_radius_intersection` or
`rename_replay` findings that would otherwise yield `needs_review`.

## Why it matters

`merge.post_merge_relint_required` is the last line of defense: even if the
collision and blast-radius heuristics miss a semantic break, the merged corpus
must lint clean before the tool reports success. A "clean" verdict on a tree
that would be rejected by the crate's own linter is the false-negative failure
mode the merge spec exists to prevent.

## Check mode

`assert` over generated triples with at least one lint-failing or dangling
merged tree.

## Public API / seam

`merge::relint_findings` (`src/merge.rs:382-420`) and the verdict precedence at
`:170-182`; `lint::lint_corpus` (`src/lint/mod.rs:284`) and `graph::build`
(`src/graph.rs:994`).

## Generator sketch

- Base and branches that produce a textually clean union tree.
- Inject one of: a dangling `[[ghost]]`; an EARS/schema/shape lint violation; a
  file that fails to parse after the union; a missing required section.
- Ensure at least one non-failed finding (blast radius or rename replay) is also
  present, so the property tests precedence, not just failure detection.
- Assert `failed` and a `relint_failure` naming the cause.

## Predicted at HEAD `c76e2b0c`

**GREEN.** `relint_findings` pushes one finding per parse error, lint issue, or
dangling reference; `"relint_failure"` is in the failed set, so it dominates
`needs_review`.

Note: the union tree is internal. The property can only observe the finding
messages, not the tree. A lint failure caused solely by the kept side of an
add/add conflict is the `merge-verdict-symmetry` case.

## What would make it red

- `lint_corpus` being skipped when `merged.is_empty()` (the code intentionally
  skips linting an empty tree; an empty union tree with no findings would then
  report `merged`). Is an empty merged tree a valid clean merge? If the base and
  both branches are empty, yes; if a branch deleted everything and the other was
  unchanged, the deletion-wins path leaves an empty tree, which may be intended.
- A lint issue kind that `relint_findings` fails to surface because parsing
  short-circuits (`details` non-empty skips lint entirely). A parse failure
  still yields a `relint_failure`, so the verdict is failed; the lint findings
  are simply not also reported.
- A verdict-precedence regression that treats `relint_failure` as
  `needs_review`.
- `graph::build` not seeing a dangling reference because the link lives in an
  unparsed file; the parse error still fails the verdict.

## Existing instrumentation

Partial: the clean-merge test asserts the empty-findings side; no test forces a
relint failure on an otherwise clean merge.

## Assumptions and Open Questions

- [ASSUMPTION] An empty union tree is not a lint failure; the guard
  `!merged.is_empty()` is intentional.
- Open: should relint findings include *all* lint issues or only those the merge
  could have introduced? The current code reports every issue in the merged
  tree, including pre-existing ones. That is conservative (fails more) but may
  blame the merge for an inherited defect. (needs human input)

## Investigation Log

- Examined: `relint_findings` (`src/merge.rs:382-420`), verdict precedence
  (`:170-182`), `lint::lint_corpus`, and `graph::build`.
- Found: parse failures, lint issues, and dangling references all become
  `relint_failure` findings, which are in the `failed` set. The lint pass is
  skipped only for an empty merged tree.
- Not done: no test forces a relint failure on an otherwise clean merge; the
  union tree is not exposed for inspection.