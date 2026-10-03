# Change: Add acset-core — fixed schema as data, typed instances, one traversal primitive

## Why

The typed reference machinery is spread across three private, partially
duplicated implementations: `src/graph.rs` holds the Reference Typing
table as `NodeKind`/`kind_index` plus per-field match arms in
`typing_violation` and builds `fan_in`/`fan_out` in two separate edge
loops (`src/graph.rs:71-127`, `:345`, `:438`); `src/merge.rs` builds its
own `dependents`/`reaches` adjacency (`src/merge.rs:122-130`) despite
`specs/merge.md`'s mandate to depend on `graph.md`'s blast-radius query —
a live spec↔code drift; `src/refactor.rs` consumes `GraphReport` counts.
An external proposal (the `acset.zip` corpus of six intent specs + a
`theory.md` patch, reviewed under Rule of 5, 2026-10-03) diagnosed the
same duplication from the spec side and proposed an attributed C-set
core. The review's verdict: adopt the cheap, parity-testable core now
(schema-as-data + instance builder + query primitive); defer the writer
and the pushout merge to their own changes (filed as beads tickets).

This change makes the diagnosis's fix structural: the schema becomes a
value every module reads, dangling references become data, and
forward/backward closure becomes the only traversal primitive — with
parity gates against the existing walks so nothing changes behaviour on
day one.

## What Changes

- **Tool — internal `acset` core (no new CLI surface):** a `Schema` value
  holding the five objects and typed morphisms of the format's fixed
  finite schema; a typed instance builder over parsed corpora (interned
  dense id-sets, partial morphism values where `None` = dangling);
  forward/backward closure primitives parameterized by morphism set.
- **`graph.rs` migration:** `typing_violation` reads allowed targets from
  the `Schema` value instead of match arms; `fan_in`/`fan_out` are
  expressed through the closure primitives; the two duplicated edge loops
  collapse to one.
- **`merge.rs` migration:** the private `dependents`/`reaches` adjacency
  is deleted and blast-radius pre-merge queries are expressed through the
  same closure primitives — closing the `specs/merge.md`↔code drift as a
  side effect.
- **Lint-time schema drift gate:** the `Schema` value is checked
  row-for-row against `specs/specodelic.md`'s Reference Typing table at
  lint time, so document and code cannot drift.
- **Explicit non-goals (deferred to tickets):** the span-preserving
  writer (`acset.writer` — needs parse-time byte-span recording) and the
  pushout merge computer (`acset.pushout` — a capability change to
  `merge`, not a refactor). Both carry the Rule-of-5 fixes in their
  tickets before any change proposal for them is drafted.

## Impact

- Affected specs: new `acset-core` capability (delta in this change).
- Affected code: `src/graph.rs` (typing + fan-in/fan-out), `src/merge.rs`
  (blast-radius walk), new `src/acset/` module family; `specs/`
  corpus untouched (the `theory.md` patch ships with the later
  pushout/writer changes, not here).
- Behaviour: graph/merge/refactor outputs stay byte-identical — every
  migration step is gated by edge-for-edge parity against the existing
  walks (`adapter_graph_equivalent`, `parity_with_existing` in the
  delta). The one new observable is the schema-drift lint finding class
  (`schema_matches_typing_table`), which reports zero findings on the
  current corpus; the builder additionally surfaces a collision report
  for duplicate-id corpora, resolving them exactly as the old path did
  (first-wins).
