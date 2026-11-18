# rename-success-reachable

## Invariant

At least one generated `(corpus, old_id, new_id)` case returns
`Ok(RenameOutcome)` with `writes` non-empty, for each of the two shapes: an
Intent rename and a row rename.

## Why it matters

This is a **generator-adequacy** property pair, not a product liveness claim. A
suite of rename properties that only ever generates `UnknownId`,
`InvalidNewId`, or `Collision` refusals is green for the wrong reason. The
`sometimes` leg makes that failure visible: if no accepted case exists, the
generator is not exercising the writer at all.

## Check mode

`sometimes` — one generated case must satisfy the predicate. This is the
appropriate use of a non-universal assertion: it guards non-vacuity without
claiming a product liveness guarantee.

## Public API / seam

`rename::run` (`src/rename.rs:65`). The distinguishing signals are
`Ok(outcome)` with `!writes.is_empty()` and, for Intent renames,
`remove.is_some()`.

## Generator sketch

- Start from a fixture with a full editable family (Constraints + States +
  Transitions + Properties) and at least one link in another file.
- Intent leg: rename the Intent id to a fresh dotted id, e.g. `demo` →
  `demo2`; expect a moved file, one removal, and at least one link rewrite.
- Row leg: rename `demo.c1` → `demo.c1x`; expect the owner file's id cell and
  every link to change.
- Ensure the corpus is link-closed and the target is fresh; otherwise the case
  legitimately refuses and cannot count toward `sometimes`.

## Predicted at HEAD `c76e2b0c`

**GREEN** with a competent corpus generator. The existing fixtures already
produce accepted renames: `tests/acset_writer_parity.rs` and
`tests/rename_writer_parity.rs` build full-family specs and rename them.

**RED** with a generator that draws ids from the wrong namespace (row rename
without the `intent.` prefix → `InvalidNewId`), or that starts from unparseable
text (→ `UnknownId`).

## What would make it red

- The generator never satisfies the row-namespace precondition
  (`new_id.starts_with("intent.")`).
- The corpus generator produces only empty/prose files the parser rejects.
- A regression that refuses all renames (e.g. verify gate rejecting every
  emitted corpus). That would also turn the law properties red.

## Existing instrumentation

Missing as an explicit non-vacuity guard. The fixed fixtures prove acceptance
for their inputs, but nothing asserts that a *generated* distribution reaches
the success path.

## Assumptions and Open Questions

- [ASSUMPTION] "Success" means `Ok` with a non-empty write-set, not merely
  `Ok`. The identity early return is `Ok` with no writes and must not satisfy
  this property.
- None that change the invariant.