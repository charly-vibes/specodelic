# Design — add-component-realization

The ticket (`specodelic-u1fm`) asks one question three ways: what is
**declared**, what is **computed**, what may a **view claim**. This
document records the mechanism decisions behind the proposal's answer
table and the alternatives rejected.

## D1: Components become a sixth row kind in a `## Components` section (chosen)

**Chosen.** `component` joins the row-kind closed set in `kinds/spec.md`
as a sixth kind, carried in a new `## Components` top-level section, with
fields `{id, kind, level, scope, realizes}`.

The corpus's kind set is closed at five, one kind per top-level section
(`kinds.kind_enum_closed`: "the five kinds are exactly {Intent,
Constraint, State, Transition, Property}, one per top-level section of a
spec file"). Reopening it is *designed for*, not a hack: the acset
capability states the exact path — `acset-core`'s `objects_closed`: "the
schema's objects are exactly {Intent, Constraint, State, Transition,
Property}; a sixth object is added only under a new Revision heading of
openspec/specs/specodelic/spec.md (append-only discipline)". So this
change lands under the next free Revision heading of the format doc, and
the append-only discipline (`specodelic.append_only_variants`) is
satisfied by that heading, never by a silent edit.

Two corpus rows must be amended under the same heading, or the change
contradicts the corpus it edits:

- `kinds.kind_enum_closed` — "the five kinds are exactly {…}" becomes six.
- `kinds.unknown_kind_rejected` — its generator is
  `row_belonging_to_a_sixth_section()` and its predicate asserts the
  sixth section (`## Actions`) is rejected. With `## Components` as the
  sixth section, that property must move to the *seventh* section, or the
  corpus rejects its own new kind.

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

**Alternative rejected — a pack-declared fiber kind**: packs extend the
`kind` column of a base table (`packs`' fiber kinds), which cannot carry a
new row *shape* (a component has fields no base table has). Rejected on
mechanism, not taste.

## D2: `realizes` is a Reference Typing row (chosen)

**Chosen.** `realizes` enters the **Reference Typing table of
`openspec/specs/specodelic/spec.md`** (the table's home — `kinds/spec.md`
only *cites* it, at lines 37 and 164) as one row:

| Field | Appears on | Must resolve to |
|-------|-----------|-----------------|
| `realizes` | Component | Constraint, or Intent |

It is resolved by the existing typed-reference machinery: corpus-wide
resolution (`total_refs`), kind compatibility (`ref_kind_compatible`),
the acyclic edge set, and self-reference refusal (`no_self_ref`).

**The two mechanisms, kept distinct.** An earlier draft of this design
conflated the Reference Typing table with the per-kind column permissions
(`satisfies`/`observes`, which `kinds.spec.md` declares as *columns a kind
may carry*, and which are not table rows). They are different mechanisms:
the Reference Typing table says which kind a *reference field* may point
at; `kinds`' `constraint_row_shape` says which optional columns a
*constraint row* may carry. `realizes` is the former — a reference field
on a new kind — and `satisfies`/`observes` are the latter. The
`row_shape_closed` checker reads the reference half live from the
Schema-as-data (`src/lint/row_shape.rs`: "the reference half is read live
from `crate::acset::schema::canonical`"), so adding the `Component`
object and the `realizes` morphism to `src/acset/schema.rs` is what makes
the column legal — never a hardcoded list.

**Alternative rejected — pointing into `src/`**: code-level realization is
vampiro's lane, advisory-only per the sibling-tool constraints; partial
Rust frontend edges make any deny-level claim unwarranted. The format
stays read-only over code.

**Revision scope (G7's "revision-scoped")**: the relation binds through
the graph artifact's snapshot digest, not a per-edge stored revision — a
new per-edge field would be a second currency mechanism beside the
existing `--before` + `scope_sha256` discipline (audit). Views and impact
questions trace to the digest; edges ride the corpus they live in. The
deviation from G7's literal wording is recorded in the change's non-goals
rather than papered over.

## D3: Level closed set is {context, container} — levels 3–4 refused

M4 K7/K9: levels 3–4 go stale and behavioral violations are unreachable;
the memo's condition-5 mapping already scored this **partial — levels 1–2
only**. So the parser refuses `level: code` (and any 3–4 spelling) with a
remediation naming the closed set. Refusal beats deprecation: a level that
cannot be checked must not be declarable.

## D4: Externality is a named variant, not a boolean

**Chosen.** `scope` is a closed set `{internal, external}`.

An earlier draft used `external: true`. Two corpus facts forbid it:
`specodelic.no_boolean_columns` — "schema defines no bool column type;
states are named variants only" — and the format's own habit of naming
variants (`level`, `kind`, `scope`) rather than flagging them. A boolean
marker is an unreadable column type in a format whose schema has none.

**The completeness rule follows the variant**: a `container` with
`scope: internal` and no `realizes` edge is a finding; a `container` with
`scope: external` passes; a `context` component needs no realization (an
external actor is what a context component *is* — packs, sibling tools,
consumers). So "unrealized" is a property of *internal containers*, and
the rule names the component and both remediations.

## D5: Reachability and the new kind's linkage obligations

`single_root_reachable` covers constraint/property/state/transition rows;
a new kind must declare its own-file primary linkage. Component rows
carry `traces_to` — the field the Reference Typing table already types as
"Constraint → Intent", whose "Appears on" cell widens to
`Constraint, Component`. The `supersedes` row (Constraint, Property) is
the corpus precedent for one field on two source kinds, and the acset
`Morphism` identity is the pair `(source, name)`, so the widened source
is a new morphism, not a schema edit to the existing one.

The alternative — a separate `component_of` linkage field — is rejected:
it would add a second intent-linkage mechanism beside `traces_to`, and
`single_root_reachable` would need a second reachability path for the same
purpose. One mechanism, widened.

`row_shape_closed` reads the new kind's field set from the Schema-as-data
(never hardcoded); `ref_kind_compatible` gets its compatible-kind rows
from the same table. `no_self_ref` and `acyclic` reach the new kind
through the shared reference machinery.

## D6: The completeness rule gets a checker file, not prose

`component_realized_or_external` is a **new rule family**, whose home is
a new checker spec file `linter-component_shape.md`
(`id: linter.component_shape`), registered in the rule→file table at
`openspec/specs/specodelic/spec.md` — the table every other family is
listed in. A rule with no file in that table is a rule nobody can run.

The rule id follows the corpus's naming law: `linter.<name>`, derived from
`lint::rule_id` (`format!("linter.{rule}")`). The id contains neither
`and`/`or` nor a universal token — `no_conjoined_id` and
`no_universal_in_id` are checked on the *Intent* id only (per
`src/lint/rules.rs`, "Scope: capability ids (the Intent id) … checker row
ids like `every_state_used` describe checks, not capabilities"), so a
checker-family id is outside their reach; the name is kept clean anyway.

## D7: Views ride graph-views' existing discipline

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

## D8: Rollout, self-hosting, and the archive sidecar

Embedded format revision bumps to the next free Revision heading (23 as of
2026-11-21, subject to other in-flight changes — revision numbers are a
shared, race-prone resource, so the change names the heading it claims
rather than assuming it).

The archive path requires a **layer sidecar** for the new capability: the
archive companion refuses a delta whose layer is unrestorable
(`archive-companion`'s `fail_closed_unverifiable`: "no sibling layer.md
sidecar … is refused BEFORE the destructive openspec archive"), and a
*new* capability has no deployed spec to inherit a layer from. So this
change ships `specs/component-realization/layer.md` beside the plain
delta, in the shape `add-preservation-check` established.

The corpus self-hosts: the `specodelic` capability declares its own
components first, so `just lint-specs` dogfoods the feature before any
consumer relies on it. Espectacular contract TOMLs are staged for each
delta scenario at authoring time (adoption-decision requirement);
espectacular stays read-only over `openspec/`.

## Research lineage

- G5 supported / G7 amend, M4 K7/K9/B3/T3:
  `.wai/resources/research/2026-11-17-meth-research-decision-memo.md`
- Composition-gap scoping directive (§5.1 item 4, "What A does not
  include"): `.wai/resources/research/2026-11-18-scope-fork-spec-tool-vs-authority-lattice.md`
- D-4/D-11, acceptance condition 5, problem 8:
  `.wai/resources/research/2026-11-16-authority-lattice-goals-and-problems.md`
- Ticket: `specodelic-u1fm` (decision 2026-11-18 §5.1.4)
