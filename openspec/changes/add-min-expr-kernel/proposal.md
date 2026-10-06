# Change: Add min-expr kernel — decidable bounded logic over acset instances with per-backend emission

## Why

The matrix survey (`.wai/projects/min-expr-kernel/designs/`) found the
corpus's data-dependent cells — the equational and bounded-quantified
ones — are prose-only today: the linter and model-check can verify
structure and simulate transitions, but a cell like "resolves is
unique" or "every reachable row satisfies a bound" has no defined
semantics. Approach 03 (min-expr kernel) won the matrix on a decidable
fragment of the internal language of `Set^𝒦`: bounded ∀/∃ over the
finite instances `I(k)`, equality, comparisons, ∧/¬, and
reference-typed atomics (`resolves`/`unique`/`acyclic`/`reachable`),
implemented over the deployed acset substrate (`src/acset`,
capabilities `acset-core`/`acset-writer`) that the survey missed
entirely.

The selection came out of a Rule-of-5 review of this very plan
(converged Stage 5; CRITICAL/HIGH findings verified through the
TypeSafe pass — 6/6 verified, measured FP rate 0%) which corrected four
decisions before scaffolding:

- **Corrected (CORR-001):** the design doc's D1 suggestion — "only
  atomics with an existing lint or model-check equivalent" — would
  have collapsed Tier B into Tier A, because the cells Tier B targets
  are exactly the ones with *no* lint equivalent today. The v0 atomic
  set is instead D1's own closed list, each atomic grounded in
  *proven machinery* individually (graph cycle-detection, acset
  traversal, model_check evaluation) — preserving "kernel semantics
  never outruns proven machinery" without requiring lint equivalents.
- **Corrected (CORR-002):** slice 1 does **not** claim to be the
  Revision delta `fragment_guard_rejected` defers to. That deferral's
  condition is *"alongside a data-carrying state space"*; expressions
  over finite instances are not a program-counter state space. Slice 1
  is scoped to citation-algebra interpretation — upgrading
  `[[a]] ∧ [[b]]` / ¬ guard citations from TLA+ annotation to
  kernel-evaluated, still never a sole verdict unless executed — and
  `fragment_guard_rejected` stands as the invariant of record.
- **Corrected (CORR-003):** D7 (sibling coordination) was marked
  *"decide jointly before either change opens"* — it cannot be
  self-resolved by a coordination note. It is recorded here as
  **PENDING-JOINT-SIGN-OFF** and sequenced as the first task; this
  change is scoped so it cannot pre-empt pack-side numeric predicates
  (the bioimage pack's R2 needs stay pack-ridable under the widening
  law).
- **Corrected (CORR-004):** the Tier C `binding` column is core
  vocabulary *as kernel mechanism* (the bridge-never-absorb invariant),
  not general core growth — reconciled explicitly with the sibling
  project's thin-core position ("core grows once for the mechanism
  itself; everything else rides packs as Grothendieck fibers one level
  up") in design.md, and namespaced (`kernel.binding`) against pack
  binding columns per the pack-namespacing criterion.

Motivation for the first deliverable: slice 1 is the approach-02
increment — one green where today is red (guard citations currently
travel as inert TLA+ annotations; after slice 1 they evaluate under the
kernel's citation algebra, with honest three-valued status when a
citation cannot be discharged).

## What Changes

- **New capability `kernel` (openspec/specs/kernel):** the decidable
  bounded kernel — closed atomic grammar, decidability gate, three-valued
  Kleene status chain (honest `unknown` that propagates, never coerced
  to pass), per-atomic grounding in existing proven machinery, the
  widening law (new atomics must be decidable over finite instances;
  pack-defined predicates register under it), and the opaque binding
  column.
- **Compile (delta):** expr cells may opt into kernel translation under
  the closed grammar per cell; grammar violations are labeled
  extraction failures; prose expr cells compile byte-identically (pure
  widening); the `kernel.binding` column extracts as an opaque claim
  carrier — external checkers claim constraints through contract-TOML
  `flags`, no registry is built.
- **Model-check (delta):** guard citations evaluate under the citation
  algebra (slice 1); a backend-agreement property (⟦−⟧py ≅ ⟦−⟧rust
  over shared fixtures) runs as a CI property test; run reports persist
  backend, bound, and per-invariant three-valued status.
- **Corpus migration (no delta — specodelic-corpus edits, tasks only):**
  `specs/USAGE.md` examples, then `specs/specodelic.md` invariants,
  migrate to kernel expressions gated per-file (specodelic-lf3
  pattern); `just lint-specs` dogfood stays green with no new advisory
  class.
- **Discipline (tasks only):** FORMAT_REVISION bump + Revision-literal
  sync + compiled-artifact regeneration in the same change (turu
  specodelic-6sb); per-scenario contract TOMLs authored alongside each
  deployed scenario; every capability delta restates the full
  requirement set (`delta_self_contained`, 14b7a75 discipline).

## Decisions of record

| Row | Decision | Status |
|---|---|---|
| D1 | v0 atomics = closed set {`==`, comparisons, bounded ∀/∃, ∧/¬, `resolves`, `unique`, `acyclic`, `reachable`}, each grounded in proven machinery individually | decided (CORR-001) |
| D2 | Backend agreement starts as a CI property test over shared fixtures; promotion to `law_requires_cases`-shaped Property row deferred until l8l (py emitter) and aby land | decided |
| D3 | Slice 1 = citation-algebra interpretation only; `fragment_guard_rejected` stands; executable guard fragments remain rejected of record | decided (CORR-002) |
| D4 | `kernel.binding` is Tier C mechanism vocabulary — opaque string, never interpreted, namespaced against pack binding columns | decided (CORR-004) |
| D5 | Migration order: `specs/USAGE.md` examples, then `specs/specodelic.md` invariants; gated per-file | decided |
| D6 | No type-vocabulary freeze in this change; atomics per D1; E1 (namespaced kinds) / CLAR-002 stay open | decided |
| D7 | Kernel-first sequencing, conditional on sibling joint sign-off; pack numeric predicates stay pack-side | **decided** — kernel-first countersigned by the sibling 2026-10-06 (CORR-003) |
| D8 | Widening law: new atomics must be decidable over finite instances; pack predicates register under the same gate | decided |

## Deferred (follow-up changes, not tasked here)

- **Property-row promotion** of the backend-agreement property (blocked
  on l8l/aby; D2 records the CI-test interim).
- **Executable guard fragments** (re-opening `fragment_guard_rejected`)
  — requires the data-carrying state-space argument the deferral names;
  not attempted here.
- **Type vocabulary** for kernel expression typing (E1/CLAR-002) — the
  kernel v0 ships with reference-typed atomics only.
- **D7's resolution** — countersigned 2026-10-06: kernel-first. Recorded
  in both projects' decision records (`.wai/projects/*/designs/matrix/decision.md`).
  Implementation phases are open.

## Sequencing

- **D7 joint sign-off first (task 1.1):** no red/green phase opens
  before the sibling project countersigns the kernel-first sequencing.
- **`add-graph-views` (gre, approved, 0/21)** touches the same graph
  primitives the kernel's `acyclic`/`reachable` atomics ground in
  (`src/graph.rs` `find_supersedes_cycles`/`dfs_supersedes`). Kernel
  slice 2 grounds those atomics in acset traversal rather than the
  graph projection, and a sequencing note in both proposals prevents
  two active changes pulling the same module (EDGE-004).
- **specodelic-lf3 pattern** governs the corpus migration: per-file
  gating, no corpus-wide runner dependency.
