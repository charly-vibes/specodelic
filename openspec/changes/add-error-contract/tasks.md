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
      failure states. The automated red: `spk graph` piped to a fixture
      assertion `corpus_emitting_failure_states == 0` fails (≥0 expected
      by the new contract), and `corpus_error_constraints == 0` fails
      against `error_property_names_label`'s requirement of ≥1 per label.
      The absence of inbound `satisfies` edges to the new extension_point
      rows is observable in the same graph run.
- [ ] 1.2 **GREEN (format)**: create `specs/errors.md` — intent, the
      nine constraints from the delta (`error_expr_shape`,
      `failure_state_emits`, `failure_class_is_state`,
      `guard_negation_typed`, `single_labeled_failure`,
      `remediation_hint_present`, `error_property_names_label`,
      `contract_published`, `enforcement_routed`), Model, Properties
      (§1.1's rows). File id `errors` per the naming law. No
      `specodelic.md` change (design D7).
- [x] 1.3 **GREEN (format)**: resolve the Open Question in design.md —
      one `extension_point` row per concern vs one merged row; keep the
      corpus reading that `satisfies` edges compose best with.
      **RESOLVED pre-approval (2026-09-30)**: one row per concern
      (`envelope_error_kind` / `exit_code_mapping` / `remediation_hint_present`)
      — the delta already reflects it; carry the same three rows into
      `specs/errors.md` verbatim.
- [ ] 1.4 **REFACTOR**: re-read the diff against `AGENTS.md`'s Revision
      discipline (new file, no Revision heading needed — nothing widened);
      `just lint-specs` green; coverage over every new constraint confirmed.

## 2. D2/D4 — per-tool corpus restructure (emitting failure states, typed guards)

- [ ] 2.1 **RED**: the §1.1 graph fixture assertions now name the
      per-file gaps mechanically: `compile.md` failure terminals == 1
      (expected 2, extract/emit split), failure terminals with `emits`
      edges == 0, failure transitions citing zero intra-file constraints
      and absent from the carve-out list == 6 (compile, rename ×2,
      linter-coverage, linter-referential_integrity ×2). All assertions
      fail before the phase's edits and pass after.
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
      D2 carve-out list and citation-set equality). The class rule is the
      graph-decidable form (classes ⟺ distinct citation sets, D2a) —
      never prose-dependent, so the checker cannot over-reject single-class
      files. No Checker Ownership row (D6 — the table's invariant is that
      every listed checker exists).
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
      Note in the same commit: renaming a label touches three sites
      together — the error Constraint, its falsifying property, and the
      property note (EDGE-002 from the second review).
