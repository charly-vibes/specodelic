# Tasks — add-check-judge-verb

Each numbered step is one red→green cycle and its own commit; the refactor
step (6) is never mixed with behavior. The mapping table in `design.md` §D3
and the capability spec's "Verify-stage outcome mapping" / "Recorded and
citation evidence mapping" requirements are the acceptance matrix — every
table row gets at least one fixture case (the spec encodes the matrix as
scenario-per-row because `openspec validate --strict` rejects >500-character
requirement bodies; the matrix itself stays in `design.md` and the deployed
docs).

**Hard constraints for every step:** do not add `Verdict::Inconclusive` (no
producer exists; T1 records termination, never a verdict); do not change
`verify`, `conform`, or `audit` status strings, schemas, or exit codes; do not
add an adapter, registry, or suite-binding flag; stage only session-authored
paths.

## 1. Judgement matrix — red

- [ ] 1.1 `tests/check_matrix.rs` (or the crate's CLI-integration convention):
  fixture cases for every row of the mapping table — property pass/fail, model
  clean, `counterexample_found`, required claim `refuted`/`unknown`,
  `exploration_only`, model `timed_out`, each conform verdict, pack
  violations/empty checked-set, bind row, stale/new overlays — asserting the
  exact record (key, verdict, class) and exit code.
- [ ] 1.2 Refusal cases: lint findings, stale/missing artifacts, stale/missing
  model report, missing/uncompilable props artifact, runner unavailable, killed
  runner, oracle parse failure, `--before` digest mismatch, zero rows — each
  asserting zero records plus exit 2 and a remediation hint.
- [ ] 1.3 Ladder cases: both gates green ⇒ rung 4 with the verify-pass basis;
  failed properties ⇒ no lift; `counterexample_found` ⇒ rung 5 unchanged.
- [ ] 1.4 Interop cases: an audit report is a valid `--before` prior for
  `check`, and a check report is a valid prior for `audit`; identical inputs
  serialize byte-identically.
- [ ] 1.5 Scope case: the CLI surface has no adapter/registry/suite flag and
  output never spells `inconclusive`.

## 2. Verb — green

- [ ] 2.1 Add `Commands::Check` in `src/main.rs` and `src/commands/check.rs`
  (orchestration only): gate via the existing gate call, stage 2 via
  `verify::evaluate_file_verdicts` with the real bounded `PropertiesRunner`,
  stage 3 via conform's walk when `--oracle` is present, stage 4 via audit's
  row assembly, `AuditClass::of` classification, fold, prior diff and rung
  derivation. No duplicated evaluator, gate, or report serializer. Keep the
  new module under the pretender file cap (300 lines) — split by cohesion, not
  by padding.
- [ ] 2.2 Set `EvidenceMix.verify_pass` from this run's verify verdict before
  rung derivation (`src/audit/rung.rs` call site) — the documented collector
  gap; no other rung logic changes.
- [ ] 2.3 Implement the exit contract as one function over (merged report,
  prior declaration) and render both views from it; audit's `exit_code` stays
  the shared predicate for the class layer.
- [ ] 2.4 Emit the report through `genesis::guide::Output::emit` with the
  audit report shape; no new persisted artifact, no `check`-named file.

## 3. Refusal discipline — red→green

- [ ] 3.1 Red: a refusal test asserting the error envelope is `ok: false`,
  names the cause, carries a non-empty remediation hint, and leaves the spec
  and artifacts byte-identical (checksum before/after).
- [ ] 3.2 Green: the single `check.evidence_unavailable(detail)` label and
  hint mapping for every refusal cause in design §D3.

## 4. Docs

- [ ] 4.1 `docs/src/commands.md`: a `spk check <file> [...]` section — flags,
  the mapping pointer, exit codes, and the relation to
  `verify`/`conform`/`audit` (one judge verb, three evidence sources).
- [ ] 4.2 `docs/src/graded-evidence.md`: a "the judge verb" section pointing at
  the capability spec's mapping table and exit contract; correct the
  `inconclusive` reserved row's trigger to the remaining producer (a contract
  suite that executes and cannot decide) and state that `check` does not
  produce it.
- [ ] 4.3 `docs/src/verification-boundaries.md`: rung 4 is reachable through
  `spk check`; `spk audit` keeps its documented ceiling until verify persists
  a report.
- [ ] 4.4 Goals doc `.wai/resources/research/2026-11-16-authority-lattice-goals-and-problems.md`
  §2.2 status column: G3 (judge verb shipped), G13/G14 (`check` built; three
  verbs stand).

## 5. Gates

- [ ] 5.1 `openspec validate add-check-judge-verb --strict` → clean.
- [ ] 5.2 `just lint-deltas` (the corpus lint, layer sidecar included) →
  clean; `python3 scripts/check_evidence_axes.py .` → clean with no new value
  row.
- [ ] 5.3 `just ci` → green.

## 6. Refactor — separate commit

- [ ] 6.1 If row-key construction or view rendering duplicates audit's helpers,
  extract them once and re-run 1.x + 5.3; no behavior change.

## 7. Archive and close-out

- [ ] 7.1 `just archive-change id=add-check-judge-verb` (the companion
  re-attaches the layer; the stock archive must not be used).
- [ ] 7.2 Close specodelic-8hjn; `bd export`; confirm `origin/main` in sync.