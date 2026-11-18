# rename-writer-parity

**Status: INVALIDATED at HEAD (`c76e2b0c`) — vacuous self-comparison.**

## Invariant (as originally stated)

For every corpus and rename, the write-set produced by the hand-wired driver
`rename::run` is byte-identical to the write-set produced by the span-preserving
driver `rename::run_via_writer`, per file, after canonical path sorting.

## Why it matters

During the acset-writer migration, `rename.rs` had two realizations: the
hand-wired line/table rewriting and the writer-driven span splicing. The parity
property was the oracle that let the writer take over without changing a byte:
any intentional difference would be an explicit reviewed snapshot update, never
a silent drift (`tests/rename_writer_parity.rs:1-19`).

## Check mode

`assert` — but there is no discriminating comparison left.

## Public API / seam

`src/rename.rs:65-73`:

```rust
pub fn run(files, old_id, new_id) -> Result<RenameOutcome, RenameError> {
    run_via_writer(files, old_id, new_id)
}
```

`tests/rename_writer_parity.rs:202-267` imports `run` and `run_via_writer`,
constructs the same generated corpus twice, and calls both. At HEAD both names
resolve to the same implementation, so the proptest compares a function with
itself. It cannot fail unless the function is non-deterministic, which it is
not.

## Generator sketch (historical)

- Two files: a definition file with all editable families and a referring file
  whose constraint links to the definition's row.
- Row rename and Intent rename legs, collision-free, with padded cells,
  backticked ids, padded links, and optional CRLF.
- Compare canonicalized outcomes: writes sorted by path, then byte equality.

## Predicted at HEAD

**VACUOUS GREEN.** The test passes by construction. It remains useful only as a
compile-time guard: if a future change reintroduces a separate `run`
implementation, the test starts measuring parity again.

## What would make it red (i.e. regain meaning)

- Reintroducing a second driver behind `run` (the original hand-wired
  implementation, or a new fast path).
- Replacing the self-comparison with a comparison against the checked-in byte
  snapshot (`tests/snapshots/rename_baseline/`), which still has independent
  content. The snapshot test `tests/acset_writer_parity.rs` is the surviving
  oracle.
- Making `run` non-deterministic (would also violate
  `compile-artifact-determinism`-style purity expectations).

## Existing instrumentation

The proptest exists, has a regression seed file
(`tests/rename_writer_parity.proptest-regressions`), and would pass vacuously.
No run was executed in this pass. The snapshot oracle
(`tests/acset_writer_parity.rs`) is the real byte reference.

## Investigation Log

- Examined: `src/rename.rs:65-95`; `tests/rename_writer_parity.rs` imports and
  `assert_parity`; the snapshot store under `tests/snapshots/rename_baseline/`.
- Found: `run` delegates wholesale; the `rewrite_text` fallback is used *inside*
  `run_via_writer` for unparsable files, not as an alternative top-level path.
- Conclusion: invalidate the parity property, keep the snapshot parity, and do
  not spend Hegel adoption budget re-running a self-comparison.

## Assumptions and Open Questions

- [ASSUMPTION] The migration's design decision D6 intended the hand-wired path
  to be removed after parity, so the self-comparison is expected residue, not a
  bug.
- Open: should the now-vacuous proptest be deleted or repurposed to compare
  against snapshots on generated corpora? Deleting it loses the
  regression-seed history; repurposing it creates a second snapshot mechanism.
  (needs human input)