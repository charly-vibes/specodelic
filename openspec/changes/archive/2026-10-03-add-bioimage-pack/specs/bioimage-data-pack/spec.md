# bioimage-data-pack Specification

## Purpose

Ship the bioimage-data domain pack — the D6 pilot for the domain-pack
mechanism: the thin instance that stress-tests what the three standard
packs could not exercise alone. It consumes the data/lineage pack's
`## Data` section (`same_shape_as` resolution, the `dtype_is` predicate
class), the numeric-predicates pack's opaque unit domain
(`units_convertible` bridging), and the empirical-registry pack's
stat-test rows (empirically held transform claims) via the mechanism's
first cross-pack `## Requires` table. OME/NGFF and BioImage.IO stay
authoritative external standards — the pack bridges them through opaque
`scale`/`unit` columns, it never absorbs them. Checkers ship as
declarations (honest-empty): this pack declares vocabulary, the
mechanism (`packs.md`) interprets it.

## Constraints

| id                        | kind      | expr                                                                                                                                                                                                                                                                               | traces_to |
|---------------------------|-----------|-------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|-----------|
| pack_file_declared        | invariant | `the bioimage-data domain pack is a kind: profile spec file at packs/bioimage-data.md (id bioimage.data) whose six manifest tables declare the ## Axes section, the bioimage.transform kind, the same_shape_as reference field, the bioimage.dtype_is, bioimage.shape_eq, and bioimage.units_convertible checkers, the transform case-label floor, and the Requires table pinning base specodelic.md Revision 14 plus the data.lineage, numeric.predicates, and empirical.registry pack deps` | [[bioimage.data.pack]]  |
| axes_section_shape        | invariant | `## Axes rows carry exactly \| name \| axis \| scale \| unit \| — the workspace's declared image axes (OME-NGFF-style semantics); the scale and unit columns are opaque prose the format never parses, and no base closed set grows and no existing entry narrows`                                                                                    | [[bioimage.data.pack]]  |
| same_shape_as_cross_pack  | invariant | `same_shape_as is a pack-added reference field resolving to a ## Data row of the declaring file — the data-lineage pack's section, the pack's cross-pack-ness living in its Requires pack deps while the resolution stays intra-file, an outbound leaf joining no reachability path and no acyclic edge set; a dangling reference is a labeled finding naming the row id and both remediations, never a generic dangling message` | [[bioimage.data.pack]]  |
| predicates_closed_named   | invariant | `the R2 data-shaped predicate grammar is declared as the named pack-qualified checkers bioimage.dtype_is, bioimage.shape_eq, and bioimage.units_convertible — a closed set over declared ## Data rows, declaration-only (the mechanism names them, never executes them), honest-empty with no ## Data rows, and the bare predicate tokens are never activation vocabulary` | [[bioimage.data.pack]]  |
| transform_floor_kind_dependent | invariant | `the per-kind floor requires bioimage.transform properties to enumerate their preserves and dtype case labels — the kind-dependent floor the base law floor's non-universality critique motivated; the base law floor is byte-identical with the pack discovered, a kind with no declared floor inherits none, the floor is a floor not a ceiling (labels from consumed packs ride along additively), and floor labels are never activation vocabulary` | [[bioimage.data.pack]]  |
| kind_column_typeable      | invariant | `when the pack is active, base-table kind columns accept its pack-qualified fiber kinds — property_kind_closed's effective set is base ∪ active-pack-fiber, never narrower; without the pack active the token stays outside the closed set and the labeled finding fires — never a silent pass` | [[bioimage.data.pack]]  |
| requires_declares_pack_deps | invariant | `the pack-authored ## Requires table pins the base format_revision and names the data.lineage, numeric.predicates, and empirical.registry pack deps with the base corpus revision each was verified against — the mechanism's first cross-pack Requires consumer; a dep with no discovered pack is a labeled advisory, never silent` | [[bioimage.data.pack]]  |
| vocabulary_prose_safe     | invariant | `no pack vocabulary token is a bare English word that appears in corpus prose — bare bioimage appears in corpus prose as documented asymmetry (dotted tokens cannot match prose words), pipeline is prose-heavy and rejected, the predicate names ride pack-qualified checker rule names, Axes and same_shape_as and preserves are verified absent from the corpus, and dtype survives only as an opaque column header and floor case label — corpus lint warnings stay byte-identical with the pack discovered` | [[bioimage.data.pack]]  |

## Model

### States
- `draft`
- `published`
- `deprecated`

### Transitions

| id        | from     | to         | guard                                                                                      |
|-----------|----------|------------|--------------------------------------------------------------------------------------------|
| publish   | draft    | published  | `[[bioimage.data.pack.pack_file_declared]] ∧ [[bioimage.data.pack.axes_section_shape]] — pack_shape reports zero findings over the pack's declared vocabulary` |
| deprecate | published| deprecated | `[[bioimage.data.pack.pack_file_declared]] — maintainer marks the pack deprecated; findings name the deprecation, vocabulary still checks` |

## Properties

| id                              | kind | derives_from                       | generator                                              | predicate                                                                     |
|---------------------------------|------|-------------------------------------|--------------------------------------------------------|-------------------------------------------------------------------------------|
| discovery_finds_bioimage_pack   | unit | [[bioimage.data.pack.pack_file_declared]]         | `lint_workspace_with_bioimage_pack_present()`          | `the pack is discovered by corpus scan alone, no config file, no registry`     |
| pack_shape_clean_all_states     | unit | [[bioimage.data.pack.pack_file_declared]]         | `pack_parsed_in_every_lifecycle_state()`               | `pack_shape reports zero findings over the declared vocabulary in draft, published, and deprecated` |
| consumer_activates_advisory     | unit | [[bioimage.data.pack.axes_section_shape]]         | `corpus_spec_uses_bioimage_vocabulary()`               | `the pack's checkers activate advisory-first naming the pack; findings attributed per pack` |
| orphan_names_bioimage_pack      | unit | [[bioimage.data.pack.axes_section_shape]]         | `workspace_uses_bioimage_vocabulary_without_pack()`    | `the orphan finding names the candidate pack bioimage.data and both remediations, exit non-zero` |
| same_shape_as_resolves          | unit | [[bioimage.data.pack.same_shape_as_cross_pack]]   | `constraint_row_same_shape_as_declared_data_row()`     | `the same_shape_as reference resolves to the declared ## Data row of the same file; a dangling reference is a labeled finding naming the row id and both remediations` |
| requires_deps_resolve           | unit | [[bioimage.data.pack.requires_declares_pack_deps]] | `bioimage_pack_requires_table_parsed_with_deps()`     | `the pack deps resolve to discovered pack ids; a dep with no discovered pack is a labeled advisory naming it, never silent` |
| predicates_honest_empty         | unit | [[bioimage.data.pack.predicates_closed_named]]    | `pack_active_without_data_rows()`                      | `the predicate checkers report an empty checked-set with no ## Data rows; no fabricated findings; the bare predicate tokens are never declared` |
| transform_floor_labels          | unit | [[bioimage.data.pack.transform_floor_kind_dependent]] | `transform_property_missing_case_label()`          | `a bioimage.transform property owes **preserves:** and **dtype:** case labels per the floor declaration` |
| law_floor_unchanged             | unit | [[bioimage.data.pack.transform_floor_kind_dependent]] | `law_property_linted_with_and_without_pack()`      | `law floor findings byte-identical with the pack discovered; a kind with no floor inherits none` |
| kind_column_accepts_fiber_kinds | unit | [[bioimage.data.pack.kind_column_typeable]]       | `property_row_typed_bioimage_transform_with_pack_active()` | `property_kind_closed accepts the fiber kind when the pack is active; the same row without the pack fires the labeled finding` |
| no_bare_word_vocabulary         | unit | [[bioimage.data.pack.vocabulary_prose_safe]]      | `corpus_linted_with_and_without_bioimage_pack()`       | `lint warnings byte-identical for every corpus file — no vocabulary-match activations from prose words` |

## ADDED Requirements

### Requirement: The bioimage-data domain pack is a declared profile file
The format SHALL ship the bioimage-data domain pack as an in-repo
`kind: profile` spec file (`packs/bioimage-data.md`, id `bioimage.data`)
whose manifest declares the `## Axes` section, the `bioimage.transform`
kind, the `same_shape_as` reference field, the `bioimage.dtype_is`,
`bioimage.shape_eq`, and `bioimage.units_convertible` checkers, the
transform case-label floor, and the `## Requires` table pinning the
`specodelic.md Revision 14` base plus the `data.lineage`,
`numeric.predicates`, and `empirical.registry` pack deps — discovered by
corpus scan alone, with no config file and no registry.

#### Scenario: Corpus scan discovers the pack
- **WHEN** `spk lint` runs over any workspace subdirectory of a repository containing `packs/bioimage-data.md`
- **THEN** the pack is discovered without configuration and reported in the lint envelope
- **AND** files using none of the pack's vocabulary keep byte-identical lint findings

#### Scenario: Pack lints clean under pack_shape in every lifecycle state
- **WHEN** the pack file is parsed in draft, published, or deprecated state
- **THEN** pack_shape reports zero findings over the pack's declared vocabulary

### Requirement: The Axes registry is namespaced with opaque scale and unit
The pack SHALL declare a `## Axes` section with row shape
`| name | axis | scale | unit |` — the workspace's declared image axes
with OME-NGFF-style semantics — and the namespaced kind
`bioimage.transform`, pack-fiber-relative vocabulary that grows no base
closed set and narrows no existing base set entry; the `scale` and
`unit` columns stay opaque prose the format never parses, keeping OME/NGFF
and BioImage.IO authoritative as external standards.

#### Scenario: Consumer activates the pack advisory-first
- **WHEN** a corpus spec uses `bioimage.transform` or `## Axes` vocabulary without a declared `uses` edge
- **THEN** the pack's declared checking activates advisory-first for that file, naming the pack

#### Scenario: Orphan vocabulary names the bioimage pack
- **WHEN** a workspace uses `bioimage.transform` vocabulary with no bioimage-data pack discovered or declared
- **THEN** the orphan finding names the candidate pack `bioimage.data` and both remediations, with non-zero exit

### Requirement: same_shape_as is a typed outbound leaf resolving cross-pack, file-locally
The pack SHALL add the `same_shape_as` reference field, resolving to a
`## Data` row of the declaring file — the data-lineage pack's section —
with the pack dependency living in the pack's `## Requires` table while
the resolution itself stays intra-file, joining no reachability path and
no acyclic edge set; a dangling reference is a labeled finding naming the
row id and both remediations.

#### Scenario: Cross-pack section resolution stays file-local
- **WHEN** a constraint row declares `same_shape_as` pointing at a `## Data` row declared in the same file, with the data-lineage pack active for the file
- **THEN** the reference resolves with no corpus-wide pass and no reachability join

#### Scenario: Dangling same_shape_as reference is labeled
- **WHEN** a row references a `## Data` row id via `same_shape_as` that no `## Data` row in the file declares
- **THEN** the finding is labeled, names the row id and both remediations, and is never a generic dangling message

### Requirement: The R2 predicate grammar is a declared closed set of named checkers
The pack SHALL declare the data-shaped predicate grammar as the named
pack-qualified checkers `bioimage.dtype_is`, `bioimage.shape_eq`, and
`bioimage.units_convertible` — a closed set resolving against declared
`## Data` rows — declaration-only, with the bare predicate tokens never
declared as activation vocabulary and the checkers reporting an empty
checked-set honestly when no `## Data` rows exist.

#### Scenario: Predicates resolve against declared Data rows
- **WHEN** a law predicate uses a declared predicate name over `## Data` rows in a file with the pack active
- **THEN** the corresponding named checker covers the predicate occurrence against the declared `## Data` rows

#### Scenario: Honest-empty predicates with no data rows
- **WHEN** the pack is active in a workspace whose specs declare no `## Data` rows
- **THEN** the predicate checkers report an empty checked-set with no fabricated findings

### Requirement: The transform floor is kind-dependent and additive
The pack SHALL declare the floor requiring `bioimage.transform`
properties to enumerate their `**preserves:**` and `**dtype:**` case
labels — the kind-dependent floor making the base `law` floor's
non-universality critique concrete — while the base `law` floor stays
byte-identical with the pack discovered, a kind with no declared floor
inherits none, case labels from the consumed packs ride along
additively, and floor labels never join the activation vocabulary.

#### Scenario: Law properties keep their base floor
- **WHEN** a base `law` property is linted with the bioimage-data pack discovered
- **THEN** its floor findings are byte-identical to the lint without the pack

#### Scenario: Transform floor names its case labels
- **WHEN** a `bioimage.transform` property is checked against the pack's floor declaration
- **THEN** the floor names `preserves` and `dtype` as the required case labels

### Requirement: Pack-fiber kinds are typeable in base-table kind columns
When the pack is active, the format SHALL accept its pack-qualified
fiber kinds in base-table kind columns — `property_kind_closed`'s
effective set is the base set extended with active packs' fiber
vocabulary, never narrower than the base set — and without the pack
active the token stays outside the closed set, firing the labeled
finding, never passing silently.

#### Scenario: Active pack makes the fiber kind typeable
- **WHEN** a Properties row carries `kind = bioimage.transform` and the bioimage-data pack is discovered/declared
- **THEN** `property_kind_closed` accepts the row — no finding

#### Scenario: Inactive pack keeps the closed set closed
- **WHEN** a Properties row carries `kind = bioimage.transform` with no bioimage-data pack discovered or declared
- **THEN** the labeled `property_kind_closed` finding fires naming the closed set

### Requirement: The Requires table names its pack dependencies
The pack SHALL author a `## Requires` table pinning the
`specodelic.md Revision 14` base and naming the `data.lineage`,
`numeric.predicates`, and `empirical.registry` pack deps with the base
corpus revision each was verified against — the mechanism's first
cross-pack `## Requires` consumer — where a dep naming no discovered
pack is a labeled advisory, never silent.

#### Scenario: Pack deps resolve to discovered packs
- **WHEN** the bioimage-data pack is discovered in a workspace that also carries the three standard packs
- **THEN** each `## Requires` dep row resolves to a discovered pack id with its verified-at revision recorded

#### Scenario: Missing pack dep is a labeled advisory
- **WHEN** a `## Requires` dep row names a pack id that no discovered pack in the workspace declares
- **THEN** the advisory names the missing dep on the warnings channel and lint does not fail silently

### Requirement: Pack vocabulary is prose-safe
The pack SHALL NOT declare any vocabulary token that is a bare English
word appearing in corpus prose: bare `bioimage` appears in corpus prose
as documented asymmetry (dotted tokens cannot match prose words),
`pipeline` is prose-heavy and rejected from this release, the predicate
names ride pack-qualified checker rule names, `Axes`, `same_shape_as`,
and `preserves` are verified absent from the corpus, and `dtype`
survives only as the `## Axes` row's opaque column header and the
transform floor's case label, which the activation scanner never reads —
corpus lint warnings stay byte-identical with the pack discovered.

#### Scenario: Corpus prose does not falsely activate the pack
- **WHEN** the corpus lints with the pack discovered and again with it removed
- **THEN** issues and warnings are byte-identical — no vocabulary-match activations from prose words

#### Scenario: Prose bioimage mentions stay inert
- **WHEN** a corpus doc's prose mentions the word `bioimage` (the packs paragraph in USAGE)
- **THEN** the pack is not activated by that word — no warning, no finding

## Requirements

### Requirement: The bioimage-data domain pack is a declared profile file
The format SHALL ship the bioimage-data domain pack as an in-repo
`kind: profile` spec file (`packs/bioimage-data.md`, id `bioimage.data`)
whose manifest declares the `## Axes` section, the `bioimage.transform`
kind, the `same_shape_as` reference field, the `bioimage.dtype_is`,
`bioimage.shape_eq`, and `bioimage.units_convertible` checkers, the
transform case-label floor, and the `## Requires` table pinning the
`specodelic.md Revision 14` base plus the `data.lineage`,
`numeric.predicates`, and `empirical.registry` pack deps — discovered by
corpus scan alone, with no config file and no registry.

#### Scenario: Corpus scan discovers the pack
- **WHEN** `spk lint` runs over any workspace subdirectory of a repository containing `packs/bioimage-data.md`
- **THEN** the pack is discovered without configuration and reported in the lint envelope
- **AND** files using none of the pack's vocabulary keep byte-identical lint findings

#### Scenario: Pack lints clean under pack_shape in every lifecycle state
- **WHEN** the pack file is parsed in draft, published, or deprecated state
- **THEN** pack_shape reports zero findings over the pack's declared vocabulary

### Requirement: The Axes registry is namespaced with opaque scale and unit
The pack SHALL declare a `## Axes` section with row shape
`| name | axis | scale | unit |` — the workspace's declared image axes
with OME-NGFF-style semantics — and the namespaced kind
`bioimage.transform`, pack-fiber-relative vocabulary that grows no base
closed set and narrows no existing base set entry; the `scale` and
`unit` columns stay opaque prose the format never parses, keeping OME/NGFF
and BioImage.IO authoritative as external standards.

#### Scenario: Consumer activates the pack advisory-first
- **WHEN** a corpus spec uses `bioimage.transform` or `## Axes` vocabulary without a declared `uses` edge
- **THEN** the pack's declared checking activates advisory-first for that file, naming the pack

#### Scenario: Orphan vocabulary names the bioimage pack
- **WHEN** a workspace uses `bioimage.transform` vocabulary with no bioimage-data pack discovered or declared
- **THEN** the orphan finding names the candidate pack `bioimage.data` and both remediations, with non-zero exit

### Requirement: same_shape_as is a typed outbound leaf resolving cross-pack, file-locally
The pack SHALL add the `same_shape_as` reference field, resolving to a
`## Data` row of the declaring file — the data-lineage pack's section —
with the pack dependency living in the pack's `## Requires` table while
the resolution itself stays intra-file, joining no reachability path and
no acyclic edge set; a dangling reference is a labeled finding naming the
row id and both remediations.

#### Scenario: Cross-pack section resolution stays file-local
- **WHEN** a constraint row declares `same_shape_as` pointing at a `## Data` row declared in the same file, with the data-lineage pack active for the file
- **THEN** the reference resolves with no corpus-wide pass and no reachability join

#### Scenario: Dangling same_shape_as reference is labeled
- **WHEN** a row references a `## Data` row id via `same_shape_as` that no `## Data` row in the file declares
- **THEN** the finding is labeled, names the row id and both remediations, and is never a generic dangling message

### Requirement: The R2 predicate grammar is a declared closed set of named checkers
The pack SHALL declare the data-shaped predicate grammar as the named
pack-qualified checkers `bioimage.dtype_is`, `bioimage.shape_eq`, and
`bioimage.units_convertible` — a closed set resolving against declared
`## Data` rows — declaration-only, with the bare predicate tokens never
declared as activation vocabulary and the checkers reporting an empty
checked-set honestly when no `## Data` rows exist.

#### Scenario: Predicates resolve against declared Data rows
- **WHEN** a law predicate uses a declared predicate name over `## Data` rows in a file with the pack active
- **THEN** the corresponding named checker covers the predicate occurrence against the declared `## Data` rows

#### Scenario: Honest-empty predicates with no data rows
- **WHEN** the pack is active in a workspace whose specs declare no `## Data` rows
- **THEN** the predicate checkers report an empty checked-set with no fabricated findings

### Requirement: The transform floor is kind-dependent and additive
The pack SHALL declare the floor requiring `bioimage.transform`
properties to enumerate their `**preserves:**` and `**dtype:**` case
labels — the kind-dependent floor making the base `law` floor's
non-universality critique concrete — while the base `law` floor stays
byte-identical with the pack discovered, a kind with no declared floor
inherits none, case labels from the consumed packs ride along
additively, and floor labels never join the activation vocabulary.

#### Scenario: Law properties keep their base floor
- **WHEN** a base `law` property is linted with the bioimage-data pack discovered
- **THEN** its floor findings are byte-identical to the lint without the pack

#### Scenario: Transform floor names its case labels
- **WHEN** a `bioimage.transform` property is checked against the pack's floor declaration
- **THEN** the floor names `preserves` and `dtype` as the required case labels

### Requirement: Pack-fiber kinds are typeable in base-table kind columns
When the pack is active, the format SHALL accept its pack-qualified
fiber kinds in base-table kind columns — `property_kind_closed`'s
effective set is the base set extended with active packs' fiber
vocabulary, never narrower than the base set — and without the pack
active the token stays outside the closed set, firing the labeled
finding, never passing silently.

#### Scenario: Active pack makes the fiber kind typeable
- **WHEN** a Properties row carries `kind = bioimage.transform` and the bioimage-data pack is discovered/declared
- **THEN** `property_kind_closed` accepts the row — no finding

#### Scenario: Inactive pack keeps the closed set closed
- **WHEN** a Properties row carries `kind = bioimage.transform` with no bioimage-data pack discovered or declared
- **THEN** the labeled `property_kind_closed` finding fires naming the closed set

### Requirement: The Requires table names its pack dependencies
The pack SHALL author a `## Requires` table pinning the
`specodelic.md Revision 14` base and naming the `data.lineage`,
`numeric.predicates`, and `empirical.registry` pack deps with the base
corpus revision each was verified against — the mechanism's first
cross-pack `## Requires` consumer — where a dep naming no discovered
pack is a labeled advisory, never silent.

#### Scenario: Pack deps resolve to discovered packs
- **WHEN** the bioimage-data pack is discovered in a workspace that also carries the three standard packs
- **THEN** each `## Requires` dep row resolves to a discovered pack id with its verified-at revision recorded

#### Scenario: Missing pack dep is a labeled advisory
- **WHEN** a `## Requires` dep row names a pack id that no discovered pack in the workspace declares
- **THEN** the advisory names the missing dep on the warnings channel and lint does not fail silently

### Requirement: Pack vocabulary is prose-safe
The pack SHALL NOT declare any vocabulary token that is a bare English
word appearing in corpus prose: bare `bioimage` appears in corpus prose
as documented asymmetry (dotted tokens cannot match prose words),
`pipeline` is prose-heavy and rejected from this release, the predicate
names ride pack-qualified checker rule names, `Axes`, `same_shape_as`,
and `preserves` are verified absent from the corpus, and `dtype`
survives only as the `## Axes` row's opaque column header and the
transform floor's case label, which the activation scanner never reads —
corpus lint warnings stay byte-identical with the pack discovered.

#### Scenario: Corpus prose does not falsely activate the pack
- **WHEN** the corpus lints with the pack discovered and again with it removed
- **THEN** issues and warnings are byte-identical — no vocabulary-match activations from prose words

#### Scenario: Prose bioimage mentions stay inert
- **WHEN** a corpus doc's prose mentions the word `bioimage` (the packs paragraph in USAGE)
- **THEN** the pack is not activated by that word — no warning, no finding
