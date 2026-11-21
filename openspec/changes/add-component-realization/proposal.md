# Change: Component realization — declared structure, computed views

> Scopes the composition gap (G5/G7) per ticket `specodelic-u1fm`
> (decision 2026-11-18 §5.1.4): "what is declared, what is computed, what
> a view may claim." Scope-A work — no runner, no registry, no foreign-repo
> coupling.

## Why

System structure in the corpus is prose-only and unverifiable (Nov 2
grounding, problem 8: "Composition is file-level only"). The research
settled the question:

- **G7 verdict — amend** (`2026-11-17-meth-research-decision-memo.md`):
  realization is a typed, revision-scoped **relation**, structurally
  checkable only. C4 levels 3–4 go stale and behavioral violations are out
  of reach (M4 K7/K9). "Derived from source facts" holds for **levels 1–2
  only**.
- **G5 verdict — supported**: typed relations with checker identity and
  revision digest are the implementable graph primitive (M2 CL; M4 B3).
  Composition is a per-edge rule yielding a residual assumption set, not a
  `composes=true` flag (M4).
- **Acceptance condition 5** ("a generated C4 view and an impact
  explanation trace back to the same snapshot") is **partial** — computable
  for levels 1–2 only. This change lands the computable half and refuses
  the rest, loudly.
- **D-4/D-11 residue**: the C4 declaration home has been carried unresolved
  since the Nov 2 grounding. This change is that home.

Today a component/realization concept does not exist in the format: no row
kind, no typed reference, no view. The graph collapses rows to intents;
`graph-views` derives per-file state machines, traceability, and schema
shape — but nothing says *which capability realizes which architectural
part*, and nothing detects a declared part that nothing realizes.

## What Changes

- **New `component` row kind** in the format (`kinds/spec.md`): closed
  fields — `level ∈ {context, container}`, `realizes` targets, prose
  responsibility. C4 levels 1–2 only; levels 3–4 are refused with a
  remediation naming the stale-levels verdict and the advisory lane that
  owns code-level structure (vampiro).
- **`realizes` as a typed, revision-scoped relation** between spec-level
  rows (component → capability/row targets), corpus-wide resolution
  (`total_refs`), kind-compatibility (`ref_kind_compatible`),
  self-reference banned (`no_self_ref`), cycle-checked (`acyclic`).
  Each edge carries the residual assumption it does *not* guarantee —
  realization is structural, never behavioral conformance.
- **Structural completeness check**: every component is realized by ≥1
  `realizes` edge or explicitly marked as context-external; a component
  nothing realizes is a lint finding, not a silent row.
- **Derived views** (extends `graph-views`): a C4 levels-1–2 view rendered
  at build time from the graph artifact alone, stamped with the snapshot
  digest — same snapshot discipline as every other view
  (`graph-views.derived_views_only`, `build_time_generation`). The view
  claims structural facts only: names, ids, edges, reachability.

## Scope boundaries (the ticket's three questions)

| Question | Answer |
|---|---|
| **Declared** | Components (level: context/container) and `realizes` edges, as rows in a capability spec — one declaration, in the corpus, no second copy. |
| **Computed** | Resolution, typing, acyclicity, completeness findings; the edge-list projection rows; the rendered view. All derived, all revision-stamped. |
| **View may claim** | Structural facts over levels 1–2 only: what exists, what points at what, what is reachable, what is unrealized. Never behavioral conformance, never freshness beyond the digest, never code-level structure. |

## Out of scope

- **C4 levels 3–4** — refused at parse time (go stale; M4 K7/K9).
- **Code-level realization checking** (`src/` seams) — vampiro's
  advisory-only lane per the sibling-tool constraints; this change never
  reads `src/`.
- **Cross-repo / cross-system composition** — `svc-contracts-pack`'s lane,
  still `draft` awaiting a second consumer.
- **Contract runner, adapter registry, self-hosting inversion** — Scope B,
  demand-gated (decision 2026-11-18 §2).
- **Fan-in multiplicity rule for view counting** — `specodelic-3xc` (B3)
  decides separately; this change's views count nothing that depends on it.

## Impact

- `openspec/specs/kinds/spec.md`: `component` joins the row-kind closed
  set; Reference Typing table gains the `realizes` column; embedded format
  revision bumps (Revision 23).
- Linter: `row_shape_closed`, `ref_kind_compatible`, `total_refs`,
  `no_self_ref`, `acyclic`, `single_root_reachable` extended to the new
  kind (component rows carry `traces_to` for own-intent reachability);
  one new completeness rule (`component_realized_or_external`).
- `graph-views`: one new derived view; projection emits component nodes and
  realization edges. Existing views byte-identical.
- `spk lint`/`spk graph` gain no new verbs; no CLI surface change beyond
  view output. The corpus self-hosts the feature (`just lint-specs`
  dogfoods it once the `specodelic` capability declares its own components).
