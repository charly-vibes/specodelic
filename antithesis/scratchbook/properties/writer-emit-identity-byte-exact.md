# writer-emit-identity-byte-exact

## Invariant

For every source `x` that `spec::parse_str` accepts,
`writer::emit(x, &parse_str(x).unwrap()) == x` byte for byte, including prose,
CRLF terminators, table padding, backticks, and padded `[[ link ]]` inners.

## Why it matters

`acset.writer.emit_identity` is the gate that lets the typed core claim it does
not cost the format its byte-stable round trip. It is the precondition for every
edit property: if emission with no edit could change bytes, edits would be
indistinguishable from re-serialization drift.

## Check mode

`assert` over generated parser-accepted sources.

## Public API / seam

`writer::emit` (`src/acset/writer.rs:56-60`):

```rust
pub fn emit(source: &str, spec: &Spec) -> Result<String, WriterError> {
    verify_spans(source, spec)?;
    Ok(source.to_string())
}
```

## Why this half is near-tautological

There is **no emitter** here. `emit` returns a clone of the input string after
verifying spans; it never reconstructs Markdown from the IR. Therefore the
byte-identity equation cannot fail as long as `verify_spans` accepts the parse.
The property's real content is:

- `verify_spans` is called at all (a future refactor could drop it);
- it does not reject a legitimate parse (false positive);
- `emit` does not later become a serializer.

The substantive writer properties are `writer-span-coherence` (what
`verify_spans` actually checks), `writer-untouched-bytes-preserved` (edits leave
non-target bytes alone), and `writer-apply-faithful-roundtrip` (the reparse
oracle). This entry is retained so the catalog does not overstate the identity
claim.

## Generator sketch

- Start from a full-family spec string; vary: `n_rows` 1–3, padded vs tight
  cells, backticked vs bare ids, padded vs exact links, quoted vs bare
  frontmatter id, LF vs CRLF.
- Parse; if parse fails, reject the case (assume) rather than asserting.
- Call `emit`; assert `Ok` and byte equality with the original string.

## Predicted at HEAD `c76e2b0c`

**GREEN**, trivially. The existing proptest
`identity_emission_exact` (`tests/acset_writer_identity.rs:143-208`) covers
exactly these axes with 64 cases.

## What would make it red

- `emit` is changed to rebuild text from `Spec` (a real serializer). This is the
  intended regression alarm.
- `verify_spans` starts rejecting accepted shapes (e.g. a span recorded for a
  trimmed cell whose raw bytes include padding). That would surface as
  `spec.span_failure`, not as byte inequality, but the property's `Ok`
  expectation would fail.
- A parser/emit API mismatch where the `Spec` passed was parsed from different
  bytes than `source` — a caller error the property cannot detect because it
  parses and emits the same string.

## Existing instrumentation

The handwritten proptest `identity_emission_exact` plus corpus and fixture
tests in `tests/acset_writer_identity.rs`. This is the **one property in the
catalog that is already well covered**; Hegel would add shrink quality, not
coverage.

## Assumptions and Open Questions

- [ASSUMPTION] The property is useful only as a regression alarm; it is not
  evidence that emission handles structural variation.
- Open: should `emit` take the source and IR separately (it does), and should
  the API prevent mismatched pairs? A type-level `Parsed { source, spec }` would
  remove the caller-error case. (needs human input)

## Investigation Log

- Examined: `emit` (`src/acset/writer.rs:56-60`), `verify_spans`
  (`:64-174`), and the existing `identity_emission_exact` proptest
  (`tests/acset_writer_identity.rs:143-208`).
- Found: `emit` returns `source.to_string()`; there is no serializer, so the
  identity equation is structural. The only gate is span verification.
- Not done: no execution; the claim is visible in the function body.