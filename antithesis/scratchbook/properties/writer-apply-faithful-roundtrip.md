# writer-apply-faithful-roundtrip

## Invariant

If `writer::apply(Edit::Rename { old, new, local }, path, x, &parse(x))` returns
`Ok(WriteSet)`, then reparsing the emitted text yields the target instance:

- the Intent id is `new` when the edit is an Intent rename;
- no state bullet, transition endpoint, or table row cell still carries the old
  local id;
- the new local id appears at exactly the places the old one stood;
- the multiset of link targets equals the expected rewrite (followed links get
  the new target, unfollowed links keep theirs).

## Why it matters

`acset.writer.edit_application_faithful` is the writer's central promise: the
emitted text denotes the edited instance. It is the check that lets `rename`
delegate text realization to the writer instead of hand-wiring every family.

## Check mode

`assert`, with the explicit caveat that the predicate is **gate-enforced**:
`apply` runs `verify_roundtrip` before returning `Ok`. The property is valuable
as an observable restatement and as a harness for future changes to the oracle,
but a green run is weaker evidence than a green run of an independent oracle.

## Public API / seam

`writer::apply` (`src/acset/writer.rs:234-333`), `verify_roundtrip`
(`:506-631`). Replacement families: `wiki_link_repls` (`:345-376`),
`local_id_repls` (`:378-419`), intent-id span at `:269-281`.

## Generator sketch

- Full-family spec with 1–3 rows, states, transitions, and links.
- Row rename: fresh local id not equal to any keyword or existing id; optional
  padded/backticked cells and CRLF.
- Intent rename: fresh dotted id; file move expected.
- Apply, reparse; assert the four clauses. Compare against a **model** rename
  built from the parsed `before` Spec, not against `verify_roundtrip`'s
  internal counters.

## Predicted at HEAD `c76e2b0c`

**GREEN — gate**, for the id/link clauses. Existing proptest
`roundtrip_faithful` (`tests/acset_writer_apply.rs:409-492`) targets this
distribution; no run was executed in this pass.

The property is deliberately blind to the `kind`-column corruption: the current
oracle counts rows, not columns. See `rename-kind-preserved`, which is the
stronger property and is predicted red.

## What would make it red

- A replacement that reparses to a different token (a backticked or suffixed
  bullet head). The current oracle catches this as an unrealized count and
  returns `roundtrip_failure`, so the property would see `Err`, not a corrupted
  `Ok`.
- A link rewrite that changes the target multiset (e.g. double-rewriting a link
  whose new target itself matches the old-prefix pattern). The multiset
  comparison at `:601-627` is designed for this and handles `alpha` →
  `alpha.prime`.
- A row-local rewrite that misses an unspanned cell. `collect_cell_repls`
  returns `apply_failure` if a matching cell lacks a span, so the property sees
  `Err`; totality is a separate question.
- A future oracle change that checks only stale ids and drops the "new id
  appears where the old stood" leg, allowing a silently deleted id cell.

## Existing instrumentation

The handwritten proptest above, plus fixed tests for each family, padding,
prose, CRLF, and file moves. The distribution's local ids are `r{pick}` →
`renamed{pick}`, so keyword-colliding ids are untested.

## Assumptions and Open Questions

- [ASSUMPTION] Reparse equality is the right oracle; byte equality is covered by
  `writer-untouched-bytes-preserved`.
- Open: should `verify_roundtrip` compare columns instead of row counts? That
  would make this property catch `rename-kind-preserved`.
  (needs human input)

## Investigation Log

- Examined: `verify_roundtrip` (`src/acset/writer.rs:506-631`), especially the
  `cells` / `cells_new` row counters (`:576-591`), and
  `tests/acset_writer_apply.rs:409-492`.
- Found: the oracle checks stale/unrealized counts on states, transitions, and
  rows-with-a-matching-cell; it does not identify which column matched, so a
  `kind` cell can absorb the count. The existing generator never draws
  keyword ids.
- Not done: no execution; the blind spot is a code-path reading.