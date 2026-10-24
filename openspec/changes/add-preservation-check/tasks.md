# Tasks: add-preservation-check

## 1. Red — failing tests first

- [ ] 1.1 Fixture: two revisions of one capability file (v1/v2 style, final-frame
      bonus change from the study), one oracle corpus of 5 scenarios — 3 preserved,
      1 changed-and-declared, 1 undeclared change. `spk preserve` does not exist
      yet: command-absence test fails.
- [ ] 1.2 Classification test shape: verdict map per scenario id ∈
      {preserved, intentionally_changed, undeclared_change, unchecked};
      expected counts pinned in the test.
- [ ] 1.3 Labeling test: every record + the report header carry
      `evidence_scope: "agreement on the supplied corpus; not a proof of
      behavioral equality"`; absence fails.
- [ ] 1.4 Exit-code test: undeclared_change ⇒ 1; preserved/unchecked-only ⇒ 0;
      gate refusal ⇒ 2 with zero records.
- [ ] 1.5 Empty-evidence test: empty corpus and zero-matching ids both fail
      labeled with no report.
- [ ] 1.6 Declarations-file test: `{scenario_id, expect_change}` parses;
      unknown field refused labeled; conform's oracle record rejects
      `expect_change` (schema stays closed — pin it).
- [ ] 1.7 Identical-input pin: before == after ⇒ all preserved.

## 2. Green — make the tests pass

- [ ] 2.1 Implement `src/preserve.rs`: artifact gate (reuse conform's), run
      conform walk on both files, diff verdicts, emit report.
- [ ] 2.2 Wire `spk preserve` into the CLI + envelope output via
      `genesis::guide::Output::emit` (errors carry remediation hints).
- [ ] 2.3 Docs: decision-guide routing row + a short preserve.md page
      carrying the Limitations discipline.

## 3. Refactor — separate commit

- [ ] 3.1 Extract shared walk-driver code between conform and preserve.
- [ ] 3.2 `just ci` green; `just lint-specs` unaffected (new capability spec
      deployed only at archive).

## 4. Dual-format layer (fail-closed gate)

- [ ] 4.1 `specs/preservation/layer.md` sidecar authored and validated by the
      archive companion (`just archive-change id=add-preservation-check`).
