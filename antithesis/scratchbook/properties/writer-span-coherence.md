# writer-span-coherence

## Invariant

For every parser-accepted source `x`, every span the parser records resolves in
bounds to exactly the bytes of the parsed element it denotes:

- frontmatter Intent id span slices the id;
- each table row's `id_span` slices the row id, and each `cell_spans[col]`
  slices the parsed value of that column;
- transition `id_span`/`from_span`/`to_span` and each cell span slice their
  values;
- each `[[…]]` link span slices an occurrence whose inner text, trimmed, equals
  the parsed target.

`emit`/`apply` return `spec.span_failure` on a mismatch or out-of-bounds range
and never panic.

## Why it matters

This is the real content behind the writer's identity claim. If a span is
recorded approximately (covering padding, or a superset of the value), a later
edit splices over the wrong bytes and corrupts the file while still producing
valid-looking Markdown. Span coherence is also the precondition for the parser
and the writer not drifting apart as the grammar grows.

## Check mode

`assert`. Universal over accepted sources; the property calls `emit` and checks
`Ok`, which means `verify_spans` passed.

## Public API / seam

`writer::verify_spans` (`src/acset/writer.rs:64-174`), reached by `emit`
(`:56`) and `apply` (`:234`). The parser records spans in `src/spec.rs`:
`frontmatter_id_span` (`:888`), `push_table_row` (`:377-475`),
`state_bullet_spans` (`:847`), and `collect_links_spanned`.

## Generator sketch

- Build full-family specs with 1–3 constraints, 1–3 states, 1–2 transitions,
  1–3 properties, and several links to defined ids.
- Vary: tight vs padded cells (`| c1 |` vs `|  c1  |`), backticked id cells,
  padded link inners (`[[ demo ]]`), CRLF.
- Call `emit`; assert `Ok` (coherence) and, as a separate corruption leg, mutate
  the source after parse (replace a cell value, truncate) and assert
  `spec.span_failure`.
- A second leg checks duplicate links to the same target get distinct spans.

## Predicted at HEAD `c76e2b0c`

**GREEN.** The parser records the trimmed cell content span for table cells and
the full `[[…]]` occurrence for links; `verify_spans` compares exactly those
values. The existing identity proptest's padded-cell cases would fail here if
coherence were approximate.

## What would make it red

- A parser change that records a span including surrounding padding or a
  trailing table pipe, while the parsed value is trimmed.
- A cell-span map keyed by a header that was duplicated (last duplicate wins in
  `cells`, while `cell_spans` zips headers with spans; a duplicate header could
  bind a span to the wrong cell).
- Link spans that include an unterminated `[[` or a nested bracket shape.
- Multi-byte UTF-8 near a cell boundary where byte offsets drift (span `get`
  would return `None` rather than panic, producing `spec.span_failure`; the
  property would see an unexpected `Err`).

## Existing instrumentation

Partial. `tests/acset_writer_spans.rs` asserts span presence and exact slices
for one fixture plus duplicate links and CRLF. The identity proptest reaches
`verify_spans` indirectly. No generated coherence oracle compares every span to
its element explicitly.

## Assumptions and Open Questions

- [ASSUMPTION] The intended semantics is "span == exact parsed value bytes",
  including the trimmed link inner. The doc comment at `writer.rs:64-71` states
  this.
- Open: how should duplicate table headers be handled? The current zip can bind
  spans incorrectly even if the cell map is last-wins. (partial: code path
  identified; no fixture exercises duplicate headers)

## Investigation Log

- Examined: `verify_spans` (`src/acset/writer.rs:64-174`), `push_table_row`'s
  span recording (`src/spec.rs:377-475`), and the padded-cell cases in the
  identity proptest.
- Found: table cells record the trimmed content span and links record the full
  `[[…]]` occurrence; the existing proptest exercises padded cells and CRLF.
  Duplicate headers are not covered by any fixture.
- Not done: no generated coherence sweep; no duplicate-header execution.