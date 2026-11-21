## ADDED Requirements

### Requirement: Components are declared as first-class rows

The format SHALL define a `component` row kind — carried in a `## Components`
top-level section of a spec file, under the next free Revision heading of the
format doc — with the closed field set `{id, kind, level, scope, realizes}`
where `level` is drawn from the closed set `{context, container}`, `scope` is
drawn from the closed set `{internal, external}`, and `realizes` is a typed
reference column; and `spk lint` SHALL accept a spec file whose `## Components`
section declares component rows.

#### Scenario: A component row parses

- **WHEN** a spec file declares a component row with `level: container` and one resolvable `realizes` target
- **THEN** `spk lint` reports no finding for the row and `spk graph` emits the component node

#### Scenario: A level outside the closed set is rejected

- **WHEN** a component row carries `level: code`
- **THEN** the lint fails naming the row, the offending value, the closed set `{context, container}`, and the stale-levels remediation (levels 3–4 go stale — M4 K7/K9)

#### Scenario: A scope outside the closed set is rejected

- **WHEN** a component row carries `scope: shared`
- **THEN** the lint fails naming the row, the offending value, and the closed set `{internal, external}` — the format defines no bool column type, so externality is a named variant, not a flag

#### Scenario: A component row with an undeclared field is rejected

- **WHEN** a component row carries a field outside the closed field set
- **THEN** `row_shape_closed` fails naming the undeclared column

### Requirement: Realization edges resolve and stay structural

The `realizes` relation SHALL be a typed reference declared in the format doc's
Reference Typing table with source kind `Component` and target kinds
`Constraint` or `Intent`: every target SHALL resolve to a real row of a
kind-compatible type anywhere in the corpus, a component SHALL NOT reference
itself, and the graph formed by `realizes` edges SHALL contain no cycle.

#### Scenario: A dangling realization target fails

- **WHEN** a component's `realizes` names an id that resolves to no row anywhere in the corpus
- **THEN** the lint fails with a `total_refs` finding naming the component and the dangling target

#### Scenario: A kind-incompatible target fails typing

- **WHEN** a component realizes a target kind the Reference Typing row excludes (e.g. a State row)
- **THEN** the lint fails with a `ref_kind_compatible` finding naming the edge and the compatible kinds

#### Scenario: A realization cycle fails

- **WHEN** two components realize each other through `realizes` edges
- **THEN** the lint fails with an `acyclic` finding naming the cycle's members

### Requirement: Every container component is realized or marked external

`spk lint` SHALL report a finding when a `container` component has no
`realizes` edge and its `scope` is not `external`, so a declared part that
nothing realizes is visible — never silent machinery.

#### Scenario: An unrealized container component is flagged

- **WHEN** a container component declares no `realizes` target and its scope is `internal`
- **THEN** the lint reports the completeness finding naming the component and both remediations (add a `realizes` edge, or set `scope: external`)

#### Scenario: A container marked external passes

- **WHEN** a `container` component carries `scope: external` and no `realizes` edge
- **THEN** the lint reports no completeness finding for it

#### Scenario: A context component needs no realization

- **WHEN** a `context` component declares no `realizes` edge
- **THEN** the lint reports no completeness finding for it

### Requirement: Views are derived, digest-stamped, and bounded

The graph-views capability SHALL render a C4 levels-1–2 view at build time
from the graph artifact alone — never from prose, never committed — stamped
with the snapshot digest of the artifact it was rendered from, and the view
SHALL claim structural facts only: components, realization edges,
reachability, and unrealized parts. The view SHALL NOT claim behavioral
conformance, freshness beyond its digest, or code-level structure.

#### Scenario: The view names its snapshot

- **WHEN** the C4 view is rendered
- **THEN** it carries the digest of the graph artifact it was derived from, and re-rendering over an unchanged graph artifact is byte-identical

#### Scenario: An unrealized component appears annotated in the view

- **WHEN** the corpus contains a container component with no realization
- **THEN** the view renders it annotated (dashed), consistent with the violations-survive-projection discipline — never silently clean

#### Scenario: A levels-3–4 request is refused

- **WHEN** a view consumer requests level 3 or level 4 rendering
- **THEN** the renderer refuses with a remediation naming the closed level set `{context, container}` — a level that cannot be checked must not be declarable
