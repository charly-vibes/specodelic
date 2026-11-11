# Design: add-evidence-audit

## Context

conform already classifies external traces against a compiled model with a
closed verdict taxonomy, evidence-scope labeling, scope digests, and
deterministic re-runs. Every graded-evidence gap the studies found is that
machinery applied to evidence the toolchain already touches but never
re-validates: recorded example corpora, pack checker results, fragment/claim
bindings, and prior audit reports.

## Goals / Non-Goals

- Goals: one verb, one verdict vocabulary, one aggregate; reuse conform
  unchanged; staged independently-shippable slices.
- Non-Goals: any equivalence or refinement claim; new checking semantics
  (nothing here checks what conform could not); first-class example tables
  in the format (the oracle-corpus path is the v1 representation —
  first-class examples stay a format-level decision, explicitly deferred);
  replacing model-check or verify.

## Decisions

- **Decision: diff two audit reports for staleness, not files.** `spk audit
  --before <report.json>` classifies the current model against the oracle
  corpus and compares per-row verdicts with the prior report: same class ⇒
  `current`, class dropped/worsened ⇒ `stale`, no prior row ⇒ `new`. This
  makes revision preservation a by-product (the parked
  add-preservation-check's v1 exactly), with no second walk implementation.
  - Alternatives considered: walking both models in one invocation (reuses
    nothing new; the report-diff is smaller and works with any prior run,
    including ones from before this change — rejected the two-model walk).
- **Decision: `binds_nothing` on the warnings channel.** The study's I5
  finding cuts both ways: vacuity exposes eliminable concepts AND correct
  phase-models are legitimately sparse. Advisory-only, file-scoped, never
  exit-nonzero — same discipline as specodelic-xci.
- **Decision: rung aggregate is derived, not stored.** Highest rung with
  `current` evidence, computed at report time from audit/conform/verify
  outputs already present. No new state, no staleness machinery for the
  rung itself.
- **Decision: verdict vocabulary is audit-local and closed.** conform's
  {permitted, forbidden, underspecified, unknown, unsupported} stays
  untouched; audit adds the evidence-lifecycle layer
  {current, stale, unchecked, contradicted, binds_nothing} on top of it —
  an audit record carries BOTH (the conform verdict and the audit class).

## Risks / Trade-offs

- Risk: one verb grows into a grab-bag → mitigated by the four staged
  slices, each with its own red→green cycle and shippable alone; any slice
  can be cut without the others.
- Risk: `stale` misread as equivalence failure → every record carries
  evidence_scope verbatim; docs state the boundary.
- Risk: diffing reports from different corpus versions → both reports carry
  `scope_sha256`; a mismatch is a labeled finding (same discipline as
  conform's digest).

## Open Questions

- Should the rung aggregate also surface in `spk doctor` output? Deferred —
  the parked add-doctor-advice covers that if demand shows.
- Does `binds_nothing` want a graph-backed signal (fan-in zero) before v1?
  Start with the cheap substring-free check (citations/fragments vs declared
  ids); graph-backed refinement later if false positives appear.
