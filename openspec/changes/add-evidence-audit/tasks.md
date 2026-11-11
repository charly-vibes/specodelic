# Tasks: add-evidence-audit

Staged slices — each is an independent red→green→refactor cycle and
shippable alone; cut freely without the others.

## Slice 1 — oracle-row classification + diff (supersedes fqte; parked
## preservation's v1)

- [x] 1.1 Red: fixture corpus + current model — audit classifies each record
      {current, unchecked, contradicted} carrying the underlying conform
      verdict; report carries evidence_scope + scope_sha256.
- [x] 1.2 Red: `--before <report.json>` — a record whose class dropped
      classifies `stale`; new records `new`; digest mismatch fails labeled.
- [x] 1.3 Red: exit contract — 1 iff `contradicted` (or `stale` when the
      prior report declared an expectation); 2 on gate refusal, zero records.
- [x] 1.4 Green: `src/audit.rs` (walk reuse), CLI wiring, envelope output.
- [x] 1.5 Refactor: extract the report-diff into a testable unit.

## Slice 2 — pack-check rows (supersedes va3q)

- [x] 2.1 Red: pack vocabulary + empty checked-set yields `unchecked`,
      countable, never a pass; non-empty checked-set results carry through.
- [x] 2.2 Green: surface pack checker outcomes as audit rows.
- [x] 2.3 Refactor: unify row-emission code across row kinds.

## Slice 3 — binds_nothing (supersedes the model-binding half of h5ks)

- [x] 3.1 Red: fragment/claim citing no declared model element ⇒
      `binds_nothing` on the warnings channel; a legitimately sparse
      phase-model with declared bad states must NOT flag (red 19f6589).
- [x] 3.2 Green: citation-vs-declared-ids check; never exit-nonzero
      (green e337e10).
- [x] 3.3 Refactor: share the declared-ids index with lint.

## Slice 4 — rung aggregate (supersedes 4m3y)

- [x] 4.1 Red: corpus with mixed evidence — summaries report per-file rung
      1–5, never exceeding present evidence; lint/orchestrate summaries
      unchanged otherwise.
- [x] 4.2 Green: derive rungs from existing outputs at report time.
- [x] 4.3 Refactor: `just ci` green.

## Docs + layer

- [ ] 5.1 Graded-evidence docs page (consolidates the Wave-1 scatter; the
      ladder becomes the audit's vocabulary; Limitations discipline).
- [ ] 5.2 `specs/audit/layer.md` sidecar authored; validated via
      `just archive-change id=add-evidence-audit` (fail-closed).
