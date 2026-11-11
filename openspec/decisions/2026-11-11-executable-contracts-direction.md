# Decision: adopt the executable-contracts direction (contract suites as normative truth)

*Decision note, 2026-11-11. Grilling session over the grounded Nov 10
external build spec (ChatGPT "Executable Specification Strategy",
`6aedeb14`); the grounding and the session's resolutions are preserved in
`.wai/resources/research/2026-11-10-executable-contracts-build-spec-grounding.md`
and
`.wai/resources/research/2026-11-10-executable-contracts-design-resolutions.md`
(including a Rule-of-5 review of the converged design, 4/4 HIGH findings
TypeSafe-verified, 0 dropped). Owner instruction to record the direction the
same day.*

## What is decided

The tool's evolution target is the **executable-contracts design**: contract
suites (hand-authored executable tests with DI, origin-annotated) are the
sole normative source of behavioral truth; the declarative markdown Model
survives as "adapter #0" (pre-code simulation, demand-gated by recorded
formalization triggers); prose lives in docstrings and the markdown tier
(`openspec/specs/` as a single-format EARS + scenarios + Model file,
demoted in place per capability); the end-state CLI is three verbs —
`check` (fusion of verify + conform + evidence-audit), `graph` (the acset
closure engine over a dual front-end), `doctor` (advisory).

The migration sequence of record:

1. **`add-evidence-audit` ships exactly as proposed** — it is the `check`
   engine in embryo; fusion happens after it exists, never instead of it.
   No new proposal is needed for this step.
2. **wild-core vertical slice** (in the wild repo): one scenario, its
   existing pytest binding, evidence recorded and staleness-checked. This
   step exists to decide the adapter-registry shape empirically.
3. **Dual front-end + graph port, behind a flag** — additive; the corpus
   keeps working throughout.
4. **Self-hosting inversion** — the tool's own checkers get their contract
   suite; conformance-matrix expectations become annotated contracts.
5. **Per-capability demotion gradient** — each `openspec/specs/<cap>/`
   file drops its normative layers (frontmatter/Constraints/Properties)
   when that capability's contract coverage lands; verb fusion lands with
   the first demotion. No big-bang retirement event; the current gates run
   until the gradient starts.

## What is explicitly NOT decided (gated, belongs to future changes)

- **Adapter registry storage** (annotations vs sidecar) — decided
  empirically by step 2's outcome; a front-end proposal that assumes an
  answer is out of order.
- **`check`'s contract-verdict mapping** onto the audit vocabulary (the
  grounding doc's D-2 residue) — one vocabulary, no fourth entrant; the
  mapping is written when the fusion change is proposed.
- **C4 context/container declaration home** (Nov 2 grounding D-4) —
  carried, unresolved.
- **EARS advisory, origin annotation syntax, `#[supersedes]`** — design
  level only; each lands with the change that first needs it, not as
  standalone schema proposals.

## Invariants that bind every future change in this program

- In-flight work is consumed, never abandoned (the py/ts park stays parked;
  `add-evidence-audit` and siblings proceed as designed).
- Sibling-tool constraints hold: espectacular stays read-only over
  `openspec/`; no new write path into `openspec/specs/` beyond the archive
  companion; pretender owns hooks; vampiro advisory-only.
- Demotion of any spec file requires that capability's measured contract
  coverage (step 4's number), never a vote.
- Every step independently shippable; current behavior is the
  preservation floor until its replacement ships.

## Why

1. The kata evidence (zai rounds 1–2, the Nov 5 numeric-pack round) is
   convergent: executable checking earns its cost on lifecycle-shaped
   domains; mandated four-layer structure and prose arithmetic were
   convicted (checked shell around an unchecked core; dual-format mirror
   as drift generator).
2. The Nov 2 and Nov 10 groundings both concluded the same thing from
   different directions: consolidate and shrink, don't build a successor
   framework — "the simplest successful outcome may be no Specodelic
   successor at all."
3. The design makes spec↔implementation drift structurally impossible
   (never sharing an artifact) instead of checked-but-possible, which is
   the only answer to the question that started the November thread.
4. The sequencing gates are the design's own falsification discipline:
   the two riskiest claims (adapter registry for a foreign consumer;
   contract coverage reaching the matrix) are tested at minimum cost
   before the tool commits.

## Relationship to deployed behavior

None today. No spec, no gate, no command changes with this decision. The
first observable change remains `add-evidence-audit`'s, already
independently justified by the catitas evidence-scope failures — this
decision only records that its engine is the seed of `check`.
