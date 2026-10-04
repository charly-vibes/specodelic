# Change: Add acset-writer — span-preserving writer over recorded parse spans

## Why

Moving `rename` and `merge` onto the typed acset core (epic
`specodelic-84i`) is blocked by one missing piece: a module that can turn
an instance edit back into text without destroying the document. A spec
file is a human-edited markdown document whose prose, line endings, and
table padding must survive an edit untouched
(`rename.prose_untouched_by_rename`). Today that skill exists only as
hand-wired table-cell rewriting inside `src/rename.rs` (554 lines), with
no spec of its own, no span recording in the parser, and no identity gate
proving that emission adds nothing beyond the edit.

The external `acset.zip` proposal (Rule of 5, 2026-10-03 — see
`openspec/changes/archive/2026-10-04-add-acset-core/design.md`) specifies
exactly this module as `acset.writer`: the one module allowed to turn an
instance edit back into text, working from recorded source spans and
returning a write-set, never re-serializing a table and never doing I/O.
The review deferred it to its own change (`specodelic-p5b`) because it
requires parse-time byte-span recording — a wide blast radius into the
parser — and carried two must-fix items before drafting: the unnamed
surface (CLAR-001) and the un-noticed misalignment policy (EDGE-001).
Both are decided in this proposal's design.

## What Changes

- **Parser records byte spans.** `src/spec.rs`/`src/parse.rs` gains byte
  spans for every id cell, every `[[link]]` occurrence, and every state
  bullet — additive fields only; the parse result's semantics are
  unchanged and every existing snapshot stays byte-identical.
- **Internal `acset` writer module (no new CLI surface).** `src/acset/writer.rs`
  exposes `emit(parse(x), no edit) == x` (identity emission) and
  `apply(f, x)` for edits, returning a write-set of `(path, full new
  contents)` plus optional removals. No subcommand, no flag, no exit-code
  path; errors surface through the consuming command's envelope per
  `specs/errors.md`.
- **`rename.rs` migrates onto the writer.** Its hand-wired id-cell,
  wiki-link, and state-bullet rewriting is re-expressed as writer edits;
  `spk rename --json` outputs stay byte-identical (parity gate), and
  `rename.rs` remains the text oracle until parity runs through the whole
  corpus.
- **Misalignment accepted (EDGE-001).** When an id edit changes a cell's
  width the writer leaves surrounding padding unchanged — the column may
  become unaligned, no lint rule fires. Realignment is a separate
  explicit formatting operation, never a side effect of an edit.
- **`packs.rs` `table_after` evaluation.** The second, independent
  markdown-table parser (`packs.rs:126`) is evaluated in this slice for
  folding onto the now-spanning parse path; the outcome is recorded in
  the change, folding or keeping the parallel path by documented
  justification.
- **Explicit non-goals:** the pushout merge computer (`specodelic-9um`,
  slice S4 — a capability change to `merge`) and the schema-view re-point
  (`specodelic-hya`) stay their own changes.

## Impact

- Affected specs: new `acset-writer` capability (delta in this change);
  `specs/rename.md`'s behaviour contract is consumed unchanged (the
  writer is how its prose-untouched guarantee is realized mechanically).
- Affected code: `src/spec.rs`/`src/parse.rs` (additive span fields),
  new `src/acset/writer.rs`, `src/rename.rs` (migration), `src/packs.rs`
  (evaluation only); `specs/` corpus untouched.
- Behaviour: `rename`/`merge`/`graph` outputs stay byte-identical —
  every migration step is gated by parity against the existing paths.
  The one new guarantee is `emit_identity`: emission over the whole
  corpus is provably byte-exact, so the typed core never costs the
  format its byte-stable round trip.
