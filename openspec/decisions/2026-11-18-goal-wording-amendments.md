# Decision: amend the G2, G4, and G15 goal wordings

*Decision note, 2026-11-18. Applies the meth research (`M1 v2`, `M2`, `M5`,
`M6`, `M7`) to the goal set recorded in
`.wai/resources/research/2026-11-16-authority-lattice-goals-and-problems.md`.
Evidence base: the eight topic syntheses and their audits, the cross-topic
decision memo (`2026-11-17-meth-research-decision-memo.md` §5), and the T1
clause audit (`2026-11-18-t1-clause-audit.md`).*

## What is decided

Three goal rows are amended. The goals themselves stand; the claims their
wording made do not.

| # | Before | After |
|---|---|---|
| **G2** | "Independence" — evidence must not be derived from the thing it judges; `origin` separates `authored-from-intent` from `characterizing-observed` | Independence is **structural and lineage-scoped**: different author, different lineage, intent before code. `origin` carries a third class, `authored-from-same-agent`; implementation-blind, cross-family authoring is the untested probe. |
| **G4** | "Drift **impossible** by construction" | "Drift **mechanically visible or refused**" — no second authoritative copy, and every copy's acceptance is revision-bound to the authority's revision. |
| **G15** | "Demotion earned by measured **coverage**, never a vote" | Demotion gated on **behavioural-guarantee preservation** — mutation score, seeded-bug replay, differential execution — never a coverage number and never a vote. |

The amended text and a pointer note live in the goals document; the research
record is not rewritten silently. All other G-rows are untouched.

## Why

1. **G2 rests on the design's least-supported assumption.** Independence is
   structural, not social (M2; M7 C13 6/6), and two suites authored from one
   specification are one evidence (M1 v2 C12 4/4). An agent authored the same
   way as the implementation is a copy, not a second authority — which is the
   design's own case as a third `origin` class (M1 v2 C13 4/4). The only
   probe with standing is implementation-blind, cross-family authoring
   (M7 3/6), and it has not been run.
2. **"Impossible" overclaims a mechanism.** The corpus supports "made
   mechanically visible or refused" and revision-binding of copies, not
   impossibility (M2 K10/G4). The enforceable statement is the one that can
   be tested; the amended wording is that statement.
3. **Coverage is contradicted as an adequacy measure.** No provider treats
   coverage as adequacy (M7 C5 5/6; M6 C9 5/5; M5), and a flat coverage
   number destroys the one thing DO-178C models well — the criticality-graded
   ladder (M5 C6). G11 keeps coverage as a *technique menu*; G15 must not let
   it become a *demotion license*.
4. **The amendments are quote-level, not directional.** No goal is added,
   removed, or re-prioritized; the migration sequence and the preservation
   floor are unchanged. This is the cheapest alignment available before the
   next proposal quotes these rows.

## What is explicitly NOT decided

- **No implementation, spec, gate, or command change.** The lattice's
  mechanism claims are unchanged; only their wording is.
- **No evidence that the amended goals are achievable at the claimed cost.**
  The corpus has no measured effect anywhere (no cost data, no controlled
  comparison, no maintenance ratio). The amendments remove unsupported
  wording; they do not supply the missing warrant.
- **No D-9 acceptance change.** The six acceptance conditions and the E1–E6
  gap list stand separately.
- **G1, G3, G5–G14 untouched** — including G3, whose amendment (vacuity,
  `inconclusive`, computed `stale`) belongs to the D-1 proposal rather than a
  wording fix.

## Relationship to deployed behavior

None. No file under `openspec/specs/`, no code, and no gate changes. The
amended goal set is a research record used to steer proposals; the current
deployment keeps its existing behavior.

## Provenance of the amendment

- Research: `~/cotorreo/synthesized/_conversations/meth-{executable-contracts,authority-models,evidence-assurance,adoption-review,agent-authored}.md` (+ `.audit.md`)
- Synthesis: `.wai/resources/research/2026-11-17-meth-research-decision-memo.md` §5
- Clause audit: `.wai/resources/research/2026-11-18-t1-clause-audit.md`
- Amended rows: `.wai/resources/research/2026-11-16-authority-lattice-goals-and-problems.md` §2.2 (G2, G4, G15)