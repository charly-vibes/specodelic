---
reviews: 2026-11-13-ro5u-findings-specodelic-04c-kernel-vocab-pack.md
verdict: pass
severity: {critical: 0, high: 0, medium: 2, low: 1}
tags: [pipeline-run:tdd-ro5-2026-11-13-specodelic-04c-kernel-vocabulary-pack, pipeline-step:ro5u-review]
---

# RO5U review — specodelic-04c kernel vocabulary pack

Diff under review: `packs/kernel-pack.md` (new), `tests/kernel_pack_vocabulary.rs` (new),
`README.md`, `docs/src/index.md`, `docs/src/faq.md`, `docs/src/commands.md`, `llms.txt`,
`llm.txt` (pack-inventory prose).

## Stage 1 — DRAFT (shape, architecture, scope)

The change ships exactly one artifact — a `kind: profile` pack file in the `kernel`
namespace — plus its tests and the doc inventory the pack-inventory guard demands.
No `src/**` change, no linter carve-out, no config, no registry. The pack's shape is
section-for-section identical to the six sibling packs
(`packs/numeric-predicates.md` vs `packs/kernel-pack.md`: same frontmatter keys, same
`## Constraints` / `## Model` / `## Properties` / six-facet-manifest section order,
same two-column facet tables) — evidence: `grep -n '^#\|^##' packs/*.md`.
Scope is tight: the pack declares vocabulary; the mechanism interprets it.

Stage 1 findings: 0 critical, 0 high, 1 medium (M1 below), 1 low (L1).

## Stage 2 — CORRECTNESS (errors, logical flaws)

- **Enablement path is corpus discovery, not a carve-out.** `spk lint openspec/specs`
  reports zero issues and lists `kernel.pack` with vocabulary
  `[kernel.predicate, KernelPredicates, kernel.binding]` and `base_pin` 20 — the pack
  is found by scan alone (`src/packs.rs` scans Sections/Kinds/References).
- **`kernel.binding` is declared in the facet the rule scans.** It sits in the
  `## References` table; the lint envelope's `vocabulary` list for the pack contains
  `kernel.binding` (evidence: the corpus lint JSON `data.packs[].vocabulary`).
- **The pack artifact itself is clean.** `spk lint packs/kernel-pack.md` → 0 issues,
  exit 0 — so `pack_shape` (six non-empty two-column facets, pack-qualified `## Kinds`
  row, legal `identity` floor label), `coverage` (every constraint has a deriving
  property), `guard_required`, `total_refs`, `acyclic` and `single_root_reachable`
  all hold.
- **Base pin is current.** `specodelic.md Revision 20` equals the embedded revision
  (`spk doctor --json` → `format_revision: specodelic.md Revision 20`), so no
  `skew_advisory` fires.
- **The four layers are wired.** 4 constraints ↔ 4 properties; each property's
  `derives_from` resolves to its constraint row; each transition guard cites
  `[[kernel.pack.vocab_declared]]`.
- **Files that use no pack vocabulary are untouched.** Test
  `file_without_the_binding_column_is_untouched_by_the_kernel_pack` asserts zero
  findings and no pack activation for such a file.

Stage 2 findings: 0 new critical, 0 new high.

## Stage 3 — CLARITY (comprehensibility)

- The pack's prose states the two laws the owner named (the binding stays opaque, the
  predicates stay pack-side) and says what the mechanism does with the file.
- The header comment of the test module names the purpose, responsibilities and
  rationale, and states the landing site (`packs/kernel-pack.md`).
- The doc inventory prose reads naturally in all five surfaces; the counts moved
  6 → 7 total and 5 → 6 standard, matching `packs/` (seven files, one pilot).

Stage 3 findings: 0 new.

## Stage 4 — EDGE CASES (boundary conditions)

- **Pack absent in a consumer workspace** → `orphan_vocabulary` still labels the file
  and names both remediations (pre-existing behavior, unchanged by this diff).
- **Vocabulary used in a row cell, not a column header** → not a structured field
  position; the fixture's prose cell (`` `-k kernel_binding` ``) stays inert.
- **Pack present, vocabulary unused** → no activation, no finding (tested).
- **Corpus lint without the pack** → the RED evidence captured the labeled failure
  before the artifact existed.

Stage 4 findings: 0 new.

## Stage 5 — EXCELLENCE (final polish)

Only cosmetic residue remains (L1).

### Findings

**Critical:** none.

**High:** none.

**Medium**

- **M1 — the pack has no companion corpus capability spec.**
  Location: `packs/kernel-pack.md` (whole artifact).
  Evidence: the six sibling packs each ship a capability spec
  (`openspec/specs/{data-lineage,numeric-predicates,empirical-registry,quant-finance,svc-contracts,bioimage-data}-pack/spec.md`);
  the kernel pack ships none, so it has no rendered book page and no scenario-scoped
  contract TOML.
  Why not fixed here: the corpus tree holds capability specs only — the reference
  oracle requires every `openspec/specs/*/spec.md` to be `kind: intent` and fired
  `oracle.frontmatter_kind` on the profile pack; and the kernel capability itself is
  already documented by `openspec/specs/kernel/spec.md`.
  Concrete fix: a follow-up ticket adding `openspec/specs/kernel-pack/spec.md`
  (`kind: intent`, the capability companion) with its `.espectacular/` contracts.

- **M2 — `spk doctor` does not surface the pack.**
  Location: `src/doctor.rs::discover_workspace_packs` (read-only observation; no
  change made — `src/**` is out of scope for this ticket).
  Evidence: `spk doctor --json` output has no `packs` key; the doctor scans only the
  corpus root's top-level `*.md` and `specs/`, so `packs/*.md` (all seven packs,
  including the six pre-existing ones) are invisible to it. The pack is discoverable
  through the lint envelope's `data.packs` list and through `spk lint openspec/specs`.
  Concrete fix: a follow-up ticket widening the doctor's scan to `packs/`.

**Low**

- **L1 — uneven table padding.** Location: `packs/kernel-pack.md`, `## Properties`,
  the `predicates_stay_external` row (pipe columns not aligned with the header).
  Evidence: visual inspection of the table.
  Concrete fix: realign the row's pipes.

## Final report

Verdict: **pass** — 0 critical, 0 high, 2 medium (both deferred with reasons and
concrete follow-ups), 1 low (cosmetic, to fix in the fix-review step).

Convergence check after Stage 4: new CRITICAL 0, total new issues 3 vs 3 at Stage 1,
new-issue rate 0%, estimated false-positive rate 0% → **CONVERGED**.

Stage 5 is polish only; L1 is fixed in the `fix-review` step.

