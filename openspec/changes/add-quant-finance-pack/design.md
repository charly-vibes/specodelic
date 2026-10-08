# Design: add-quant-finance-pack

## Context

The pack mechanism (archived 2026-10-03) makes standard packs thin
instances — the mechanism interprets declared vocabulary, advisory-first.
Quant is the D6 "second" pack: two vendors shipped independent
quant-finance designs and the synthesis's recurring verdict sentence is
quant's. Unlike the bioimage pilot (which exercised cross-pack reference
resolution and a base-table kind column), the quant pack's job is the
opposite stress: **restraint** — most of what vendors asked for already
ships in the standard packs, so this design's main decisions are
rejections (D6) plus one new section and one new floor.

## Goals / Non-Goals

- **Goal:** the quant-finance domain pack as a thin profile file —
  limits vocabulary, risk floor, outbound-leaf edge, cross-pack Requires.
- **Goal:** zero core growth — no base closed set changes, no `src/`
  edits, no checker code.
- **Non-goal:** re-declaring `tolerance` or `statistic` (both live in
  standard packs; see proposal Out-of-scope and D6).
- **Non-goal:** temporal/hybrid modeling, an EARS sixth pattern, unit
  conversion, or executable predicate fragments.

## Decisions

### D1 — Thin vocabulary (one section, two kinds, one field, two checkers, one floor)

| Facet    | Declaration |
|----------|-------------|
| Sections | `## Limits` with row shape `\| name \| kind \| unit \| bound \|` |
| Kinds    | `quant.risk` — *a bounded risk measure (a VaR, expected shortfall, drawdown, or exposure cap stated with its bound)*; `quant.pricing` — *a valuation or pricing invariant (no-arbitrage, put-call parity, or NAV consistency claim)* |
| References | `capped_by` — resolves to a `## Limits` row of the declaring file (outbound leaf, intra-file, the `measured_by` precedent) |
| Checkers | `quant.limit_closed` (every `capped_by` name resolves to a declared `## Limits` row; honest-empty otherwise), `quant.risk_labels` (risk laws enumerate their `**horizon:**` and `**confidence:**` case labels per the pack's floor) |
| Floors   | `quant.risk` requires `horizon` + `confidence` case labels (D4) |
| Requires | `base` pinned at `specodelic.md Revision 18` (current; D5), `numeric.predicates`, `data.lineage` |

Kept minimal deliberately; `append_only_packs` lets later releases add
temporal vocabulary, fixture sections, or executable bound predicates
without breaking this one.

`quant.pricing` is a **law-shaped kind** (parity/no-arbitrage claims
want case-label enumeration, like algebraic laws want
identity/associativity) but gets **no floor in v1**: parity laws owe
their instrument-set enumeration, and no vendor design converged on a
required case set for it. The floor registry is data — a later release
adds a `quant.pricing` row to `## Floors` when the domain enumerates it.

### D2 — Vocabulary hygiene applied up front (the D6 lesson, pre-paid)

Every vocabulary-carrying facet was audited against `grep -rilw` over
`specs/` and `openspec/specs/` (word-boundary, case-insensitive counts).

- `risk` — **0 / 0** (specs / openspec/specs). Safe as a kind
  component; `quant.risk` is dotted and word-boundary-safe.
- `Limits` (capitalized plural) — **0 / 0**. Safe as the section token,
  with the same accepted residual risk the numeric pack's `Quantities`
  and the data pack's `Data` carry: future corpus prose that
  capitalizes the word would falsely activate the pack;
  `append_only_packs` means the token is chosen at its safest form now
  and can never be un-declared later.
- `limit` (bare) — **4 / 1** corpus uses. **Excluded** from every
  vocabulary-carrying facet; appears only as a `## Limits` row column
  header shape (columns are not scanned vocabulary) and in the
  `bound` column semantics.
- `confidence` — **5 / 0** corpus uses. **Excluded** from vocabulary;
  survives only as a floor case label (`**confidence:**`), which the
  activation scanner never reads (floors are not in `vocabulary()` —
  the mechanism scans kinds + sections + references only).
- `horizon` — **0 / 0**. Safe; also used only as a floor case label.
- `Fixtures` / `fixture` — **4 / 4** corpus uses. **Rejected as a
  section token**; the vendor demand (bounded backtest fixtures with
  provenance) is satisfied by the data-lineage pack's `## Data` rows —
  see D6.
- `capped_by` — **0 / 0**. Verified absent; `cap` reads correctly as a
  directed edge (the capped row points at its limit row).
- `quant`, `finance`, `backtest`, `pricing` — **0 / 0** each. The
  namespace and the pack id (`quant.finance` ↔ `packs/quant-finance.md`,
  per the file-naming law) are prose-safe.
- `quant.risk`, `quant.pricing` (dotted) — **0 / 0**. Dotted kinds are
  word-boundary-safe by construction.

### D3 — Limit rows resolve intra-file, outbound-leaf

`capped_by` resolves to a `## Limits` row of the declaring file —
file-local checking (advisory-first, no corpus-wide pass), no
reachability join, no acyclic edge set (the `measured_by` precedent).
A dangling reference is a labeled finding naming the row id and both
remediations, never a generic dangling message. With no `## Limits`
rows in a workspace the checked-set is empty and reported honestly.

### D4 — The risk floor is grok-quant's tolerance, reduced to case labels

Grok-quant's `tolerance` Constraint kind carried `{metric, abs, rel,
unit}`. Under namespacing (E1/E2) a second tolerance kind is the
collision the mechanism exists to prevent — the numeric pack already
declares `numeric.tolerance` with its `bound, against` floor. What is
*quant-specific* in grok's design and absent from every standard pack:
a risk bound owes its **measurement horizon** and **confidence level**.
The floor reuses the machine-findable `**name:**` case-label
enumeration mechanism with a different required label set — exactly
mistral's reuse phrasing (R4) that the numeric and bioimage packs made
concrete. Floor cases: `horizon, confidence`.

### D5 — Requires pins base Revision 18 + two standard packs

The existing standard packs pin `Revision 14` (the mechanism revision at
their authoring). This pack is authored at Revision 18 (current: 18,
single-tree id derivation per v0.7.0) and pins it honestly. Revisions
15–18 introduced no pack-mechanism narrowing — `append_only_variants`
guarantees the 14-pinned standard packs still check — and revision skew
is a labeled advisory (`skew_advisory`), never a failure.

Pack deps: `numeric.predicates` (a `## Limits` row bounds a quantity;
`quant.risk` laws that need tolerance semantics declare
`numeric.tolerance` rows) and `data.lineage` (backtest fixtures and
datasets are `## Data` rows). The empirical-registry pack is a natural
companion (statistical claims on backtests → `empirical.statistic`) but
is **not required**: a quant spec file without statistical claims should
not pull the registry in. Workspaces wanting the full quant stack
declare `uses` edges to all four packs.

### D6 — Rejections recorded (the restraint decisions)

| Vendor ask | Source | Resolution |
|------------|--------|------------|
| Second `tolerance` kind `{metric, abs, rel, unit}` | grok-quant | Reuse `numeric.tolerance` (E1/E2 collision; namespacing makes re-declaration legal but pointless) |
| `statistic` kind with `stat_test` set | mistral-quant | Reuse `empirical.statistic` + `tested_by` (already shipped) |
| `## Fixtures` section | mistral-quant | Token corpus-contaminated (D2); satisfied by `## Data` rows |
| quantity/unit/limit fields on constraint rows | mistral-quant | `## Quantities` (numeric pack) + `## Limits` (this pack) carry them as typed rows |
| Workspace `units.md` unit table as extension point | mistral-quant | Unit systems stay opaque prose (the `unit` column); bridge, never absorb |
| Temporal guards, `### Continuous` | mistral-quant, grok-quant | No time pack exists; deferred to a time pack or format Revision |
| Sixth EARS pattern `WITHIN <ε> [<unit>]` | mistral (matrix E4) | Core candidate, rides a format Revision — not pack work |

### D7 — Spec delta shape (dual-format, mirrors the pack trio)

`specs/quant-finance-pack/spec.md` in the change delta: Purpose +
Constraints + Model (draft/published/deprecated) + Properties +
`## ADDED Requirements` (five requirements, one or more scenarios each)
+ mirrored `## Requirements` (verbatim) — the `add-bioimage-pack`
scaffold shape, gated by `just sync-sections`.

## Risks / Trade-offs

- **Restraint risk inverted:** the pack is thin enough that a reviewer
  may ask "why ship it at all?" — answer: the `## Limits` section and
  the risk floor are real new checking surface grok/mistral both
  demanded, and `append_only_packs` gives later releases room to grow
  without a breaking change.
- **`Limits` token residual risk** (D2): accepted, same trade as
  `Quantities`/`Data`.
- **Revision 18 pin vs. the 14-pinned packs:** skew is advisory by law;
  no action needed until a pack-side change forces re-pinning.

## Migration Plan

None needed: packs are optional, files without packs lint identically
(`no_pack_no_change` property of the mechanism). The pack artifact lands
in `packs/` in phase 2 after approval; discovery is corpus scan.

## Open Questions

- None blocking. The `quant.pricing` floor (D1) is deliberately deferred
  to a later release under `append_only_packs`.

## Review outcome

Ro5 review pending — run at proposal time per the pack-change pattern
(bioimage tasks 1.4); findings folded into D1–D7 before approval.
