# Tasks: add-observability-contracts

Each phase is one red→green→refactor cycle; the failing test is written
and observed to fail before the implementation that makes it pass.
Tidying commits are separate from feature commits.

## 1. Format corpus Revisions (spec first — the corpus is the primary fixture)

- [x] 1.1 **RED**: add lint fixtures (synthetic, tempdir) exercising the new
      shape: observes→effect cross-file accepted, observes→invariant
      rejected with remediation hint, mutual cross-file observation passes
      acyclic, effect-without-observer emits the advisory warning
      `linter.observability` on the warnings channel with exit 0 (design
      D5), graph classifies extension_point-hosting file
      as external boundary. Run `just test` — all new tests must fail.
- [x] 1.2 **GREEN (format)**: `specs/specodelic.md` Revision 9 — one new
      Reference Typing row for `observes` (Constraint, any file →
      Constraint, kind == effect only), worded on the `satisfies` precedent,
      including the D7 pre-answered checklist (guard exclusion,
      single_root_reachable, referential_integrity genericity). Bump
      `src/guide.rs`'s `FORMAT_REVISION` to `specodelic.md Revision 9` in
      the same commit — the doctor knowledge-currency check and the guide
      drift-guard both key on it. (Revision 8 is taken: backend
      pluggability, landed with specodelic-len.)
- [x] 1.3 **GREEN (format)**: `specs/kinds.md` Revision 5 — reconcile
      `constraint_row_shape` with optional typed reference columns
      (`satisfies` tension from design.md D3); state the general rule, not
      an `observes` special case. Landing this resolves HITL ticket
      `specodelic-mp1` row 9 — update that ticket's note when it lands.
- [x] 1.4 **GREEN (format)**: `specs/graph.md` — `observes` added to
      `total_extraction`'s field enumeration; acyclic-exemption stated;
      external-boundary derivation as a derived-not-authored constraint.
      `specs/linter-graph_shape.md` — note that `acyclic`'s edge set is
      unchanged (D4).
- [x] 1.5 **GREEN (format)**: new `specs/linter-observability.md` — the
      check's semantics: the observation universe is the lint invocation's
      file set (corpus-wide `just lint-specs` is the canonical run);
      every effect in scope is observed or warned; advisory severity means
      warnings-channel emission, exit 0 (design D5 — decide the
      self-observation question here too, see design Open Questions);
      no waivers in v1 (model on `linter-external_completeness.md`'s file
      shape); a dangling observes is a referential-integrity failure,
      never an observability finding — the two checks compose without
      double-reporting the same row.
- [x] 1.6 **REFACTOR**: re-read the corpus diffs against `AGENTS.md`'s
      Revision discipline (nothing silent, one Revision heading per
      widening); `just lint-specs` clean or gaps filed to beads.

## 2. Parser and typing (red→green→refactor)

- [x] 2.1 **RED**: parser test — optional `observes` column on the
      Constraints table parses into the typed reference set; absent column
      parses as before (no behavior change for the 26 existing corpus
      files). Observe failure.
- [x] 2.2 **GREEN**: extend `src/spec.rs` reference extraction with
      `observes`, reading its target-kind rule from the typing table the
      way `satisfies`/`emits` already do. Observe 1.1's typing tests pass.
- [x] 2.3 **REFACTOR**: confirm `ref_kind_compatible` stayed generic
      (design.md D7: zero special cases in
      `linter-referential_integrity` territory); extract any duplicated
      column-parsing logic with `satisfies` into one helper.

## 3. Linter check (red→green→refactor)

- [x] 3.1 **RED**: `linter.observability` unit tests — unobserved effect →
      advisory warning carrying rule id + semantics string
      (`lint-findings` contract) + row id, on the warnings channel with
      exit 0 (design D5); observed effect → silent; zero effects in corpus →
      zero warnings; check result never affects lint exit code or lifecycle
      gate outcomes. Observe failure.
- [x] 3.2 **GREEN**: implement the cross-file check in `src/lint.rs`
      (corpus-wide pass over extracted `observes` edges vs effect rows);
      emit the warning through `genesis::guide::Output::emit`'s warnings
      channel with remediation hint, never as an `Issue` (which would fail
      the run).
- [x] 3.3 **REFACTOR**: share the cross-file edge-collection plumbing with
      `linter.external_completeness` if the shapes match; don't force it.
      (Note: those rules are unimplemented — deferred to `specodelic-mp1`
      row 10 — so this is at most a forward-looking shape note, not a
      dependency.)

## 4. Graph boundary derivation (red→green→refactor)

- [x] 4.1 **RED**: graph tests — file hosting `extension_point` row is
      classified external boundary in `spk graph --json`; file without one
      is not; removing all extension_points drops the classification
      (stale-tag-impossible property). Observe failure.
- [x] 4.2 **GREEN**: derived classification in `src/graph.rs` — pure
      function over extracted edges, no new authored inputs.
- [x] 4.3 **REFACTOR**: fold into the existing graph query surface
      (blast-radius style) rather than a parallel structure.

## 5. Dogfood and decide (informs the deferred gating question)

- [x] 5.1 Add the USAGE.md §2-style worked example carrying real `emits` +
      `observes` rows (consumer file observing a published effect), and a
      matching fixture corpus file under `tests/`.
- [x] 5.2 Run `spk lint` over the corpus with one deliberately unobserved
      effect; capture the advisory output as the evidence base for the
      gating decision; record the friction observations in this change's
      directory (decision note, no wiring — see beads specodelic-4ae
      precedent for adoption-note discipline). The corpus's own
      `refactor.advisory_finding_emitted` is already unobserved — the
      deliberate case may simply be the corpus itself.
- [x] 5.3 `specs/STATUS.md` — add the status-table row for
      `linter-observability.md`; `specs/CHANGELOG.md` entry.
- [x] 5.4 Full gates: `just ci`, `just lint-specs`,
      `openspec validate add-observability-contracts --strict`,
      `just sync-sections`.
