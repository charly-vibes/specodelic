# Proposal: add-numeric-predicates-pack

## Why

R1+R2+R4 of the external-review synthesis are the numeric half of the
vendor convergence: every one of the four independent domains invented a
quantity layer (R1: `## Quantities` tables, unit/limit fields, UCUM-ish
units), real-valued predicates that cannot currently gate a transition
(R2: "the strongest finance/science constraints are exactly the ones
prose_untouched makes uncheckable"), and tolerance/statistical property
kinds reusing the `**name:**` case-label machinery with a different
required label set (R4: mistral's reuse phrasing — tolerance laws owe
`**bound:**` + `**against:**`, not identity/associativity). Demand is
≥3 independent domains (mistral ×2, grok, qwen, zai) — exactly the
governance rule in `specs/packs.md`. The mechanism
(`add-domain-pack-mechanism`, archived 2026-10-03) made packs
first-class; data/lineage (archived 2026-10-03) was the first standard
pack; this is the second, and together with the empirical-registry pack
it completes what the bioimage D6 pilot consumes.

## What Changes

- **New capability `numeric-predicates-pack`** (delta in this change):
  the contract for the numeric-predicates standard pack — a declared
  `kind: profile` pack artifact, a namespaced `## Quantities` section,
  the `numeric.quantity` / `numeric.bound` / `numeric.tolerance` kinds,
  `measured_by` as an outbound-leaf reference field, the
  `numeric.quantity_closed` and `numeric.tolerance_labels` checker
  declarations, the tolerance case-label floor, and a
  `specodelic.md Revision 14` base pin.
- **New in-repo pack artifact** `packs/numeric-predicates.md` (id
  `numeric.predicates`, per the file-naming law): six manifest tables
  declaring exactly the vocabulary above. No checker code — pack
  checkers ship as declarations per `specs/packs.md` (honest-empty
  convention); nothing in `src/` changes.

## Capabilities

- **New capability `numeric-predicates-pack`**: quantity section
  declaration, namespaced numeric kinds, tolerance case-label floor,
  measured_by as an outbound leaf, checkers declared honest-empty.

## Out of scope

- The empirical kind + floor registry standard pack (specodelic-aal) —
  separate proposal; the tolerance floor declared here is
  pack-fiber-relative, not the general registry.
- The R2 data-resolved predicate *grammar* (`dtype_is`, `shape_eq`,
  `same_shape_as`) — that is the bioimage pilot's data-shaped half;
  this pack ships only the domain-neutral numeric half (`within`-class
  bounds declared as vocabulary the mechanism can name, not execute).
- Any `src/` change: no new lint rules, no checker implementations.
- Unit systems (UCUM, ISO4217, microscopy units) in any closed set —
  the `unit` column stays opaque prose the format never parses, same
  bridge-never-absorb rule the data pack applies to `binding`.

## Alignment

- `append_only_packs` applies to the pack itself: releases may only add
  vocabulary, never remove or narrow.
- `prose_untouched` holds: the pack declares vocabulary, never prose
  rules. The vocabulary-hygiene lesson from `add-data-lineage-pack`
  design D6 is applied up front: **no bare-English tokens are declared**
  — `within`, `bound`, `against`, `unit`, `domain` all appear in corpus
  prose and are excluded from every vocabulary-carrying facet; kinds are
  dotted (`numeric.*`), the section name (`Quantities`) and the
  reference field (`measured_by`) are verified absent from the corpus.
- `single_root_reachable` needs no carve-out: `measured_by` is an
  outbound leaf (the `uses`/`observes` precedent).
- Base closed sets frozen: `numeric.*` kinds are pack-fiber-relative;
  `INTENT_KINDS` and Reference Typing are identical with or without the
  pack discovered.
- Honest-empty: with no `## Quantities` rows in a workspace, the pack's
  checkers report empty checked-sets, never fabricated findings.

## Governance

- Demand rule: ≥3 independent domains (R1: mistral ×2, grok, qwen, zai;
  R2: all four; R4: mistral ×2, grok, zai) per `specs/packs.md`.
- Ro5 review at proposal time before implementation (recorded in
  design.md "Review outcome").
- Spec-first: the capability delta states the pack contract; the pack
  artifact implements it; `specs/packs.md` governs the mechanism.