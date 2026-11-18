# Decision: scope A — the specification tool; the authority lattice demand-gated

*Decision note, 2026-11-18. Ratified by the owner ("ok") after a Rule-of-5
review of the decision brief
`.wai/resources/research/2026-11-18-scope-fork-spec-tool-vs-authority-lattice.md`
(converged stage 4, no CRITICAL findings; §4.1 shipped-machinery citations
spot-verified). Supersedes the direction ruling in
`2026-11-11-executable-contracts-direction.md` — that note's step 1 stands as
shipped; its steps 2–5 become demand-gated, not abandoned.*

## What is decided

1. **Scope A is the product**: a specification tool that makes drift
   *visible and graded* inside one repo — markdown spec (EARS + Model +
   claims) normative; tests are *evidence, graded and dated*, never truth.
   The honest claim of record: "drift is visible at every revision" — never
   "no second authoritative copy exists".
2. **Scope B (the authority lattice: contract suites as normative truth,
   adapter registry, contract runner, corpus demotion, self-hosting
   inversion, independence claims) is demand-gated.** Entry condition — all
   three, or an explicit owner override:
   1. a real external consumer asks to bind its own suite (not a sample we
      pick), **or** a measured benefit appears on one module (seeded-defect
      replay or equivalent); **and**
   2. the preservation harness exists, so demotion is measurable.
   (Brief §5 stated clauses 1–3; collapsed here to the two load-bearing
   gates: a warrant, and a way to measure preservation.)
3. **Approved for implementation**: `add-evidence-record-axes`
   (`specodelic-re29`) — the five-axis evidence record + drift gate;
   `inconclusive`/`origin`/`binding_kind`/`bound` gain producers.
4. **Authorized next proposal**: `check` as the single judge verb — a fusion
   of `verify` + `conform` + `audit` over *existing* evidence (no runner, no
   registry). The proposal defines the verdict mapping and the exit
   contract; `inconclusive` enters the vocabulary only when a real producer
   exists (a run that executes and cannot decide).

## What the 2026-11-11 note keeps and what it loses

- **Keeps**: step 1 (`add-evidence-audit`) — shipped; the invariants that
  bind in-flight work; the "in-flight work is consumed, never abandoned"
  rule (py/ts park stays parked).
- **Demand-gated**: steps 2 (wild slice — dropped from the plan, not
  deferred; no wild ticket is filed), 3 (dual front-end/graph port),
  4 (self-hosting inversion), 5 (per-capability demotion gradient) — each
  resumes only under the entry condition above.
- **Lost, stated plainly**: the strong single-source promise and any
  independence claim for agent-authored evidence. The vocabulary and docs
  must not claim them (M1 v2 C12/C13, M7 C13; G2/G4/G15 amendments of
  2026-11-18).

## The value question stays open

A's founding value claim (spec-first reduces decision back/forth) remains
unmeasured — an79's first honest read is inconclusive. The warrant paths of
record: the seeded-defect replay probe (brief §5.1.5 — which is also the
experiment that would satisfy B's entry gate, so A's program and B's warrant
are the same instrument) and the paired-attempt experiment (an79 verdict doc,
§Prospective capture).

## Consequences carried by this note

- `openspec/decisions/2026-11-11-executable-contracts-direction.md` — banner
  pointer added; text untouched.
- Goals doc §2.2 status column and `docs/src/graded-evidence.md` wording are
  updated with the implementation work they accompany (re29 / check), not as
  standalone edits.
- Outcome tickets filed for brief §5.1 items 2–4 (+5 optional probe, +
  paired-attempt): one-judge-verb proposal, staleness-on-real-evidence demo,
  composition-gap scoping (G5/G7), warrant probe, paired attempts.
- Meth memo §7 item 6 reads "ratify scope: A now, B gated".
