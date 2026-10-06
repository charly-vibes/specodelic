# Grounded review — unverified-expressions bundles (`~/Downloads/cv`)

**Project:** min-expr-kernel · **Date:** 2026-10-06 · **Reviewer session:** repo-side verification of four model-produced analysis bundles against `specodelic` at snapshot `028fc50` (v0.5.2, the bundles' anchor) and at HEAD `d847837`.

**Canonical findings source for the design matrix** (`designs/matrix/`). The bundles are inputs; this file is what survives contact with the implementation.

## The four bundles

| Bundle | Producer | Character |
|---|---|---|
| `specodelic-unverified-expressions-survey-&-category-theory-model.md` | GLM (via Vibe) | Measured corpus survey (536 cells, 9 marker-bearing at `028fc50`) + design eval + category model + tiered recommendations. Best-grounded of the four. |
| `claude.zip` (`dl/1` notes) | Claude Sonnet 5.5 | Proposes `dl/1` — typed first-order bodies + least-fixpoint recursion (Datalog-like). Self-declared "reported from reading, not verified line by line". |
| `specodelic-expr-model.zip` | unnamed model | Third independent inventory + category model + minimum-language proposal. |
| `zai-evaluate-unverified-expressions.md` | GLM chat export | Evaluation conversation; self-reports "repository internals were never read" — README-only inference, least grounded. |

## Verified claims (hold at snapshot and at HEAD)

- **Honest failure is real:** `todo_predicate!` is a `todo!` placeholder that panics only at execution (`src/verify.rs:15`, `src/compile.rs:675,771`); prose-only guards yield `invariants_checked: []`, never a fabricated counterexample (`src/model_check.rs:21,116,204`).
- **Seam is real:** `PropertiesRunner` trait, `CargoRunner` sole adapter (`src/verify.rs:338,425,442`).
- **Guard blocker is fragment-scoped:** `specs/compile.md:30` rejects only `**rust:**` fragments in guards ("no data binding a guard could constrain… deferred alongside a data-carrying state space"). The citation algebra (`[[a]] ∧ [[b]]`, `¬`) is a different, uninterpreted surface — guards travel as TLA+ annotations "never a verdict" (`src/model_check.rs:165`). The survey's separation of the two is correct.
- **theory.md starting point accurate:** 𝒦 schema category (`specs/theory.md:29`), copresheaf instance (`:48`), Grothendieck namespacing (`:75`).
- **Counts corroborated:** independent recount at `028fc50` found 10 marker-bearing rows vs. the survey's 9 — within parse-method noise.

## Critical drift findings (bundle-aging)

1. **The "pending" change landed one day after the snapshot.** `add-language-neutral-property-binding` is deployed and archived (`openspec/changes/archive/2026-10-06-add-language-neutral-property-binding/`; commits `d77871e`, `1fea62a`). Revision 16 of record (`specs/specodelic.md:768`); closed tag set `{rust, py, ts}` shipped (`FRAGMENT_LANGUAGES` `src/compile.rs:145`; `fragment_language_closed`, `no_emitter_labeled_failure` with follow-up pointers, `src/compile.rs:371–451`). py/ts emitters are filed as beads `specodelic-l8l` (carries generator-vacuity decision D5) and `specodelic-aby`.
2. **Counts are one-day stale:** ~9 marked cells at `028fc50` → 28 `**rust:**` occurrences in `specs/`+`packs/` at HEAD. Tier sizes must be re-measured before any task list depends on them.
3. **acset blind spot (biggest grounding gap):** `src/acset` (`src/lib.rs:13`) and deployed capabilities `openspec/specs/acset-core`, `openspec/specs/acset-writer` landed 2026-10-04 — *before* the survey's snapshot. Doc 04 proposes exactly this substrate (instance-as-copresheaf, merge-as-pushout, typed rows) without citing it; `acset.pushout` is explicitly deferred (`archive/2026-10-04-add-acset-core/proposal.md:45–46`). Any kernel implementation should be grounded over `src/acset`, and the survey never says so.
4. **Four bundles, three grammars, no adjudication:** survey kernel (decidable fragment of `Set^𝒦` internal language) vs. `dl/1` (least-fixpoint recursion — in direct tension with the decidability floor) vs. the expr-model zip's own grammar. The zai bundle is evaluation-only and README-grounded.

## Do not double-file

- py/ts emitters → `specodelic-l8l` / `specodelic-aby`.
- Generator adequacy / vacuity (D5) → inside `specodelic-l8l`.
- Guard-typing corpus sync gap → `specodelic-814`.

## Disposition

Survey bundle declared canonical input; other bundles archived as context only. Matrix built from the grounded facts above.
