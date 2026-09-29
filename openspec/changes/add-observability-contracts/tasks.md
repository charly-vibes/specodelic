# Tasks: add-observability-contracts

Each phase is one red→green→refactor cycle; the failing test is written
and observed to fail before the implementation that makes it pass.
Tidying commits are separate from feature commits.

## 1. Format corpus Revisions (spec first — the corpus is the primary fixture)

- [ ] 1.1 **RED**: add lint fixtures (synthetic, tempdir) exercising the new
      shape: observes→effect cross-file accepted, observes→invariant
      rejected with remediation hint, mutual cross-file observation passes
      acyclic, effect-without-observer emits advisory
      `linter.observability`, graph classifies extension_point-hosting file
      as external boundary. Run `just test` — all new tests must fail.
- [ ] 1.2 **GREEN (format)**: `specs/specodelic.md` Revision 8 — one new
      Reference Typing row for `observes` (Constraint, any file →
      Constraint, kind == effect only), worded on the `satisfies` precedent,
      including the D7 pre-answered checklist (guard exclusion,
      single_root_reachable, referential_integrity genericity).
- [ ] 1.3 **GREEN (format)**: `specs/kinds.md` Revision 5 — reconcile
      `constraint_row_shape` with optional typed reference columns
      (`satisfies` tension from design.md D3); state the general rule, not
      an `observes` special case.
- [ ] 1.4 **GREEN (format)**: `specs/graph.md` — `observes` added to
      `total_extraction`'s field enumeration; acyclic-exemption stated;
      external-boundary derivation as a derived-not-authored constraint.
      `specs/linter-graph_shape.md` — note that `acyclic`'s edge set is
      unchanged (D4).
- [ ] 1.5 **GREEN (format)**: new `specs/linter-observability.md` — the
      check's semantics: every effect is observed or reported, advisory,
      cross-file, no waivers in v1, never gates (model on
      `linter-external_completeness.md`'s file shape).
- [ ] 1.6 **REFACTOR**: re-read the corpus diffs against `AGENTS.md`'s
      Revision discipline (nothing silent, one Revision heading per
      widening); `just lint-specs` clean or gaps filed to beads.

## 2. Parser and typing (red→green→refactor)

- [ ] 2.1 **RED**: parser test — optional `observes` column on the
      Constraints table parses into the typed reference set; absent column
      parses as before (no behavior change for the 26 existing corpus
      files). Observe failure.
- [ ] 2.2 **GREEN**: extend `src/spec.rs` reference extraction with
      `observes`, reading its target-kind rule from the typing table the
      way `satisfies`/`emits` already do. Observe 1.1's typing tests pass.
- [ ] 2.3 **REFACTOR**: confirm `ref_kind_compatible` stayed generic
      (design.md D7: zero special cases in
      `linter-referential_integrity` territory); extract any duplicated
      column-parsing logic with `satisfies` into one helper.

## 3. Linter check (red→green→refactor)

- [ ] 3.1 **RED**: `linter.observability` unit tests — unobserved effect →
      advisory finding carrying rule id + semantics string
      (`lint-findings` contract) + row id; observed effect → silent; zero
      effects in corpus → zero findings; check result never affects
      lifecycle gate outcomes. Observe failure.
- [ ] 3.2 **GREEN**: implement the cross-file check in `src/lint.rs`
      (corpus-wide pass over extracted `observes` edges vs effect rows);
      emit through `genesis::guide::Output::emit` with remediation hint.
- [ ] 3.3 **REFACTOR**: share the cross-file edge-collection plumbing with
      `linter.external_completeness` if the shapes match; don't force it.

## 4. Graph boundary derivation (red→green→refactor)

- [ ] 4.1 **RED**: graph tests — file hosting `extension_point` row is
      classified external boundary in `spk graph --json`; file without one
      is not; removing all extension_points drops the classification
      (stale-tag-impossible property). Observe failure.
- [ ] 4.2 **GREEN**: derived classification in `src/graph.rs` — pure
      function over extracted edges, no new authored inputs.
- [ ] 4.3 **REFACTOR**: fold into the existing graph query surface
      (blast-radius style) rather than a parallel structure.

## 5. Dogfood and decide (informs the deferred gating question)

- [ ] 5.1 Add the USAGE.md §2-style worked example carrying real `emits` +
      `observes` rows (consumer file observing a published effect), and a
      matching fixture corpus file under `tests/`.
- [ ] 5.2 Run `spk lint` over the corpus with one deliberately unobserved
      effect; capture the advisory output as the evidence base for the
      gating decision; record the friction observations in this change's
      directory (decision note, no wiring — see beads specodelic-4ae
      precedent for adoption-note discipline).
- [ ] 5.3 `specs/STATUS.md` — add the status-table row for
      `linter-observability.md`; `specs/CHANGELOG.md` entry.
- [ ] 5.4 Full gates: `just ci`, `just lint-specs`,
      `openspec validate add-observability-contracts --strict`,
      `just sync-sections`.
