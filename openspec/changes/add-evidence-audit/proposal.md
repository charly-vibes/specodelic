# Change: Evidence audit — one graded-evidence verb

## Why

The catitas studies' failures were all one concept: **evidence whose scope
silently widened** — stale examples read as green (O2), "empty checked-set"
read as a pass (pack rounds), vacuous fragments read as verified (rev1),
`unknown` verdicts under-read (I4), no re-validation across revisions (O4).
The toolchain already names graded evidence (the ladder, `evidence_scope`,
`exploration_only`) but computes it nowhere uniformly. This change makes one
verb compute it, reusing conform's walk machinery unchanged, and supersedes
four separate near-term features (stale-example revalidation, pack checked-set
honesty, ladder-rung surfacing, model-binding vacuity advisories) as
*scenarios of one capability* rather than commands.

## What Changes

- New command `spk audit <file> --oracle <corpus.jsonl>`: classifies every
  evidence row against the current compiled model with a closed verdict set
  `{current, stale, unchecked, contradicted, binds_nothing}`:
  - **example/oracle rows** — the conform walk classifies each record;
    a row whose class differs from a prior report (via `--before`) is `stale`
    (covers stale-example revalidation AND revision preservation diffing);
  - **pack-check rows** — a checker whose checked-set is empty yields
    `unchecked` (never a silent pass);
  - **bind rows** — fragments/claims referencing no declared model element
    yield `binds_nothing` (warnings channel — correct phase-models are
    legitimately sparse; advisory only, never exit-nonzero).
- Aggregate rung: the summary reports the highest ladder rung for which
  `current` evidence exists — the ladder becomes machine-readable, not just
  documentation.
- Exit-code contract mirrors conform's: 1 on `contradicted` (and `stale`
  only under a declared expectation), never on `unchecked`/`binds_nothing`.
- **Staged slices, each independently shippable** (anti-bundling, per review
  CLAR-001): 1) oracle-row classification + diff, 2) pack rows, 3)
  `binds_nothing`, 4) rung aggregate in lint/orchestrate summaries.
- Read-only like conform: never compiles, never writes artifacts, never
  advances a lifecycle transition. `evidence_scope` labeling verbatim.

## Impact

- Affected specs: new capability `audit`; no existing capability requirements
  modified.
- Affected code: new `src/audit.rs` reusing `src/conform.rs` walk machinery;
  summary surfacing in lint/orchestrate; docs (graded-evidence page
  consolidates the Wave-1 doc scatter).
- Supersedes as scenarios: specodelic-fqte (A3), specodelic-va3q (A7),
  specodelic-4m3y (B2), the model-binding half of specodelic-h5ks (A2).
- Provenance: catitas corpus (Claude bundle v3 O2/O4/I5/I4, pack rounds,
  mistral ladder reading) + the simplification review (2026-10-25).
