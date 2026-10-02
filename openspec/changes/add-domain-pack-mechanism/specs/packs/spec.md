---
id: spec
kind: intent
statement: "WHEN a workspace enables a domain pack, THE format SHALL make the extension mechanism first-class — a kind: profile spec file whose six per-facet manifest tables declare exactly the sections, namespaced kinds, reference fields, checkers, floors, and requirements the pack introduces — and SHALL activate pack checking advisory-first by vocabulary use or a declared uses edge, never by config files, never by narrowing any base closed set, and never by changing how files without packs lint."
---

# packs Specification

## Purpose

Make specodelic's extension mechanism first-class: a domain pack is a
declared, discoverable, versioned artifact in the corpus itself — not a
convention followed by hand. Base closed sets stop growing; vocabulary is
pack-fiber-relative (the Grothendieck construction one level up per
theory.md); independent packs coexist namespaced instead of colliding in
the global kind sets.

## Constraints

| id                        | kind      | expr                                                                                                                                                                                                                                  | traces_to |
|---------------------------|-----------|---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|-----------|
| pack_is_profile_file      | invariant | `a pack is a four-layer spec file with kind: profile in frontmatter — no separate artifact format, no external manifest, self-hosting preserved`                                                                                       | [[spec]]  |
| manifest_six_tables       | invariant | `a pack manifest is exactly six per-facet closed tables — ## Sections, ## Kinds, ## References, ## Checkers, ## Floors, ## Requires — each with a fixed row shape checked by the pack_shape lint rule`                                 | [[spec]]  |
| discovery_corpus_scan     | invariant | `every kind: profile file in the workspace is a candidate pack, found by corpus scan alone — no config file, no registry outside the corpus`                                                                                           | [[spec]]  |
| namespaced_vocabulary     | invariant | `pack-declared kinds, sections, reference fields, and predicate names are pack-qualified in fiber scope; two packs claiming the same mechanism is legitimate, collision exists only at vocabulary level and resolves by pack-qualified naming` | [[spec]]  |
| base_closed_sets_frozen   | invariant | `base closed sets (constraint kinds, property kinds, intent kinds, reference typing) stop growing except by true format Revision; a pack can never narrow or retype an existing set entry`                                             | [[spec]]  |
| advisory_first_opt_in     | invariant | `pack checking activates advisory-first when a file uses the pack's declared vocabulary, upgradeable to a declared uses edge; files without packs lint byte-identically to pre-mechanism behavior`                                     | [[spec]]  |
| orphan_vocabulary_labeled | invariant | `declared vocabulary used with no pack discovered or declared produces a labeled finding naming the pack — never a generic dangling message, never silent`                                                                              | [[spec]]  |
| uses_is_outbound_leaf     | invariant | `the uses reference field targets a profile file's frontmatter id and is an outbound leaf — nothing targets it, single_root_reachable needs no carve-out`                                                                               | [[spec]]  |
| lifecycle_labeled         | invariant | `pack states are draft, published, deprecated; consumers pin base format_revision and pack deps via ## Requires; deprecated packs stay advisory with deprecation named in findings`                                                     | [[spec]]  |
| skew_advisory             | invariant | `revision skew between an enabled pack and the workspace corpus revision is a labeled advisory on the warnings channel — never silent, never failing`                                                                                   | [[spec]]  |
| honest_empty              | invariant | `a pack checker with no workspace fixtures, data, or vocabulary instances reports an empty checked-set honestly — no fabricated discoveries`                                                                                            | [[spec]]  |
| append_only_packs         | invariant | `append_only_variants applies to packs themselves — a pack release may only add vocabulary, never remove or narrow previously declared entries`                                                                                          | [[spec]]  |

## Model

### States
- `draft`
- `published`
- `deprecated`

### Transitions

| id        | from     | to         | guard                                                                                      |
|-----------|----------|------------|--------------------------------------------------------------------------------------------|
| publish   | draft    | published  | `[[spec.pack_is_profile_file]] ∧ [[spec.manifest_six_tables]] — pack_shape reports zero findings over the pack's own declared vocabulary` |
| deprecate | published| deprecated | `[[spec.lifecycle_labeled]] — maintainer marks the pack deprecated; findings name the deprecation, vocabulary still checks` |

## Properties

| id                          | kind | derives_from                     | generator                                   | predicate                                                                    |
|-----------------------------|------|----------------------------------|---------------------------------------------|------------------------------------------------------------------------------|
| manifest_round_trips        | unit | [[spec.manifest_six_tables]]     | `pack_file_parsed_then_manifest_read()`     | `declared entries equal parsed manifest rows — no drift, no prose leakage`   |
| discovery_finds_all_packs   | unit | [[spec.discovery_corpus_scan]]   | `corpus_with_n_profile_files()`             | `scan reports exactly n packs, order-independent`                             |
| no_pack_no_change           | unit | [[spec.advisory_first_opt_in]]   | `corpus_linted_before_and_after_mechanism()`| `lint findings byte-identical for a corpus using no pack vocabulary`          |
| orphan_finding_names_pack   | unit | [[spec.orphan_vocabulary_labeled]] | `corpus_using_vocabulary_without_pack()`  | `finding names the candidate pack and both remediations`                      |
| uses_edge_typing            | unit | [[spec.uses_is_outbound_leaf]]   | `file_with_uses_edge_to_profile_id()`       | `graph edge typed uses resolves to the profile id; wrong-kind targets violate typing` |
| skew_is_warning_not_failure | unit | [[spec.skew_advisory]]           | `pack_pinned_to_older_revision()`           | `exit 0 with warning-channel advisory naming both revisions`                  |
| empty_vocabulary_honest     | unit | [[spec.honest_empty]]            | `pack_enabled_without_instances()`          | `checker reports empty checked-set; no fabricated findings`                   |
| narrowing_rejected          | unit | [[spec.append_only_packs]]       | `pack_removing_previously_declared_kind()`  | `pack_shape fails naming the removed entry and the append-only law`           |
| profile_file_is_four_layer  | unit | [[spec.pack_is_profile_file]]    | `pack_parsed_as_standard_spec()`            | `all four layers parse from the pack file alone — no sidecar artifact needed` |
| pack_vocabulary_qualified   | unit | [[spec.namespaced_vocabulary]]   | `two_packs_claim_same_mechanism()`          | `both packs' declared vocabulary resolves without cross-pack collision`       |
| base_sets_unchanged_by_packs| unit | [[spec.base_closed_sets_frozen]] | `guide_sets_compared_across_pack_enable()`  | `INTENT_KINDS and Reference Typing sets identical with and without packs discovered` |
| lifecycle_states_parsed     | unit | [[spec.lifecycle_labeled]]       | `pack_model_section_parsed()`               | `each lifecycle state yields exactly its declared checking behavior`          |

## ADDED Requirements

### Requirement: Pack artifact is a declared profile file
The system SHALL treat a domain pack as a four-layer spec file with
`kind: profile` whose manifest is exactly six per-facet closed tables
(`## Sections`, `## Kinds`, `## References`, `## Checkers`, `## Floors`,
`## Requires`) — no external manifest, no config file — and SHALL validate
manifest structure with an append-only `pack_shape` lint rule.

#### Scenario: Well-formed pack lints clean
- **WHEN** a `kind: profile` file carries all six manifest tables with well-formed rows
- **THEN** `spk lint` reports no `pack_shape` findings for it

#### Scenario: Malformed manifest is a labeled finding
- **WHEN** a `kind: profile` file is missing a required manifest table or has a row not matching the facet's shape
- **THEN** `pack_shape` emits a finding naming the missing or malformed facet

### Requirement: Discovery by corpus scan
The system SHALL discover packs by scanning the workspace corpus for
`kind: profile` files — every such file is a candidate pack, and no pack
registration may live outside the corpus.

#### Scenario: Profile file found without configuration
- **WHEN** a workspace contains a `kind: profile` file and no configuration anywhere
- **THEN** `spk lint` and `spk doctor` discover and report the pack

#### Scenario: No profile files means no packs
- **WHEN** the corpus contains no `kind: profile` files
- **THEN** no pack machinery activates and lint output is unchanged

### Requirement: Vocabulary is pack-namespaced and base sets stay frozen
The system SHALL scope pack-declared kinds, sections, reference fields,
and predicate names to the declaring pack (pack-qualified naming), SHALL
let two packs claim the same mechanism without collision, and SHALL NOT
grow any base closed set except by a true format Revision.

#### Scenario: Two packs may declare the same mechanism
- **WHEN** two discovered packs each declare a data section mechanism
- **THEN** both lint clean — collision exists only for identical vocabulary, resolved by pack-qualified naming

#### Scenario: Pack cannot narrow a base set
- **WHEN** a pack manifest claims to remove or retype a base-set entry
- **THEN** `pack_shape` fails naming the entry and the append-only law

### Requirement: Advisory-first opt-in with declared upgrade path
The system SHALL activate a pack's checking on a file when that file uses
the pack's declared vocabulary (advisory-first), SHALL support explicit
opt-in via a declared `uses` edge to the pack's frontmatter id, SHALL
produce a labeled finding naming the pack when declared vocabulary appears
with no pack present, and SHALL leave files without packs byte-identical
in lint behavior.

#### Scenario: Vocabulary use triggers advisory checking
- **WHEN** a file uses vocabulary declared by a discovered pack
- **THEN** the pack's checkers run for that file and report advisory-first results

#### Scenario: Declared uses edge pins the pack
- **WHEN** a file declares `uses` targeting a discovered pack's id
- **THEN** the pack's checks apply with pinning/skew validation enabled

#### Scenario: Orphan vocabulary is labeled
- **WHEN** a file uses vocabulary matching a pack's declared namespace but no pack is discovered
- **THEN** a finding names the candidate pack and both remediations (enable/declare the pack, or fix the vocabulary)

### Requirement: Pack lifecycle and revision skew are labeled advisories
The system SHALL carry pack lifecycle states `draft`, `published`, and
`deprecated` in the pack's Model section, SHALL let consumers pin the base
format_revision and pack dependencies via `## Requires`, SHALL report
revision skew as a labeled advisory on the warnings channel, and SHALL
name deprecation in findings for deprecated packs.

#### Scenario: Skew warns without failing
- **WHEN** a consumer pins a pack requiring an older base format_revision than the workspace corpus
- **THEN** lint exits 0 with an advisory naming both revisions

#### Scenario: Deprecated pack stays honest
- **WHEN** a deprecated pack's vocabulary is used
- **THEN** findings still fire and each names the deprecation

### Requirement: Honest-empty pack checking
The system SHALL report an empty checked-set when a pack's checkers find
no workspace fixtures, data, or vocabulary instances — never fabricating
discoveries, preserving the honest-empty convention.

#### Scenario: Pack with nothing to check stays honest
- **WHEN** a pack is enabled but the workspace has no instances of its vocabulary
- **THEN** its checker output reports an empty checked-set and exits clean

## Requirements

### Requirement: Pack artifact is a declared profile file
The system SHALL treat a domain pack as a four-layer spec file with
`kind: profile` whose manifest is exactly six per-facet closed tables
(`## Sections`, `## Kinds`, `## References`, `## Checkers`, `## Floors`,
`## Requires`) — no external manifest, no config file — and SHALL validate
manifest structure with an append-only `pack_shape` lint rule.

#### Scenario: Well-formed pack lints clean
- **WHEN** a `kind: profile` file carries all six manifest tables with well-formed rows
- **THEN** `spk lint` reports no `pack_shape` findings for it

#### Scenario: Malformed manifest is a labeled finding
- **WHEN** a `kind: profile` file is missing a required manifest table or has a row not matching the facet's shape
- **THEN** `pack_shape` emits a finding naming the missing or malformed facet

### Requirement: Discovery by corpus scan
The system SHALL discover packs by scanning the workspace corpus for
`kind: profile` files — every such file is a candidate pack, and no pack
registration may live outside the corpus.

#### Scenario: Profile file found without configuration
- **WHEN** a workspace contains a `kind: profile` file and no configuration anywhere
- **THEN** `spk lint` and `spk doctor` discover and report the pack

#### Scenario: No profile files means no packs
- **WHEN** the corpus contains no `kind: profile` files
- **THEN** no pack machinery activates and lint output is unchanged

### Requirement: Vocabulary is pack-namespaced and base sets stay frozen
The system SHALL scope pack-declared kinds, sections, reference fields,
and predicate names to the declaring pack (pack-qualified naming), SHALL
let two packs claim the same mechanism without collision, and SHALL NOT
grow any base closed set except by a true format Revision.

#### Scenario: Two packs may declare the same mechanism
- **WHEN** two discovered packs each declare a data section mechanism
- **THEN** both lint clean — collision exists only for identical vocabulary, resolved by pack-qualified naming

#### Scenario: Pack cannot narrow a base set
- **WHEN** a pack manifest claims to remove or retype a base-set entry
- **THEN** `pack_shape` fails naming the entry and the append-only law

### Requirement: Advisory-first opt-in with declared upgrade path
The system SHALL activate a pack's checking on a file when that file uses
the pack's declared vocabulary (advisory-first), SHALL support explicit
opt-in via a declared `uses` edge to the pack's frontmatter id, SHALL
produce a labeled finding naming the pack when declared vocabulary appears
with no pack present, and SHALL leave files without packs byte-identical
in lint behavior.

#### Scenario: Vocabulary use triggers advisory checking
- **WHEN** a file uses vocabulary declared by a discovered pack
- **THEN** the pack's checkers run for that file and report advisory-first results

#### Scenario: Declared uses edge pins the pack
- **WHEN** a file declares `uses` targeting a discovered pack's id
- **THEN** the pack's checks apply with pinning/skew validation enabled

#### Scenario: Orphan vocabulary is labeled
- **WHEN** a file uses vocabulary matching a pack's declared namespace but no pack is discovered
- **THEN** a finding names the candidate pack and both remediations (enable/declare the pack, or fix the vocabulary)

### Requirement: Pack lifecycle and revision skew are labeled advisories
The system SHALL carry pack lifecycle states `draft`, `published`, and
`deprecated` in the pack's Model section, SHALL let consumers pin the base
format_revision and pack dependencies via `## Requires`, SHALL report
revision skew as a labeled advisory on the warnings channel, and SHALL
name deprecation in findings for deprecated packs.

#### Scenario: Skew warns without failing
- **WHEN** a consumer pins a pack requiring an older base format_revision than the workspace corpus
- **THEN** lint exits 0 with an advisory naming both revisions

#### Scenario: Deprecated pack stays honest
- **WHEN** a deprecated pack's vocabulary is used
- **THEN** findings still fire and each names the deprecation

### Requirement: Honest-empty pack checking
The system SHALL report an empty checked-set when a pack's checkers find
no workspace fixtures, data, or vocabulary instances — never fabricating
discoveries, preserving the honest-empty convention.

#### Scenario: Pack with nothing to check stays honest
- **WHEN** a pack is enabled but the workspace has no instances of its vocabulary
- **THEN** its checker output reports an empty checked-set and exits clean
