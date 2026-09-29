# Tasks: add-error-contract

Each phase is one red→green→refactor cycle; the failing test is written
and observed to fail before the change that makes it pass. Tidying commits
are separate from feature commits. `just lint-specs` must be green after
every phase — the corpus is its own primary fixture.

## 1. D1 — `specs/errors.md`, the contract file (spec first)

- [ ] 1.1 **RED**: author the per-error unit properties first (property
      rows whose predicates assert the exact labels
      `compile.extraction_failure` / `compile.emission_failure` / the
      per-tool failure labels chosen in 2.x) — they fail against the
      current corpus, which has no error Constraints and no emitting
      failure states. Run `just lint-specs` — the new file's absence of
      satisfying consumers is observable via `spk graph` (no inbound
      `satisfies` edges to the extension_point rows).
- [ ] 1.2 **GREEN (format)**: create `specs/errors.md` — intent, the
      eight constraints from the delta (`error_expr_shape`,
      `failure_state_emits`, `failure_class_is_state`,
      `guard_negation_typed`, `single_labeled_failure`,
      `remediation_hint_present`, `error_property_names_label`,
      `contract_published`), Model, Properties (§1.1's rows). File id
      `errors` per the naming law. No `specodelic.md` change (design D7).
- [ ] 1.3 **GREEN (format)**: resolve the Open Question in design.md —
      one `extension_point` row per concern vs one merged row; keep the
      corpus reading that `satisfies` edges compose best with.
- [ ] 1.4 **REFACTOR**: re-read the diff against `AGENTS.md`'s Revision
      discipline (new file, no Revision heading needed — nothing widened);
      `just lint-specs` green; coverage over every new constraint confirmed.

## 2. D2/D4 — per-tool corpus restructure (emitting failure states, typed guards)

- [ ] 2.1 **RED**: `spk graph` over the corpus asserts zero emitting
      failure states and ≥6 untyped `¬x_ok.guard` guards — record as the
      before-fixture (this is the review's evidence, made mechanical).
- [ ] 2.2 **GREEN (compile.md)**: split `failed` → `extract_failed` /
      `emit_failed` (D4; the file's own `compile_is_total` Notes argue the
      two classes); add effect Constraints `compile.extraction_failure(row_id,
      reason)` and `compile.emission_failure(detail)`; `emits` edges from
      both failure states; retype the failure guards as typed negations
      (`¬([[compile.constraint_table_to_toml]] ∧ [[compile.model_to_tla]] ∧
      [[compile.properties_to_proptest]])`, D2); add unit properties
      asserting the exact labels; add `satisfies` edges to the contract
      rows.
- [ ] 2.3 **GREEN (emits-only files)**: `linter-coverage.md`,
      `linter-referential_integrity.md`, `rename.md` — add file-owned
      labeled error Constraints, `emits` edges on the existing `failed`
      states, typed negation guards where the negated constraints are
      intra-file, unit properties naming each label, `satisfies` edges.
- [ ] 2.4 **GREEN (orchestrate.md — the D2 carve-out)**: add the file's
      labeled stage-failure error Constraints and `emits` edges; stage-fail
      guards keep their prose form (per `orchestrate.md:96-98`'s own
      don't-restate discipline); add `satisfies` edges; note the carve-out
      inline so it is stated, not silent.
- [ ] 2.5 **REFACTOR**: `just lint-specs` green; `spk graph` after-fixture
      asserts every failure terminal emits and the only remaining prose
      guards are orchestrate's carved-out ones; re-run the round-trip
      property mentally against `compile.md`'s new state set (states map
      1:1 under `model_to_tla`; stuttering disjunct unchanged).

## 3. D3/D6 — `specs/linter-failure_shape.md`, spec-only

- [ ] 3.1 **Spec**: new checker file on the
      `linter-external_completeness.md` shape — constraints
      `terminal_states_emit` (v1 scope: failure terminals; phase 2 lists
      `timed_out` / `exploration_only` explicitly as a stated non-goal,
      D5), `error_labels_unique` (trivially satisfied corpus-wide by D3's
      namespacing — stated as such), `guard_negation_total` (checking the
      D2 carve-out list). No Checker Ownership row (D6 — the table's
      invariant is that every listed checker exists).
- [ ] 3.2 **Tracking**: file the implementation ticket in beads
      (`linter-failure_shape: enforce terminal_states_emit,
      error_labels_unique, guard_negation_total`) — not gated by
      `specodelic-6pi`, which is closed (design D6); note the AGENTS.md
      staleness to the maintainer instead of editing governance.
- [ ] 3.3 **REFACTOR**: `STATUS.md` §1 file inventory gains `errors.md`
      and `linter-failure_shape.md` rows; `just lint-specs` and
      `just ci` green (section-sync, capability-format, dogfood).

## 4. Wrap-up

- [ ] 4.1 Full gates: `just ci`; `openspec validate add-error-contract
      --strict`; `just lint-specs`.
- [ ] 4.2 Update the per-error unit properties' notes to cite the labels
      they pin, so the label set is greppable from the properties alone.
