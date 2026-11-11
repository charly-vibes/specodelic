# 2026-11-10 — Executable contracts: design resolutions from the grilling session

**Status: design only — no implementation authorized.** This document records
the design decisions resolved in the interview session that followed the
grounding of the Nov 10 build spec
(`2026-11-10-executable-contracts-build-spec-grounding.md` — read that first;
every D-n referenced below is defined there). A Rule-of-5 review of the
converged design was run during the session (4/4 HIGH findings TypeSafe-
verified); its fixes are folded into the decisions below. This document is
the companion answer to the grounding doc: there the external proposal was
checked; here the design is settled.

## The one-sentence design

Code is truth, markdown is reviewed intent and simulation, proposals carry
both — spec and implementation cannot drift because they never share an
artifact.

## Resolved decisions

### 1. Authority: the contract suite (D-3 resolved)

A **contract suite** — hand-authored executable tests with DI, run against
adapters — is the sole normative source of behavioral truth. Test
populations, epistemically ranked:

| Population | Example | Status |
|---|---|---|
| (a) generated fragments | `specodelic/*_props.rs` | projection of a source — never evidence |
| (b) checker pinners | `tests/conformance_matrix.rs` | pins the tool; expectations from specs |
| (c) implementation tests | wild's sidecar-bound pytest | descriptive today |
| (d) **contract suites** | new | **normative — authored-first, adapter-bound** |

Independence argument (adjointness §7.6): tests compiled from specs are one
projection of a single source; only authored-against-oracle artifacts are
independent. Retrofit rule (Rule-of-5 EDGE-001, verified): every contract
carries an origin — `authored-from-intent` (normative) or
`characterizing-observed` (non-normative until review promotes it).

### 2. Triangle, not pipeline

One adapter seam (a `PropertiesRunner`-shaped trait), two routed suite
families: normative contracts and derived compiled properties. The
implementation binds through adapters; the Model (below) is adapter #0.
Disagreement between the two families is a verdict in the closed taxonomy —
the adjunction's unit/counit becomes a machine-checkable finding, not an
essay.

### 3. No derived spec layer (D-1 resolved by deletion)

No extraction, no NL synthesis — ever. Prose lives in docstrings, read in
place. `underspecified` (conform's existing verdict) names the honest gap
when behavior has no prose. Promotion of a sketch to normative happens only
by replacement — writing the contract — never by elevation.

### 4. Model = adapter #0 (simulation preserved)

The declarative markdown Model (States/Transitions → TLA+/IR → embedded
stateright BFS) survives as the first, cheapest adapter: pre-code
simulation. Contract-first even pre-code: a failing contract against the
Model is a Model defect by default; speculative contracts (flagged) are
excluded from the normative set and the ladder. Orchestrate's machine-checked
stage gates (Rule-of-5 CORR-002, verified) become a tier-1 Model of the
process itself — the absorption is literal, not aspirational.

### 5. Tiering with demand-gated models (kata verdict as policy)

| Tier | Artifacts | Domain shape | Pre-code simulation |
|---|---|---|---|
| 0 | contracts only | data-shaped, arithmetic, policy | none |
| 1 | contracts + Model | lifecycle-shaped, stateful | Model adapter |

The Model may only be authored with a recorded formalization trigger (the
observed problem, one docstring line). Tier-0 "contract-verified" is a
complete ladder answer. The ladder is machine-read (consumes
add-evidence-audit's rung aggregate) — never decoration.

### 6. Three verbs

- **`check`** — fusion of verify + conform + the evidence-audit verb: suites
  × adapters → verdicts + scope-digested, staleness-checked, rung-tagged
  evidence. Absorbs `lint` as `--draft` mode (structural verdict on markdown
  tier input).
- **`graph`** — the acset closure engine with a dual front-end (annotated
  code + markdown tier). Views (dot/mermaid/TSV/wiring) emitted as today.
- **`doctor`** — advisory; gains the tier advisory ("lifecycle-shaped suite,
  no model, no trigger on record") and the EARS advisory (below).

`model-check` and `compile` become internal engines. `lint`/`new` are
absorbed (`--draft`, scaffolding). Fusion and death — never rebuilding.

### 7. Structure-in-code, prose-in-docstrings

Docstrings are prose-only — **nothing ever parses a docstring as data**.
All structure is compiler-validated syntax:

| Structure | Home |
|---|---|
| id | derived: path + function/fixture name |
| binding | `#[binds(component.x)]` |
| origin | `#[origin(intent)]` / `#[origin(observed)]` |
| speculative | `#[speculative]` |
| cross-suite supersede | `#[supersedes("old")]` — one edge, one direction |
| EARS intent | one line in suite docstring — doctor advisory, never a gate |

The four-layer tables (guards, traces_to, kind, law floors) live nowhere —
the format stays dead (Rule-of-5 CORR-001's fix keeps only the EARS
discipline, as advisory).

### 8. The markdown REPL tier (agent ergonomics preserved)

Any agent + one binary keeps the loop the corpus rewarded: `spk new`
scaffolds into `specs/`, agents write EARS + scenario tables (+ optional
Model), `check --draft` gives the instant structural verdict, `graph` gives
impact. `exploration_only` by default. The ladder reads: sketch → model →
tier 0 → tier 1 — progressive formalization with machine-labeled rungs
(deepseek's "no optional progressive path" finding, answered).

### 9. `openspec/specs/` = the markdown tier's home

The end-state spec file is **today's dual-format file minus its normative
layers**: frontmatter/Constraints/Properties die; EARS intent, scenario
blocks, and Model tables stay. Migration is demotion in place, per
capability, gated on that capability's contract-suite coverage — a gradient,
not an event.

Three anti-mirror rules (the dual-format drift generator stays dead):

1. Single format, single parser; the layer.md sidecar machinery retires; the
   archive companion survives as plain delta applier.
2. Citations one-way and component-scoped: contracts bind components via
   `#[binds]`; nothing executable cites a scenario or EARS line. Espectacular
   reads the scenario blocks (its rung, read-only as today) — two tools, one
   ladder, neither writes the other's files.
3. Proposals carry rationale + markdown-tier delta + contract diff,
   explicitly separated; `check` enforces the separation (contradiction
   between a proposal's scenario delta and its contract diff is a verdict).

### 10. Migration sequence (recommended — ratification pending)

1. Ship `add-evidence-audit` as designed (the `check` engine in embryo).
2. Wild-core vertical slice: one scenario, existing pytest binding, evidence
   recorded + staleness-checked; decides the adapter registry empirically.
3. Dual front-end + graph port behind a flag (additive, corpus keeps
   working).
4. Self-hosting inversion: the tool's own contract suite; conformance-matrix
   expectations become annotated contracts.
5. Per-capability demotion gradient: each capability's spec file drops its
   normative layers when its contract coverage lands; verb fusion lands with
   the first demoted capability.

Every step independently shippable; in-flight work consumed, never
abandoned; current gates run until the gradient starts.

## Behavior-preservation audit (Rule-of-5 final report, condensed)

| Current behavior | End-state home | Verdict |
|---|---|---|
| verify's evidence discipline | `check` | preserved by fusion |
| conform verdicts | `check` (+ CLI wiring it never had) | preserved, net gain |
| evidence-audit verb | `check` engine | preserved |
| graph/impact/closures | `graph`, dual front-end | preserved (port) |
| model-check simulation | engine behind `check` | preserved + strengthened |
| rename/refactor safety | `graph` impact over code | preserved in spirit |
| agent markdown loop | REPL tier + `specs/` home | preserved, honestly labeled |
| orchestrate stage gates | lifecycle-as-Model | preserved (CORR-002 fix) |
| EARS precision | advisory line | preserved as advisory (CORR-001 fix) |
| self-hosting corpus | tool's own contract suite | preserved by staging gate |
| lint, packs, dual-format mirror | — | deliberately killed (convicted by evidence) |

Review convergence: stages 2–4, 0 CRITICALs, measured FP rate 0/4 (0%),
CONVERGED, no escalation.

## Open questions

- Exact verdict mapping when `check` consumes the audit vocabulary (D-2's
  residue — no fourth vocabulary, but the contract-verdict mapping is
  unwritten).
- C4 context/container declaration home (Nov 2 doc D-4, carried).
- Glossary: "contract suite" vs espectacular's "contract tests" naming
  collision — one line needed in any user-facing doc.
- Migration sequence ratification (decision 10 is recommended, not yet
  ratified).

## Non-goals

No implementation, no tickets, no schema proposals. Nothing here overrides
sibling-tool constraints (espectacular read-only over `openspec/`;
pretender hook ownership; vampiro advisory-only). The four OpenSpec
open-questions from the external build spec that this design supersedes are
answered by deletion or by decisions 1–9; the design doc that would carry
them into openspec is a future, separately-reviewed artifact.

