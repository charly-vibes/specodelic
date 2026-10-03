# Proposal: add-bioimage-pack

## Why

The decision of record (2026-10-02, synthesis §10 D6) named the bioimage
pack the D6 pilot: the most mechanically complete vendor design, and the
instance that exercises the pack mechanism end-to-end — it is the first
pack whose vocabulary *resolves against another pack's section* and the
first whose fiber kind naturally wants a base-table kind column. R2
converged on a smallest-credible closed predicate grammar (mistral
bioimage: `dtype_is`, `shape_eq`, `same_shape_as`, `within`,
`units_convertible` resolving against a `## Data` table — nobody proposed
an open DSL), and grok-bioimage held the counter-position that governs
the shape: OME/NGFF and BioImage.IO stay authoritative external
standards — bridge, don't absorb. The three standard packs the pilot
consumes (data/lineage, numeric predicates, empirical registry — all
archived 2026-10-03) are exactly what makes this pack *thin*: its
data-shaped half (`same_shape_as`, the `dtype_is`/`shape_eq` class
resolving against `## Data`) was explicitly deferred here by the numeric
pack's design, and its empirical half rides the empirical registry's
stat-test rows. This is the stress test the mechanism was built for.

## What Changes

- **New capability `bioimage-data-pack`** (delta in this change): the
  contract for the bioimage-data domain pack (`packs/bioimage-data.md`,
  id `bioimage.data`) — a declared `kind: profile` pack artifact with a
  namespaced `## Axes` section (axis semantics, scale/unit columns
  opaque), the `bioimage.transform` property kind, `same_shape_as` as a
  reference field resolving to a `## Data` row of the declaring file
  (the data-lineage pack's section — the first cross-pack reference
  resolution), the R2 data-shaped predicate set declared as named
  pack-qualified checkers (`bioimage.dtype_is`, `bioimage.shape_eq`,
  `bioimage.units_convertible` — declaration-only, honest-empty), a
  kind-dependent case-label floor (`preserves` + `dtype` — the
  domain-universality critique of the base `law` floor made concrete),
  and a `## Requires` table pinning base `specodelic.md Revision 14`
  plus the three standard packs — the first cross-pack `## Requires`
  consumer.
- **New in-repo pack artifact plan** `packs/bioimage-data.md` (id
  `bioimage.data`, per the file-naming law): six manifest tables
  declaring exactly the vocabulary above. No checker code — pack
  checkers ship as declarations per `specs/packs.md` (honest-empty
  convention); nothing in `src/` changes.

## Capabilities

- **New capability `bioimage-data-pack`**: axis-semantics section
  declaration, the namespaced `bioimage.transform` kind, cross-pack
  reference resolution (`same_shape_as` → `## Data`), the R2 closed
  predicate grammar as named checkers, a kind-dependent transform floor,
  and pack dependencies on the three standard packs.

## Out of scope

- The R2 *executable* predicate-fragment track (`specodelic-rjb`) — the
  predicates here are declared and nameable, never executed; compilation
  into `invariants_checked` is that ticket's mechanism work.
- The R6 external-artifact binding layer (`validated_against`, benchmark
  manifests with checksums, fail-closed verify) — same deferral as the
  empirical registry's; OME/NGFF URIs stay opaque `binding` pointers.
- `within` — prose-toxic (13 corpus files); excluded from every
  vocabulary-carrying facet exactly as the numeric pack ruled.
- A `bioimage.pipeline` kind — `pipeline` is prose-heavy (9 corpus
  files) and mistral's handoff unification is mechanism work; a later
  append-only release may add it.
- Any `src/` change in THIS change: no new lint rules, no checker
  implementations, no base closed set growth. The mechanism capabilities
  the pilot consumes — cross-pack reference resolution and kind-column
  fiber acceptance — are already landed (Revision 14's `uses` typing row
  and beads `specodelic-ung`); the pilot *exercises* them, it does not
  extend them.

## Alignment

- `append_only_packs` applies to the pack itself: releases may only add
  vocabulary, never remove or narrow.
- `prose_untouched` holds: the pack declares vocabulary, never prose
  rules. The hygiene lesson pre-paid by both sibling packs is applied up
  front a third time: **no bare-English vocabulary tokens are declared**
  — `bioimage` appears in corpus prose (USAGE §packs, STATUS, CHANGELOG)
  but is documented asymmetry (dotted tokens cannot match prose words;
  `contains_word` verified in the empirical pack's D2); `pipeline` is
  prose-heavy and rejected outright; the predicate names
  (`dtype_is`, `shape_eq`, `units_convertible`) ride pack-qualified
  checker rule names so the bare tokens are never declared; `Axes`,
  `same_shape_as`, `preserves` are verified absent from the corpus;
  `dtype` (1 file — the data pack's own `## Data` column) appears only
  as an opaque column header and a floor case label, never as activation
  vocabulary.
- `single_root_reachable` needs no carve-out: `same_shape_as` is an
  outbound leaf (the `measured_by`/`tested_by` precedent), and its
  cross-pack target is the declaring file's own `## Data` row — the
  *pack* is cross-pack, the *resolution* stays intra-file (advisory-
  first, file-local checking, no corpus-wide pass, no acyclic edge set).
- Base closed sets frozen: `bioimage.transform` is pack-fiber-relative;
  `INTENT_KINDS` and Reference Typing are identical with or without the
  pack discovered — and the ung mechanism makes the fiber kind *typeable*
  in a base Properties row's `kind` column when the pack is active,
  never silently passing when it is not.
- Honest-empty: with no `## Data` or `## Axes` rows in a workspace, the
  pack's checkers report empty checked-sets, never fabricated findings.
- Bridge-never-absorb, held twice: OME/NGFF axes stay authoritative —
  the `## Axes` table names the workspace's axis vocabulary with opaque
  `scale`/`unit` columns; unit systems stay the numeric pack's opaque
  `unit` domain, which `units_convertible` bridges without parsing.

## Governance

- Decision of record: synthesis §10 D6 (pilot = bioimage-data); demand
  R2/R3 (mistral-bioimage, grok-bioimage) per `specs/packs.md`.
- Ro5 review at proposal time before implementation (recorded in
  design.md "Review outcome").
- Spec-first: the capability delta states the pack contract; the pack
  artifact implements it; `specs/packs.md` governs the mechanism.
