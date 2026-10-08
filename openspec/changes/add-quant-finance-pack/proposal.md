# Proposal: add-quant-finance-pack

## Why

D6 of the decision of record (2026-10-02, `.wai/projects/domain-specific-extensions/designs/matrix/decision.md`) named the quant pack **second** after the bioimage-data pilot: *"quant second"* — and the external-review synthesis (`openspec/research/2026-10-02-external-review-synthesis/synthesis.md`) shows quant is where the vendor convergence started. Two independent vendors shipped quant-finance designs (grok `specodelic-quant-proposal`, mistral `specodelic-quant-finance`), and the recurring verdict sentence is quant's: *"the strongest finance/science constraints are exactly the ones prose_untouched makes uncheckable."* The mechanism (`add-domain-pack-mechanism`, archived 2026-10-03) and the three standard packs it rides (data/lineage, numeric predicates, empirical registry — all archived 2026-10-03) are exactly what makes this pack **thin**: it declares only the vocabulary no standard pack already carries, and reuses the rest. What it does declare is real new checking surface both vendors demanded — the typed `## Limits` section and the risk case-label floor — and `append_only_packs` leaves every later release (temporal vocabulary, executable bound predicates, a pricing floor) room to grow without breaking this one.

## What Changes

- **New capability `quant-finance-pack`** (delta in this change): the
  contract for the quant-finance domain pack (`packs/quant-finance.md`,
  id `quant.finance`) — a declared `kind: profile` pack artifact with a
  namespaced `## Limits` section (`| name | kind | unit | bound |`),
  the `quant.risk` and `quant.pricing` kinds, `capped_by` as an
  outbound-leaf reference field resolving to a `## Limits` row of the
  declaring file, the `quant.limit_closed` and `quant.risk_labels`
  checker declarations, a kind-dependent risk floor (`horizon` +
  `confidence` — derived per design D4 from the R4 per-kind floor
  precedent; grok-quant's {metric, abs, rel, unit} tolerance fields are
  already covered by `numeric.tolerance`), and a `## Requires` table
  pinning base `specodelic.md Revision 18` (the current revision; the
  earlier packs' Revision 14 pins stay valid under `append_only_variants`
  and revision skew is advisory — design D5) plus the
  `numeric.predicates` and `data.lineage` standard packs — a cross-pack
  consumer like the bioimage pilot.
- **New in-repo pack artifact plan** `packs/quant-finance.md` (id
  `quant.finance`, per the file-naming law): six manifest tables
  declaring exactly the vocabulary above. No checker code — pack
  checkers ship as declarations per `specs/packs.md` (honest-empty
  convention); nothing in `src/` changes.

## Capabilities

- **New capability `quant-finance-pack`**: limits-section declaration,
  namespaced quant kinds, the risk case-label floor, `capped_by` as an
  outbound leaf, checker declarations honest-empty, and pack
  dependencies on the two standard packs the vendor designs consume
  (`numeric.predicates`, `data.lineage`) — with the empirical-registry
  pack as the optional companion for statistical claims on backtests
  (`empirical.statistic`, declared not-required per design D5).

## Out of scope

- **No second `tolerance` kind** (E1/E2, the collision the mechanism
  exists to prevent): grok-quant's `tolerance` Constraint kind carrying
  `{metric, abs, rel, unit}` is already `numeric.tolerance` in the
  numeric-predicates pack. Namespacing makes a re-declaration legitimate
  but pointless; quant laws that need tolerance semantics declare
  `numeric.tolerance` rows.
- **No second `statistic` kind**: mistral-quant's `statistic` kind with
  `stat_test ∈ {stationarity, mean_eq, ci_within, distribution}` is
  already the empirical-registry pack's `empirical.statistic` +
  `tested_by` machinery. Quant spec files that state statistical claims
  declare `empirical.statistic` rows against the registry.
- **No `## Fixtures` section** (design D2): the corpus-contamination
  audit rejects the token (4+4 files already use `fixture`/`Fixtures`
  as prose). The vendor demand — bounded backtest fixtures with declared
  provenance — is satisfied by the data-lineage pack's `## Data` rows
  (`data.dataset` kind, `produced_by`/`consumed_by` lineage), which this
  pack requires.
- **Temporal guards / `### Continuous`** (synthesis R7; the same Time
  requirement is matrix R6, Could priority): no time standard pack
  exists yet; temporal vocabulary is deferred until a time pack or a
  format Revision carries it.
- **The EARS sixth quantitative pattern** `WITHIN <ε> [<unit>]`
  (matrix E4, the EARS gap): a core-candidate intent-statement change,
  not pack work — it rides a true format Revision, not this pack.
- Any `src/` change: no new lint rules, no checker implementations.

## Alignment

- `append_only_packs` applies to the pack itself: releases may only add
  vocabulary, never remove or narrow.
- `prose_untouched` holds: the pack declares vocabulary, never prose
  rules. The vocabulary-hygiene lesson (`add-data-lineage-pack` design
  D6, applied up front by `add-numeric-predicates-pack` D2) is applied
  here with a fresh audit (design D2): **no bare-English tokens are
  declared** — `limit`, `confidence`, `Fixtures` are corpus-contaminated
  and excluded; `horizon` survives only as a floor case label, which the
  activation scanner never reads.
- `single_root_reachable` needs no carve-out: `capped_by` is an outbound
  leaf (the `uses`/`observes`/`measured_by` precedent).
- Base closed sets frozen: `quant.*` kinds are pack-fiber-relative; the
  `## Limits` section is a pack-declared section; the mechanism's
  per-facet closed sets absorb all growth.
- Honest-empty: `quant.limit_closed` reports an empty checked-set when
  no `## Limits` rows exist; no fabricated discoveries.
- Self-hosting: the pack is itself a four-layer spec file in the corpus,
  linted by the same tool it extends.

## Governance

- The adoption bar is met by the decision matrix's Demand column — R1
  (quantities/units) and R2 (numeric predicates) each converged 4/4
  vendors, and two independent quant-finance designs (grok,
  mistral) plus FP&A's independent reach for empirical bounds (grok,
  with FP&A-specific gaps noted by the synthesis) clear the
  two-independent-designs bar the pilot set. The ≥2-independent-domains
  rule cited by the pack proposals is project governance, not yet
  format law in `specs/packs.md` — a follow-up should either add it
  there or strike the citation from this proposal's ancestors.
- The change follows the `add-bioimage-pack` / `add-numeric-predicates-pack`
  scaffold pattern: proposal + design + tasks + dual-format delta gated
  on user approval before the pack artifact lands.
