# Design matrix — minimum expression kernel for unverified expression cells

**Project:** min-expr-kernel · **Date:** 2026-10-06
**Canonical findings source:** `research/2026-10-06-grounded-review-unverified-expressions-bundles.md` — this doc is the decision record; that file is the findings source of truth.
**Backing matrix:** `designs/matrix/` — 12 criteria × 4 approaches = **48/48 cells filled**, each cell a `fact.md` grounded in the current implementation plus a `green`/`yellow`/`red` verdict.

**Canonicality:** the matrix directories are the decision record of substance — when this doc's table and the cells diverge, the **cells win**; this doc's table is a projection, regenerated after cell edits (Ro5 review 2026-10-06: two cells corrected, see §Corrections). **Legend:** `green` = criterion satisfied with implementation-grounded evidence · `yellow` = partially satisfied or grounded-but-costly · `red` = criterion violated or anti-pattern of record.

## Problem

The specodelic pipeline is deliberately bimodal: cells with a `**rust:**` fragment become executable (invariant-kind Constraints only); everything else — ~98% of the corpus's expression content at the analyzed snapshot — is parsed positionally, structurally linted, and never interpreted. The unverified cells carry three distinct jobs: data-dependent relations (need a language), structural meta-properties (already owned by lint), and human-only prose (correctly uninterpreted). Four external analysis bundles (`~/Downloads/cv`) propose extensions; this matrix adjudicates them against the implementation as of `d847837` (Revision 16 of record).

## Approaches

| # | Approach | One line |
|---|---|---|
| 01 | status-quo | Bimodal as-is; external binding via artifacts + contract-TOML `flags` only |
| 02 | guard-citation-semantics | Near-term slice: meet-in-Sub(S) semantics for the guard citation algebra, compiled into TLA+ instead of comments |
| 03 | min-expr-kernel | Three-tier: Tier A stays prose/lint · Tier B kernel = decidable fragment of `Set^𝒦`'s internal language over the acset substrate, unique-functor emission · Tier C opaque `**tag:**` + `binding` column bridging external checkers |
| 04-dl1-datalog | typed first-order bodies + least-fixpoint recursion (positive-only), 3-valued Kleene status, built-in predicate registry | 1g/10y/1r |

## Verdict matrix

| Criterion | 01 status-quo | 02 guard-semantics | 03 min-expr-kernel | 04 dl/1 |
|---|---|---|---|---|
| guard-citation-semantics | red | green | green | yellow |
| data-dependent-coverage | red | red | green | yellow |
| decidability-floor | green (vacuous) | green | green | green |
| unique-functor-emission | red | yellow | green | yellow |
| honest-failure | green | green | green | yellow |
| prose-untouched | green | green | green | yellow |
| append-only-evolution | green | green | green | yellow |
| acset-substrate | yellow | yellow | green | yellow |
| self-hosting | green | green | green | yellow |
| sibling-boundary | green | green | green | yellow |
| migration-cost | green | green | yellow | red |
| opaque-binding-bridge | green | green | green | yellow |
| **Totals** | 8g/1y/3r | 9g/2y/1r | **11g/1y/0r** | 1g/10y/1r |

Totals are unweighted — a red dominates the reading regardless of count; read cells, not sums.

**Scope exclusion (recorded, not analyzed):** a fifth approach — *closed per-domain predicate grammars riding packs* — is deliberately absent: it is the sibling project `domain-specific-extensions`' decided mechanism for machine-checkable numeric predicates (its matrix R2 → decision 02-profile-intent). The overlap question (kernel vs pack grammars) is deferred to that project's sequencing, tracked as D7 below.

## Reading

- **01 status-quo** shows exactly what is wrong with today: the two red cells (guards, data-dependent tier) are the measured gap; its greens are the guardrails any design must preserve.
- **02** is the minimal honest increment — one green where today is red — but leaves the ~204-cell tier untouched and does not establish the unique-functor pattern py/ts emitters (`specodelic-l8l`/`aby`) will need.
- **03** is the only approach green on the substrate criterion: it implements row-typing over the deployed `src/acset` layer instead of erecting a parallel structure, and its single yellow (migration cost) is opt-in, lint-gated, file-by-file — the `specodelic-lf3` pattern.
- **04** no longer fails on the decidability floor (Ro5 correction: positive-only fixpoints over finite instances are decidable, `02-categorical-model.md:25`) — its remaining reds are substrate (paper 𝒦-mapping, no `src/acset` plan) and migration. Crucially, 04 **near-converges with 03**: both are 𝒦-typed bounded coherent logic over finite instances with overlapping atomics (`resolves`/`unique`/`acyclic`/`reaches`). The bundle's genuine contributions the kernel design should absorb: the three-valued Kleene status chain (honest `unknown`), the built-in predicate registry, and pack-defined predicate registration. "Three grammars" overstates the divergence — the survey kernel and dl/1 v1 are essentially one grammar with two honesty layers.

## Recommendation

**03 min-expr-kernel**, with 02 as its first slice (guard citation semantics = kernel slice 1), **absorbing dl/1's three-valued Kleene status chain and built-in-predicate registry into the kernel's honest-failure semantics** (the two proposals near-converge; the absorb is cheaper than the rivalry). Recommendation, pending grill before any openspec proposal is opened (candidate id: `add-min-expr-kernel`).

## Decision rows open for grilling

- **D1** — Kernel grammar v0 scope: which atomics ship first (`resolves`, `unique`, `acyclic`, `reachable`, `==`, bounded ∀/∃)? Suggest: only atomics with an existing lint or model-check equivalent, so kernel semantics never outruns proven machinery.
- **D2** — Backend agreement: is ⟦−⟧py ≅ ⟦−⟧rust asserted as a cross-backend Property row (natural-isomorphism framing, `law_requires_cases`-shaped) or as a CI property test over shared fixtures? Decides what `specodelic-l8l` carries beyond D5.
- **D3** — Guard semantics placement: does the citation-algebra interpretation ship inside `add-min-expr-kernel` or as its own Revision delta (the `fragment_guard_rejected` decision-of-record says future Revision; does *this* kernel change qualify)?
- **D4** — Binding column: per-constraint opaque `binding` on Constraints — new pack pattern or core vocabulary? Must not collide with pack `binding` columns (see `domain-specific-extensions` matrix, pack-namespacing criterion).
- **D5** — Migration gating: which corpus files migrate first? Suggest `specs/USAGE.md` examples (worked-example rows) then `specs/specodelic.md` invariants — self-hosting before breadth.
- **D6** — Kind-column overload (CLAR-002) interacts with kernel typing; resolve before the grammar freezes its type vocabulary (the survey's open problem 4, also tracked in the sibling matrix as E1).
- **D7** — Sequencing vs the sibling project: the `domain-specific-extensions` pilot pack (bioimage-data) needs machine-checkable numeric predicates (its R2). Does a pack-scoped closed grammar land first (pack mechanism, no kernel) with the kernel absorbing it later, or does the kernel land first and the pack grammar become kernel vocabulary? Decide jointly with that project before either change opens.
- **D8** — Decidability gate for extensions: dl/1's pack-defined predicate registration has no decidability guard. The kernel's widening law should state explicitly that new atomics must be decidable over finite instances (or PBT-sampled with the honest-unknown status), so the decidability floor survives grammar growth.

## Corrections (Ro5 review 2026-10-06)

TypeSafe-verified review pass corrected two cells in approach 04 and their projection here: `04/03-decidability-floor` red → green (dl/1 v1 is decidable by construction — positive least fixpoints over finite instances), `04/08-acset-substrate` red → yellow (dl/1 grounds in 𝒦/copresheaf vocabulary; the real gap is no `src/acset` engagement). Measured false-positive rate of the original pass: 1 in 5 findings (20%). Counts in criteria (`30 guard cells`, `~204 equational cells`) are anchored to the 2026-10-06 corpus — re-measure before any task list depends on them (marker-bearing rows already drifted 9 → 28 between 2026-10-05 and 2026-10-06).

## Invariants every approach must preserve (non-negotiable)

`prose_untouched` · honest failure (`todo_predicate!` panic, labeled extraction failures, `invariants_checked: []`) · `append_only_variants` / pure-widening Revisions · byte-stable compile artifacts · self-hosting · bridge-never-absorb (`**tag:**` + pack `binding` pattern) · espectacular read-only over `openspec/` · no language-runner registry in specodelic.

## Do not double-file

- py/ts emitters → `specodelic-l8l` / `specodelic-aby` · generator adequacy (D5) → inside `specodelic-l8l` · guard-typing corpus sync → `specodelic-814`.
