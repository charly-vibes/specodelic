---
id: linter.referential_integrity
kind: intent
statement: "WHEN a spec repo is parsed, THE linter SHALL reject it if any row id is duplicated within a file or any wiki-link reference fails to resolve to a defining row."
---

# Linter: Referential Integrity Check

Runs after `linter.frontmatter` passes for every file in the repo — it needs
every file's declared `id` to already be trustworthy before it can build the
id→row index this check relies on.

## Constraints

| id                  | kind      | expr                                                                                   | traces_to                          |
|----------------------|-----------|-------------------------------------------------------------------------------------------|---------------------------------------|
| unique_within_file   | invariant | `∀ file: no two rows in file share the same local id`                                     | [[specodelic.unique_id]]           |
| unique_across_repo   | invariant | `∀ repo: no two files declare the same fully-qualified id`                                | [[specodelic.unique_id]]           |
| ref_resolves         | invariant | `∀ [[ref]] in file: qualify(ref) ∈ index(repo)`                                            | [[specodelic.total_refs]]          |
| ref_kind_compatible  | invariant | `∀ [[ref]] in any reference field defined by specodelic.md's Reference Typing table: target row's kind ∈ allowed_targets(field)` — read from that table, not hardcoded per field name, so a Revision adding a field (`supersedes`, `emits`, ...) needs no change here | [[specodelic.total_refs]]          |

## Model

### States
- `unindexed`
- `indexing`
- `indexed`
- `resolving`
- `passed`
- `failed`

### Transitions

| id             | from       | to         | guard                                                                                       |
|----------------|------------|------------|----------------------------------------------------------------------------------------------|
| build_index    | unindexed  | indexing   | `all files in repo passed linter.frontmatter`                                                |
| index_ok       | indexing   | indexed    | [[linter.referential_integrity.unique_within_file]] ∧ [[linter.referential_integrity.unique_across_repo]] |
| index_fail     | indexing   | failed     | `¬index_ok.guard`                                                                             |
| resolve_refs   | indexed    | resolving  | `index built successfully`                                                                    |
| accept         | resolving  | passed     | [[linter.referential_integrity.ref_resolves]] ∧ [[linter.referential_integrity.ref_kind_compatible]] |
| reject         | resolving  | failed     | `¬accept.guard`                                                                                |

## Properties

| id                     | kind | derives_from                                             | generator                                          | predicate                                                                 |
|-------------------------|------|-------------------------------------------------------------|--------------------------------------------------------|-----------------------------------------------------------------------------|
| duplicate_id_rejected    | unit | [[linter.referential_integrity.unique_within_file]]        | `spec_file_with(duplicate_row_id: true)`               | `check(file) == failed`                                                    |
| cross_file_collision     | unit | [[linter.referential_integrity.unique_across_repo]]        | `two_spec_files_sharing_id()`                          | `check(repo) == failed`                                                    |
| dangling_ref_rejected    | unit | [[linter.referential_integrity.ref_resolves]]              | `spec_file_with(ref_to_nonexistent_id: true)`           | `check(file) == failed`                                                    |
| wrong_target_kind        | unit | [[linter.referential_integrity.ref_kind_compatible]]       | `guard_field_pointing_at_a_property_row()`              | `check(file) == failed`                                                    |
| rename_naturality        | law  | [[specodelic.rename_naturality]]                          | `arbitrary_spec_repo(), arbitrary_id_rename()`          | **identity:** `resolve(rename(I,a,a)) == resolve(I)`  **associativity:** `resolve(rename(rename(I,a,b),b,c)) == resolve(rename(I,a,c))` |
| clean_repo_passes        | unit | [[linter.referential_integrity.ref_resolves]]              | `arbitrary_well_formed_repo()`                          | `check(repo) == passed`                                                    |

## Notes

`rename_naturality` is restated here rather than only living in
`specodelic.md`, because *this* is the check that would actually break if
a naive rename tool patched an `id` field without updating every `[[ref]]`
pointing at it — the property belongs at the layer that enforces it, with
`specodelic.md` keeping the higher-level restatement as its own
traceability anchor. This is the same "same claim, two altitudes" pattern
as `ah` reading `dont`'s events: one tool states the law, another is where
violating it actually fails.

The open question this file's Notes used to raise — that `ref_kind_compatible`
assumes a fixed "which kind may fill which field" table that wasn't
specified anywhere — was resolved in `specodelic.md` Revision 2, which
added the Reference Typing table for exactly this reason. What was never
fixed, until this pass (`specodelic.md` Revision 6), is that this file's
own `expr` still named the three original fields by hand
(`traces_to/derives_from/guard`) instead of pointing at that table
generically — so it silently fell behind twice, first when `supersedes`
was added (Revision 5) and again when `emits` was (Revision 6), despite
`specodelic.md`'s own Revision 5 notes claiming this check needed **zero
code changes** for either addition. That claim was true of the
*implementation* (which already reads `allowed_targets(field)` from the
table) but not of this file's *wording*, which had drifted out of sync
with what the implementation actually does. Reworded above to describe
the table-driven behavior directly, so the next Reference Typing addition
requires no follow-up here either in code or in wording.
