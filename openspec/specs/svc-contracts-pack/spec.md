---
id: svc.contracts.pack
kind: intent
statement: "WHEN a workspace enables the svc-contracts standard pack, THE format SHALL provide a namespaced ## Interfaces section whose rows carry name, kind, repo, binding, and evolution; a svc.compat kind with a compatibility case-label floor; and a published_as outbound-leaf reference field resolving to declared ## Interfaces rows of the same file — adding no cross-file edge and no consumer-listing field, with the binding and repo columns staying opaque and files using none of this vocabulary linting byte-identically."
---

# svc-contracts-pack Specification

## Purpose

Ship the svc-contracts standard pack (proposed, draft): vocabulary for
services whose published contracts live in different repositories. The
base format already carries the cross-file edges (`extension_point` +
`satisfies`, `effect` + `observes`), so this pack adds none; it declares
only a typed `## Interfaces` registry row (name, kind, repo, opaque
binding to the authoritative external artifact, evolution mode), a
`svc.compat` law kind whose floor forces the `added` and `removed`
cases, and the `published_as` outbound leaf tying a contract row to its
registry row. Checkers ship as declarations (honest-empty), as in the
existing standard packs.

## Constraints

| id                        | kind      | expr                                                                                                                                                                                                                                                          | traces_to              |
|---------------------------|-----------|---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|------------------------|
| pack_file_declared        | invariant | `the svc-contracts standard pack is a kind: profile spec file at packs/svc-contracts.md (id svc.contracts) whose six manifest tables declare the Interfaces section, the svc.compat kind, the published_as reference field, two declaration-only checkers, the compat floor, and the specodelic.md Revision 18 base pin — discovered by corpus scan alone, no config file, no registry` | [[svc.contracts.pack]] |
| interfaces_section_shape  | invariant | `Interfaces rows carry exactly name, kind, repo, binding, evolution; kind is closed per-pack to request, event, schema and evolution to additive, versioned, frozen — per-pack closed sets the mechanism does not check; repo and binding are opaque strings never parsed; no base closed set grows and no existing entry narrows`                                       | [[svc.contracts.pack]] |
| published_as_outbound     | invariant | `published_as is a pack-added reference field resolving to an Interfaces row of the declaring file — an outbound leaf joining no reachability path and no acyclic edge set; its closure checker is declared, and reports an empty checked-set honestly when no Interfaces rows exist`                                                                              | [[svc.contracts.pack]] |
| compat_floor_declared     | invariant | `svc.compat laws enumerate the added and removed case labels per the pack floor, reusing the machine-findable bold-name case-label mechanism with a different required label set; the floor is a declaration the mechanism names and never executes`                                                                                                              | [[svc.contracts.pack]] |
| no_new_cross_repo_edge    | invariant | `the pack declares no cross-file edge field and no field or section listing a contract's consumers: cross-repo edges stay the base satisfies and observes fields, and the provider-to-consumer direction stays derived from the graph, never authored (USAGE.md section 2.6)`                                                                                      | [[svc.contracts.pack]] |
| vocabulary_prose_safe     | invariant | `no pack vocabulary token is a bare word that appears with exact case in the corpus prose of specs/ or openspec/specs/ — the token audit is recorded in the change design and re-run at landing`                                                                                                                                                                  | [[svc.contracts.pack]] |

## Model

### States
- `draft`
- `published`
- `deprecated`

### Transitions

| id        | from      | to         | guard                                                                                                                                          |
|-----------|-----------|------------|------------------------------------------------------------------------------------------------------------------------------------------------|
| publish   | draft     | published  | `[[svc.contracts.pack.pack_file_declared]] ∧ [[svc.contracts.pack.interfaces_section_shape]] — pack_shape reports zero findings over the pack's declared vocabulary` |
| deprecate | published | deprecated | `[[svc.contracts.pack.pack_file_declared]] — maintainer marks the pack deprecated; findings name the deprecation, vocabulary still checks`                               |

## Properties

| id                          | kind | derives_from                                 | generator                                        | predicate                                                                                          |
|-----------------------------|------|----------------------------------------------|--------------------------------------------------|----------------------------------------------------------------------------------------------------|
| discovery_finds_svc_pack    | unit | [[svc.contracts.pack.pack_file_declared]]    | `lint_workspace_with_svc_pack_present()`         | `the pack is discovered by corpus scan alone and the activation advisory names its draft status`   |
| no_pack_no_change           | unit | [[svc.contracts.pack.pack_file_declared]]    | `corpus_linted_before_and_after_svc_pack()`      | `lint output byte-identical for a corpus using no svc vocabulary`                                  |
| interfaces_shape_lints_clean| unit | [[svc.contracts.pack.interfaces_section_shape]] | `pack_file_linted_alone()`                    | `pack_shape reports zero findings`                                                                 |
| honest_empty                | unit | [[svc.contracts.pack.published_as_outbound]] | `svc_pack_active_with_no_interfaces_rows()`      | `the closure checker reports an empty checked-set with no fabricated findings`                     |
| floor_names_cases           | unit | [[svc.contracts.pack.compat_floor_declared]] | `read_floor_row_for_svc_compat()`                | `the floor names added and removed as the required case labels`                                    |
| no_consumer_field_declared  | unit | [[svc.contracts.pack.no_new_cross_repo_edge]]| `scan_manifest_for_consumer_listing_fields()`    | `no declared section, kind, or reference field lists consumers or adds a cross-file edge`          |
| vocabulary_audit_holds      | unit | [[svc.contracts.pack.vocabulary_prose_safe]] | `audit_pack_tokens_against_corpus_prose()`       | `zero exact-case hits for every vocabulary token in specs/ and openspec/specs/`                    |

## ADDED Requirements

### Requirement: The svc-contracts standard pack is a declared profile file
The format SHALL ship the svc-contracts standard pack as an in-repo
`kind: profile` spec file (`packs/svc-contracts.md`, id `svc.contracts`)
whose manifest declares the `## Interfaces` section, the `svc.compat`
kind, the `published_as` reference field, the `svc.interface_closed` and
`svc.compat_cases` checkers, the compatibility floor, and the
`specodelic.md Revision 18` base pin — discovered by corpus scan alone,
with no config file and no registry.

#### Scenario: Corpus scan discovers the pack
- **WHEN** `spk lint` runs over a workspace subdirectory of a repository containing `packs/svc-contracts.md`
- **THEN** the pack is discovered without configuration
- **AND** files using none of the pack's vocabulary keep byte-identical lint findings

#### Scenario: Pack lints clean under pack_shape
- **WHEN** the pack file is parsed in its landing lifecycle state
- **THEN** pack_shape reports zero findings over the pack's declared vocabulary

### Requirement: Interfaces vocabulary is namespaced and per-pack closed
The pack SHALL declare an `## Interfaces` section with row shape
`| name | kind | repo | binding | evolution |`, `kind` closed per-pack to
`request`, `event`, `schema`, `evolution` closed per-pack to `additive`,
`versioned`, `frozen`, and the namespaced kind `svc.compat` — pack-fiber-
relative vocabulary that grows no base closed set and narrows no
existing base set entry. The `repo` and `binding` columns SHALL stay
opaque strings that the pack never parses.

#### Scenario: Consumer activates the pack advisory-first
- **WHEN** a corpus spec uses `svc.compat`, `Interfaces`, or `published_as` without a declared `uses` edge
- **THEN** the pack's declared checking activates advisory-first for that file, naming the pack and its draft status

#### Scenario: Orphan vocabulary names the svc pack
- **WHEN** a workspace uses `svc.compat` with no svc-contracts pack discovered or declared
- **THEN** the orphan finding names the candidate namespace `svc` and both remediations, with non-zero exit

### Requirement: published_as is a typed outbound leaf resolving to declared rows
The pack SHALL add the `published_as` reference field, resolving to an
`## Interfaces` row of the declaring file and joining no reachability
path and no acyclic edge set, and SHALL declare the `svc.interface_closed`
checker that reports an empty checked-set honestly when no `## Interfaces`
rows exist.

#### Scenario: Honest-empty closure with no interfaces rows
- **WHEN** the pack is active in a workspace whose specs declare no `## Interfaces` rows
- **THEN** the closure checker reports an empty checked-set with no fabricated findings

#### Scenario: Closure is a declaration, not an executed check
- **WHEN** a constraint row names an `## Interfaces` row id via `published_as` that no row in the file declares
- **THEN** the pack's Checkers facet names the closure rule and the mechanism reports its checked-set honestly — the pack never claims an executed check the mechanism does not run

### Requirement: The compatibility floor names its case labels
The pack SHALL declare a kind-dependent floor: `svc.compat` laws
enumerate their `**added:**` and `**removed:**` case labels per the floor
declaration, reusing the machine-findable `**name:**` case-label
enumeration mechanism with a different required label set, and SHALL
NOT re-declare any base or other-pack kind.

#### Scenario: Compat floor names its case labels
- **WHEN** a `svc.compat` property is read against the pack's floor declaration
- **THEN** the floor names `added` and `removed` as the required case labels

#### Scenario: Floor is declaration-only
- **WHEN** a `svc.compat` property omits the floor's case labels
- **THEN** the pack's declared floor is the contract and the mechanism does not fabricate a finding the checker declaration does not execute

### Requirement: The pack adds no cross-file edge and no consumer listing
The pack SHALL NOT declare any section, kind, or reference field that
lists a contract's consumers or that adds a cross-file edge: cross-repo
edges SHALL remain the base `satisfies` (to an `extension_point`) and
`observes` (to an `effect`) fields, and the provider-to-consumer
direction SHALL stay derived from the reference graph.

#### Scenario: Cross-repo contract uses only base edges
- **WHEN** a consumer repo's constraint row cites a provider repo's `extension_point` through `satisfies` and an `effect` through `observes`
- **THEN** both resolve corpus-wide with no pack involved, and the provider's file names no consumer

#### Scenario: Manifest declares no consumer field
- **WHEN** the pack's `## Sections`, `## Kinds`, and `## References` facets are read
- **THEN** none declares a consumers or consumed-by listing or a second cross-file edge

## Requirements

### Requirement: The svc-contracts standard pack is a declared profile file
The format SHALL ship the svc-contracts standard pack as an in-repo
`kind: profile` spec file (`packs/svc-contracts.md`, id `svc.contracts`)
whose manifest declares the `## Interfaces` section, the `svc.compat`
kind, the `published_as` reference field, the `svc.interface_closed` and
`svc.compat_cases` checkers, the compatibility floor, and the
`specodelic.md Revision 18` base pin — discovered by corpus scan alone,
with no config file and no registry.

#### Scenario: Corpus scan discovers the pack
- **WHEN** `spk lint` runs over a workspace subdirectory of a repository containing `packs/svc-contracts.md`
- **THEN** the pack is discovered without configuration
- **AND** files using none of the pack's vocabulary keep byte-identical lint findings

#### Scenario: Pack lints clean under pack_shape
- **WHEN** the pack file is parsed in its landing lifecycle state
- **THEN** pack_shape reports zero findings over the pack's declared vocabulary

### Requirement: Interfaces vocabulary is namespaced and per-pack closed
The pack SHALL declare an `## Interfaces` section with row shape
`| name | kind | repo | binding | evolution |`, `kind` closed per-pack to
`request`, `event`, `schema`, `evolution` closed per-pack to `additive`,
`versioned`, `frozen`, and the namespaced kind `svc.compat` — pack-fiber-
relative vocabulary that grows no base closed set and narrows no
existing base set entry. The `repo` and `binding` columns SHALL stay
opaque strings that the pack never parses.

#### Scenario: Consumer activates the pack advisory-first
- **WHEN** a corpus spec uses `svc.compat`, `Interfaces`, or `published_as` without a declared `uses` edge
- **THEN** the pack's declared checking activates advisory-first for that file, naming the pack and its draft status

#### Scenario: Orphan vocabulary names the svc pack
- **WHEN** a workspace uses `svc.compat` with no svc-contracts pack discovered or declared
- **THEN** the orphan finding names the candidate namespace `svc` and both remediations, with non-zero exit

### Requirement: published_as is a typed outbound leaf resolving to declared rows
The pack SHALL add the `published_as` reference field, resolving to an
`## Interfaces` row of the declaring file and joining no reachability
path and no acyclic edge set, and SHALL declare the `svc.interface_closed`
checker that reports an empty checked-set honestly when no `## Interfaces`
rows exist.

#### Scenario: Honest-empty closure with no interfaces rows
- **WHEN** the pack is active in a workspace whose specs declare no `## Interfaces` rows
- **THEN** the closure checker reports an empty checked-set with no fabricated findings

#### Scenario: Closure is a declaration, not an executed check
- **WHEN** a constraint row names an `## Interfaces` row id via `published_as` that no row in the file declares
- **THEN** the pack's Checkers facet names the closure rule and the mechanism reports its checked-set honestly — the pack never claims an executed check the mechanism does not run

### Requirement: The compatibility floor names its case labels
The pack SHALL declare a kind-dependent floor: `svc.compat` laws
enumerate their `**added:**` and `**removed:**` case labels per the floor
declaration, reusing the machine-findable `**name:**` case-label
enumeration mechanism with a different required label set, and SHALL
NOT re-declare any base or other-pack kind.

#### Scenario: Compat floor names its case labels
- **WHEN** a `svc.compat` property is read against the pack's floor declaration
- **THEN** the floor names `added` and `removed` as the required case labels

#### Scenario: Floor is declaration-only
- **WHEN** a `svc.compat` property omits the floor's case labels
- **THEN** the pack's declared floor is the contract and the mechanism does not fabricate a finding the checker declaration does not execute

### Requirement: The pack adds no cross-file edge and no consumer listing
The pack SHALL NOT declare any section, kind, or reference field that
lists a contract's consumers or that adds a cross-file edge: cross-repo
edges SHALL remain the base `satisfies` (to an `extension_point`) and
`observes` (to an `effect`) fields, and the provider-to-consumer
direction SHALL stay derived from the reference graph.

#### Scenario: Cross-repo contract uses only base edges
- **WHEN** a consumer repo's constraint row cites a provider repo's `extension_point` through `satisfies` and an `effect` through `observes`
- **THEN** both resolve corpus-wide with no pack involved, and the provider's file names no consumer

#### Scenario: Manifest declares no consumer field
- **WHEN** the pack's `## Sections`, `## Kinds`, and `## References` facets are read
- **THEN** none declares a consumers or consumed-by listing or a second cross-file edge
