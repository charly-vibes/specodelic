---
id: linter.referential_integrity
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

## Resolution algorithm

`qualify(ref)` (the `ref_resolves` expr) and `spk graph` share one
algorithm, pinned by unit tests and documented here because it lives
nowhere user-facing otherwise (gh#1: "the resolution algorithm is
undocumented"):

1. **Exact file id** — the whole target names a file; file id wins over
   any split interpretation.
2. **Bare-local row** — a dotless target naming one of the SOURCE
   file's own rows resolves to that row. In practice this arm is masked
   by the metasyntactic skip (a dotless target naming no file id is
   format prose, not a reference) — it is the safety net that keeps a
   dotless spelling meaningful when a file id shadows it. Revision 18
   note: the old "only self-contained `id: spec` deltas reach this
   arm" reading retired with `id: spec` itself; every file resolves
   corpus-wide and authors spell rows file-qualified.
3. **Every dot split, last to first** — `prefix` must be a known file
   id and the remainder must be one of: a row of that file, the
   `model.state`/`model.transition` section anchor, or `row.member`
   where the row is the single segment right after the split and the
   member path may itself be dotted. Last-dot wins because dotted file
   ids (`linter.frontmatter`) are the common case; the first-dot
   fallback covers `[[specodelic.model.state]]` (file `specodelic`,
   anchor `model.state`) and multi-segment member paths
   (`[[a.b.c.d]]` = file `a`, row `b`, member `c.d`).

Row ids are single-segment: a dotted tail is always read as
`row.member`, never as a dotted row id, so `[[a.b.c.d]]` does not
resolve when file `a` declares only a row literally named `b.c`.
## Notes

**Reference-typing reconciliation (2026-09-30, `specodelic-cxq`):** the
Constraints table's `traces_to` cells previously pointed at the
`specodelic.md` rows these checks re-own — a Constraint→Constraint target,
which the Reference Typing table forbids (`traces_to` resolves to Intent
only). Each now traces to this file's own intent: `specodelic.md` keeps
the corpus-wide statement of record, this file owns the checkable one
(the "same claim, two altitudes" pattern the Notes below already use).

**Decision of record (2026-09-30, `specodelic-cxq`):** this file's
`rename_naturality` law previously carried only identity + associativity;
its naturality case (the cross-artifact commutation that gives the law its
name, `lookup_all(rename(I)) == rename(lookup_all(I))`) was implicit in the
Notes prose. It is now an explicit case column entry — the law the Notes
describe as "the layer that enforces it" states all three cases
`[[specodelic.law_requires_cases]]` requires.

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

## Constraints

| id                  | kind      | expr                                                                                   | traces_to | satisfies |
|----------------------|-----------|-------------------------------------------------------------------------------------------|--------------------------------------------------|-----------|
| unique_within_file   | invariant | `∀ file: no two rows in file share the same local id`                                     | [[linter.referential_integrity]]           |          |
| unique_across_repo   | invariant | `∀ repo: no two files declare the same fully-qualified id`                                | [[linter.referential_integrity]]           |          |
| ref_resolves         | invariant | `∀ [[ref]] in file: qualify(ref) ∈ index(repo)`                                            | [[linter.referential_integrity]]          |          |
| ref_kind_compatible  | invariant | `∀ [[ref]] in any reference field defined by specodelic.md's Reference Typing table: target row's kind ∈ allowed_targets(field)` — read from that table, not hardcoded per field name, so a Revision adding a field (`supersedes`, `emits`, ...) needs no change here | [[linter.referential_integrity]]          |          |
| index_failure | effect | `linter.referential_integrity.index_failure(detail)` | [[linter.referential_integrity]] | [[errors.envelope_error_kind]] ∧ [[errors.exit_code_mapping]] ∧ [[errors.remediation_hint_present]] |
| resolution_failure | effect | `linter.referential_integrity.resolution_failure(detail)` | [[linter.referential_integrity]] | [[errors.envelope_error_kind]] ∧ [[errors.exit_code_mapping]] ∧ [[errors.remediation_hint_present]] |
| schema_is_decision | invariant | `the allowed targets for a typed field are read from the canonical Schema via its typing check — no per-field match arm exists in the lint rule` | [[linter.referential_integrity]] |
| typed_columns_only | invariant | `the rule walks only links whose column carries schema morphism rows — frontmatter, expr, predicate, and generator cells are format prose, never typed references` | [[linter.referential_integrity]] |
| dangling_is_total_refs | invariant | `a typed link that resolves to no row or file produces exactly the dangling finding — typing never double-reports an unresolved target` | [[linter.referential_integrity]] |
| severity_error | invariant | `a kind-incompatible resolved target is an issues-channel finding (exit 1) — the invariant gates the checker's accept transition, never the warnings channel` | [[linter.referential_integrity]] |
| member_arm_types_row | invariant | `a member-arm target (file.row.member) canonicalizes to file.row and is typed as the row's kind — the stricter of the graph and oracle derivations` | [[linter.referential_integrity]] |
| uses_types_frontmatter | invariant | `a file-id target under uses is typed as Intent with the file's frontmatter kind — a non-profile target fires, a profile pack id stays clean` | [[linter.referential_integrity]] |
| from_to_not_duplicated | invariant | `from/to State typing stays the model-shape checker's beat — this rule adds no second finding for the same cell` | [[linter.referential_integrity]] |
| corpus_stays_green | invariant | `the rule firing over the repo's own corpus yields zero findings — dogfooding is the gate` | [[linter.referential_integrity]] |

## Model

### States
- `unindexed`
- `indexing`
- `indexed`
- `resolving`
- `passed`
- `index_failed` (emits: `[[linter.referential_integrity.index_failure]]`)
- `resolution_failed` (emits: `[[linter.referential_integrity.resolution_failure]]`)

### Transitions

| id             | from       | to         | guard                                                                                       |
|----------------|------------|------------|----------------------------------------------------------------------------------------------|
| build_index    | unindexed  | indexing   | `all files in repo passed linter.frontmatter`                                                |
| index_ok       | indexing   | indexed    | [[linter.referential_integrity.unique_within_file]] ∧ [[linter.referential_integrity.unique_across_repo]] |
| index_fail | indexing | index_failed | `¬([[linter.referential_integrity.unique_within_file]] ∧ [[linter.referential_integrity.unique_across_repo]])` |
| resolve_refs   | indexed    | resolving  | `index built successfully`                                                                    |
| accept         | resolving  | passed     | [[linter.referential_integrity.ref_resolves]] ∧ [[linter.referential_integrity.ref_kind_compatible]] |
| reject | resolving | resolution_failed | `¬([[linter.referential_integrity.ref_resolves]] ∧ [[linter.referential_integrity.ref_kind_compatible]])` |

## Properties

| id                     | kind | derives_from                                             | generator                                          | predicate                                                                 |
|-------------------------|------|-------------------------------------------------------------|--------------------------------------------------------|-----------------------------------------------------------------------------|
| duplicate_id_rejected    | unit | [[linter.referential_integrity.unique_within_file]]        | `spec_file_with(duplicate_row_id: true)`               | `check(file) == failed`                                                    |
| cross_file_collision     | unit | [[linter.referential_integrity.unique_across_repo]]        | `two_spec_files_sharing_id()`                          | `check(repo) == failed`                                                    |
| dangling_ref_rejected    | unit | [[linter.referential_integrity.ref_resolves]]              | `spec_file_with(ref_to_nonexistent_id: true)`           | `check(file) == failed`                                                    |
| wrong_target_kind        | unit | [[linter.referential_integrity.ref_kind_compatible]]       | `guard_field_pointing_at_a_property_row()`              | `check(file) == failed`                                                    |
| rename_naturality        | law  | [[specodelic.rename_naturality]]                          | `arbitrary_spec_repo(), arbitrary_id_rename()`          | **identity:** `resolve(rename(I,a,a)) == resolve(I)`  **associativity:** `resolve(rename(rename(I,a,b),b,c)) == resolve(rename(I,a,c))`  **naturality:** `lookup_all(rename(I)) == rename(lookup_all(I))` — every `[[ref]]` pointing at a renamed id is updated with it, the naturality case this checker is the enforcement layer for |
| clean_repo_passes        | unit | [[linter.referential_integrity.ref_resolves]]              | `arbitrary_well_formed_repo()`                          | `check(repo) == passed`                                                    |
| index_failure_label_asserted | unit | [[linter.referential_integrity.index_failure]] | `index_failure_raised()` | `error_label == "linter.referential_integrity.index_failure"` — renaming the label touches the error Constraint, this property, and its note together (EDGE-002) |
| resolution_failure_label_asserted | unit | [[linter.referential_integrity.resolution_failure]] | `resolution_failure_raised()` | `error_label == "linter.referential_integrity.resolution_failure"` — renaming the label touches the error Constraint, this property, and its note together (EDGE-002) |
| p_schema_decision | unit | [[linter.referential_integrity.schema_is_decision]] | `arbitrary_typed_link()` | `the lint verdict matches the graph report on the same link` |
| p_typed_only | unit | [[linter.referential_integrity.typed_columns_only]] | `links_in_untyped_cells()` | `no finding names an untyped column` |
| p_dangling_once | unit | [[linter.referential_integrity.dangling_is_total_refs]] | `typed_cell_with_dangling_target()` | `findings == [the dangling finding]` |
| p_severity | unit | [[linter.referential_integrity.severity_error]] | `violating_corpus()` | `exit code 1 ∧ finding on the issues channel` |
| p_member_arm | unit | [[linter.referential_integrity.member_arm_types_row]] | `member_arm_links()` | `typed as the canonical row's kind` |
| p_uses_profile | unit | [[linter.referential_integrity.uses_types_frontmatter]] | `uses_links()` | `profile target passes ∧ intent target fires` |
| p_from_to_once | unit | [[linter.referential_integrity.from_to_not_duplicated]] | `transition_with_ghost_endpoint()` | `exactly the model-shape finding fires` |
| p_enforcement_clean | unit | [[linter.referential_integrity.corpus_stays_green]] | `arbitrary_well_formed_corpus()` | `exit 0 ∧ zero typing findings` |

## Requirements

### Requirement: Property coverage mirror

Every property row SHALL be verified by exactly one dedicated scenario;
`ah sync` SHALL derive one contract per VERIFIES link.

#### Scenario: duplicate_id_rejected

- **WHEN** `spec_file_with(duplicate_row_id: true)`
- **THEN** `check(file) == failed`
- **VERIFIES** [[linter.referential_integrity.duplicate_id_rejected]]

#### Scenario: cross_file_collision

- **WHEN** `two_spec_files_sharing_id()`
- **THEN** `check(repo) == failed`
- **VERIFIES** [[linter.referential_integrity.cross_file_collision]]

#### Scenario: dangling_ref_rejected

- **WHEN** `spec_file_with(ref_to_nonexistent_id: true)`
- **THEN** `check(file) == failed`
- **VERIFIES** [[linter.referential_integrity.dangling_ref_rejected]]

#### Scenario: wrong_target_kind

- **WHEN** `guard_field_pointing_at_a_property_row()`
- **THEN** `check(file) == failed`
- **VERIFIES** [[linter.referential_integrity.wrong_target_kind]]

#### Scenario: rename_naturality

- **WHEN** `arbitrary_spec_repo(), arbitrary_id_rename()`
- **THEN** **identity:** `resolve(rename(I,a,a)) == resolve(I)`  **associativity:** `resolve(rename(rename(I,a,b),b,c)) == resolve(rename(I,a,c))`  **naturality:** `lookup_all(rename(I)) == rename(lookup_all(I))` — every `[[ref]]` pointing at a renamed id is updated with it, the naturality case this checker is the enforcement layer for
- **VERIFIES** [[linter.referential_integrity.rename_naturality]]

#### Scenario: clean_repo_passes

- **WHEN** `arbitrary_well_formed_repo()`
- **THEN** `check(repo) == passed`
- **VERIFIES** [[linter.referential_integrity.clean_repo_passes]]

#### Scenario: index_failure_label_asserted

- **WHEN** `index_failure_raised()`
- **THEN** `error_label == "linter.referential_integrity.index_failure"` — renaming the label touches the error Constraint, this property, and its note together (EDGE-002)
- **VERIFIES** [[linter.referential_integrity.index_failure_label_asserted]]

#### Scenario: resolution_failure_label_asserted

- **WHEN** `resolution_failure_raised()`
- **THEN** `error_label == "linter.referential_integrity.resolution_failure"` — renaming the label touches the error Constraint, this property, and its note together (EDGE-002)
- **VERIFIES** [[linter.referential_integrity.resolution_failure_label_asserted]]

### Requirement: Reference Typing enforcement

`spk lint` SHALL reject a corpus when any resolved `[[link]]` in a
reference field defined by the format's Reference Typing table targets a
row whose kind the table forbids, reading the allowed targets from the
Schema-as-data table (never hardcoded per field name), and each finding
SHALL carry the `linter.ref_kind_compatible` rule id, the rule's
one-line semantics, and a remediation hint.

#### Scenario: Guard citing a non-invariant constraint fires

- **WHEN** a transition's guard cites an `advisory` or `effect` constraint
- **THEN** lint exits 1 with a `linter.ref_kind_compatible` finding naming
  the guard target and the invariant-or-State requirement

#### Scenario: Emits targeting a non-effect constraint fires

- **WHEN** a state's `emits` cell cites a constraint whose kind is not
  `effect`
- **THEN** lint exits 1 with a `linter.ref_kind_compatible` finding

#### Scenario: Constraint traces rejected

- **WHEN** a constraint's `traces_to` cell cites a row that is not an Intent
- **THEN** lint exits 1 with a `linter.ref_kind_compatible` finding

#### Scenario: Derives-from at a non-constraint target fires

- **WHEN** a `unit` property's `derives_from` cell cites a row that is not
  a constraint (a law may derive from the law-Property it restates)
- **THEN** lint exits 1 with a `linter.ref_kind_compatible` finding

#### Scenario: Supersedes across kinds rejected

- **WHEN** a `supersedes` cell cites a row of a different kind than the row
  carrying it
- **THEN** lint exits 1 with a `linter.ref_kind_compatible` finding

#### Scenario: Satisfies and observes at wrong-kind contracts fire

- **WHEN** a `satisfies` cell cites a non-`extension_point` constraint, or
  an `observes` cell cites a non-`effect` constraint
- **THEN** lint exits 1 with a `linter.ref_kind_compatible` finding naming
  the requirement

#### Scenario: Uses targeting a non-profile file fires

- **WHEN** a `uses` cell cites a file whose frontmatter kind is not
  `profile`
- **THEN** lint exits 1 with a `linter.ref_kind_compatible` finding

#### Scenario: Kind-compatible references lint clean

- **WHEN** every typed cell cites a kind-compatible target (guard →
  invariant constraint or State, emits → effect constraint, derives_from →
  constraint or own-law restatement, satisfies → extension_point,
  observes → effect, uses → profile file, member-arm targets typed as
  their row)
- **THEN** lint reports no `ref_kind_compatible` findings

#### Scenario: Dangling target is not double-reported

- **WHEN** a typed cell cites an id that resolves to no row or file
- **THEN** exactly the `linter.total_refs` finding fires — no duplicate
  typing finding for the same dangling target

#### Scenario: Corpus stays green under enforcement

- **WHEN** the linter runs over its own corpus
- **THEN** zero `ref_kind_compatible` findings fire (dogfooding gate)
