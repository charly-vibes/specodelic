# Add: first-class domain-pack mechanism

## Why

26 external vendor deliverables (grok, mistral, qwen, zai) evaluated against
v0.4.0 independently converged on the same extension moves: optional new
sections, append-only kind growth, new outbound-leaf reference fields,
per-kind required-case-label floors, and honest-empty advisory checkers.
Today those moves are only a *convention followed by hand* — nothing in the
format names an extension, so independent packs collide in the global closed
kind sets (two vendors claimed `tolerance` on different layers; two mistral
proposals collided on `## Data`).

Synthesis + decision of record:
`openspec/research/2026-10-02-external-review-synthesis/synthesis.md`
(§7–§10) and `.wai/projects/domain-specific-extensions/designs/matrix/decision.md`
(approach 02-profile-intent).

## What changes

Make the extension mechanism **first-class** — a *domain pack*:

1. **Pack artifact (D1 v2):** a pack is a four-layer spec file with
   `kind: profile` whose manifest is six per-facet closed tables —
   `## Sections`, `## Kinds`, `## References`, `## Checkers`, `## Floors`,
   `## Requires` — declaring exactly what the pack introduces.
2. **Core grows once, narrowly (D7 strict thin-core):** `profile` joins
   INTENT_KINDS and `uses` joins Reference Typing as an explicit format
   Revision (Revision 9 of `specs/specodelic.md`). This is the only core
   growth; everything vocabulary-like is pack-fiber-relative (the
   Grothendieck construction one level up, per `specs/theory.md`
   precedent) and the base closed sets stop growing.
3. **Discovery = corpus scan:** every `kind: profile` file in the workspace
   is a candidate pack. No config file, no registry outside the corpus —
   self-hosting preserved.
4. **Opt-in, advisory-first:** checking activates when a file uses the
   pack's declared vocabulary (upgradeable to a declared `uses` edge);
   files without packs lint identically to today; vocabulary used with
   no pack discovered or declared = labeled failure finding naming the
   candidate pack and both remediations (error-contract).
5. **Lifecycle:** pack Model = draft → published → deprecated; the pack
   itself pins the base format_revision and pack deps via `## Requires`
   (consumers declare enablement via `uses`); revision skew between the
   pack's `base` pin and the workspace corpus revision is a labeled
   advisory, never silent.

Governance: standard packs = in-repo, dogfooded, versioned, ≥3-independent-
domain demand rule + Ro5 at proposal time — recorded spec-first in
`specs/packs.md` (created before any checker code, per repo convention).

## Capabilities

- **New capability `packs`** (delta in this change): pack artifact shape,
  manifest tables, discovery, opt-in, namespacing, lifecycle, honest-empty
  checking.

## Out of scope (follow-on changes)

- The three standard packs the pilot needs (data/lineage, numeric
  predicates, empirical kind + floor registry) — separate proposals after
  the mechanism lands.
- `add-bioimage-pack` — the thin domain instance (D6 pilot).
- Any corpus file *becoming* a pack; `specs/` corpus files keep their
  current kinds.

## Alignment

- `append_only_variants`, `prose_untouched`, typed `[[wiki-link]]` foreign
  keys, DAG-ness, `single_root_reachable` (`uses` is an outbound leaf),
  byte-stable artifacts, honest-empty convention — all preserved.
- Zero migration: packs are optional; no existing file changes meaning.
