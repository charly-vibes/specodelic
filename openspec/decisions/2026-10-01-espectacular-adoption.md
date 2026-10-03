# Decision: espectacular scenario-conformance over specodelic's openspec specs

- **Status**: **ADOPTED-WITH-SEQUENCING** (supersedes the 2026-10-01 DEFER
  below, which is kept as historical context) — specodelic-tlv, owner
  instruction 2026-10-03.
- **Hard constraints that survive the reversal** (AGENTS.md, enforced by
  `scripts/guards/sibling-blockers.sh` + tests/sibling_blockers.rs):
  - espectacular stays **read-only over `openspec/`** — contracts live in
    `.espectacular/`, never inside `openspec/`.
  - The sibling-blockers guard must be updated in the SAME commit as any
    wiring (EDGE-002).
- **Sequencing**: `ah init` + gate wiring (`just ci` + lefthook) land first;
  full per-scenario contract authoring (~123 deployed `#### Scenario:`
  blocks) is a separate follow-up tracked in beads. Until contracts exist,
  `ah check` is wired advisory-only, not gating.

## Historical record: the 2026-10-01 DEFER (superseded by specodelic-tlv)

- **Date**: 2026-10-01
- **Ticket**: specodelic-4ae (P3)
- **Decision**: **DEFER** — revisit at orchestrate (`specodelic-8kk`), per this
  ticket's own advisory. No wiring now, and no pilot unless that revisit
  asks for one.

## What espectacular would do here

`espectacular check` discovers `#### Scenario:` blocks under
`openspec/specs/<cap>/spec.md`, resolves each slug against a contract TOML
in `.espectacular/<spec>/<id>.toml`, and runs the contract's test entries
(cargo/shell/property/custom) — producing structural findings (missing
contract, slug collision, overlay conflict) and execution findings
(failing bound test), plus quality findings. It is **read-only over
`openspec/`** (contracts live outside it), so adoption satisfies the
sibling-tool constraint in AGENTS.md unchanged. No repo-side blockers
remain (specodelic-6pi closed 2026-09-28; lint-deltas is recursive and
gate-chainable).

## Fit assessment

- **Coverage surface**: 123 deployed `#### Scenario:` blocks across 11
  capability specs (compile 13, error-contract 18, hooks 20,
  observability 14, spec-integration 16, embedded-guide 10, migrate 10,
  docs-site 8, doctor 7, model-check 5, lint-findings 2), plus 26
  in-flight delta scenarios in the active `add-graph-views` change.
- **Test correspondence is the real cost**: the repo's ~144 tests are
  not 1:1 named per scenario. Full adoption means authoring 123 contract
  TOMLs *and* establishing named-test correspondence (rename/cluster
  splitting) — estimated **2–4 days initial**, plus ongoing friction:
  every new delta scenario needs a staged contract at authoring time
  (`add-graph-views` alone would add 26).
- **Redundancy risk — the deciding factor**: specodelic is already
  building its own spec↔test execution loop: `verify` (specs/verify.md,
  ticket specodelic-1pv) executes `compile.md`'s proptest! scaffolding
  and folds the result with `model_check` into the corpus `verified`
  gate. Two conformance systems adopted in parallel would duplicate the
  seam (runner invocation, finding schema, CI gating). Deciding **after**
  verify's shape is known lets capability-scenario conformance reuse the
  same execution seam instead of a parallel one.
- **What it uniquely wins**: drift detection on scenario *prose* —
  a hand-edited deployed scenario produces an overlay/structural finding
  today's gates cannot see (section-sync only checks the intra-file
  ADDED↔Requirements duplication). This is a real, unowned gap — but a
  narrow one, and the corpus's scenario texts have not drifted in
  practice (the drift incidents so far were artifacts and SUMMARY gaps,
  both now gated).

## Revisit criteria (at specodelic-8kk)

1. `verify` (specodelic-1pv) has landed — does its execution seam cover
   or subsume per-scenario test binding for capability specs?
2. A real scenario-prose drift incident has occurred (evidence the gap
   is not theoretical), **or** orchestrate's stage reports need
   conformance output.
3. If both: run a bounded pilot first — `just conformance` over
   `lint-findings` (2 scenarios) or `doctor` (7), **not** wired into CI,
   measuring authoring cost and findings quality — before any full
   adoption (~0.5 day pilot; full adoption per the estimate above).

## Rejection clause

If at revisit the verify seam covers the need, reject outright and note
it here; the deferral is not a permanent hold-open.