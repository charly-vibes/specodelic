---
id: spec
kind: intent
statement: "WHEN a workspace enables the data/lineage standard pack, THE format SHALL provide the pack as a declared kind: profile artifact — a namespaced ## Data section, data.dataset/data.artifact/data.environment kinds, produced_by/consumed_by/produces lineage reference fields resolving to declared ## Data rows, and a binding column that bridges external data standards without absorbing them into any base closed set — while files not using the pack's vocabulary lint byte-identically."
---

# data-lineage-pack Specification

## Purpose

Ship the first standard pack on the pack mechanism: a data/lineage pack
declaring a typed `## Data` table, namespaced dataset/artifact/environment
kinds, and lineage edges (`produced_by`/`consumed_by`/`produces`) whose
external binding is typed but opaque. External data standards stay
authoritative — the pack bridges, never absorbs. It is the mechanism's
first real instance and the prerequisite for the bioimage D6 pilot.

## Constraints

| id                       | kind      | expr                                                                                                                                                                                                                                       | traces_to |
|--------------------------|-----------|--------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|-----------|
| pack_file_declared       | invariant | `the data/lineage standard pack is a kind: profile spec file at packs/data-lineage.md (id data.lineage) whose six manifest tables declare the ## Data section, the data.dataset/data.artifact/data.environment kinds, the produces/produced_by/consumed_by reference fields, the data.lineage.closure checker, the artifact provenance floor, and the Revision 14 base pin` | [[spec]]  |
| data_section_row_shape   | invariant | `## Data rows carry exactly \| name \| kind \| dtype \| units \| binding \| with kind closed to dataset, artifact, environment — a per-pack closed set; no base closed set grows and no existing entry narrows`                             | [[spec]]  |
| lineage_edges_outbound   | invariant | `produces, produced_by, and consumed_by are pack-added reference fields resolving to a ## Data row of the declaring file — outbound leaves joining no reachability path and no acyclic edge set; a dangling lineage reference is a labeled finding naming the row id and both remediations, never a generic dangling message` | [[spec]]  |
| binding_stays_external   | invariant | `the binding column carries a typed pointer to an external data standard and the format never parses it — no dtype enum, shape convention, or URI scheme joins a base closed set, and lint behavior is identical regardless of the binding target` | [[spec]]  |

## Model

### States
- `draft`
- `published`
- `deprecated`

### Transitions

| id        | from     | to         | guard                                                                                      |
|-----------|----------|------------|--------------------------------------------------------------------------------------------|
| publish   | draft    | published  | `[[spec.pack_file_declared]] ∧ [[spec.data_section_row_shape]] — pack_shape reports zero findings over the pack's declared vocabulary` |
| deprecate | published| deprecated | `[[spec.pack_file_declared]] — maintainer marks the pack deprecated; findings name the deprecation, vocabulary still checks` |

## Properties

| id                             | kind | derives_from                     | generator                                       | predicate                                                                    |
|--------------------------------|------|----------------------------------|-------------------------------------------------|------------------------------------------------------------------------------|
| discovery_finds_lineage_pack   | unit | [[spec.pack_file_declared]]      | `lint_workspace_with_pack_present()`            | `the pack is discovered by corpus scan alone, no config file, no registry`    |
| pack_shape_clean_all_states    | unit | [[spec.pack_file_declared]]      | `pack_parsed_in_every_lifecycle_state()`        | `pack_shape reports zero findings over the declared vocabulary in draft, published, and deprecated` |
| consumer_activates_advisory    | unit | [[spec.data_section_row_shape]]  | `corpus_spec_uses_data_dataset_vocabulary()`    | `the pack's checkers activate advisory-first naming the pack; findings attributed per pack` |
| orphan_names_lineage_pack      | unit | [[spec.data_section_row_shape]]  | `workspace_uses_data_vocabulary_without_pack()` | `the orphan finding names the candidate pack data.lineage and both remediations, exit non-zero` |
| lineage_reference_resolves     | unit | [[spec.lineage_edges_outbound]]  | `constraint_row_produced_by_declared_data_row()`| `the lineage reference resolves to the declared ## Data row; a dangling reference is a labeled finding naming the row id and both remediations` |
| closure_honest_empty           | unit | [[spec.lineage_edges_outbound]]  | `pack_active_without_data_rows()`               | `the closure checker reports an empty checked-set; no fabricated findings`    |
| no_base_set_absorption         | unit | [[spec.binding_stays_external]]  | `guide_sets_compared_with_and_without_data_pack()` | `INTENT_KINDS and Reference Typing sets identical with the pack discovered or not` |
| binding_opaque_to_lint         | unit | [[spec.binding_stays_external]]  | `data_row_binds_to_ome_zarr_vs_parquet()`       | `lint findings byte-identical across different binding targets`               |

## ADDED Requirements

### Requirement: The data/lineage standard pack is a declared profile file
The format SHALL ship the data/lineage standard pack as an in-repo
`kind: profile` spec file (`packs/data-lineage.md`, id `data.lineage`)
whose manifest declares the `## Data` section, the `data.dataset`,
`data.artifact`, and `data.environment` kinds, the `produces`,
`produced_by`, and `consumed_by` reference fields, the
`data.lineage.closure` checker, the artifact provenance floor, and the
`specodelic.md Revision 14` base pin — discovered by corpus scan alone,
with no config file and no registry.

#### Scenario: Corpus scan discovers the pack
- **WHEN** `spk lint` runs over any workspace subdirectory of a repository containing `packs/data-lineage.md`
- **THEN** the pack is discovered without configuration and reported in the lint envelope
- **AND** files using none of the pack's vocabulary keep byte-identical lint findings

#### Scenario: Pack lints clean under pack_shape in every lifecycle state
- **WHEN** the pack file is parsed in draft, published, or deprecated state
- **THEN** pack_shape reports zero findings over the pack's declared vocabulary

### Requirement: Data vocabulary is namespaced and per-pack closed
The pack SHALL declare a `## Data` section with row shape
`| name | kind | dtype | units | binding |`, kind closed to `dataset`,
`artifact`, `environment`, and namespaced kinds `data.dataset`,
`data.artifact`, `data.environment` — pack-fiber-relative vocabulary that
grows no base closed set and narrows no existing base set entry.

#### Scenario: Consumer activates the pack advisory-first
- **WHEN** a corpus spec uses `data.dataset` vocabulary without a declared `uses` edge
- **THEN** the pack's declared checking activates advisory-first for that file, naming the pack

#### Scenario: Orphan vocabulary names the lineage pack
- **WHEN** a workspace uses `data.dataset` vocabulary with no data/lineage pack discovered or declared
- **THEN** the orphan finding names the candidate pack `data.lineage` and both remediations, with non-zero exit

### Requirement: Lineage edges are typed outbound leaves resolving to declared rows
The pack SHALL add `produces`, `produced_by`, and `consumed_by` reference
fields, each resolving to a `## Data` row of the declaring file and
joining no reachability path and no acyclic edge set, and SHALL declare a
`data.lineage.closure` checker that reports an empty checked-set honestly
when no `## Data` rows exist.

#### Scenario: Dangling lineage reference is labeled
- **WHEN** a constraint row references a `## Data` row id via `produced_by` that no `## Data` row in the file declares
- **THEN** the finding is labeled, names the row id and both remediations, and is never a generic dangling message

#### Scenario: Honest-empty closure with no data rows
- **WHEN** the pack is active in a workspace whose specs declare no `## Data` rows
- **THEN** the closure checker reports an empty checked-set with no fabricated findings

### Requirement: External binding stays external
The pack SHALL bridge, never absorb, external data standards: the
`binding` column carries a typed pointer to a schema, URI, or standard
reference that the format never parses, and no dtype enum, shape
convention, or URI scheme ever joins a base closed set — lint behavior
is identical regardless of the binding target.

#### Scenario: Any external standard binds without lint change
- **WHEN** a `## Data` row binds to OME-Zarr, Parquet, or any other external standard
- **THEN** lint findings are byte-identical across the different binding targets

## Requirements

### Requirement: The data/lineage standard pack is a declared profile file
The format SHALL ship the data/lineage standard pack as an in-repo
`kind: profile` spec file (`packs/data-lineage.md`, id `data.lineage`)
whose manifest declares the `## Data` section, the `data.dataset`,
`data.artifact`, and `data.environment` kinds, the `produces`,
`produced_by`, and `consumed_by` reference fields, the
`data.lineage.closure` checker, the artifact provenance floor, and the
`specodelic.md Revision 14` base pin — discovered by corpus scan alone,
with no config file and no registry.

#### Scenario: Corpus scan discovers the pack
- **WHEN** `spk lint` runs over any workspace subdirectory of a repository containing `packs/data-lineage.md`
- **THEN** the pack is discovered without configuration and reported in the lint envelope
- **AND** files using none of the pack's vocabulary keep byte-identical lint findings

#### Scenario: Pack lints clean under pack_shape in every lifecycle state
- **WHEN** the pack file is parsed in draft, published, or deprecated state
- **THEN** pack_shape reports zero findings over the pack's declared vocabulary

### Requirement: Data vocabulary is namespaced and per-pack closed
The pack SHALL declare a `## Data` section with row shape
`| name | kind | dtype | units | binding |`, kind closed to `dataset`,
`artifact`, `environment`, and namespaced kinds `data.dataset`,
`data.artifact`, `data.environment` — pack-fiber-relative vocabulary that
grows no base closed set and narrows no existing base set entry.

#### Scenario: Consumer activates the pack advisory-first
- **WHEN** a corpus spec uses `data.dataset` vocabulary without a declared `uses` edge
- **THEN** the pack's declared checking activates advisory-first for that file, naming the pack

#### Scenario: Orphan vocabulary names the lineage pack
- **WHEN** a workspace uses `data.dataset` vocabulary with no data/lineage pack discovered or declared
- **THEN** the orphan finding names the candidate pack `data.lineage` and both remediations, with non-zero exit

### Requirement: Lineage edges are typed outbound leaves resolving to declared rows
The pack SHALL add `produces`, `produced_by`, and `consumed_by` reference
fields, each resolving to a `## Data` row of the declaring file and
joining no reachability path and no acyclic edge set, and SHALL declare a
`data.lineage.closure` checker that reports an empty checked-set honestly
when no `## Data` rows exist.

#### Scenario: Dangling lineage reference is labeled
- **WHEN** a constraint row references a `## Data` row id via `produced_by` that no `## Data` row in the file declares
- **THEN** the finding is labeled, names the row id and both remediations, and is never a generic dangling message

#### Scenario: Honest-empty closure with no data rows
- **WHEN** the pack is active in a workspace whose specs declare no `## Data` rows
- **THEN** the closure checker reports an empty checked-set with no fabricated findings

### Requirement: External binding stays external
The pack SHALL bridge, never absorb, external data standards: the
`binding` column carries a typed pointer to a schema, URI, or standard
reference that the format never parses, and no dtype enum, shape
convention, or URI scheme ever joins a base closed set — lint behavior
is identical regardless of the binding target.

#### Scenario: Any external standard binds without lint change
- **WHEN** a `## Data` row binds to OME-Zarr, Parquet, or any other external standard
- **THEN** lint findings are byte-identical across the different binding targets