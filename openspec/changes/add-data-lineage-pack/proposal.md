# Proposal: add-data-lineage-pack

## Why

External-review demand R3 (data/lineage layer) was the highest-frequency
cross-domain request after the pack mechanism itself: a typed `## Data`
table plus lineage edges (`produced_by`/`consumed_by`/`produces`) with an
outbound-leaf external binding. It was demanded by ≥3 independent domains —
bioimaging (mistral-bioimage, grok-bioimage: OME/NGFF-bound datasets),
quant/ML pipelines (mistral-quant, qwen: benchmark manifests, statistical
inputs), and data engineering/ETL (qwen, zai: pipeline lineage) — which is
exactly the ≥3-independent-domain rule recorded in `specs/packs.md`
governance. The mechanism (`add-domain-pack-mechanism`, archived
2026-10-03) made packs first-class; this change ships the first standard
pack so the mechanism has a real instance before the D6 bioimage pilot.

## What Changes

- **New capability `data-lineage-pack`** (delta in this change): the
  contract for the data/lineage standard pack — a declared `kind: profile`
  pack artifact, the namespaced `## Data` vocabulary, typed lineage
  reference fields, and the bridge-never-absorb external binding rule.
- **New in-repo pack artifact** `packs/data-lineage.md` (id `data.lineage`,
  per the file-naming law): `kind: profile`, six manifest tables declaring
  the `## Data` section row shape, `data.dataset` / `data.artifact` /
  `data.environment` kinds, `produces` / `produced_by` / `consumed_by`
  outbound-leaf reference fields, the `data.lineage.closure` checker
  declaration (honest-empty), the `binding` floor, and a
  `specodelic.md Revision 14` base pin.
- **No checker code.** Pack checkers ship as declarations per
  `specs/packs.md` (honest-empty convention): the mechanism activates them
  advisory-first; nothing in `src/` changes.

## Capabilities

- **New capability `data-lineage-pack`**: pack artifact declaration,
  namespaced data vocabulary, lineage edges as outbound leaves, external
  binding stays external.

## Out of scope

- The other standard packs: `numeric predicates` (specodelic-0fb),
  `empirical kind + floor registry` (specodelic-aal) — separate proposals.
- `add-bioimage-pack` (specodelic-0dn, D6 pilot) — depends on this pack.
- Any `src/` change: no new lint rules, no checker implementations — the
  mechanism already executes declared pack checkers advisory-first.
- Absorbing external data standards (OME/NGFF, Parquet, BioImage.IO) into
  any closed set — the pack bridges them, it never owns them.

## Alignment

- `append_only_variants` applies to the pack itself: releases may only add
  vocabulary, never remove or narrow.
- `prose_untouched` holds: the pack adds vocabulary and declarations, not
  prose rules.
- `single_root_reachable` needs no carve-out: lineage reference fields are
  outbound leaves (the `uses` precedent from the mechanism change).
- Base closed sets frozen: `data.*` kinds are pack-fiber-relative;
  `INTENT_KINDS` and Reference Typing are identical with or without the
  pack discovered.
- Honest-empty: with no `## Data` rows in a workspace, the pack's checker
  reports an empty checked-set, never a fabricated finding.

## Governance

- Demand rule: ≥3 independent domains (bioimaging, quant/ML, data
  engineering) per `openspec/research/2026-10-02-external-review-synthesis/synthesis.md` R3.
- Ro5 review at proposal time: verdict READY WITH_NOTES, fix pass applied
  before implementation (see design.md "Review outcome").
- Spec-first: the capability delta (this change) states the pack contract;
  the pack artifact implements it; `specs/packs.md` governs the mechanism.