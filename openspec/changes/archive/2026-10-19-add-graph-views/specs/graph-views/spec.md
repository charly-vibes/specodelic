# graph-views Specification

## Purpose
Make the typed reference graph of any specodelic-compliant corpus
thinkable-with rather than merely machine-readable: a deterministic,
pipeline-friendly edge list as the single projection boundary, and
human-readable views (per-file state machines, file-level traceability,
the schema's own shape) rendered from that boundary at build time. Views
are derived facts — never authored, never committed, never cleaner than
the artifact they came from.

## ADDED Requirements

### Requirement: Deterministic edge-list projection
The system SHALL provide `spk graph --format edges` emitting a sorted TSV
edge list (source id, source kind, typed reference field, target kind,
target id, annotation) with exactly one row per recorded edge — plus
annotation rows per the violations requirement below — using canonical node IDs (intent IDs and qualified row IDs), byte-identical across re-runs on an unchanged corpus. When
`--format edges` is given, raw TSV goes to stdout, overriding envelope
formatting.

#### Scenario: Projection over a compliant corpus
- **WHEN** `spk graph --format edges` runs over a corpus with ≥1 lint-clean intent
- **THEN** the output is a sorted TSV whose every endpoint is a canonical node ID (intent ID or qualified row ID)

#### Scenario: Re-run determinism
- **WHEN** the projection runs twice over an unchanged corpus
- **THEN** both outputs are byte-identical

#### Scenario: Display labels excluded
- **WHEN** the corpus contains label-qualified pseudo-nodes in the raw graph output
- **THEN** the projection contains only canonical ids — no label-qualified endpoints

### Requirement: Violations never silently hidden
The edge-list projection SHALL include one annotation row per violation in
the graph artifact, and views SHALL render violations as annotated (not
omitted, not silently clean) elements.

#### Scenario: Violation-bearing corpus projects annotated rows
- **WHEN** the graph artifact reports N typing violations
- **THEN** the edge list contains exactly N annotation rows

#### Scenario: Views annotate rather than omit
- **WHEN** a view is rendered from an artifact carrying violations
- **THEN** each violation appears rendered as a dashed/annotated element

### Requirement: Derived per-file state-machine view
The system SHALL derive, per corpus file whose artifact contains
transition edges, a state-machine view from `transitions.from`,
`transitions.to`, and `transitions.guard` edges alone — no re-walking of
markdown.

#### Scenario: State machine from edges only
- **WHEN** a file's spec defines states and guarded transitions
- **THEN** the derived view shows the same states, transitions, and guard annotations as the file's Model tables

#### Scenario: File without transitions skipped cleanly
- **WHEN** a file has no transition edges
- **THEN** no state-machine view is emitted for it and the build succeeds

### Requirement: Derived file-level traceability view
The system SHALL derive a corpus-level view collapsing all edges to intent
(file) granularity, annotating fan-in per intent, from the edge list alone.

#### Scenario: Collapse to intents
- **WHEN** the projection is derived from the specodelic corpus
- **THEN** the view's node set equals the corpus's intent set and edges connect only intents

#### Scenario: Fan-in annotation
- **WHEN** an intent is the target of edges from k distinct intents
- **THEN** the view annotates that intent with fan-in k

### Requirement: Derived revision-labeled schema view
The system SHALL derive the schema view (kinds as nodes, typed reference
fields as typed edges with allowed targets) from the acset `Schema`
value (`acset::schema::canonical()` — the Reference Typing table as
data, lint-gated against `openspec/specs/specodelic/spec.md` by
`schema_matches_typing_table`) and the format revision marker, labeled
with the revision it was derived from.

#### Scenario: Schema from the Schema value
- **WHEN** the schema view is rendered
- **THEN** its nodes and typed edges match the `Schema` value's objects and morphisms, and the format revision is named in the output

### Requirement: The schema export is a typed genesis envelope
`spk guide --schema --json` SHALL provide the schema view through a
genesis envelope whose data contains schema_version 1, format_revision,
sorted object names, and morphisms sorted by source/name. Each morphism
SHALL carry name, column, source, target, refinements (side/kind pairs
sorted by side/kind), source_rule (unchecked, appears_on or same_kind),
and endo_acyclic (boolean or null).

#### Scenario: Schema export connects producer and renderer
- **WHEN** the production exporter serializes canonical Schema and two valid constructed schemas differing in one morphism
- **THEN** its canonical payload matches the source rows and rendering the two constructed exports changes the corresponding edge without a renderer edit

### Requirement: The schema renderer validates the export
The schema renderer SHALL consume the serialized export, display
refinement labels and reject unsuccessful envelopes, missing fields,
unknown versions, duplicate identities and absent endpoints with
schema_export_invalid and a regeneration hint before writing output.
It SHALL neither parse Rust source nor reconstruct schema structure
from corpus edges.

#### Scenario: Invalid schema export is refused
- **WHEN** the renderer receives an unsuccessful envelope or schema data with a missing field, unknown version, duplicate identity or absent endpoint
- **THEN** it fails as schema_export_invalid with a regeneration hint before writing a diagram

### Requirement: The schema export derives from canonical Schema alone
The export SHALL derive directly from canonical Schema; guide typing
constants SHALL NOT supply its rows. Ordinary guide --json SHALL expose
kinds, row shapes and revision independently.

#### Scenario: Revision bump changes the view
- **WHEN** a new Revision adds a Reference Typing field (Schema + corpus doc updated together under the lint gate)
- **THEN** the regenerated schema view includes the new field without any renderer change

### Requirement: Build-time generation, never committed
Rendered views SHALL be generated into the docs build at build time via a
`just docs-graphs` recipe and SHALL NOT be committed to version control;
no hand-edit path exists.

#### Scenario: Build leaves the worktree clean
- **WHEN** `just docs-graphs` runs and the docs build completes
- **THEN** `git status` reports no new or modified tracked files from view generation

#### Scenario: Empty corpus is refused at the transform, clean at the projection
- **WHEN** views are generated over a directory with zero spec files
- **THEN** the scope gate exits non-zero with a remediation hint (`out_of_scope_refused`), while the tool-level projection itself exits 0 with empty output (`empty_corpus_valid`)

### Requirement: Row identity survives raw projection
Raw edge projection SHALL retain canonical intent IDs and qualified row
IDs without collapsing rows to their file. File-level traceability and
wiring views MAY collapse rows to their owning intent; state-machine
views SHALL retain distinct state and transition identities.

#### Scenario: Two states in one file remain distinct
- **WHEN** one file defines transition order.t from order.s1 to order.s2
- **THEN** raw projection retains both distinct state IDs and both edges, and the state-machine view renders two states
