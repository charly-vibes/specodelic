# Design — add-component-realization

The ticket (`specodelic-u1fm`) asks one question three ways: what is
**declared**, what is **computed**, what may a **view claim**. This
document records the mechanism decisions behind the proposal's answer
table and the alternatives rejected.

## D1: Components become a row kind, not a manifest (chosen)

**Chosen.** `component` joins the row-kind closed set in
`kinds/spec.md`, with fields `{id, kind, level, realizes, notes}` and
`level ∈ {context, container}`. A component is normative structure — part
of what the spec *means* — so it lives in the four-layer shape, not beside
it.

**Alternative rejected — external manifest** (`*.components.md`, the
checklist-manifest precedent): the mp1 row-10 decision scoped manifests to
*external completeness* artifacts ("not a sixth spec kind"). Structure that
the spec's meaning depends on is exactly what the checklist carve-out
excludes. G12's verdict convicts the alternative too: unbound structure
drifts 5/5 and "survives only as a generated or ID-linked projection" — a
manifest is ID-linked but never reaches intent rows, so
`single_root_reachable` could not vouch for it.

**Alternative rejected — reuse existing row kinds** (components as
`extension_point` constraints): an unreadable kind cell is one thing, but a
constraint whose expr is a structural declaration abuses the closed kind
semantics and makes `constraint_kind_closed`'s contract lie.

## D2: Realization is a typed reference column (chosen)

**Chosen.** `realizes` enters the Reference Typing table
(`kinds/spec.md`) as a component→row reference, resolved corpus-wide — the
same machinery as `satisfies`/`observes` (Revision 18 retired
self-containment; cross-file typed edges are established). Targets are
spec-level rows (capability intents, constraints, properties); kind
compatibility is read from the table, not hardcoded. Each edge's guard
prose carries the **residual assumption set** (G5/M4: composition is a
per-edge rule, not a `composes=true` flag): what the edge does *not*
guarantee — behavior, currency, correctness of the realized claim.

**Alternative rejected — pointing into `src/`**: code-level realization is
vampiro's lane, advisory-only per the sibling-tool constraints; partial
Rust frontend edges make any deny-level claim unwarranted. The format
stays read-only over code.

**Revision scope (G7's "revision-scoped")**: the relation binds through the
graph artifact's snapshot digest, not a per-edge stored revision — a new
per-edge field would be a second currency mechanism beside the existing
`--before` + `scope_sha256` discipline (audit). Views and impact questions
trace to the digest; edges ride the corpus they live in.

## D3: Level closed set is {context, container} — levels 3–4 refused

M4 K7/K9: levels 3–4 go stale and behavioral violations are unreachable;
the memo's condition-5 mapping already scored this **partial — levels 1–2
only**. So the parser refuses `level: code` (and any 3–4 spelling) with a
remediation naming the closed set and the advisory lane. Refusal beats
deprecation: a level that cannot be checked must not be declarable.

## D4: Completeness — realized-or-external, on the lint channel

A container component with no `realizes` edge is the structure-shaped
version of `every_state_used`'s failure ("machinery the model can never
enter or leave"). New rule `component_realized_or_external`: finding unless
the edge exists or `external: true` is set (context actors — packs,
sibling tools, consumers — are declared but not realized in-corpus).
Fail-closed on unreadable level/external values.

## D5: Reachability and the new kind's linkage obligations

`single_root_reachable` covers constraint/property/state/transition rows;
a new kind must declare its own-file primary linkage. Component rows carry
`traces_to` (to their own intent, directly or through the realizing rows)
— the same own-file discipline as constraints — so the rule's machinery
generalizes unchanged. `row_shape_closed` reads the new kind's field set
from the kinds table (schema-as-data, never hardcoded);
`ref_kind_compatible` gets its compatible-kind rows from the same table.
`no_self_ref` extends to `realizes` by the general self-reference clause.
`acyclic` adds `realizes` to its edge union.

## D6: Views ride graph-views' existing discipline

The C4 view is one more derived view: consumed from the edge-list
artifact + lint-gated inputs only (`derived_views_only`), generated at
build time, never committed (`build_time_generation`), violations rendered
annotated, never silently clean (`violations_annotated`). Snapshot
traceability = the artifact digest already carried by the projection; the
view prints it. Byte-identical re-render follows from
`deterministic_projection`. Levels-3–4 requests are refused at the
renderer boundary naming the closed set.

Counting semantics that would touch the fan-in question are avoided —
`specodelic-3xc` (B3) decides multiplicity separately; this view counts
nothing.

## D7: Rollout and self-hosting

Embedded format revision bumps to 23 (new row kind + typing column =
schema change; `AGENTS.md` managed block refreshes via `spk init --force`
at implementation time). The corpus self-hosts: the `specodelic`
capability declares its own components first, so `just lint-specs`
dogfoods the feature before any consumer relies on it. Espectacular
contract TOMLs are staged for each delta scenario at authoring time
(adoption-decision requirement); espectacular stays read-only over
`openspec/`.

## Research lineage

- G5 supported / G7 amend, M4 K7/K9/B3/T3:
  `.wai/resources/research/2026-11-17-meth-research-decision-memo.md`
- Composition-gap scoping directive (§5.1 item 4, "What A does not
  include"): `.wai/resources/research/2026-11-18-scope-fork-spec-tool-vs-authority-lattice.md`
- D-4/D-11, acceptance condition 5, problem 8:
  `.wai/resources/research/2026-11-16-authority-lattice-goals-and-problems.md`
- Ticket: `specodelic-u1fm` (decision 2026-11-18 §5.1.4)
