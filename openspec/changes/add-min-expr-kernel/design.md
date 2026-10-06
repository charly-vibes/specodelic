# Design: min-expr kernel

## Context

The originating survey mapped the corpus's data-dependent cells and
found no defined semantics for them; the matrix
(`.wai/projects/min-expr-kernel/designs/matrix/`) scored four
approaches against ten criteria, and approach 03 won on the acset
grounding (cell 08): row-typing and reference atomics implement over
the deployed `src/acset` layer — Schema-as-data supplies the 𝒦-typed
instances `I(k)` and the traversal primitive the kernel quantifies
over.

The Rule-of-5 review of the proposal plan (converged Stage 5; 6/6
CRITICAL/HIGH findings verified through the TypeSafe pass, measured FP
rate 0%) corrected four decisions; this design restates the verified
evidence directly.

Grounding evidence (verified against the live workspace):

- `specs/compile.md:30` — `fragment_guard_rejected` deferral:
  *"decision of record: deferred to a future Revision alongside a
  data-carrying state space"* — the deferral names its own condition;
  this change does not meet it and does not claim it.
- Matrix cell `03/02-data-dependent-coverage` — *"Tier B targets
  exactly the equational + bounded-quantified cells — a closed kernel
  grammar (bounded ∀/∃ over I(k), ==, comparisons, ∧/¬, reference-typed
  atomics resolves/unique/acyclic/reachable) with per-backend
  emission."*
- Matrix cell `03/10-sibling-boundary` — *"Tier C keeps `**tag:**`
  opaque; the per-constraint `binding` column lets external checkers
  claim constraints through contract-TOML `flags` ... without
  specodelic learning their language."*
- Sibling decision record (2026-10-02) — thin-core position:
  *"core grows once for the mechanism itself (kind: profile as an
  explicit Revision delta), everything else rides packs as
  Grothendieck fibers one level up."*
- Proven machinery for grounding: `src/graph.rs`
  (`find_supersedes_cycles`, `dfs_supersedes`), `src/acset` traversal,
  model_check bounded evaluation; `src/guide.rs:20` FORMAT_REVISION;
  revision-literal sites in `tests/cli/model_check.rs` (×3) and
  `tests/cli/parse_misc.rs:787`.

## Decisions

### D1 — v0 atomic set is closed and individually grounded (CORR-001)

The design doc's suggestion "only atomics with an existing lint or
model-check equivalent" is **rejected as written** (it would empty
Tier B: the targeted cells have no lint equivalent — that is why they
are unverified). The v0 atomic set is the doc's own closed list —
`==`, comparisons, bounded ∀/∃ over `I(k)`, ∧/¬, `resolves`, `unique`,
`acyclic`, `reachable` — and the underlying principle ("semantics
never outruns proven machinery") is preserved per-atomic: each v0
atomic names its grounding machinery (graph traversal, acset
traversal, model_check evaluation) in the kernel spec's grounding
table, and an atomic without a grounding entry cannot ship.

### D2 — Backend agreement as CI property first; Property-row promotion deferred

⟦−⟧py ≅ ⟦−⟧rust runs as a CI property test over shared fixtures
(rust backend exists today; the py backend arrives with l8l). Promotion
to a `law_requires_cases`-shaped Property row happens in the l8l/aby
change, not here — the property needs both backends to exist before it
can be law-shaped.

### D3 — Slice 1 is citation-algebra interpretation only (CORR-002)

Guard citations (`[[a]] ∧ [[b]]`, ¬) upgrade from inert TLA+
annotations to kernel-evaluated claims, with honest three-valued
status when a citation cannot be discharged. Executable guard
fragments (`**rust:**` in guard cells) remain rejected:
`fragment_guard_rejected` stands as the invariant of record, because
the deferral's condition (data-carrying state space) is not met by
instance-grounded expressions. The kernel Revision does **not** claim
that delta; if a future change makes the state-space argument, it
re-opens the decision explicitly.

### D4 — `kernel.binding` is Tier C mechanism, namespaced (CORR-004)

Reconciliation with the sibling thin-core position: the `binding`
column is not general core growth — it is the kernel mechanism itself,
carrying the bridge-never-absorb invariant (specodelic never interprets
external checkers' claims; checkers claim constraints through
contract-TOML `flags` without specodelic learning their language).
The name is namespaced (`kernel.binding`) to satisfy the
pack-namespacing criterion, shrinking collision surface with pack
binding columns (E1's rationale). No registry: the rescinded
dl/1-registry stance stays rescinded.

### D5 — Migration order: USAGE.md examples, then specodelic.md invariants

Gated per-file (specodelic-lf3 pattern): each migrated file must stay
lint-clean and runner-independent before the next migrates. `just
lint-specs` dogfood stays green with no new advisory class — a
migration that would introduce one is a migration bug, not a corpus
fact.

### D6 — No type-vocabulary freeze; atomics per D1

The kernel v0 ships reference-typed atomics only; typing kernel
expressions (E1/CLAR-002) stays open of record. Nothing in this change
forecloses a typed kernel later; the widening law is the extension
seam.

### D7 — Kernel-first, conditional on joint sign-off (CORR-003)

The sibling project's thin-core decision (2026-10-02) and the design
doc's HITL marking make this a joint decision, not a unilateral one.
This change records kernel-first as its intended sequencing, scopes
itself so pack-side numeric predicates (bioimage R2) remain pack-ridable
under the widening law, and **does not open implementation phases until
the sibling countersigns** (task 1.1). If the sibling prefers
pack-first, the kernel change re-scopes to slice 1 only.

### D8 — Widening law with a decidability gate

New atomics — including pack-defined predicates — must be decidable
over finite instances to register. The gate is the kernel's honesty
guarantee: an undecidable predicate cannot enter the closed set, so no
evaluation can silently diverge or vacuously pass. Pack predicates
that fail the gate stay pack-side (checked by contract-TOML runners,
not by the kernel).

## Risks

| Risk | Mitigation |
|---|---|
| D7 unresolved blocks all implementation | Task 1.1 is first; slice-1-only fallback scope is pre-agreed in this design |
| Three deltas in one change → archive blast radius (DRAFT-002, verified) | `delta_self_contained` task rows per capability; each delta restates the full requirement set; archive follows the dual-format recipe (`just archive-change`) |
| Contract-TOML debt blocks commits (EDGE-001, verified) | Per-scenario contract TOMLs authored in the same phase as the scenario deploys (l8l precedent: 6 TOMLs bound phase-2 tests) |
| Revision-literal drift aborts commits (turu specodelic-6sb) | Bump FORMAT_REVISION + all `specodelic.md Revision N` literals + regenerate compiled artifacts as one chore commit inside the change |
| Backend divergence (py vs rust semantics) | Shared fixtures + the CI agreement property runs on every push from slice 2 onward |
| Corpus migration breaks dogfood | Per-file gating; `just lint-specs` gate on every migration commit |
