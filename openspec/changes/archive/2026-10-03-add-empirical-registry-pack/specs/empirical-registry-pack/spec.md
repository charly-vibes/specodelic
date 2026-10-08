# empirical-registry-pack Specification

## Purpose

Ship the empirical-registry standard pack: the Floors facet's first real
consumer — a namespaced `## StatTests` registry table, the
`empirical.statistic` property kind, and a per-kind case-label floor
(`alpha` + `window`) reusing the `**name:**` enumeration machinery with
a different required label set than the base `law` floor. Statistical
tests stay authoritative outside the format — the pack declares the
registry, the mechanism interprets it. It is the third standard pack
and, with data/lineage and numeric predicates, completes what the
bioimage D6 pilot consumes.

## Constraints

| id                       | kind      | expr                                                                                                                                                                                                                                              | traces_to |
|--------------------------|-----------|---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|-----------|
| pack_file_declared       | invariant | `the empirical-registry standard pack is a kind: profile spec file at packs/empirical-registry.md (id empirical.registry) whose six manifest tables declare the ## StatTests section, the empirical.statistic kind, the tested_by reference field, the empirical.statistic_labels and empirical.stat_test_closed checkers, the statistic case-label floor, and the Revision 14 base pin` | [[empirical.registry.pack]]  |
| stat_test_section_shape  | invariant | `## StatTests rows carry exactly \| name \| metric \| alpha \| window \| — the registry of the workspace's statistical tests; alpha and window columns are opaque prose the format never parses, and no base closed set grows and no existing entry narrows`                                                        | [[empirical.registry.pack]]  |
| tested_by_outbound       | invariant | `tested_by is a pack-added reference field resolving to a ## StatTests row of the declaring file — an outbound leaf joining no reachability path and no acyclic edge set; a dangling reference is a labeled finding naming the row id and both remediations, never a generic dangling message`                       | [[empirical.registry.pack]]  |
| floors_are_additive      | invariant | `the per-kind floor requires statistic properties to enumerate their alpha and window case labels; the base law floor (identity/associativity) is byte-identical with the pack discovered, a kind with no declared floor inherits none, the base floor is a floor not a ceiling (a law-kind row using the pack's vocabulary may carry the labels additively), and floor labels are never activation vocabulary` | [[empirical.registry.pack]]  |
| fiber_kinds_typeable     | invariant | `when the pack is active, base-table kind columns accept its pack-qualified fiber kinds — property_kind_closed's effective set is base ∪ active-pack-fiber, never narrower; without the pack active the token stays outside the closed set and the labeled finding fires — never a silent pass`                        | [[empirical.registry.pack]]  |
| vocabulary_prose_safe    | invariant | `no pack vocabulary token is a bare English word that appears in corpus prose — window appears in corpus prose and is excluded from every vocabulary-carrying facet, surviving only as a floor case label and a table column; the kind is dotted; StatTests and tested_by are verified absent from the corpus`      | [[empirical.registry.pack]]  |

## Model

### States
- `draft`
- `published`
- `deprecated`

### Transitions

| id        | from     | to         | guard                                                                                      |
|-----------|----------|------------|--------------------------------------------------------------------------------------------|
| publish   | draft    | published  | `[[empirical.registry.pack.pack_file_declared]] ∧ [[empirical.registry.pack.stat_test_section_shape]] — pack_shape reports zero findings over the pack's declared vocabulary` |
| deprecate | published| deprecated | `[[empirical.registry.pack.pack_file_declared]] — maintainer marks the pack deprecated; findings name the deprecation, vocabulary still checks` |

## Properties

| id                             | kind | derives_from                      | generator                                            | predicate                                                                     |
|--------------------------------|------|-----------------------------------|------------------------------------------------------|-------------------------------------------------------------------------------|
| discovery_finds_empirical_pack | unit | [[empirical.registry.pack.pack_file_declared]]       | `lint_workspace_with_empirical_pack_present()`       | `the pack is discovered by corpus scan alone, no config file, no registry`     |
| pack_shape_clean_all_states    | unit | [[empirical.registry.pack.pack_file_declared]]       | `pack_parsed_in_every_lifecycle_state()`             | `pack_shape reports zero findings over the declared vocabulary in draft, published, and deprecated` |
| consumer_activates_advisory    | unit | [[empirical.registry.pack.stat_test_section_shape]]  | `corpus_spec_uses_empirical_statistic_vocabulary()`  | `the pack's checkers activate advisory-first naming the pack; findings attributed per pack` |
| orphan_names_empirical_pack    | unit | [[empirical.registry.pack.stat_test_section_shape]]  | `workspace_uses_empirical_vocabulary_without_pack()` | `the orphan finding names the candidate pack empirical.registry and both remediations, exit non-zero` |
| tested_by_resolves             | unit | [[empirical.registry.pack.tested_by_outbound]]       | `property_row_tested_by_declared_stat_test_row()`    | `the tested_by reference resolves to the declared ## StatTests row; a dangling reference is a labeled finding naming the row id and both remediations` |
| closure_honest_empty           | unit | [[empirical.registry.pack.tested_by_outbound]]       | `pack_active_without_stat_test_rows()`               | `the closure checkers report an empty checked-set; no fabricated findings`     |
| law_floor_unchanged            | unit | [[empirical.registry.pack.floors_are_additive]]      | `law_property_linted_with_and_without_pack()`        | `law floor findings byte-identical with the pack discovered; a kind with no floor inherits none` |
| statistic_floor_labels         | unit | [[empirical.registry.pack.floors_are_additive]]      | `statistic_property_missing_case_label()`            | `an empirical.statistic property owes **alpha:** and **window:** case labels per the floor declaration` |
| kind_column_accepts_fiber_kinds | unit | [[empirical.registry.pack.fiber_kinds_typeable]]     | `property_row_typed_empirical_statistic_with_pack_active()` | `property_kind_closed accepts the fiber kind when the pack is active; the same row without the pack fires the labeled finding` |
| no_bare_word_vocabulary        | unit | [[empirical.registry.pack.vocabulary_prose_safe]]    | `corpus_linted_with_and_without_empirical_pack()`    | `lint warnings byte-identical for every corpus file — no vocabulary-match activations from prose words` |

## ADDED Requirements

### Requirement: The empirical-registry standard pack is a declared profile file
The format SHALL ship the empirical-registry standard pack as an in-repo
`kind: profile` spec file (`packs/empirical-registry.md`, id
`empirical.registry`) whose manifest declares the `## StatTests`
section, the `empirical.statistic` kind, the `tested_by` reference
field, the `empirical.statistic_labels` and `empirical.stat_test_closed`
checkers, the statistic case-label floor, and the
`specodelic.md Revision 14` base pin — discovered by corpus scan alone,
with no config file and no registry.

#### Scenario: Corpus scan discovers the pack
- **WHEN** `spk lint` runs over any workspace subdirectory of a repository containing `packs/empirical-registry.md`
- **THEN** the pack is discovered without configuration and reported in the lint envelope
- **AND** files using none of the pack's vocabulary keep byte-identical lint findings

#### Scenario: Pack lints clean under pack_shape in every lifecycle state
- **WHEN** the pack file is parsed in draft, published, or deprecated state
- **THEN** pack_shape reports zero findings over the pack's declared vocabulary

### Requirement: The StatTests registry is namespaced and per-pack closed
The pack SHALL declare a `## StatTests` section with row shape
`| name | metric | alpha | window |` — the workspace's declared
statistical tests — and the namespaced kind `empirical.statistic`,
pack-fiber-relative vocabulary that grows no base closed set and
narrows no existing base set entry; the `alpha` and `window` columns
stay opaque prose the format never parses.

#### Scenario: Consumer activates the pack advisory-first
- **WHEN** a corpus spec uses `empirical.statistic` vocabulary without a declared `uses` edge
- **THEN** the pack's declared checking activates advisory-first for that file, naming the pack

#### Scenario: Orphan vocabulary names the empirical pack
- **WHEN** a workspace uses `empirical.statistic` vocabulary with no empirical-registry pack discovered or declared
- **THEN** the orphan finding names the candidate pack `empirical.registry` and both remediations, with non-zero exit

### Requirement: tested_by is a typed outbound leaf resolving to declared rows
The pack SHALL add the `tested_by` reference field, resolving to a
`## StatTests` row of the declaring file and joining no reachability
path and no acyclic edge set, and SHALL declare the
`empirical.stat_test_closed` checker that reports an empty checked-set
honestly when no `## StatTests` rows exist.

#### Scenario: Dangling tested_by reference is labeled
- **WHEN** a property row references a `## StatTests` row id via `tested_by` that no `## StatTests` row in the file declares
- **THEN** the finding is labeled, names the row id and both remediations, and is never a generic dangling message

#### Scenario: Honest-empty closure with no stat-test rows
- **WHEN** the pack is active in a workspace whose specs declare no `## StatTests` rows
- **THEN** the closure checker reports an empty checked-set with no fabricated findings

### Requirement: The per-kind floor is additive enumeration
The pack SHALL declare the floor requiring `empirical.statistic`
properties to enumerate their `**alpha:**` and `**window:**` case
labels, reusing the `**name:**` enumeration machinery with a different
required label set — while the base `law` floor stays byte-identical
with the pack discovered and a kind with no declared floor inherits
none; floor labels never join the activation vocabulary.

#### Scenario: Law properties keep their base floor
- **WHEN** a base `law` property is linted with the empirical-registry pack discovered
- **THEN** its floor findings are byte-identical to the lint without the pack

#### Scenario: Statistic floor names its case labels
- **WHEN** an `empirical.statistic` property is checked against the pack's floor declaration
- **THEN** the floor names `alpha` and `window` as the required case labels

### Requirement: Pack-fiber kinds are typeable in base-table kind columns
When the pack is active, the format SHALL accept its pack-qualified
fiber kinds in base-table kind columns — `property_kind_closed`'s
effective set is the base set extended with active packs' fiber
vocabulary, never narrower than the base set — and without the pack
active the token stays outside the closed set, firing the labeled
finding, never passing silently.

#### Scenario: Active pack makes the fiber kind typeable
- **WHEN** a Properties row carries `kind = empirical.statistic` and the empirical-registry pack is discovered/declared
- **THEN** `property_kind_closed` accepts the row — no finding

#### Scenario: Inactive pack keeps the closed set closed
- **WHEN** a Properties row carries `kind = empirical.statistic` with no empirical-registry pack discovered or declared
- **THEN** the labeled `property_kind_closed` finding fires naming the closed set

### Requirement: Pack vocabulary is prose-safe
The pack SHALL NOT declare any vocabulary token that is a bare English
word appearing in corpus prose: `window` appears in corpus prose and is
excluded from the Sections, Kinds, and References facets, surviving only
as the statistic floor's case label and a table column, which the
activation scanner never reads; the kind is dotted
(`empirical.statistic`); `StatTests` and `tested_by` are verified absent
from the corpus — corpus lint warnings stay byte-identical with the pack
discovered.

#### Scenario: Corpus prose does not falsely activate the pack
- **WHEN** the corpus lints with the pack discovered and again with it removed
- **THEN** issues and warnings are byte-identical — no vocabulary-match activations from prose words

#### Scenario: Prose window mentions stay inert
- **WHEN** a corpus spec's prose contains the English word `window` (the rate-limit window pattern)
- **THEN** the pack is not activated by that word — no warning, no finding

## Requirements

### Requirement: The empirical-registry standard pack is a declared profile file
The format SHALL ship the empirical-registry standard pack as an in-repo
`kind: profile` spec file (`packs/empirical-registry.md`, id
`empirical.registry`) whose manifest declares the `## StatTests`
section, the `empirical.statistic` kind, the `tested_by` reference
field, the `empirical.statistic_labels` and `empirical.stat_test_closed`
checkers, the statistic case-label floor, and the
`specodelic.md Revision 14` base pin — discovered by corpus scan alone,
with no config file and no registry.

#### Scenario: Corpus scan discovers the pack
- **WHEN** `spk lint` runs over any workspace subdirectory of a repository containing `packs/empirical-registry.md`
- **THEN** the pack is discovered without configuration and reported in the lint envelope
- **AND** files using none of the pack's vocabulary keep byte-identical lint findings

#### Scenario: Pack lints clean under pack_shape in every lifecycle state
- **WHEN** the pack file is parsed in draft, published, or deprecated state
- **THEN** pack_shape reports zero findings over the pack's declared vocabulary

### Requirement: The StatTests registry is namespaced and per-pack closed
The pack SHALL declare a `## StatTests` section with row shape
`| name | metric | alpha | window |` — the workspace's declared
statistical tests — and the namespaced kind `empirical.statistic`,
pack-fiber-relative vocabulary that grows no base closed set and
narrows no existing base set entry; the `alpha` and `window` columns
stay opaque prose the format never parses.

#### Scenario: Consumer activates the pack advisory-first
- **WHEN** a corpus spec uses `empirical.statistic` vocabulary without a declared `uses` edge
- **THEN** the pack's declared checking activates advisory-first for that file, naming the pack

#### Scenario: Orphan vocabulary names the empirical pack
- **WHEN** a workspace uses `empirical.statistic` vocabulary with no empirical-registry pack discovered or declared
- **THEN** the orphan finding names the candidate pack `empirical.registry` and both remediations, with non-zero exit

### Requirement: tested_by is a typed outbound leaf resolving to declared rows
The pack SHALL add the `tested_by` reference field, resolving to a
`## StatTests` row of the declaring file and joining no reachability
path and no acyclic edge set, and SHALL declare the
`empirical.stat_test_closed` checker that reports an empty checked-set
honestly when no `## StatTests` rows exist.

#### Scenario: Dangling tested_by reference is labeled
- **WHEN** a property row references a `## StatTests` row id via `tested_by` that no `## StatTests` row in the file declares
- **THEN** the finding is labeled, names the row id and both remediations, and is never a generic dangling message

#### Scenario: Honest-empty closure with no stat-test rows
- **WHEN** the pack is active in a workspace whose specs declare no `## StatTests` rows
- **THEN** the closure checker reports an empty checked-set with no fabricated findings

### Requirement: The per-kind floor is additive enumeration
The pack SHALL declare the floor requiring `empirical.statistic`
properties to enumerate their `**alpha:**` and `**window:**` case
labels, reusing the `**name:**` enumeration machinery with a different
required label set — while the base `law` floor stays byte-identical
with the pack discovered and a kind with no declared floor inherits
none; floor labels never join the activation vocabulary.

#### Scenario: Law properties keep their base floor
- **WHEN** a base `law` property is linted with the empirical-registry pack discovered
- **THEN** its floor findings are byte-identical to the lint without the pack

#### Scenario: Statistic floor names its case labels
- **WHEN** an `empirical.statistic` property is checked against the pack's floor declaration
- **THEN** the floor names `alpha` and `window` as the required case labels

### Requirement: Pack-fiber kinds are typeable in base-table kind columns
When the pack is active, the format SHALL accept its pack-qualified
fiber kinds in base-table kind columns — `property_kind_closed`'s
effective set is the base set extended with active packs' fiber
vocabulary, never narrower than the base set — and without the pack
active the token stays outside the closed set, firing the labeled
finding, never passing silently.

#### Scenario: Active pack makes the fiber kind typeable
- **WHEN** a Properties row carries `kind = empirical.statistic` and the empirical-registry pack is discovered/declared
- **THEN** `property_kind_closed` accepts the row — no finding

#### Scenario: Inactive pack keeps the closed set closed
- **WHEN** a Properties row carries `kind = empirical.statistic` with no empirical-registry pack discovered or declared
- **THEN** the labeled `property_kind_closed` finding fires naming the closed set

### Requirement: Pack vocabulary is prose-safe
The pack SHALL NOT declare any vocabulary token that is a bare English
word appearing in corpus prose: `window` appears in corpus prose and is
excluded from the Sections, Kinds, and References facets, surviving only
as the statistic floor's case label and a table column, which the
activation scanner never reads; the kind is dotted
(`empirical.statistic`); `StatTests` and `tested_by` are verified absent
from the corpus — corpus lint warnings stay byte-identical with the pack
discovered.

#### Scenario: Corpus prose does not falsely activate the pack
- **WHEN** the corpus lints with the pack discovered and again with it removed
- **THEN** issues and warnings are byte-identical — no vocabulary-match activations from prose words

#### Scenario: Prose window mentions stay inert
- **WHEN** a corpus spec's prose contains the English word `window` (the rate-limit window pattern)
- **THEN** the pack is not activated by that word — no warning, no finding
