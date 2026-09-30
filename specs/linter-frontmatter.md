---
id: linter.frontmatter
kind: intent
statement: "WHEN a spec file is parsed, THE linter SHALL reject it unless its frontmatter declares id, kind, and statement, with kind equal to intent."
---

# Linter: Frontmatter Check

The first gate every spec file passes through. Every other linter check
assumes this one already succeeded — a file with no valid `id` has nothing
for later checks to key off of.

## Constraints

| id              | kind      | expr                                                              | traces_to | satisfies |
|------------------|-----------|----------------------------------------------------------------------|------------------------------------|
| has_id           | invariant | `frontmatter.id != null and matches(id, /^[a-z][a-z0-9_.]*$/)`        | [[linter.frontmatter]] |          |
| has_kind         | invariant | `frontmatter.kind == "intent"`                                       | [[linter.frontmatter]] |          |
| has_statement    | invariant | `frontmatter.statement != null and len(statement) > 0`               | [[linter.frontmatter]] |          |
| id_matches_file  | invariant | `frontmatter.id == expected_id_from_filename(path)`                  | [[linter.frontmatter]] |          |
| check_failure | effect | `linter.frontmatter.check_failure(detail)` | [[linter.frontmatter]] | [[errors.envelope_error_kind]] ∧ [[errors.exit_code_mapping]] ∧ [[errors.remediation_hint_present]] |

## Model

### States
- `unchecked`
- `checking`
- `passed`
- `failed` (emits: `[[linter.frontmatter.check_failure]]`)

### Transitions

| id            | from      | to        | guard                                                               |
|---------------|-----------|-----------|--------------------------------------------------------------------|
| begin         | unchecked | checking  | `file exists and is UTF-8 markdown`                                 |
| accept        | checking  | passed    | [[linter.frontmatter.has_id]] ∧ [[linter.frontmatter.has_kind]] ∧ [[linter.frontmatter.has_statement]] ∧ [[linter.frontmatter.id_matches_file]] |
| reject | checking | failed | `¬([[linter.frontmatter.has_id]] ∧ [[linter.frontmatter.has_kind]] ∧ [[linter.frontmatter.has_statement]] ∧ [[linter.frontmatter.id_matches_file]])` |


## Properties

| id                  | kind | derives_from                          | generator                              | predicate                                                                 |
|----------------------|------|------------------------------------------|-------------------------------------------|-----------------------------------------------------------------------------|
| missing_id_rejected  | unit | [[linter.frontmatter.has_id]]           | `frontmatter_without("id")`               | `check(frontmatter) == failed`                                              |
| wrong_kind_rejected  | unit | [[linter.frontmatter.has_kind]]         | `frontmatter_with_kind(≠ "intent")`       | `check(frontmatter) == failed`                                              |
| valid_passes         | unit | [[linter.frontmatter.has_id]]           | `arbitrary_valid_frontmatter()`           | `check(frontmatter) == passed`                                              |
| filename_mismatch    | unit | [[linter.frontmatter.id_matches_file]]  | `(id, filename)` pairs where `id ≠ stem(filename)` | `check(id, filename) == failed`                                    |
| missing_statement_rejected | unit | [[linter.frontmatter.has_statement]] | `frontmatter_without("statement")` and `frontmatter_with(statement: "")` | `check(frontmatter) == failed` — absent and empty are both rejections |
| check_failure_label_asserted | unit | [[linter.frontmatter.check_failure]] | `check_failure_raised()` | `error_label == "linter.frontmatter.check_failure"` — renaming the label touches the error Constraint, this property, and its note together (EDGE-002) |
## Notes

**Reference-typing reconciliation (2026-09-30, `specodelic-cxq`):** the
Constraints table's `traces_to` cells previously pointed at the
`specodelic.md` rows these checks re-own — a Constraint→Constraint target,
which the Reference Typing table forbids (`traces_to` resolves to Intent
only). Each now traces to this file's own intent: `specodelic.md` keeps
the corpus-wide statement of record, this file owns the checkable one
(the "same claim, two altitudes" pattern the Notes below already use).

`id_matches_file` is not in the meta-spec's `frontmatter_valid` list yet —
it's a new invariant this decomposition surfaced: nothing in `specodelic.md`
currently requires a file's declared `id` to match its own filename, which
means two files could silently claim the same logical feature under
different names. Flagging as a gap to fold back into `specodelic.md`
(`Needs Human Review`: does this apply to non-root spec files too, or only
top-level feature files?) rather than resolving it here.
