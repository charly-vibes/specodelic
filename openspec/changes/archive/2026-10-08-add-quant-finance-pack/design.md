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

Ro5 proposal-time review completed 2026-10-09 (rule-of-5-universal, TypeSafe-
verified on the HIGH finding; converged at Stage 4, verdict NEEDS_REVISION →
fixes applied):

- CORR-003 (HIGH, REVIEW_REQUIRED): the Governance bullet cited "the demand
  rule in `specs/packs.md` (≥2 independent domains)" — no such rule exists in
  that file (or anywhere in `specs/`, `openspec/specs/`, or the matrix); the
  phantom citation was inherited from the numeric-predicates proposal. Fix:
  Governance re-grounded on the decision matrix's Demand column, with the
  phantom-rule status named and a follow-up suggested (add the rule or strike
  the citation from ancestor proposals).
- CORR-001 (MEDIUM): the synthesis verdict-sentence quote dropped
  "/science"; restored verbatim.
- CORR-002 (MEDIUM): the risk floor was attributed to grok-quant's
  {metric, abs, rel, unit} tolerance; `horizon`/`confidence` derive from the
  R4 per-kind floor precedent (design D4), not grok's fields. Fix: proposal
  wording now credits the derivation.
- DRAFT-001 / EDGE-001 / CLAR-002 / EXCL-001 (MEDIUM/LOW): positive-case
  sentence added to Why; Revision 18 pin rationale clause added; empirical
  registry named as optional companion in Capabilities; FP&A claim softened.
- CLAR-001 / EDGE-002 (LOW): source-gloss added to the temporal bullet;
  `quant.pricing`-on-Limits motivation left to design D1 (reviewer judged it
  coherent as stated; the delta's section-kind closure is unchanged).

## Probe outcome

Six dogfood probes run 2026-10-10 (specodelic-9h3z) against the landed
pack (a407ff0), each RED (pack toggled OUT of a scratch git worktree at
`/var/tmp/qfp-9h3z/main`, fixtures in `probes/` there — never in the
repo tree) then GREEN (pack present). Binary:
`target/release/specodelic` at 827854a; JSON envelopes preserved under
`/var/tmp/qfp-9h3z/evidence/`. Verdict: **all six probes pass**; two
deviations from the brief's expected wording recorded honestly (c,
dual-format exception), neither a delta-scenario failure.

**(a) Cross-pack `capped_by`, intra-file (D3) — PASS.** Positive
fixture: a Constraint row whose `capped_by` cell targets a declared
`## Limits` row of the same file lints exit 0 (RED: silent, no false
activation — `capped_by`/`Limits` are undotted tokens, the
structured-position orphan signal correctly does not fire; GREEN: adds
`pack \`quant.finance\` activated for \`a_ok\` (vocabulary match:
Limits, capped_by) — empty checked-set (no violations to report)`).
Dangling fixture (`[[a_dangling.missing_row]]`, a `## Limits` row id no
row declares): labeled `linter.total_refs` — `dangling reference
\`[[a_dangling.missing_row]]\` from constraints.capped_by — target not
defined in any spec file` — names the row id and the typed column,
exit 1, both with and without the pack (the resolution rule is base;
the pack declares `quant.limit_closed` as its closure checker).
`spk graph` over the positive fixture: 0 dangling, 0 violations, and
**no `capped_by` edge in the projection** — the reference joins no
reachability path and no acyclic edge set; no `single_root_reachable`
carve-out needed (outbound leaf, intra-file, D3 verbatim). Recorded
verbatim: the labeled dangling finding is the base `total_refs` rule —
it names the row id and column but not the orphan rule's
"both-remediations" wording; the closure checker is declaration-only
(the pack's own Checkers row: "the mechanism names this rule, never
executes it"), the bioimage pilot's same precedent.

**(b) Kind column (the ung mechanism) — PASS.** RED (pack removed):
`linter.property_kind_closed` ×2 (`property \`b_p1\` has kind
\`quant.risk\` — outside the closed set {unit, law}`;
`quant.pricing` likewise) plus `linter.orphan_vocabulary`
(prefix-derived: `candidate pack \`quant.*\``) — exit 1. GREEN: all
base findings gone — the fiber kinds `quant.risk`/`quant.pricing`
extend the property closed set when the file activates the pack —
leaving only the activation advisory on the warnings channel, exit 0.

**(c) `## Requires` dep row naming an absent pack — OBSERVED,
declaration-only.** A well-formed scratch pack (`probe.scratch`,
vocabulary disjoint from quant) whose `## Requires` names
`quant.finance`: RED (pack absent) and GREEN (pack present) both lint
exit 0 with zero findings; a dep naming a pack absent from any
workspace (`time.models`) likewise fires nothing. The mechanism parses
`## Requires` rows (pack_shape two-column shape) but resolves only the
`base` pin (skew advisory) — pack deps are declaration-only, exactly
the bioimage pilot's D8 fix-pass note ("a dep naming no discovered pack
fires nothing; recorded, not waved at"). Deviation from the brief's
"labeled advisory" expectation recorded; a labeled unresolvable-dep
advisory is unimplemented mechanism surface, filed for the orchestrator
to route (no `src/` edits in this change).

**(d) Risk floor (D4) — PASS (additivity byte-identical; floor is
declaration-only).** Additivity: a base `law` property missing its
floor cases fires `linter.law_cases` (`missing floor case(s)
["identity", "associativity"]`) BYTE-IDENTICAL with the pack present
vs removed (diff empty). Ride-along labels: a `law` property
enumerating identity + associativity plus `**horizon:**`,
`**confidence:**`, `**bound:**`, `**against:**` lints clean with the
pack present — extra named cases are checkable declarations, labels
from consumed packs (numeric.predicates, and quant's own floor labels)
ride along. Risk floor: a `quant.risk` property WITH
`**horizon:**`/`**confidence:**` lints clean + activation advisory
(exit 0); WITHOUT them, no mechanical floor finding fires — the pack
floor is declaration-only per the pack's `quant.risk_labels` row ("the
mechanism names this rule, never executes it"); the floor "names"
`horizon, confidence` via the pack's `## Floors` row, discovered in
the envelope. The delta scenario holds at the declaration level; a
mechanically-executed pack floor is follow-on mechanism surface (the
pack's own statement scopes this).

**(e) Corpus safety (`no_pack_no_change`, now with all five packs) —
PASS.** Worktree with all FIVE standard packs + scratch pack
discovered: `lint openspec` → 0 issues, 9 warnings, exit 0; `lint
specs` → 0 issues, 40 warnings, exit 0. Pack toggled out: issues AND
warnings byte-identical on both corpora (diff empty; the `packs`
envelope field differs only by the removed pack's entry). Real repo
confirmed (all five packs): 0 issues, exit 0 both corpora. Honest-empty
closure: no `## Limits` rows exist in this repo; every activation
advisory reads `— empty checked-set (no violations to report)` —
`quant.limit_closed`'s empty checked-set, no fabricated findings.
Dual-format exception: this change's delta
(`openspec/changes/add-quant-finance-pack/specs/quant-finance-pack/spec.md`)
is frontmatter-less and is skipped by the linter (`skipped (no
frontmatter — not a spec file)`), so **no pre-archive activation
exception exists for this change** — the corpus diff needed no carve-
out at this stage. The exception's live shape is captured on the
archived bioimage capability spec (`openspec/specs/bioimage-data-pack/
spec.md`, frontmatter `id: bioimage.data.pack`):
`pack \`bioimage.data\` activated for \`bioimage.data.pack\`
(vocabulary match: bioimage.transform, Axes, same_shape_as) — empty
checked-set (no violations to report)` — warnings channel only, exit
0 (bioimage D7 verbatim). At archive, the quant capability spec
(frontmatter `id: quant.finance.pack`) will vocabulary-activate
`quant.finance` the same way.

**(f) Orphan vocabulary — PASS.** RED, declared
`uses: [[quant.finance]]` edge, pack absent: labeled
`linter.orphan_vocabulary` — `orphan vocabulary: \`uses\` edge targets
\`quant.finance\`, but no \`kind: profile\` pack with that id is
discovered — candidate pack \`quant.finance\`; remediations: add/enable
a \`kind: profile\` pack file declaring it, or fix the vocabulary
(remove or retype the \`uses\` edge)` — candidate pack and both
remediations named, exit 1 (plus the base `total_refs` dangle on the
unresolved link, which disappears in GREEN when the pack file joins the
corpus). RED, vocabulary-only (`kind: quant.risk` in a Properties kind
cell, no uses edge): prefix-derived `candidate pack \`quant.*\`` + both
remediations, exit 1 (plus `property_kind_closed`). GREEN both: the
activation advisories (`declared uses edge` / `vocabulary match:
quant.risk`), exit 0.

Probe evidence files: `/var/tmp/qfp-9h3z/evidence/`
(`a_red_ok.json`, `a_green_ok.json`, `a_red_dangling.json`,
`a_green_dangling.json`, `b_red.json`, `b_green.json`, `c_red.json`,
`c_green.json`, `d_law_red_issues.json`, `d_law_green_issues.json`,
`f_uses_red.json`, `f_uses_green.json`, `f_vocab_red.json`,
`f_vocab_green.json`, `e_green_openspec.json`, `e_red_openspec.json`,
`e_green_specs.json`, `e_red_specs.json`, `e_repo_openspec.json`,
`e_repo_specs.json`).
