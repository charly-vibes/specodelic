# Proposal: add-empirical-registry-pack

## Why

R4 of the external-review synthesis is the statistical half of the
vendor convergence: every domain that measured anything wanted an
empirically held property kind (mistral ×2 `statistic` with
`stat_test ∈ {stationarity, mean_eq, ci_within, distribution}`, grok
empirical bounds with scaling generators, zai statistical properties)
and, critically, mistral-phrased the reuse of the case-label machinery:
*same machine-findable `**name:**` enumeration, a different required
label set* — the base `law` floor (identity/associativity) is not
domain-universal, per-kind floors should be kind-dependent while the
enumeration *form* stays unchanged. Demand is ≥3 independent domains
(mistral ×2, grok, zai) — exactly the governance rule in
`specs/packs.md`. The mechanism (`add-domain-pack-mechanism`, archived
2026-10-03) made packs first-class; data/lineage and numeric-predicates
(archived 2026-10-03) are the first two standard packs; this is the
third — the **Floors facet's first real consumer** — and together with
them it completes what the bioimage D6 pilot consumes.

## What Changes

- **New capability `empirical-registry-pack`** (delta in this change):
  the contract for the empirical-registry standard pack — a declared
  `kind: profile` pack artifact, a namespaced `## StatTests` section,
  the `empirical.statistic` property kind, `tested_by` as an
  outbound-leaf reference field, the `empirical.statistic_labels` and
  `empirical.stat_test_closed` checker declarations, the per-kind
  case-label floor (`empirical.statistic` requires `alpha` + `window`),
  and a `specodelic.md Revision 14` base pin.
- **New in-repo pack artifact plan** `packs/empirical-registry.md` (id
  `empirical.registry`, per the file-naming law): six manifest tables
  declaring exactly the vocabulary above. No checker code — pack
  checkers ship as declarations per `specs/packs.md` (honest-empty
  convention); nothing in `src/` changes.

## Capabilities

- **New capability `empirical-registry-pack`**: stat-test registry
  section declaration, the namespaced `empirical.statistic` kind,
  per-kind case-label floor registry (the Floors facet's first
  consumer), `tested_by` as an outbound leaf, checkers declared
  honest-empty.

## Out of scope

- The bioimage D6 pilot pack (specodelic-0dn) — it consumes this pack's
  vocabulary via `uses` edges; its domain invariants are its own.
- The R6 external-artifact binding layer (`validated_against`, benchmark
  manifests with checksums, fail-closed verify) — reference datasets
  stay outside this pack; the `## StatTests` rows carry only
  metric/alpha/window. A later release may add a binding reference field
  (`append_only_packs` permits additions, never removals).
- grok's FP&A "empirical bounds with scaling generators" as a second
  kind — `numeric.bound`/`numeric.tolerance` (the numeric pack) already
  carry stated and empirically held limits; a parallel bound kind would
  be redundant cross-pack vocabulary. Domain packs add their own kinds.
- Any `src/` change in THIS change: no new lint rules, no checker
  implementations. The one mechanism capability this pack's contract
  needs — base-table kind columns accepting active-pack fiber kinds
  (design D8; `property_kind_closed` today walks the static base set) —
  is filed as its own follow-up ticket, not folded in here.
- The base `law` floor: per-kind floors are purely additive — laws keep
  identity/associativity; nothing narrows.

## Alignment

- `append_only_packs` applies to the pack itself: releases may only add
  vocabulary, never remove or narrow.
- `prose_untouched` holds: the pack declares vocabulary, never prose
  rules. The vocabulary-hygiene lesson from `add-data-lineage-pack`
  design D6, pre-paid by the numeric pack, is applied up front again:
  **no bare-English tokens are declared** — `window` appears in corpus
  prose (the rate-limit window in USAGE §2.8-adjacent state machines)
  and is excluded from every vocabulary-carrying facet; it survives only
  as a floor case label and a table column, neither of which the
  activation scanner reads (`vocabulary()` = kinds + sections +
  references). The kind is dotted (`empirical.statistic`); the section
  name (`StatTests`) and the reference field (`tested_by`) are verified
  absent from the corpus.
- `single_root_reachable` needs no carve-out: `tested_by` is an outbound
  leaf (the `uses`/`observes`/`measured_by` precedent).
- Base closed sets frozen: `empirical.statistic` is pack-fiber-relative
  (the `kind` overload CLAR-002 dissolves at the fiber); `INTENT_KINDS`
  and Reference Typing are identical with or without the pack
  discovered.
- Honest-empty: with no `## StatTests` rows in a workspace, the pack's
  checkers report empty checked-sets, never fabricated findings.
- Contract depth honesty: like both sibling packs, the delta states
  end-state semantics (kind columns, dangling references, floors) whose
  mechanism interpretation is currently declaration-only — never
  silently passing; the kind-column follow-up is filed, not waved at.

## Governance

- Demand rule: ≥3 independent domains (R4: mistral ×2, grok, zai) per
  `specs/packs.md`.
- Ro5 review at proposal time before implementation (recorded in
  design.md "Review outcome").
- Spec-first: the capability delta states the pack contract; the pack
  artifact implements it; `specs/packs.md` governs the mechanism.
