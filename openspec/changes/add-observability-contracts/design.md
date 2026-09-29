# Design: observability contracts

## Context

The specodelic format grows only through sanctioned Revisions
(`append_only_variants`): closed kind value-sets and the Reference Typing
table widen under a new Revision heading, never silently. Revision 7 added
`extension_point`/`satisfies` for external contracts; Revision 6 added
`emits` for state outputs. Neither makes "what must be observable" or "what
crosses the boundary" checkable facts. This design adds the minimal
vocabulary to close that, shaped on the Revision 7 precedent end to end.

A Rule-of-5 review of the originating investigation converged with these
binding corrections: D1 (no authored boundary tag), D2 (Constraint-only
sources), D4 (acyclic exemption stated explicitly), D5 (advisory-first,
gating deferred), D6 (cross-file check modeled on external_completeness,
not coverage).

## Goals / Non-Goals

- Goals:
  - A spec author can declare, per behavior, what must be observable — and
    have "declared an output nobody observes" be a lint finding.
  - `spk graph` can answer "what does this component expose externally"
    without any new authored metadata.
  - Zero breaking change; the corpus remains lint-clean throughout.
- Non-Goals:
  - Verifying that conformers of `extension_point` contracts exist or are
    correct (Revision 7's explicitly rejected scope).
  - A sixth schema kind (`Component`); `kind_enum_closed` stays closed.
  - Waiver machinery in v1.
  - Gating lifecycle transitions on the new check in v1.

## Decisions

### D1 — External boundaries are derived, not tagged

The investigation's original mechanism (optional frontmatter field
`boundary: external`) was rejected: it makes external-ness an *authored*
fact that drifts from the edges, against `graph.md`'s
`graph_is_derived_not_authored` philosophy. A file is an external boundary
iff it hosts ≥1 `extension_point` Constraint. The classification disappears
automatically when the contracts move — no stale tag, no new Revision in the
frontmatter field set.

*Alternatives considered:* authored frontmatter tag (rejected above);
tagging individual Constraints (same drift, more surface).

### D2 — `observes` sources from Constraint rows only

The investigation said "any Constraint/Property". `kinds.md`
`property_row_shape`'s field set is exactly
`{id, kind, derives_from, generator, predicate}` — a Property source would
require a second row-shape Revision for zero demonstrated need. The
`satisfies` precedent (Constraint, any file → extension_point Constraint)
is followed exactly: Constraint, any file → effect Constraint.

### D3 — `observes` is authored as an optional Constraints-table column

`satisfies` is authored as its own optional column (USAGE.md §2.6's
five-column example). `observes` mirrors that shape exactly: one optional
column, one Reference Typing row, `ref_kind_compatible` needs zero code
changes (it reads `allowed_targets(field)` generically).

**Pre-existing tension to reconcile, not inherit:** `kinds.md`
`constraint_row_shape` says Constraint fields are *exactly*
`{id, kind, expr, traces_to}` — already inconsistent with `satisfies`'
column. `kinds.md` Revision 5 must state the general rule (optional typed
reference columns alongside `traces_to`) rather than special-casing
`observes`. If the tension is already tracked under `specodelic-qc8`'s
coverage gaps, fix it there per its triage; either way it is not weakened
to land this change.

### D4 — `observes` is acyclic-exempt, stated explicitly

`acyclic`'s edge set is the closed union `traces_to ∪ derives_from ∪
guard-as-edge` (`linter-graph_shape.md`); `satisfies`, `emits`, and
`supersedes` each got explicit treatment. `observes` joins none of them:
an observation claim is not a dependency, and two files mutually observing
each other's effects is well-formed. The edge set is *not* silently grown.

### D5 — Advisory first; the gating decision is deferred until after dogfooding

No corpus file uses `emits` today, so "every effect has an observer" passes
vacuously — the advisory-vs-invariant question cannot be answered honestly
in the abstract. Sequence: land the format Revision + advisory finding,
dogfood on a real worked example (`USAGE.md` §2 pattern), decide gating
from observed friction. The check never gates any lifecycle transition in
v1, and `orchestrate.md`'s stage guards are untouched.

### D6 — Cross-file check modeled on `linter-external_completeness`, not `linter-coverage`

The unobserved-effect check crosses file boundaries (observer in file A,
effect in file B). `linter-coverage`'s checks are intra-file
(`no_orphan_property` scopes to "this edge"). v1 ships the finding without
waiver machinery; when a waiver need is demonstrated,
`linter-external_completeness.md`'s covered/waived claim structure is the
candidate — not a second, invented waiver mechanism.

### D7 — Proposal structured on Revision 7's own checklist

`specodelic.md` Revision 7 pre-answered "what does this addition need
nothing new for" (guard-typing exclusion, `single_root_reachable`
carve-out-freedom, referential_integrity genericity). The same three checks
apply to `observes` and are pre-answered here:

- **guard already excludes effect from gating** — `guard`'s typing row is
  `kind == invariant only`; no new "cannot gate" invariant.
- **`single_root_reachable` needs no carve-out** — a Constraint row carrying
  `observes` keeps its ordinary `traces_to` to its own file's Intent;
  `observes` is an extra outbound pointer beside `traces_to`, like `guard`,
  `emits`, and `satisfies`.
- **`linter-referential_integrity.md` needs zero code changes** —
  `ref_kind_compatible` reads the Reference Typing table generically.

## Risks / Trade-offs

- *Vacuous enforcement at first* (no effect rows in corpus) → mitigated by
  the dogfooding task; a check nobody's corpus exercises is still cheaper
  than a mis-designed gating rule.
- *Column proliferation on the Constraints table* (`traces_to`, `satisfies`,
  `observes`) → acceptable: each is a distinct typed claim; D3's general
  rule makes further fields cheap. If a fourth appears, revisit a generic
  "claims" column with typed entries.
- *Semantics drift between effect-as-output and effect-as-telemetry* →
  documented in the Revision text: an `observes`-targeted effect is a
  declared observable; an unobserved effect is only *reported*, never
  reinterpreted.

## Migration Plan

Purely additive: existing files parse identically (the `observes` column is
optional), the new finding is advisory, and the boundary classification is
purely derived. Rollback = revert the extraction rule, the linter check, and
the graph classification; no corpus data migration exists or is needed.

## Open Questions

- **Gating** (deferred by D5): does `linter.observability` become an
  invariant gate after dogfooding, and if so, does it enter the orchestrate
  pipeline or stay a standalone lint check? Decided in a follow-up change.
- **Waivers** (deferred by D6): is `linter-external_completeness`'s
  checklist the waiver home when the need is demonstrated?
- **Subkind vs typing**: if telemetry claims need richer structure than an
  `observes` edge carries (metric name, cardinality), the answer is a new
  constraint subkind via `kind_field_extensible` — out of scope until the
  dogfood shows the edge is insufficient. (`kinds.md` CLAR-002's
  kind/subkind naming collision is live for any such widening; resolve it
  first per its own Notes.)
