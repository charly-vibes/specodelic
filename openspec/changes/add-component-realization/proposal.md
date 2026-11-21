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

- **A sixth row kind, `component`**, declared in a new `## Components`
  top-level section of a spec file. The format's kind set is a closed set
  of five (`kinds.kind_enum_closed`: "the five kinds are exactly
  {Intent, Constraint, State, Transition, Property}, one per top-level
  section"), so this change reopens that closure under the corpus's own
  append-only discipline: a new Revision heading in
  `openspec/specs/specodelic/spec.md` (the file `acset-core`'s
  `objects_closed` names as the gate: "a sixth object is added only under a
  new Revision heading of openspec/specs/specodelic/spec.md").
- **Component row shape** (`kinds.component_row_shape`), read from the
  Schema-as-data like every other kind's shape: base fields
  `{id, kind, level, scope, realizes}` plus the typed reference columns the
  Reference Typing table declares for it.
  - `level` — closed set `{context, container}`. C4 levels 3–4 are **not
    declarable** (D3).
  - `scope` — closed set `{internal, external}`. `external` replaces the
    boolean `external: true` marker an earlier draft of this proposal
    used: the format's schema defines **no bool column type**
    (`specodelic.no_boolean_columns`: "schema defines no bool column
    type; states are named variants only"), so an in-band boolean would
    have been an unreadable column. A named variant carries the same
    information in a form the format already has.
  - `realizes` — a typed reference column (below).
- **`realizes` as a typed reference column**, declared in the Reference
  Typing table (`openspec/specs/specodelic/spec.md` § Reference Typing —
  the table's home, not `kinds/spec.md`, which only *cites* it). One
  Reference Typing row:

  | Field | Appears on | Must resolve to |
  |-------|-----------|-----------------|
  | `realizes` | Component | Constraint, or Intent |

  The row is resolved by the existing typed-reference machinery —
  corpus-wide resolution (`total_refs`), kind compatibility
  (`ref_kind_compatible`), and the acyclic edge set — exactly as
  `traces_to` is, because `realizes` is a Reference Typing row like
  `traces_to`, not like the `satisfies`/`observes` *column permissions*
  (those are declared per-kind in `kinds.spec.md`, not as table rows).
  Each edge's guard prose carries the **residual assumption set** (G5/M4:
  composition is a per-edge rule, not a `composes=true` flag): what the
  edge does *not* guarantee — behavior, currency, correctness of the
  realized claim.
- **Component rows reach their own intent** through the existing
  `traces_to` field, which the Reference Typing table's "Appears on"
  column must widen from `Constraint` to `Constraint, Component` (the
  `supersedes` row is the precedent for one field on two source kinds).
  The alternative — a separate linkage field — would add a second currency
  mechanism beside `traces_to` and is rejected (design.md D5).
- **Structural completeness check** — a new rule
  `component_realized_or_external`: a `container` component with no
  `realizes` edge and no `scope: external` marker is a finding, so a
  declared part that nothing realizes is visible, never silent machinery.
  A `context` component with `scope: external` passes. The rule's home is
  a new checker spec file `linter-component_shape.md`
  (`id: linter.component_shape`), registered in the rule→file table at
  `openspec/specs/specodelic/spec.md` — the table every rule family is
  listed in, so the rule is never prose-only.
- **Derived views** (extends `graph-views`): a C4 levels-1–2 view rendered
  at build time from the graph artifact alone, stamped with the snapshot
  digest — same snapshot discipline as every other view
  (`graph-views.derived_views_only`, `build_time_generation`). The view
  claims structural facts only: names, ids, edges, reachability.

## Scope boundaries (the ticket's three questions)

| Question | Answer |
|---|---|
| **Declared** | Component rows (`level`, `scope`, `realizes`) in a `## Components` section, one declaration, in the corpus, no second copy. |
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

- `openspec/specs/specodelic/spec.md`: new Revision heading (next free
  heading — 23 as of 2026-11-21, subject to other in-flight changes);
  Reference Typing gains the `realizes` row and widens `traces_to`'s
  "Appears on"; `objects_closed` in `acset-core` now names six objects.
- `openspec/specs/kinds/spec.md`: `component` joins the row-kind closed
  set; `kind_enum_closed` and `unknown_kind_rejected` (which currently
  asserts a sixth section is rejected) are amended under the new Revision
  heading; `component_row_shape` joins the row-shape family.
- `openspec/specs/acset-core/spec.md`: the Schema value gains the
  `Component` object and the `realizes` morphism — `schema_matches_typing_table`
  requires the doc's Reference Typing table and the Schema value to agree
  row for row, so neither may move alone.
- `src/acset/schema.rs`: `CLOSED_OBJECTS` gains `Component`; the canonical
  schema gains the `realizes` morphism (source `Component`, target
  `Constraint`/`Intent`) and the widened `traces_to` source.
- Linter: `row_shape_closed` extends to the new kind (fields read from
  the Schema-as-data, never hardcoded); `total_refs`, `ref_kind_compatible`,
  `no_self_ref`, `acyclic` and `single_root_reachable` reach the new kind
  through the shared machinery; one new rule family
  (`linter.component_shape`) owns the completeness check.
- `graph-views`: one new derived view; the projection emits component
  nodes and realization edges. Existing views byte-identical.
- `spk lint`/`spk graph` gain no new verbs; no CLI surface change beyond
  view output. The corpus self-hosts the feature (`just lint-specs`
  dogfoods it once the `specodelic` capability declares its own components).
