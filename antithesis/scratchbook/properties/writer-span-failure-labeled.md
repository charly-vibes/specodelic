# writer-span-failure-labeled

## Invariant

When the source no longer matches the parse — a cell's bytes changed after
parsing, the source was truncated, or a recorded span falls outside the source —
`emit` and `apply` return `Err(WriterError { label: "spec.span_failure", detail,
remediation })` with a non-empty detail naming the element and a non-empty
remediation. They never panic and never return partially edited text.

## Why it matters

Stale parses are the realistic failure mode at the rename/verify boundary: a
caller parses a corpus, another process changes a file, and the edit is applied
to the old `Spec`. The contract must be a labeled refusal, not a splice at the
wrong offset. This is also the fleet error contract's "labeled cause with
remediation" requirement.

## Check mode

`assert` over mutated sources; an additional adversarial leg asserts no panic
for arbitrary byte mutations (catch-unwind or a process-isolated runner).

## Public API / seam

`writer::verify_spans` (`src/acset/writer.rs:64-174`), `span_failure`
(`:178-188`), called by `emit` (`:56`) and `apply` (`:234`).

## Generator sketch

- Parse a valid full-family source.
- Mutation families: replace a cell value with same-length garbage; insert or
  delete bytes before a span; truncate at random byte boundaries (respecting
  UTF-8 or using a byte-level API that returns `None`); take a suffix slice.
- Call `emit(mutated, &spec)` and `apply(edit, path, mutated, &spec)`.
- Assert `Err` with the exact label and non-empty remediation; assert the
  message names the affected id or column where the mutation is targeted.

## Predicted at HEAD `c76e2b0c`

**GREEN.** `verify_spans` uses `str::get(start..end)` for bounds and UTF-8
safety, compares covered bytes to the parsed value, and returns the labeled
error on the first disagreement. `splice_replacements` runs only after a full
`verify_spans` pass, so no partial splice is reachable from a coherent error
path.

## What would make it red

- A panic path: an indexing operation on a stale span before verification
  (currently `verify_spans` is called first in both `emit` and `apply`).
- Returning `Ok` after a mismatch because a `None` span is treated as "nothing
  to verify". That is correct for legitimately unrecorded spans (quoted ids,
  unrecognized bullets), but a mutation that turns a recorded span into `None`
  by reparsing differently is a different API misuse.
- `apply`'s **identity short-circuit**: `apply(id, x_stale)` returns `Ok` with
  no writes before `verify_spans`. This is not corruption (nothing is spliced)
  but it violates a naive "apply always verifies spans" reading. The property
  should exclude the no-op edit or state this as a separate case.
- A UTF-8 boundary slice that panics because a future change replaces `get`
  with direct indexing.

## Existing instrumentation

Fixed tests `span_failure_label_asserted_on_disagreement` and
`..._on_out_of_bounds` (`tests/acset_writer_identity.rs:225-254`), plus the
apply-failure label tests for unspanned intent ids/bullets
(`tests/acset_writer_apply.rs:291-338`). No generated mutation sweep.

## Assumptions and Open Questions

- [ASSUMPTION] Mutation sweeps use the same `Spec` parsed from the pre-mutation
  text; reparsing the mutated text would produce a different `Spec` and is a
  different scenario.
- Open: should `apply` verify spans on the identity path? Current behavior is
  harmless but asymmetric with `emit`. (needs human input if the API promises
  verification on every call)

## Investigation Log

- Examined: `verify_spans` (`src/acset/writer.rs:64-174`), `splice_replacements`
  (`:420-433`), `emit`/`apply` ordering, and the existing mutation tests.
- Found: `verify_spans` runs before any splice in `emit` and non-identity
  `apply`; the identity branch returns earlier. `str::get` makes out-of-bounds
  and non-boundary slices return `None`, not panic.
- Not done: no generated mutation sweep.