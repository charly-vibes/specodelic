---
id: quant.finance.pack
kind: intent
statement: "WHEN a workspace enables the quant-finance standard pack, THE format SHALL provide a namespaced ## Limits section whose rows carry name, kind, unit, and bound; quant.risk and quant.pricing kinds; a capped_by reference field resolving to declared ## Limits rows of the same file; the limit-closure and risk-label pack-qualified checkers (quant.limit_closed, quant.risk_labels); and a kind-dependent case-label floor enumerating horizon and confidence on risk laws — while ISO4217 and market-data conventions stay authoritative external standards, no base closed set grows, and files using none of this vocabulary lint byte-identically."
---

# quant-finance-pack Specification

## Purpose

Ship the quant-finance standard pack: a typed `## Limits` section for
bounded risk measures and pricing invariants, namespaced quant kinds, a
kind-dependent risk case-label floor, and a `capped_by` outbound-leaf
edge. Most of the vendor convergence already ships in the standard
packs — this pack declares only the vocabulary no standard pack
carries, and requires `numeric.predicates` + `data.lineage` for the
rest. It is the D6 second domain pack.

## Constraints

| id                    | kind      | expr                                                                                                                                                                                                                                                                                     | traces_to |
|-----------------------|-----------|------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|-----------|
| pack_file_declared    | invariant | `the quant-finance standard pack is a kind: profile spec file at packs/quant-finance.md (id quant.finance) whose six manifest tables declare the ## Limits section, the quant.risk and quant.pricing kinds, the capped_by reference field, the quant.limit_closed and quant.risk_labels checkers, the risk case-label floor, and the Revision 18 base pin plus the numeric.predicates and data.lineage pack deps` | [[quant.finance.pack]]  |
| limits_section_shape  | invariant | `## Limits rows carry exactly \| name \| kind \| unit \| bound \| with kind closed to risk, pricing — a per-pack closed set; no base closed set grows and no existing entry narrows`                                                                                                       | [[quant.finance.pack]]  |
| capped_by_outbound    | invariant | `capped_by is a pack-added reference field resolving to a ## Limits row of the declaring file — an outbound leaf joining no reachability path and no acyclic edge set; a dangling reference is a labeled finding naming the row id and both remediations, never a generic dangling message` | [[quant.finance.pack]]  |
| fixtures_stay_data    | invariant | `backtest fixtures and datasets are data.lineage ## Data rows — the pack declares no fixtures section and no second dataset vocabulary; the vendor Fixtures demand is satisfied by the required data.lineage pack`                                                                        | [[quant.finance.pack]]  |
| vocabulary_prose_safe | invariant | `no pack vocabulary token is a bare English word that appears in corpus prose — the limit/confidence/Fixtures audit excluded them from vocabulary-carrying facets; horizon and confidence survive only as floor case labels, which the mechanism never scans`                             | [[quant.finance.pack]]  |

## Model

### States
- `draft`
- `published`
- `deprecated`

### Transitions

| id        | from     | to         | guard                                                                                      |
|-----------|----------|------------|--------------------------------------------------------------------------------------------|
| publish   | draft    | published  | `[[quant.finance.pack.pack_file_declared]] ∧ [[quant.finance.pack.limits_section_shape]] — pack_shape reports zero findings over the pack's declared vocabulary` |
| deprecate | published| deprecated | `[[quant.finance.pack.pack_file_declared]] — maintainer marks the pack deprecated; findings name the deprecation, vocabulary still checks`  |

## Properties

| id                          | kind | derives_from                                  | generator                                    | predicate                                                                       |
|-----------------------------|------|-----------------------------------------------|----------------------------------------------|---------------------------------------------------------------------------------|
| discovery_finds_quant_pack  | unit | [[quant.finance.pack.pack_file_declared]]          | `lint_workspace_with_quant_pack_present()`   | `the pack is discovered by corpus scan alone, no config file, no registry`      |
| no_pack_no_change           | unit | [[quant.finance.pack.pack_file_declared]]          | `corpus_linted_before_and_after_quant_pack()`| `lint findings byte-identical for a corpus using no quant vocabulary`           |
| honest_empty                | unit | [[quant.finance.pack.capped_by_outbound]]          | `quant_pack_active_with_no_limits_rows()`    | `quant.limit_closed reports an empty checked-set with no fabricated findings`   |
| limits_rows_namespaced      | unit | [[quant.finance.pack.limits_section_shape]]        | `limits_row_shape_checked_by_pack_shape()`   | `## Limits rows carry exactly \| name \| kind \| unit \| bound \|; the row-kind labels stay closed to risk, pricing — a per-pack closed set that never grows a base set` |
| fixtures_are_data_rows      | unit | [[quant.finance.pack.fixtures_stay_data]]          | `corpus_declares_no_fixtures_section()`      | `the pack declares no Fixtures section; backtest fixtures and datasets are data.lineage ## Data rows via the required data.lineage pack dep` |
| vocabulary_prose_safe_verified | unit | [[quant.finance.pack.vocabulary_prose_safe]]    | `vocabulary_surface_equals_declared_tokens()`| `the pack's vocabulary() is exactly [quant.risk, quant.pricing, Limits, capped_by]; floor case labels never join it; corpus prose using none of it lints byte-identically` |

## ADDED Requirements

### Requirement: The quant-finance standard pack is a declared profile file
The format SHALL ship the quant-finance standard pack as an in-repo
`kind: profile` spec file (`packs/quant-finance.md`, id
`quant.finance`) whose manifest declares the `## Limits` section, the
`quant.risk` and `quant.pricing` kinds, the `capped_by` reference
field, the `quant.limit_closed` and `quant.risk_labels` checkers, the
risk case-label floor, and the `specodelic.md Revision 18` base pin
plus `numeric.predicates` and `data.lineage` pack deps — discovered by
corpus scan alone, with no config file and no registry.

#### Scenario: Corpus scan discovers the pack
- **WHEN** `spk lint` runs over any workspace subdirectory of a repository containing `packs/quant-finance.md`
- **THEN** the pack is discovered without configuration and reported in the lint envelope
- **AND** files using none of the pack's vocabulary keep byte-identical lint findings

#### Scenario: Pack lints clean under pack_shape in every lifecycle state
- **WHEN** the pack file is parsed in draft, published, or deprecated state
- **THEN** pack_shape reports zero findings over the pack's declared vocabulary

### Requirement: Limits vocabulary is namespaced and per-pack closed
The pack SHALL declare a `## Limits` section with row shape
`| name | kind | unit | bound |`, kind closed to `risk`, `pricing`,
and namespaced kinds `quant.risk`, `quant.pricing` —
pack-fiber-relative vocabulary that grows no base closed set and
narrows no existing base set entry.

#### Scenario: Consumer activates the pack advisory-first
- **WHEN** a corpus spec uses `quant.risk` vocabulary without a declared `uses` edge
- **THEN** the pack's declared checking activates advisory-first for that file, naming the pack

#### Scenario: Orphan vocabulary names the quant pack
- **WHEN** a workspace uses `quant.risk` vocabulary with no quant-finance pack discovered or declared
- **THEN** the orphan finding names the candidate pack `quant.finance` and both remediations, with non-zero exit

### Requirement: capped_by is a typed outbound leaf resolving to declared rows
The pack SHALL add the `capped_by` reference field, resolving to a
`## Limits` row of the declaring file and joining no reachability path
and no acyclic edge set, and SHALL declare the `quant.limit_closed`
checker that reports an empty checked-set honestly when no `## Limits`
rows exist.

#### Scenario: Dangling capped_by reference is labeled
- **WHEN** a constraint row references a `## Limits` row id via `capped_by` that no `## Limits` row in the file declares
- **THEN** the finding is labeled, names the row id and both remediations, and is never a generic dangling message

#### Scenario: Honest-empty closure with no limits rows
- **WHEN** the pack is active in a workspace whose specs declare no `## Limits` rows
- **THEN** the closure checker reports an empty checked-set with no fabricated findings

### Requirement: The risk floor names its case labels
The pack SHALL declare a kind-dependent floor: `quant.risk` laws and
limits enumerate their `**horizon:**` and `**confidence:**` case labels
per the floor declaration, reusing the machine-findable `**name:**`
case-label enumeration mechanism with a different required label set —
and SHALL NOT re-declare a second `tolerance` kind (the numeric pack's
`numeric.tolerance` and its `bound, against` floor remain the
tolerance vocabulary).

#### Scenario: Risk floor names its case labels
- **WHEN** a `quant.risk` law property is checked against the pack's floor declaration
- **THEN** the floor names `horizon` and `confidence` as the required case labels

#### Scenario: Tolerance semantics reuse the numeric pack
- **WHEN** a quant spec file states a tolerance claim on a quantity
- **THEN** the claim is declared as a `numeric.tolerance` row under the required `numeric.predicates` pack, with no second tolerance kind from this pack

### Requirement: Fixtures stay in the data-lineage pack and vocabulary is prose-safe
The pack SHALL NOT declare a fixtures section or any vocabulary token
that is a bare English word appearing in corpus prose: backtest
fixtures and datasets are the required `data.lineage` pack's `## Data`
rows, and the `limit`, `confidence`, and `Fixtures` audit excludes them
from the Sections, Kinds, and References facets — `horizon` and
`confidence` survive only as the risk floor's case labels, which the
activation scanner never reads — corpus lint warnings stay
byte-identical with the pack discovered.

#### Scenario: Corpus prose does not falsely activate the pack
- **WHEN** the corpus lints with the pack discovered and again with it removed
- **THEN** issues and warnings are byte-identical — no vocabulary-match activations from prose words

#### Scenario: Backtest fixtures are Data rows
- **WHEN** a quant spec file declares a backtest fixture or dataset
- **THEN** it is a `data.dataset` or `data.artifact` row of the declaring file's `## Data` table under the required `data.lineage` pack

## Requirements

### Requirement: The quant-finance standard pack is a declared profile file
The format SHALL ship the quant-finance standard pack as an in-repo
`kind: profile` spec file (`packs/quant-finance.md`, id
`quant.finance`) whose manifest declares the `## Limits` section, the
`quant.risk` and `quant.pricing` kinds, the `capped_by` reference
field, the `quant.limit_closed` and `quant.risk_labels` checkers, the
risk case-label floor, and the `specodelic.md Revision 18` base pin
plus `numeric.predicates` and `data.lineage` pack deps — discovered by
corpus scan alone, with no config file and no registry.

#### Scenario: Corpus scan discovers the pack
- **WHEN** `spk lint` runs over any workspace subdirectory of a repository containing `packs/quant-finance.md`
- **THEN** the pack is discovered without configuration and reported in the lint envelope
- **AND** files using none of the pack's vocabulary keep byte-identical lint findings

#### Scenario: Pack lints clean under pack_shape in every lifecycle state
- **WHEN** the pack file is parsed in draft, published, or deprecated state
- **THEN** pack_shape reports zero findings over the pack's declared vocabulary

### Requirement: Limits vocabulary is namespaced and per-pack closed
The pack SHALL declare a `## Limits` section with row shape
`| name | kind | unit | bound |`, kind closed to `risk`, `pricing`,
and namespaced kinds `quant.risk`, `quant.pricing` —
pack-fiber-relative vocabulary that grows no base closed set and
narrows no existing base set entry.

#### Scenario: Consumer activates the pack advisory-first
- **WHEN** a corpus spec uses `quant.risk` vocabulary without a declared `uses` edge
- **THEN** the pack's declared checking activates advisory-first for that file, naming the pack

#### Scenario: Orphan vocabulary names the quant pack
- **WHEN** a workspace uses `quant.risk` vocabulary with no quant-finance pack discovered or declared
- **THEN** the orphan finding names the candidate pack `quant.finance` and both remediations, with non-zero exit

### Requirement: capped_by is a typed outbound leaf resolving to declared rows
The pack SHALL add the `capped_by` reference field, resolving to a
`## Limits` row of the declaring file and joining no reachability path
and no acyclic edge set, and SHALL declare the `quant.limit_closed`
checker that reports an empty checked-set honestly when no `## Limits`
rows exist.

#### Scenario: Dangling capped_by reference is labeled
- **WHEN** a constraint row references a `## Limits` row id via `capped_by` that no `## Limits` row in the file declares
- **THEN** the finding is labeled, names the row id and both remediations, and is never a generic dangling message

#### Scenario: Honest-empty closure with no limits rows
- **WHEN** the pack is active in a workspace whose specs declare no `## Limits` rows
- **THEN** the closure checker reports an empty checked-set with no fabricated findings

### Requirement: The risk floor names its case labels
The pack SHALL declare a kind-dependent floor: `quant.risk` laws and
limits enumerate their `**horizon:**` and `**confidence:**` case labels
per the floor declaration, reusing the machine-findable `**name:**`
case-label enumeration mechanism with a different required label set —
and SHALL NOT re-declare a second `tolerance` kind (the numeric pack's
`numeric.tolerance` and its `bound, against` floor remain the
tolerance vocabulary).

#### Scenario: Risk floor names its case labels
- **WHEN** a `quant.risk` law property is checked against the pack's floor declaration
- **THEN** the floor names `horizon` and `confidence` as the required case labels

#### Scenario: Tolerance semantics reuse the numeric pack
- **WHEN** a quant spec file states a tolerance claim on a quantity
- **THEN** the claim is declared as a `numeric.tolerance` row under the required `numeric.predicates` pack, with no second tolerance kind from this pack

### Requirement: Fixtures stay in the data-lineage pack and vocabulary is prose-safe
The pack SHALL NOT declare a fixtures section or any vocabulary token
that is a bare English word appearing in corpus prose: backtest
fixtures and datasets are the required `data.lineage` pack's `## Data`
rows, and the `limit`, `confidence`, and `Fixtures` audit excludes them
from the Sections, Kinds, and References facets — `horizon` and
`confidence` survive only as the risk floor's case labels, which the
activation scanner never reads — corpus lint warnings stay
byte-identical with the pack discovered.

#### Scenario: Corpus prose does not falsely activate the pack
- **WHEN** the corpus lints with the pack discovered and again with it removed
- **THEN** issues and warnings are byte-identical — no vocabulary-match activations from prose words

#### Scenario: Backtest fixtures are Data rows
- **WHEN** a quant spec file declares a backtest fixture or dataset
- **THEN** it is a `data.dataset` or `data.artifact` row of the declaring file's `## Data` table under the required `data.lineage` pack
