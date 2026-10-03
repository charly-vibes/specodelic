---
id: spec
kind: intent
statement: "WHEN a workspace enables the numeric-predicates standard pack, THE format SHALL provide a namespaced ## Quantities section whose rows carry name, kind, unit, and domain; numeric.quantity, numeric.bound, and numeric.tolerance kinds; a measured_by reference field resolving to declared ## Quantities rows of the same file; and a tolerance case-label floor — while unit and domain columns stay opaque and files using none of this vocabulary lint byte-identically."
---

# numeric-predicates-pack Specification

## Purpose

Ship the numeric-predicates standard pack: a typed `## Quantities`
table, namespaced numeric kinds, tolerance laws with their own case-label
floor, and a `measured_by` outbound-leaf edge. Unit systems stay
authoritative outside the format — the pack bridges, never absorbs. It
is the second standard pack and, with the empirical-registry pack,
completes what the bioimage D6 pilot consumes.

## Constraints

| id                     | kind      | expr                                                                                                                                                                                                                              | traces_to |
|------------------------|-----------|-----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|-----------|
| pack_file_declared     | invariant | `the numeric-predicates standard pack is a kind: profile spec file at packs/numeric-predicates.md (id numeric.predicates) whose six manifest tables declare the ## Quantities section, the numeric.quantity/numeric.bound/numeric.tolerance kinds, the measured_by reference field, the numeric.quantity_closed and numeric.tolerance_labels checkers, the tolerance case-label floor, and the Revision 14 base pin` | [[spec]]  |
| quantity_section_shape | invariant | `## Quantities rows carry exactly \| name \| kind \| unit \| domain \| with kind closed to quantity, bound, tolerance — a per-pack closed set; no base closed set grows and no existing entry narrows`                                                                                              | [[spec]]  |
| measured_by_outbound   | invariant | `measured_by is a pack-added reference field resolving to a ## Quantities row of the declaring file — an outbound leaf joining no reachability path and no acyclic edge set; a dangling reference is a labeled finding naming the row id and both remediations, never a generic dangling message` | [[spec]]  |
| units_stay_opaque      | invariant | `the unit and domain columns carry typed prose pointers (unit systems, domains of validity) that the format never parses — no unit system, conversion rule, or domain vocabulary joins a base closed set, and lint behavior is identical regardless of the unit system named` | [[spec]]  |
| vocabulary_prose_safe  | invariant | `no pack vocabulary token is a bare English word that appears in corpus prose — the within/bound/against/unit/domain audit excluded them from vocabulary-carrying facets; bound and against survive only as floor case labels, which the mechanism never scans` | [[spec]]  |

## Model

### States
- `draft`
- `published`
- `deprecated`

### Transitions

| id        | from     | to         | guard                                                                                      |
|-----------|----------|------------|--------------------------------------------------------------------------------------------|
| publish   | draft    | published  | `[[spec.pack_file_declared]] ∧ [[spec.quantity_section_shape]] — pack_shape reports zero findings over the pack's declared vocabulary` |
| deprecate | published| deprecated | `[[spec.pack_file_declared]] — maintainer marks the pack deprecated; findings name the deprecation, vocabulary still checks` |

## Properties

| id                            | kind | derives_from                     | generator                                      | predicate                                                                     |
|-------------------------------|------|----------------------------------|------------------------------------------------|-------------------------------------------------------------------------------|
| discovery_finds_numeric_pack  | unit | [[spec.pack_file_declared]]      | `lint_workspace_with_numeric_pack_present()`   | `the pack is discovered by corpus scan alone, no config file, no registry`     |
| pack_shape_clean_all_states   | unit | [[spec.pack_file_declared]]      | `pack_parsed_in_every_lifecycle_state()`       | `pack_shape reports zero findings over the declared vocabulary in draft, published, and deprecated` |
| consumer_activates_advisory   | unit | [[spec.quantity_section_shape]]  | `corpus_spec_uses_numeric_quantity_vocabulary()` | `the pack's checkers activate advisory-first naming the pack; findings attributed per pack` |
| orphan_names_numeric_pack     | unit | [[spec.quantity_section_shape]]  | `workspace_uses_numeric_vocabulary_without_pack()` | `the orphan finding names the candidate pack numeric.predicates and both remediations, exit non-zero` |
| measured_by_resolves          | unit | [[spec.measured_by_outbound]]    | `constraint_row_measured_by_declared_quantity_row()` | `the measured_by reference resolves to the declared ## Quantities row; a dangling reference is a labeled finding naming the row id and both remediations` |
| closure_honest_empty          | unit | [[spec.measured_by_outbound]]    | `pack_active_without_quantity_rows()`          | `the closure checkers report an empty checked-set; no fabricated findings`     |
| no_base_set_absorption        | unit | [[spec.units_stay_opaque]]       | `guide_sets_compared_with_and_without_numeric_pack()` | `INTENT_KINDS and Reference Typing sets identical with the pack discovered or not` |
| units_opaque_to_lint          | unit | [[spec.units_stay_opaque]]       | `quantity_row_names_ucum_vs_iso4217()`         | `lint findings byte-identical across different unit systems`                   |
| no_bare_word_vocabulary       | unit | [[spec.vocabulary_prose_safe]]   | `corpus_linted_with_and_without_numeric_pack()` | `lint warnings byte-identical for every corpus file — no vocabulary-match activations from prose words` |
| tolerance_floor_labels        | unit | [[spec.vocabulary_prose_safe]]   | `tolerance_law_missing_case_label()`           | `a numeric.tolerance law owes **bound:** and **against:** case labels per the floor declaration` |

## ADDED Requirements

### Requirement: The numeric-predicates standard pack is a declared profile file
The format SHALL ship the numeric-predicates standard pack as an in-repo
`kind: profile` spec file (`packs/numeric-predicates.md`, id
`numeric.predicates`) whose manifest declares the `## Quantities`
section, the `numeric.quantity`, `numeric.bound`, and
`numeric.tolerance` kinds, the `measured_by` reference field, the
`numeric.quantity_closed` and `numeric.tolerance_labels` checkers, the
tolerance case-label floor, and the `specodelic.md Revision 14` base pin
— discovered by corpus scan alone, with no config file and no registry.

#### Scenario: Corpus scan discovers the pack
- **WHEN** `spk lint` runs over any workspace subdirectory of a repository containing `packs/numeric-predicates.md`
- **THEN** the pack is discovered without configuration and reported in the lint envelope
- **AND** files using none of the pack's vocabulary keep byte-identical lint findings

#### Scenario: Pack lints clean under pack_shape in every lifecycle state
- **WHEN** the pack file is parsed in draft, published, or deprecated state
- **THEN** pack_shape reports zero findings over the pack's declared vocabulary

### Requirement: Quantity vocabulary is namespaced and per-pack closed
The pack SHALL declare a `## Quantities` section with row shape
`| name | kind | unit | domain |`, kind closed to `quantity`, `bound`,
`tolerance`, and namespaced kinds `numeric.quantity`, `numeric.bound`,
`numeric.tolerance` — pack-fiber-relative vocabulary that grows no base
closed set and narrows no existing base set entry.

#### Scenario: Consumer activates the pack advisory-first
- **WHEN** a corpus spec uses `numeric.quantity` vocabulary without a declared `uses` edge
- **THEN** the pack's declared checking activates advisory-first for that file, naming the pack

#### Scenario: Orphan vocabulary names the numeric pack
- **WHEN** a workspace uses `numeric.quantity` vocabulary with no numeric-predicates pack discovered or declared
- **THEN** the orphan finding names the candidate pack `numeric.predicates` and both remediations, with non-zero exit

### Requirement: measured_by is a typed outbound leaf resolving to declared rows
The pack SHALL add the `measured_by` reference field, resolving to a
`## Quantities` row of the declaring file and joining no reachability
path and no acyclic edge set, and SHALL declare the
`numeric.quantity_closed` checker that reports an empty checked-set
honestly when no `## Quantities` rows exist.

#### Scenario: Dangling measured_by reference is labeled
- **WHEN** a constraint row references a `## Quantities` row id via `measured_by` that no `## Quantities` row in the file declares
- **THEN** the finding is labeled, names the row id and both remediations, and is never a generic dangling message

#### Scenario: Honest-empty closure with no quantity rows
- **WHEN** the pack is active in a workspace whose specs declare no `## Quantities` rows
- **THEN** the closure checker reports an empty checked-set with no fabricated findings

### Requirement: Unit systems stay external
The pack SHALL bridge, never absorb, unit systems: the `unit` and
`domain` columns carry typed prose pointers (UCUM codes, ISO4217
currencies, domains of validity) that the format never parses, and no
unit system or conversion rule ever joins a base closed set — lint
behavior is identical regardless of the unit system named.

#### Scenario: Any unit system binds without lint change
- **WHEN** a `## Quantities` row names UCUM units, ISO4217 currencies, or any other unit system
- **THEN** lint findings are byte-identical across the different unit systems

### Requirement: Pack vocabulary is prose-safe
The pack SHALL NOT declare any vocabulary token that is a bare English
word appearing in corpus prose: the `within`, `bound`, `against`,
`unit`, and `domain` audit excludes them from the Sections, Kinds, and
References facets, and `bound`/`against` survive only as the tolerance
floor's case labels, which the activation scanner never reads — corpus
lint warnings stay byte-identical with the pack discovered.

#### Scenario: Corpus prose does not falsely activate the pack
- **WHEN** the corpus lints with the pack discovered and again with it removed
- **THEN** issues and warnings are byte-identical — no vocabulary-match activations from prose words

#### Scenario: Tolerance floor names its case labels
- **WHEN** a `numeric.tolerance` law property is checked against the pack's floor declaration
- **THEN** the floor names `bound` and `against` as the required case labels

## Requirements

### Requirement: The numeric-predicates standard pack is a declared profile file
The format SHALL ship the numeric-predicates standard pack as an in-repo
`kind: profile` spec file (`packs/numeric-predicates.md`, id
`numeric.predicates`) whose manifest declares the `## Quantities`
section, the `numeric.quantity`, `numeric.bound`, and
`numeric.tolerance` kinds, the `measured_by` reference field, the
`numeric.quantity_closed` and `numeric.tolerance_labels` checkers, the
tolerance case-label floor, and the `specodelic.md Revision 14` base pin
— discovered by corpus scan alone, with no config file and no registry.

#### Scenario: Corpus scan discovers the pack
- **WHEN** `spk lint` runs over any workspace subdirectory of a repository containing `packs/numeric-predicates.md`
- **THEN** the pack is discovered without configuration and reported in the lint envelope
- **AND** files using none of the pack's vocabulary keep byte-identical lint findings

#### Scenario: Pack lints clean under pack_shape in every lifecycle state
- **WHEN** the pack file is parsed in draft, published, or deprecated state
- **THEN** pack_shape reports zero findings over the pack's declared vocabulary

### Requirement: Quantity vocabulary is namespaced and per-pack closed
The pack SHALL declare a `## Quantities` section with row shape
`| name | kind | unit | domain |`, kind closed to `quantity`, `bound`,
`tolerance`, and namespaced kinds `numeric.quantity`, `numeric.bound`,
`numeric.tolerance` — pack-fiber-relative vocabulary that grows no base
closed set and narrows no existing base set entry.

#### Scenario: Consumer activates the pack advisory-first
- **WHEN** a corpus spec uses `numeric.quantity` vocabulary without a declared `uses` edge
- **THEN** the pack's declared checking activates advisory-first for that file, naming the pack

#### Scenario: Orphan vocabulary names the numeric pack
- **WHEN** a workspace uses `numeric.quantity` vocabulary with no numeric-predicates pack discovered or declared
- **THEN** the orphan finding names the candidate pack `numeric.predicates` and both remediations, with non-zero exit

### Requirement: measured_by is a typed outbound leaf resolving to declared rows
The pack SHALL add the `measured_by` reference field, resolving to a
`## Quantities` row of the declaring file and joining no reachability
path and no acyclic edge set, and SHALL declare the
`numeric.quantity_closed` checker that reports an empty checked-set
honestly when no `## Quantities` rows exist.

#### Scenario: Dangling measured_by reference is labeled
- **WHEN** a constraint row references a `## Quantities` row id via `measured_by` that no `## Quantities` row in the file declares
- **THEN** the finding is labeled, names the row id and both remediations, and is never a generic dangling message

#### Scenario: Honest-empty closure with no quantity rows
- **WHEN** the pack is active in a workspace whose specs declare no `## Quantities` rows
- **THEN** the closure checker reports an empty checked-set with no fabricated findings

### Requirement: Unit systems stay external
The pack SHALL bridge, never absorb, unit systems: the `unit` and
`domain` columns carry typed prose pointers (UCUM codes, ISO4217
currencies, domains of validity) that the format never parses, and no
unit system or conversion rule ever joins a base closed set — lint
behavior is identical regardless of the unit system named.

#### Scenario: Any unit system binds without lint change
- **WHEN** a `## Quantities` row names UCUM units, ISO4217 currencies, or any other unit system
- **THEN** lint findings are byte-identical across the different unit systems

### Requirement: Pack vocabulary is prose-safe
The pack SHALL NOT declare any vocabulary token that is a bare English
word appearing in corpus prose: the `within`, `bound`, `against`,
`unit`, and `domain` audit excludes them from the Sections, Kinds, and
References facets, and `bound`/`against` survive only as the tolerance
floor's case labels, which the activation scanner never reads — corpus
lint warnings stay byte-identical with the pack discovered.

#### Scenario: Corpus prose does not falsely activate the pack
- **WHEN** the corpus lints with the pack discovered and again with it removed
- **THEN** issues and warnings are byte-identical — no vocabulary-match activations from prose words

#### Scenario: Tolerance floor names its case labels
- **WHEN** a `numeric.tolerance` law property is checked against the pack's floor declaration
- **THEN** the floor names `bound` and `against` as the required case labels