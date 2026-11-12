---
id: linter.referential_integrity.layer
kind: intent
statement: "WHEN a spec repo is linted, THE linter SHALL reject it if any reference field defined by the Reference Typing table resolves to a row whose kind the table forbids, reading allowed targets from the Schema-as-data table."
---

# linter.referential_integrity.layer — dual-format layer (sidecar)

## Purpose

Give the Reference Typing table its enforcement layer: the declared
`ref_kind_compatible` invariant stops being catalog prose and becomes a
labeled lint finding, decided from the same Schema-as-data table the
graph command reads — the gate and the report cannot disagree about what
the table means, and the next typed-field addition needs no checker code.

## Constraints

| id | kind | expr | traces_to |
|----|------|------|-----------|
| schema_is_decision | invariant | `the allowed targets for a typed field are read from the canonical Schema via its typing check — no per-field match arm exists in the lint rule` | [[linter.referential_integrity.layer]] |
| typed_columns_only | invariant | `the rule walks only links whose column carries schema morphism rows — frontmatter, expr, predicate, and generator cells are format prose, never typed references` | [[linter.referential_integrity.layer]] |
| dangling_is_total_refs | invariant | `a typed link that resolves to no row or file produces exactly the resolving checker's dangling finding — typing never double-reports a dangling target` | [[linter.referential_integrity.layer]] |
| severity_error | invariant | `a kind-incompatible resolved target is an issues-channel finding (exit 1) — the invariant gates the checker's accept transition, never the warnings channel` | [[linter.referential_integrity.layer]] |
| member_arm_types_row | invariant | `a member-arm target (file.row.member) canonicalizes to file.row and is typed as the row's kind — the stricter of the graph and oracle derivations` | [[linter.referential_integrity.layer]] |
| uses_types_frontmatter | invariant | `a file-id target under uses is typed as Intent with the file's frontmatter kind — a non-profile target fires, a profile pack id stays clean` | [[linter.referential_integrity.layer]] |
| from_to_not_duplicated | invariant | `from/to State typing stays the model-shape checker's beat — the rule adds no second finding for the same cell` | [[linter.referential_integrity.layer]] |
| corpus_stays_green | invariant | `the rule firing over the repo's own corpus yields zero findings — dogfooding is the gate` | [[linter.referential_integrity.layer]] |
| check_failure | effect | `linter.referential_integrity.layer.check_failure(detail)` | [[linter.referential_integrity.layer]] |

## Model

### States
- `indexed`
- `typing`
- `passed`
- `typing_failed` (emits: `[[linter.referential_integrity.layer.check_failure]]`)

### Transitions

| id | from | to | guard |
|----|------|----|-------|
| type_check | indexed | typing | [[linter.referential_integrity.layer.typed_columns_only]] |
| accept | typing | passed | [[linter.referential_integrity.layer.schema_is_decision]] ∧ [[linter.referential_integrity.layer.severity_error]] |
| reject | typing | typing_failed | `¬([[linter.referential_integrity.layer.schema_is_decision]] ∧ [[linter.referential_integrity.layer.severity_error]])` |

## Properties

| id | kind | derives_from | generator | predicate |
|----|------|--------------|-----------|-----------|
| p_schema_decision | unit | [[linter.referential_integrity.layer.schema_is_decision]] | `arbitrary_typed_link()` | `verdict matches the graph report on the same link` |
| p_typed_only | unit | [[linter.referential_integrity.layer.typed_columns_only]] | `links_in_untyped_cells()` | `no finding names an untyped column` |
| p_dangling_once | unit | [[linter.referential_integrity.layer.dangling_is_total_refs]] | `typed_cell_with_dangling_target()` | `findings == [total_refs]` |
| p_severity | unit | [[linter.referential_integrity.layer.severity_error]] | `violating_corpus()` | `exit code 1 ∧ finding on the issues channel` |
| p_member_arm | unit | [[linter.referential_integrity.layer.member_arm_types_row]] | `member_arm_links()` | `typed as the canonical row's kind` |
| p_uses_profile | unit | [[linter.referential_integrity.layer.uses_types_frontmatter]] | `uses_links()` | `profile target passes ∧ intent target fires` |
| p_from_to_once | unit | [[linter.referential_integrity.layer.from_to_not_duplicated]] | `transition_with_ghost_endpoint()` | `exactly the model-shape finding fires` |
| p_clean_passes | unit | [[linter.referential_integrity.layer.corpus_stays_green]] | `arbitrary_well_formed_corpus()` | `exit 0 ∧ zero ref_kind_compatible findings` |
| p_check_failure | unit | [[linter.referential_integrity.layer.check_failure]] | `typing_failure_raised()` | `error_label == "linter.referential_integrity.layer.check_failure"` |
