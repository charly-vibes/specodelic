# Design: add-acset-writer

## Context

Slice S3 of the acset adoption (epic `specodelic-84i`). The source spec
is `acset.zip`'s `acset-writer.md` (kind: intent, the "largest risk in
the whole extraction" per its own Notes). The core it needs — Schema
value, typed instance builder, closure primitives — landed as
`add-acset-core` (archived 2026-10-04). What is still missing is entirely
mechanical: the parser does not record where anything lives in the file,
so no edit can be applied without re-serializing.

## Rule-of-5 findings and how this change resolves them

| id | finding (verified) | resolution here |
|----|--------------------|-----------------|
| CLAR-001 (MEDIUM) | No surface named for the writer anywhere in the source spec | **Decision D1** below: internal-only module consumed by `rename.rs`; no new subcommand, flag, or exit-code path. |
| EDGE-001 (MEDIUM) | `width_padding_policy` permits columns to become unaligned and no lint rule notices — decide: lint advisory or accept silent degradation | **Decision D2** below: accept the degradation; no advisory in this change. |
| packs.rs:126 note (below-gate, human call) | `table_after` is a second, independent markdown-table parser ("multiple markdown parsing paths"); deliberate today | **Decision D5** below: evaluated in this slice; fold or keep by documented justification. |

## Key decisions

1. **Internal-only writer (CLAR-001).** `acset.writer` is the library
   seam `rename` (and later the pushout merge) call; it performs no I/O
   and returns a write-set — "applying it is the caller's single
   transaction" (`acset-writer.md`, `write_set_atomic`). A subcommand
   would contradict that contract: there is no user-meaningful "emit
   without an edit" operation (that is the identity gate, a test), and
   every edit worth making is already reachable through an existing
   command. The three writer error labels
   (`acset.writer.span_failure`/`apply_failure`/`roundtrip_failure`) are
   internal causes wrapped by the consuming command's envelope error per
   `specs/errors.md` — no new exit codes.
2. **Misalignment accepted; no lint advisory (EDGE-001).** The source
   spec is explicit that this is a decision, not a discovery: preserving
   alignment under id edits would force table re-serialization, which
   `source_spans_recorded` forbids. The corpus already tolerates
   unaligned columns, and an advisory finding would (a) require a
   baseline entry class under `specs/.lint-baseline`'s remove-only
   ratchet, (b) fire on every width-changing rename immediately, and
   (c) police a property the format never promised. `emit_identity` is
   the guarantee that matters; if misalignment is ever observed as a
   real annoyance, an advisory lint rule is its own small change —
   decision recorded here, not silently inherited.
3. **Spans are additive fields, not a parallel parse.** The parsed
   structures in `src/spec.rs` gain byte spans for id cells, `[[link]]`
   occurrences, and state bullets; nothing else about parsing changes.
   Parity gate: every existing snapshot and the full `spk graph --json`
   corpus snapshot stay byte-identical after the parser change.
4. **`emit_identity` is the central gate, proven corpus-wide.**
   `emit(parse(x), no edit) == x` byte for byte for every file the
   parser accepts — fixture-driven over the whole `specs/` corpus plus
   proptest-generated cases, mirroring `tests/acset_parity.rs` from the
   core change. Until the identity gate runs through the whole corpus,
   `rename.rs` stays the text oracle.
5. **`packs.rs` `table_after`: evaluate, then fold or justify (D5).**
   The span-recording parse path makes a second table parser redundant
   *if* pack manifests parse through the same path. Task 6.1 evaluates;
   the fold happens in this change only if `table_after`'s semantics are
   exactly the spanned parse's table extraction; otherwise the parallel
   path stays with a comment citing this decision and the blocker that
   prevented the fold. No silent coexistence.
6. **`rename.rs` migrates last, behind byte-identical output parity.**
   The writer is proven (identity + applied-edit fixtures) before
   `rename.rs` touches it; the migration flips one rewrite family at a
   time (id cells, then wiki-links, then state bullets), each behind
   `spk rename --json` snapshots. `rename.rs`'s own checks
   (`prose_untouched_by_rename`, atomic apply) are unchanged — the
   writer realizes them mechanically.

## Risks

- **Span extraction off-by-one on table edges** — pipes, padding, and
  line endings are exactly the bytes the writer must not touch.
  Mitigation: the identity gate runs per-file over the corpus after the
  first parser change; CRLF fixtures are included (the Windows runner
  CRLF gotcha from `acset_parity` is already known — snapshots compare
  line-ending-agnostically there, but `emit_identity` must be exact, so
  CRLF corpus fixtures run in the identity test explicitly).
- **`rename.rs` writes bytes today that a span-preserving writer would
  render differently** (e.g. padding the replacement cell to the old
  width). Mitigation: parity snapshots before/after per rewrite family;
  any intentional difference (e.g. padding policy) surfaces as an
  explicit snapshot update reviewed in the change, never silently.
- **Wide blast radius in the parser** — every command consumes parsed
  specs. Mitigation: spans are additive fields; no consumer reads them
  until the writer phase; `just ci` runs at every phase boundary.
