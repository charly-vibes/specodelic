# merge-verdict-symmetry

## Invariant

For every `(base, a, b)`, `merge::run(base, a, b)` and `merge::run(base, b, a)`
return:

- the same `verdict`;
- the same multiset of finding `kind`s, after normalizing the branch label
  (`branch A` / `branch B`) in messages;
- the same set of ids named in findings, when the messages are parsed.

The merge operates on two branch tips; swapping them must not change the
semantic call.

## Why it matters

A merge check whose answer depends on argument order is not a check. Callers
invoke it as `merge(base, current, incoming)` or the reverse depending on which
branch is checked out; a verdict that flips means the tool can report "clean"
for one direction and "failed" for the other on the same history. That is
exactly the false-clean class `merge.md` exists to prevent.

## Check mode

`assert` over generated triples; compare verdict and sorted finding kinds.

## Public API / seam

`merge::run` (`src/merge.rs:140-186`), `union_tree` (`:310-380`),
`relint_findings` (`:382-420`), verdict precedence (`:170-182`).

## Generator sketch

Three trees of `(rel path, text)`:

1. `base` with 0–3 files defining ids; some files absent.
2. `a` / `b` derived by: unchanged copy, edit a row, add a file with a fresh id,
   delete a file, rename an id (delete old definition + add new), and **add the
   same path on both branches with different text and different ids**.
3. Generate link placement so relint can differ: e.g. one add/add text is
   self-contained, the other carries a dangling `[[ghost]]`.

Call both orders; normalize messages; assert verdict + kind multiset.

## Predicted at HEAD `c76e2b0c`

**RED** (source-derived). Counterexample:

- `base = {}` (empty ancestor).
- `a = {"x.md": spec id `x`, self-contained}`.
- `b = {"x.md": spec id `y`, containing a dangling `[[ghost]]`}`.

`merge::run(base, a, b)`:
- collision: ids `x` and `y` are disjoint → none;
- textual conflict: `textual_conflict_findings` requires a base entry for the
  path (`src/merge.rs:223-247`), so add/add is skipped;
- `union_tree`: copies `a`'s `x.md`; for `b`'s `x.md`, the path is already in
  `a`, `a_untouched` compares `base.texts.get(rel)` (None) to `a_text` → false,
  so `a`'s text is kept (`:310-340`);
- relint: parses `x.md` → `x` self-contained, clean; verdict `merged`.

`merge::run(base, b, a)` keeps `b`'s text, relint sees `[[ghost]]` dangling →
`relint_failure` → verdict `failed`.

Same history, opposite verdict. The asymmetry is in `union_tree` plus the
absence of an add/add finding kind.

## What would make it red

Already predicted red. It becomes green if add/add of differing text produces a
`textual_conflict` (matching git) before `union_tree` chooses a side, or if the
union tree refuses to pick and returns a deterministic error.

## Existing instrumentation

Missing. `tests/merge_flow.rs` covers collision, rename replay, both
modify/delete directions, and a clean merge — not add/add, not symmetry.

## Investigation Log

- Examined: `merge::run` verdict precedence, `textual_conflict_findings`,
  `union_tree`'s three loops, `relint_findings`.
- Found: add/add with no base entry is not flagged; A wins by construction;
  relint then depends on which text won.
- Not done: no execution. The counterexample is a code-path construction; a
  one-off `merge::run` call with the two in-memory trees would confirm it.

## Assumptions and Open Questions

- [ASSUMPTION] Add/add same-path histories are realistic (two branches create
  the same new spec file with different content). Git marks this as a conflict,
  so the tool should not silently choose.
- Open: should the merge report expose the union tree so symmetry can be checked
  on content, not only on findings? (needs human input)