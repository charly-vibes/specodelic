---
id: graph.views
kind: intent
statement: "THE graph-views capability SHALL emit a deterministic edge-list projection with canonical ids and annotated typing violations from any specodelic-compliant corpus, and SHALL derive per-file state-machine, file-level traceability, and revision-labeled schema views from that artifact plus the lint-gated acset Schema value and the format revision marker alone — never from prose."
---

# graph-views Specification

## Purpose
Make the typed reference graph of any specodelic-compliant corpus
thinkable-with rather than merely machine-readable: a deterministic,
pipeline-friendly edge list as the single projection boundary, and
human-readable views (per-file state machines, file-level traceability,
the schema's own shape) rendered from that boundary at build time. Views
are derived facts — never authored, never committed, never cleaner than
the artifact they came from.

## Constraints

| id                       | kind      | expr                                                                                                                                                                                                                                                                  | traces_to |
|--------------------------|-----------|-------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|-----------|
| process_lifecycle | invariant | `the capability advances through its declared lifecycle states under the repo's change process — each stage transition fires only when its stage gate holds` | [[graph.views]] |
| canonical_ids            | invariant | `every edge-list endpoint is a canonical node ID (intent ID or qualified row ID) value; display labels (e.g. `refactor (intent)`-style label-qualified nodes) never appear in the projection — normalization happens at or before projection`                                                                 | [[graph.views]]  |
| deterministic_projection | invariant | `re-running the projection over an unchanged corpus produces a byte-identical edge list — rows sorted, no timestamps, no iteration-order leakage`                                                                                                                        | [[graph.views]]  |
| violations_annotated     | invariant | `every violation in the graph artifact appears as an annotation row in the edge list; a view rendered from a corpus with violations is never silently clean — dashed/annotated rendering is mandatory, omission forbidden`                                                | [[graph.views]]  |
| derived_views_only       | invariant | `corpus views consume only graph artifacts; the schema view consumes only a versioned export of the lint-gated acset Schema value and format revision; no view parses prose, re-walks markdown, or embeds hand-authored structure`                                                                                            | [[graph.views]]  |
| build_time_generation    | invariant | `rendered views are generated into the docs build at build time and never committed; no hand-edit path exists, so graph_is_derived_not_authored holds by construction and no staleness check is needed`                                                                    | [[graph.views]]  |
| empty_corpus_valid       | invariant | `spk graph --format edges over any parseable corpus — including a directory with zero spec files, or a single intent with no cross-file edges — exits 0 and emits a well-formed (possibly empty) row set: graph extraction requires parsed, not linted (specs/graph.md's scope note); rendered views are transform-level and additionally gated by corpus_scope_operational — an intentless corpus is refused there with a remediation hint, never silently rendered`                                                        | [[graph.views]]  |
| corpus_scope_operational | invariant | `a corpus is in scope iff spk lint over it reports no invariant-rule findings and ≥1 intent file parses; corpora failing this (e.g. openspec-layout repositories) are out of scope until a parse-boundary adapter change lands — no view special-cases them`                                | [[graph.views]]  |

## Model

### States
- `proposed`
- `approved`
- `implemented`
- `archived`

### Transitions

| id          | from        | to          | guard                                                                                     |
|-------------|-------------|-------------|-------------------------------------------------------------------------------------------|
| approve     | proposed    | approved    | [[graph.views.process_lifecycle]] ∧ `proposal reviewed and approved by the maintainer`  |
| implement   | approved    | implemented | [[graph.views.process_lifecycle]] ∧ `all tasks.md items complete; spk lint, transform tests, and openspec validate --strict pass`  |
| archive     | implemented | archived    | [[graph.views.process_lifecycle]] ∧ `just archive-change id=add-graph-views ran with the dual-format recipe`  |

## Properties

| id                            | kind | derives_from                     | generator                                             | predicate                                                                 |
|-------------------------------|------|-----------------------------------|-----------------------------------------------------------------------------------------------------|
| process_lifecycle_checked | unit | [[graph.views.process_lifecycle]] | `lifecycle_model_present()` | `check(file) == passed` |
| ids_never_labels              | unit | [[graph.views.canonical_ids]]           | `corpus_parsed_then_projected()`                      | `no TSV field matches a label-qualified node pattern`                     |
| rerun_byte_identical          | unit | [[graph.views.deterministic_projection]] | `same_corpus_projected_twice()`                       | `both runs byte-identical; also holds across invocation order of files`   |
| violations_survive_projection | unit | [[graph.views.violations_annotated]]    | `corpus_with_known_typing_violations()`               | `edge list contains one annotation row per violation, none missing`       |
| clean_corpus_no_annotations   | unit | [[graph.views.violations_annotated]]    | `lint_clean_corpus()`                                 | `edge list contains zero annotation rows`                                 |
| views_from_artifact_only      | unit | [[graph.views.derived_views_only]]      | `view_rendered_with_prose_perturbation()`             | `rendered view byte-identical after perturbing prose blocks`              |
| schema_view_revision_labeled  | unit | [[graph.views.derived_views_only]]      | `schema_value_rendered()`                         | `schema view names the format revision it was derived from`               |
| build_output_not_committed    | unit | [[graph.views.build_time_generation]]   | `docs_build_completed()`                              | `git status clean — no rendered artifact tracked or staged`               |
| empty_corpus_projection       | unit | [[graph.views.empty_corpus_valid]]      | `directory_with_zero_spec_files()`                    | `projection exits 0 with empty output — the view half is governed by corpus_scope_operational (an intentless corpus is refused: out_of_scope_refused)` |
| single_intent_sane            | unit | [[graph.views.empty_corpus_valid]]      | `corpus_with_one_intent_no_cross_file_edges()`        | `file-level view renders one node; no crash`                              |
| out_of_scope_refused          | unit | [[graph.views.corpus_scope_operational]] | `corpus_with_invariant_findings_or_no_intents()`      | `transform exits non-zero with a remediation hint naming the failed gate` |

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
data, lint-gated against `specs/specodelic.md` by
`schema_matches_typing_table`) and the format revision marker, labeled
with the revision it was derived from. `spk guide --schema --json` SHALL
provide this value through a genesis envelope whose data contains
schema_version 1, format_revision, sorted object names, and morphisms sorted
by source/name. Each morphism SHALL carry name, column, source, target,
refinements (side/kind pairs sorted by side/kind), source_rule
(unchecked, appears_on or same_kind), and endo_acyclic (boolean or null).
The export SHALL derive directly from canonical Schema; guide typing
constants SHALL NOT supply its rows. Ordinary guide --json SHALL expose
kinds, row shapes and revision independently.

The schema renderer SHALL consume the serialized export, display refinement
labels and reject unsuccessful envelopes, missing fields, unknown versions,
duplicate identities and absent endpoints with schema_export_invalid and a
regeneration hint before writing output. It SHALL neither parse Rust source
nor reconstruct schema structure from corpus edges.

#### Scenario: Schema from the Schema value
- **WHEN** the schema view is rendered
- **THEN** its nodes and typed edges match the `Schema` value's objects and morphisms, and the format revision is named in the output

#### Scenario: Schema export connects producer and renderer
- **WHEN** the production exporter serializes canonical Schema and two valid constructed schemas differing in one morphism
- **THEN** its canonical payload matches the source rows and rendering the two constructed exports changes the corresponding edge without a renderer edit

#### Scenario: Invalid schema export is refused
- **WHEN** the renderer receives an unsuccessful envelope or schema data with a missing field, unknown version, duplicate identity or absent endpoint
- **THEN** it fails as schema_export_invalid with a regeneration hint before writing a diagram

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

## Requirements

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
data, lint-gated against `specs/specodelic.md` by
`schema_matches_typing_table`) and the format revision marker, labeled
with the revision it was derived from. `spk guide --schema --json` SHALL
provide this value through a genesis envelope whose data contains
schema_version 1, format_revision, sorted object names, and morphisms sorted
by source/name. Each morphism SHALL carry name, column, source, target,
refinements (side/kind pairs sorted by side/kind), source_rule
(unchecked, appears_on or same_kind), and endo_acyclic (boolean or null).
The export SHALL derive directly from canonical Schema; guide typing
constants SHALL NOT supply its rows. Ordinary guide --json SHALL expose
kinds, row shapes and revision independently.

The schema renderer SHALL consume the serialized export, display refinement
labels and reject unsuccessful envelopes, missing fields, unknown versions,
duplicate identities and absent endpoints with schema_export_invalid and a
regeneration hint before writing output. It SHALL neither parse Rust source
nor reconstruct schema structure from corpus edges.

#### Scenario: Schema from the Schema value
- **WHEN** the schema view is rendered
- **THEN** its nodes and typed edges match the `Schema` value's objects and morphisms, and the format revision is named in the output

#### Scenario: Schema export connects producer and renderer
- **WHEN** the production exporter serializes canonical Schema and two valid constructed schemas differing in one morphism
- **THEN** its canonical payload matches the source rows and rendering the two constructed exports changes the corresponding edge without a renderer edit

#### Scenario: Invalid schema export is refused
- **WHEN** the renderer receives an unsuccessful envelope or schema data with a missing field, unknown version, duplicate identity or absent endpoint
- **THEN** it fails as schema_export_invalid with a regeneration hint before writing a diagram

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
