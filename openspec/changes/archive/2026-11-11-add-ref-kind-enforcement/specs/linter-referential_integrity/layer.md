---
id: linter.referential_integrity.layer
kind: intent
statement: "WHEN a spec repo is linted, THE linter SHALL reject it if any row id is duplicated within a file, any wiki-link reference fails to resolve to a defining row, or any reference field defined by the Reference Typing table resolves to a row whose kind the table forbids, reading allowed targets from the Schema-as-data table."
---

# Linter: Referential Integrity Check

Runs after `linter.frontmatter` passes for every file in the repo — it needs
every file's declared `id` to already be trustworthy before it can build the
id→row index this check relies on.

## Purpose

Runs after `linter.frontmatter` passes for every file in the repo — it needs
every file's declared `id` to already be trustworthy before it can build the
id→row index this check relies on.

## Constraints

| id                  | kind      | expr                                                                                   | traces_to | satisfies |
|----------------------|-----------|-------------------------------------------------------------------------------------------|--------------------------------------------------|-----------|
| unique_within_file   | invariant | `∀ file: no two rows in file share the same local id`                                     | [[linter.referential_integrity.layer]]           |          |
| unique_across_repo   | invariant | `∀ repo: no two files declare the same fully-qualified id`                                | [[linter.referential_integrity.layer]]           |          |
| ref_resolves         | invariant | `∀ [[ref]] in file: qualify(ref) ∈ index(repo)`                                            | [[linter.referential_integrity.layer]]          |          |
| ref_kind_compatible  | invariant | `∀ [[ref]] in any reference field defined by specodelic.md's Reference Typing table: target row's kind ∈ allowed_targets(field)` — read from that table, not hardcoded per field name, so a Revision adding a field (`supersedes`, `emits`, ...) needs no change here | [[linter.referential_integrity.layer]]          |          |
| index_failure | effect | `linter.referential_integrity.layer.index_failure(detail)` | [[linter.referential_integrity.layer]] | [[errors.envelope_error_kind]] ∧ [[errors.exit_code_mapping]] ∧ [[errors.remediation_hint_present]] |
| resolution_failure | effect | `linter.referential_integrity.layer.resolution_failure(detail)` | [[linter.referential_integrity.layer]] | [[errors.envelope_error_kind]] ∧ [[errors.exit_code_mapping]] ∧ [[errors.remediation_hint_present]] |
| schema_is_decision | invariant | `the allowed targets for a typed field are read from the canonical Schema via its typing check — no per-field match arm exists in the lint rule` | [[linter.referential_integrity.layer]] |
| typed_columns_only | invariant | `the rule walks only links whose column carries schema morphism rows — frontmatter, expr, predicate, and generator cells are format prose, never typed references` | [[linter.referential_integrity.layer]] |
| dangling_is_total_refs | invariant | `a typed link that resolves to no row or file produces exactly the dangling finding — typing never double-reports an unresolved target` | [[linter.referential_integrity.layer]] |
| severity_error | invariant | `a kind-incompatible resolved target is an issues-channel finding (exit 1) — the invariant gates the checker's accept transition, never the warnings channel` | [[linter.referential_integrity.layer]] |
| member_arm_types_row | invariant | `a member-arm target (file.row.member) canonicalizes to file.row and is typed as the row's kind — the stricter of the graph and oracle derivations` | [[linter.referential_integrity.layer]] |
| uses_types_frontmatter | invariant | `a file-id target under uses is typed as Intent with the file's frontmatter kind — a non-profile target fires, a profile pack id stays clean` | [[linter.referential_integrity.layer]] |
| from_to_not_duplicated | invariant | `from/to State typing stays the model-shape checker's beat — this rule adds no second finding for the same cell` | [[linter.referential_integrity.layer]] |
| corpus_stays_green | invariant | `the rule firing over the repo's own corpus yields zero findings — dogfooding is the gate` | [[linter.referential_integrity.layer]] |

## Model

### States
- `unindexed`
- `indexing`
- `indexed`
- `resolving`
- `passed`
- `index_failed` (emits: `[[linter.referential_integrity.layer.index_failure]]`)
- `resolution_failed` (emits: `[[linter.referential_integrity.layer.resolution_failure]]`)

### Transitions

| id             | from       | to         | guard                                                                                       |
|----------------|------------|------------|----------------------------------------------------------------------------------------------|
| build_index    | unindexed  | indexing   | `all files in repo passed linter.frontmatter`                                                |
| index_ok       | indexing   | indexed    | [[linter.referential_integrity.layer.unique_within_file]] ∧ [[linter.referential_integrity.layer.unique_across_repo]] |
| index_fail | indexing | index_failed | `¬([[linter.referential_integrity.layer.unique_within_file]] ∧ [[linter.referential_integrity.layer.unique_across_repo]])` |
| resolve_refs   | indexed    | resolving  | `index built successfully`                                                                    |
| accept         | resolving  | passed     | [[linter.referential_integrity.layer.ref_resolves]] ∧ [[linter.referential_integrity.layer.ref_kind_compatible]] |
| reject | resolving | resolution_failed | `¬([[linter.referential_integrity.layer.ref_resolves]] ∧ [[linter.referential_integrity.layer.ref_kind_compatible]])` |

## Properties

| id                     | kind | derives_from                                             | generator                                          | predicate                                                                 |
|-------------------------|------|-------------------------------------------------------------|--------------------------------------------------------|-----------------------------------------------------------------------------|
| duplicate_id_rejected    | unit | [[linter.referential_integrity.layer.unique_within_file]]        | `spec_file_with(duplicate_row_id: true)`               | `check(file) == failed`                                                    |
| cross_file_collision     | unit | [[linter.referential_integrity.layer.unique_across_repo]]        | `two_spec_files_sharing_id()`                          | `check(repo) == failed`                                                    |
| dangling_ref_rejected    | unit | [[linter.referential_integrity.layer.ref_resolves]]              | `spec_file_with(ref_to_nonexistent_id: true)`           | `check(file) == failed`                                                    |
| wrong_target_kind        | unit | [[linter.referential_integrity.layer.ref_kind_compatible]]       | `guard_field_pointing_at_a_property_row()`              | `check(file) == failed`                                                    |
| rename_naturality        | law  | [[specodelic.rename_naturality]]                          | `arbitrary_spec_repo(), arbitrary_id_rename()`          | **identity:** `resolve(rename(I,a,a)) == resolve(I)`  **associativity:** `resolve(rename(rename(I,a,b),b,c)) == resolve(rename(I,a,c))`  **naturality:** `lookup_all(rename(I)) == rename(lookup_all(I))` — every `[[ref]]` pointing at a renamed id is updated with it, the naturality case this checker is the enforcement layer for |
| clean_repo_passes        | unit | [[linter.referential_integrity.layer.ref_resolves]]              | `arbitrary_well_formed_repo()`                          | `check(repo) == passed`                                                    |
| index_failure_label_asserted | unit | [[linter.referential_integrity.layer.index_failure]] | `index_failure_raised()` | `error_label == "linter.referential_integrity.layer.index_failure"` — renaming the label touches the error Constraint, this property, and its note together (EDGE-002) |
| resolution_failure_label_asserted | unit | [[linter.referential_integrity.layer.resolution_failure]] | `resolution_failure_raised()` | `error_label == "linter.referential_integrity.layer.resolution_failure"` — renaming the label touches the error Constraint, this property, and its note together (EDGE-002) |
| p_schema_decision | unit | [[linter.referential_integrity.layer.schema_is_decision]] | `arbitrary_typed_link()` | `the lint verdict matches the graph report on the same link` |
| p_typed_only | unit | [[linter.referential_integrity.layer.typed_columns_only]] | `links_in_untyped_cells()` | `no finding names an untyped column` |
| p_dangling_once | unit | [[linter.referential_integrity.layer.dangling_is_total_refs]] | `typed_cell_with_dangling_target()` | `findings == [the dangling finding]` |
| p_severity | unit | [[linter.referential_integrity.layer.severity_error]] | `violating_corpus()` | `exit code 1 ∧ finding on the issues channel` |
| p_member_arm | unit | [[linter.referential_integrity.layer.member_arm_types_row]] | `member_arm_links()` | `typed as the canonical row's kind` |
| p_uses_profile | unit | [[linter.referential_integrity.layer.uses_types_frontmatter]] | `uses_links()` | `profile target passes ∧ intent target fires` |
| p_from_to_once | unit | [[linter.referential_integrity.layer.from_to_not_duplicated]] | `transition_with_ghost_endpoint()` | `exactly the model-shape finding fires` |
| p_enforcement_clean | unit | [[linter.referential_integrity.layer.corpus_stays_green]] | `arbitrary_well_formed_corpus()` | `exit 0 ∧ zero typing findings` |
