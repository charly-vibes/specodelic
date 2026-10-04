# Tasks: add-acset-writer

Each phase is one red→green→refactor cycle; the failing test is written
and observed to fail before the implementation that makes it pass.
Tidying commits are separate from feature commits.

## 1. Baseline snapshots (parity harness)

- [x] 1.1 **BASELINE** (pinning, exempt from the red step): `spk rename
      --json` snapshots over a fixture corpus covering every rewrite
      family (id cell in every table kind, wiki-link in prose, state
      bullet, file's-own-intent-id rename with removal), plus CRLF
      variants of each fixture. Run `just test` — the snapshots must
      already pass (they pin today's behaviour); this is the parity
      oracle, not a failing test.

## 2. Parser span recording

- [x] 2.1 **RED**: a failing identity-shaped test is impossible before
      spans exist, so phase 2's red step is the span-presence test:
      parsing a fixture spec reports the byte span of every id cell,
      every `[[link]]` occurrence, and every state bullet (fails: no
      span fields exist).
- [x] 2.2 **GREEN**: additive span fields in `src/spec.rs` filled by
      `src/parse.rs`; the full-corpus `spk graph --json` snapshot and
      all other snapshots stay byte-identical (design decision D3).
- [x] 2.3 **TIDY**: dead-flag and clippy sweep; `just ci` green.

## 3. Writer core — identity emission

- [x] 3.1 **RED**: `emit(parse(x), no edit) == x` byte for byte — first
      as fixtures over the whole `specs/` corpus (fails: writer does
      not exist), then as a proptest over generated spec-shaped inputs
      (`identity_emission_exact`).
- [x] 3.2 **GREEN**: `src/acset/writer.rs` emits from recorded spans
      only — never re-serializes a table (`source_spans_recorded`).
- [x] 3.3 **RED→GREEN**: span extraction failures surface as
      `acset.writer.span_failure` with remediation hint
      (`span_failure_label_asserted`).

## 4. Edit application

- [x] 4.1 **RED**: applied-edit fixtures — CRLF survives
      (`crlf_survives_edit`), prose mentioning the old id as a word
      survives (`prose_survives_edit`), padding is not realigned
      (`padding_not_realigned`), a file's own intent-id rename maps the
      path and returns one removal (`intent_rename_moves_file`), no I/O
      in the write-set path (`writer_does_no_io`).
- [x] 4.2 **GREEN**: `apply(f, x)` realizes the edit through spans and
      returns the write-set `(path, contents)` + removals
      (`write_set_atomic`, `filename_follows_intent_id`,
      `width_padding_policy`, `untouched_bytes_preserved`).
- [x] 4.3 **RED→GREEN**: roundtrip faithfulness — `from_specs(apply(f,
      x)) == rename(from_specs(x), a, b)` over arbitrary spec/rename
      proptest cases (`edit_application_faithful`), and composition —
      `apply(g, apply(f, x)) == apply(compose(g, f), x)`
      (`writer_edit_law`); failures label `acset.writer.roundtrip_failure`.
- [x] 4.4 **RED→GREEN**: apply-time failures (edit names an id that no
      recorded span can realize) label `acset.writer.apply_failure`
      (`apply_failure_label_asserted`).

## 5. rename.rs migration (parity-gated)

- [x] 5.1 **RED**: rename parity property — for every fixture and
      arbitrary corpus/rename case, writer-driven rename output equals
      the pre-migration `rename.rs` write-set byte for byte (fails:
      rename does not call the writer yet).
- [x] 5.2 **GREEN**: migrate one rewrite family at a time (id cells →
      wiki-links → state bullets) behind the snapshots from 1.1; any
      intentional output difference is an explicit reviewed snapshot
      update (design decision D6).
- [x] 5.3 **TIDY**: delete the now-dead hand-wired rewriting paths in
      `src/rename.rs`; `spk rename --json` snapshots byte-identical;
      `just ci` green.

## 6. table_after evaluation (packs.rs:126)

- [x] 6.1 Evaluate folding `packs.rs`'s `table_after` onto the spanning
      parse path: if its semantics are exactly the spanned parse's table
      extraction, fold and delete the parallel parser; otherwise keep
      it with a comment citing this decision and the blocker. The
      outcome is recorded in this checklist item (design decision D5).
      **Outcome: KEPT.** The spanned parser extracts only
      `TableKind::{Constraints, Properties, Transitions}` under known
      headings (`src/spec.rs` `on_h2`); the six manifest facets are
      pack vocabulary outside `Spec`'s structure — their rows never
      reach the parser, spanned or otherwise. A fold needs new parser
      surface (arbitrary-heading spanned tables) plus escaped-pipe
      semantics parity, far beyond this change's blast radius, for a
      file kind the writer never edits. Decision of record is the doc
      comment on `table_after` (src/packs.rs).

## 7. Gates

- [ ] 7.1 Corpus gates green: `just ci` (now including
      `model-check-specs`), `openspec validate --all --strict`,
      `spk lint openspec` 0/0, `spk lint specs` at baseline, ah check
      contract tests for the deployed scenarios authored at archive
      time.
- [ ] 7.2 Archive via `just archive-change add-acset-writer` (dual-format
      recipe); docs SUMMARY entry; capability spec deployed verbatim.
