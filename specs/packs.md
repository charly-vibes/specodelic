---
id: packs
kind: intent
checked_against_core: clear
statement: "WHEN a workspace enables a domain pack, THE format SHALL make the extension mechanism first-class — a kind: profile spec file whose six per-facet manifest tables declare exactly the sections, namespaced kinds, reference fields, checkers, floors, and requirements the pack introduces — and SHALL activate pack checking advisory-first by vocabulary use or a declared uses edge, never by config files, never by narrowing any base closed set, and never by changing how files without packs lint."
---

# Packs

Make specodelic's extension mechanism first-class: a domain pack is a
declared, discoverable, versioned artifact in the corpus itself — not a
convention followed by hand. Base closed sets stop growing; vocabulary is
pack-fiber-relative (the Grothendieck construction one level up, per
[[theory]] precedent); independent packs coexist namespaced instead of
colliding in the global kind sets.

## Constraints

| id                        | kind      | expr                                                                                                                                                                                                                                  | traces_to |
|---------------------------|-----------|---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|-----------|
| pack_is_profile_file      | invariant | `a pack is a four-layer spec file with kind: profile in frontmatter — no separate artifact format, no external manifest, self-hosting preserved`                                                                                       | [[packs]]  |
| manifest_six_tables       | invariant | `a pack manifest is exactly six per-facet closed tables — ## Sections, ## Kinds, ## References, ## Checkers, ## Floors, ## Requires — each with a fixed row shape checked by the pack_shape lint rule`                                 | [[packs]]  |
| discovery_corpus_scan     | invariant | `every kind: profile file in the workspace is a candidate pack, found by corpus scan alone anchored at the git repository toplevel (the lint target directory when no git root exists) — no config file, no registry outside the corpus, and a subdirectory lint discovers packs living anywhere in the workspace` | [[packs]]  |
| namespaced_vocabulary     | invariant | `pack-declared kinds, sections, reference fields, and predicate names are pack-qualified in fiber scope; two packs claiming the same mechanism is legitimate, collision exists only at vocabulary level and resolves by pack-qualified naming` | [[packs]]  |
| base_closed_sets_frozen   | invariant | `base closed sets (constraint kinds, property kinds, intent kinds, reference typing) stop growing except by true format Revision; a pack can never narrow or retype an existing set entry`                                             | [[packs]]  |
| advisory_first_opt_in     | invariant | `pack checking activates advisory-first when a file uses the pack's declared vocabulary, upgradeable to a declared uses edge; activation is per-pack — identical vocabulary declared by two discovered packs activates both with findings attributed per pack — and vocabulary matching is longest-prefix; files without packs lint byte-identically to pre-mechanism behavior` | [[packs]]  |
| orphan_vocabulary_labeled | invariant | `declared vocabulary used with no pack discovered or declared produces a labeled failure finding (non-zero exit) naming the candidate pack — prefix-derived when only the namespace is known — and both remediations; never a generic dangling message, never silent` | [[packs]]  |
| uses_is_outbound_leaf     | invariant | `the uses reference field appears as a set-valued column on any Constraint row and resolves to the intent of a kind: profile file (the pack's frontmatter id); it is an outbound leaf joining no reachability path and no acyclic edge set — nothing targets it, single_root_reachable needs no carve-out, mutual pack use is well-formed` | [[packs]]  |
| pack_self_exempt          | invariant | `a pack file is never a consumer of declared vocabulary — its own manifest rows never trigger its or any pack's checkers and never produce orphan findings`                                                                            | [[packs]]  |
| lifecycle_labeled         | invariant | `pack states are draft, published, deprecated; pack_shape must pass at parse in every state and draft packs' checkers activate advisory-first naming the draft status; the pack-authored ## Requires table pins the base format_revision and pack deps; deprecated packs stay advisory with deprecation named in findings` | [[packs]]  |
| skew_advisory             | invariant | `revision skew between a pack's Requires base pin and the workspace corpus revision is a labeled advisory on the warnings channel — never silent, never failing`                                                                        | [[packs]]  |
| honest_empty              | invariant | `a pack checker with no workspace fixtures, data, or vocabulary instances reports an empty checked-set honestly — no fabricated discoveries`                                                                                            | [[packs]]  |
| append_only_packs         | invariant | `append_only_variants applies to packs themselves — a pack release may only add vocabulary, never remove or narrow previously declared entries`                                                                                          | [[packs]]  |

## Model

### States
- `draft`
- `published`
- `deprecated`

### Transitions

| id        | from     | to         | guard                                                                                      |
|-----------|----------|------------|--------------------------------------------------------------------------------------------|
| publish   | draft    | published  | `[[packs.pack_is_profile_file]] ∧ [[packs.manifest_six_tables]] — pack_shape reports zero findings over the pack's own declared vocabulary` |
| deprecate | published| deprecated | `[[packs.lifecycle_labeled]] — maintainer marks the pack deprecated; findings name the deprecation, vocabulary still checks` |

## Properties

| id                          | kind | derives_from                     | generator                                   | predicate                                                                    |
|-----------------------------|------|----------------------------------|---------------------------------------------|------------------------------------------------------------------------------|
| manifest_round_trips        | unit | [[packs.manifest_six_tables]]     | `pack_file_parsed_then_manifest_read()`     | `declared entries equal parsed manifest rows — no drift, no prose leakage`   |
| discovery_finds_all_packs   | unit | [[packs.discovery_corpus_scan]]   | `corpus_with_n_profile_files()`             | `scan reports exactly n packs, order-independent`                             |
| subdir_lint_discovers_workspace_packs | unit | [[packs.discovery_corpus_scan]] | `lint_subdirectory_with_pack_at_workspace_root()` | `pack discovered, its vocabulary checks run, zero orphan findings`       |
| no_pack_no_change           | unit | [[packs.advisory_first_opt_in]]   | `corpus_linted_before_and_after_mechanism()`| `lint findings byte-identical for a corpus using no pack vocabulary`          |
| overlapping_vocabulary_activates_both | unit | [[packs.advisory_first_opt_in]] | `two_packs_declare_identical_vocabulary_and_a_file_uses_it()` | `both packs' checkers run; findings attributed per pack`             |
| orphan_finding_names_pack   | unit | [[packs.orphan_vocabulary_labeled]] | `corpus_using_vocabulary_without_pack()`  | `failure finding names the candidate pack and both remediations, exit non-zero` |
| uses_edge_typing            | unit | [[packs.uses_is_outbound_leaf]]   | `file_with_uses_edge_to_profile_id()`       | `graph edge typed uses resolves to the profile id; wrong-kind targets violate typing` |
| pack_self_activation_absent | unit | [[packs.pack_self_exempt]]        | `pack_file_linted_alone()`                  | `its checkers report an empty checked-set; no orphan findings from the pack's own manifest rows` |
| skew_is_warning_not_failure | unit | [[packs.skew_advisory]]           | `pack_pinned_to_older_revision()`           | `exit 0 with warning-channel advisory naming the pack's base pin and the corpus revision` |
| empty_vocabulary_honest     | unit | [[packs.honest_empty]]            | `pack_enabled_without_instances()`          | `checker reports empty checked-set; no fabricated findings`                   |
| narrowing_rejected          | unit | [[packs.append_only_packs]]       | `pack_removing_previously_declared_kind()`  | `pack_shape fails naming the removed entry and the append-only law`           |
| profile_file_is_four_layer  | unit | [[packs.pack_is_profile_file]]    | `pack_parsed_as_standard_spec()`            | `all four layers parse from the pack file alone — no sidecar artifact needed` |
| pack_vocabulary_qualified   | unit | [[packs.namespaced_vocabulary]]   | `two_packs_claim_same_mechanism()`          | `both packs' declared vocabulary resolves without cross-pack collision`       |
| base_sets_unchanged_by_packs| unit | [[packs.base_closed_sets_frozen]] | `guide_sets_compared_across_pack_enable()`  | `INTENT_KINDS and Reference Typing sets identical with and without packs discovered` |
| lifecycle_states_parsed     | unit | [[packs.lifecycle_labeled]]       | `pack_model_section_parsed()`               | `each lifecycle state yields exactly its declared checking behavior`          |
