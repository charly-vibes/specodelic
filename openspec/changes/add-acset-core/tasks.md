# Tasks: add-acset-core

Each phase is one red→green→refactor cycle; the failing test is written
and observed to fail before the implementation that makes it pass.
Tidying commits are separate from feature commits.

## 1. Snapshot fixtures (parity harness)

- [x] 1.1 **BASELINE** (pinning, exempt from the red step): snapshot
      tests — `spk graph --json` over the full corpus (byte-identical
      before/after snapshots; note: the corpus currently reports 0
      typing violations — pin violation CLASSES via fixtures, never a
      count), plus a fixture corpus per violation class and per
      dangling-reference shape, plus a zero-file directory and a
      single-intent corpus. Run `just test` — the snapshots must already
      pass (they pin today's behaviour); this is the parity oracle, not
      a failing test.
- [x] 1.2 **RED**: parity property skeleton — `edges(from_specs(c)) ==
      graph::build(c).edges` property over `arbitrary_corpus()` proptest
      cases (fails: `from_specs` doesn't exist yet). Scope note: the
      gate covers every corpus the existing builder accepts, including
      lint-dirty ones (duplicate ids, dangling links) — the builder
      tolerates exactly what the old path tolerates.

## 2. Schema value + generic typing check

- [x] 2.1 **RED**: unit tests for `src/acset/schema.rs` —
      `sixth_object_rejected`, `duplicate_morphism_rejected`,
      `endo_cycle_detected` vs `unflagged_endo_cycle_tolerated` (flagged
      vs unflagged endo-morphism fixtures).
- [x] 2.2 **GREEN**: implement the `Schema` value (objects, typed
      morphisms, allowed-targets incl. the three refinement rules as
      declared predicates, endo-acyclicity flags, canonical sorted
      order).
- [x] 2.3 **RED→GREEN**: switch `typing_violation` in `src/graph.rs` to
      read allowed targets from the Schema — the snapshot fixtures from
      1.1 must stay byte-identical (`adapter_graph_equivalent` gate).
- [x] 2.4 **RED→GREEN**: lint-time drift gate
      (`schema_matches_typing_table`) — fixture: a Schema missing one row
      of specs/specodelic.md's Reference Typing table reports at lint
      time.
- [x] 2.5 **TIDY**: collapse the duplicated edge loops in `src/graph.rs`
      (`:345`/`:438`) behind one edge derivation; dead-flag and clippy
      sweep (`just ci`).

## 3. Typed instance builder

- [x] 3.1 **RED**: unit tests — `dangling_is_a_value`,
      `no_link_dropped` (stored + dangling + violations == links),
      `forbidden_edge_not_stored`, `rebuild_is_byte_stable` (shuffled
      file order), `duplicate_id_first_wins_parity` (builder on a
      duplicate-id corpus matches the existing `kind_index` or_insert
      result and names the collision — the old path never fails, so the
      builder must not either).
- [x] 3.2 **GREEN**: implement `src/acset/instance.rs` — interning
      (dense, bidirectional, sorted-order assignment), partial morphism
      vectors, attribute cells carried untouched (`cells_carried`).
- [x] 3.3 **GREEN**: satisfy the parity property from 1.2
      (`adapter_graph_equivalent`) over the snapshot fixtures + proptest
      cases; the old path stays authoritative until this is green on the
      whole corpus.
- [x] 3.4 **TIDY**: extract shared edge-derivation types between
      builder and `graph.rs`; `just ci`.

## 4. Query primitive + walk migration

- [x] 4.1 **RED**: unit tests — `unknown_seed_rejected`,
      `dangling_not_followed_in_traversal`, `closure_terminates` on a
      cyclic M, `closure_laws` (contains / idempotence / monotonicity,
      proptest), `results_ordered`.
- [x] 4.2 **GREEN**: implement `src/acset/query.rs` — forward/backward
      closure over (seed set, morphism set M).
- [x] 4.3 **RED→GREEN**: parity property (`parity_with_existing`,
      `blast_radius_parity`) — closure results equal `graph.rs` fan-in /
      fan-out / supersedes cycles and `merge.rs`'s `dependents`/`reaches`
      blast radii over fixture branches.
- [ ] 4.4 **RED→GREEN**: migrate `src/graph.rs` fan-in/fan-out onto the
      closure primitive (snapshots byte-identical).
- [ ] 4.5 **RED→GREEN**: migrate `src/merge.rs` blast-radius onto the
      closure primitive; **delete** the private `dependents`/`reaches`
      adjacency (closes the specs/merge.md↔code drift, CORR-003) —
      `merge` command snapshots byte-identical.
- [ ] 4.6 **TIDY**: `src/refactor.rs` consumes query-derived counts;
      `just ci`.

## 5. Corpus + gates

- [ ] 5.1 **RED→GREEN**: `spk lint` over `specs/` stays at zero findings
      with the new lint gate active (dogfooding).
- [ ] 5.2 Full gates: `just ci`, snapshot parity confirmed, openspec
      validate --strict.
- [ ] 5.3 Record the decision trail: design.md's Rule-of-5 table links
      into the two follow-up tickets (writer, pushout) — verify the
      ticket bodies carry their fixes.
