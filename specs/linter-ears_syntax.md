---
id: linter.ears_syntax
kind: intent
statement: "WHEN a spec file's frontmatter has passed, THE linter SHALL reject it if the intent statement does not match one of the five EARS patterns or if any row id joins two capabilities."
---

# Linter: EARS Syntax Check

Runs after `linter.frontmatter` only — unlike the other checks, it doesn't
need the reference graph or model, since it operates purely on the
`statement` string and the shape of `id` cells. Corresponds to
`testability-implementability-evaluator`'s EARS-compliance and
NLP-anti-pattern steps, narrowed to what can be checked structurally rather
than by judging prose quality.

## Constraints

| id                  | kind      | expr                                                                                                    | traces_to | satisfies |
|----------------------|-----------|--------------------------------------------------------------------------------------------------------------|----------------------------------------------------|
| ears_pattern_match    | invariant | `statement matches one of: Ubiquitous / Event-Driven / State-Driven / Unwanted-Behavior / Optional-Feature`   | [[specodelic.ears_statement]]        |          |
| has_shall             | invariant | `statement contains an imperative "SHALL" (or "SHALL NOT")`                                                   | [[specodelic.ears_statement]]        |          |
| no_conjoined_id       | invariant | `∀ row.id: id does not encode two capabilities joined by "and"/"or" (checked on the id token, not prose)`      | [[specodelic.one_capability_per_row]]|          |
| no_universal_in_id    | invariant | `∀ row.id: id does not contain "all"/"every"/"any"/"always"/"never" as a token`                                | [[specodelic.one_capability_per_row]]|          |
| pattern_failure | effect | `linter.ears_syntax.pattern_failure(detail)` | [[linter.ears_syntax]] | [[errors.envelope_error_kind]] ∧ [[errors.exit_code_mapping]] ∧ [[errors.remediation_hint_present]] |
| id_check_failure | effect | `linter.ears_syntax.id_check_failure(detail)` | [[linter.ears_syntax]] | [[errors.envelope_error_kind]] ∧ [[errors.exit_code_mapping]] ∧ [[errors.remediation_hint_present]] |

## Model

### States
- `unchecked`
- `pattern_matching`
- `id_checking`
- `passed`
- `pattern_failed` (emits: `[[linter.ears_syntax.pattern_failure]]`)
- `id_check_failed` (emits: `[[linter.ears_syntax.id_check_failure]]`)

### Transitions

| id             | from              | to                | guard                                                                                    |
|----------------|-------------------|-------------------|----------------------------------------------------------------------------------------------|
| begin          | unchecked         | pattern_matching  | `file passed linter.frontmatter`                                                              |
| pattern_ok     | pattern_matching  | id_checking       | [[linter.ears_syntax.ears_pattern_match]] ∧ [[linter.ears_syntax.has_shall]]                    |
| pattern_fail | pattern_matching | pattern_failed | `¬([[linter.ears_syntax.ears_pattern_match]] ∧ [[linter.ears_syntax.has_shall]])` |
| accept         | id_checking       | passed            | [[linter.ears_syntax.no_conjoined_id]] ∧ [[linter.ears_syntax.no_universal_in_id]]              |
| reject | id_checking | id_check_failed | `¬([[linter.ears_syntax.no_conjoined_id]] ∧ [[linter.ears_syntax.no_universal_in_id]])` |


## Properties

| id                      | kind | derives_from                                  | generator                                                | predicate                                                                 |
|--------------------------|------|---------------------------------------------------|---------------------------------------------------------------|-----------------------------------------------------------------------------|
| non_ears_rejected          | unit | [[linter.ears_syntax.ears_pattern_match]]        | `statement_not_matching_any_ears_pattern()`                     | `check(file) == failed`                                                    |
| missing_shall_rejected     | unit | [[linter.ears_syntax.has_shall]]                 | `statement_matching_pattern_but_missing("SHALL")`               | `check(file) == failed`                                                    |
| conjoined_id_rejected      | unit | [[linter.ears_syntax.no_conjoined_id]]           | `row_with(id: "order.cancel_and_refund")`                       | `check(file) == failed`                                                    |
| universal_token_rejected   | unit | [[linter.ears_syntax.no_universal_in_id]]        | `row_with(id: "order.always_validate")`                         | `check(file) == failed`                                                    |
| valid_ears_passes          | unit | [[linter.ears_syntax.ears_pattern_match]]        | `arbitrary_ears_compliant_statement()`                          | `check(file) == passed`                                                    |
| pattern_failure_label_asserted | unit | [[linter.ears_syntax.pattern_failure]] | `pattern_failure_raised()` | `error_label == "linter.ears_syntax.pattern_failure"` — renaming the label touches the error Constraint, this property, and its note together (EDGE-002) |
| id_check_failure_label_asserted | unit | [[linter.ears_syntax.id_check_failure]] | `id_check_failure_raised()` | `error_label == "linter.ears_syntax.id_check_failure"` — renaming the label touches the error Constraint, this property, and its note together (EDGE-002) |
## Notes

This check deliberately does **not** attempt weak-phrase detection
("adequate," "as appropriate"), passive-voice detection, or non-specific
temporals ("immediately," "quickly") — those are exactly the prose-quality
judgments `specodelic.md`'s `prose_untouched` invariant puts out of the
linter's scope, and are correctly `testability-implementability-evaluator`'s
job to run separately over the compiled `statement` field, not this
schema's job to structurally forbid. The line drawn here: EARS *pattern*
conformance is a grammar fact (does the sentence have the right shape),
weak-phrase/temporal/passive-voice detection is a judgment about word
choice within a grammatically valid sentence — the former belongs in the
linter, the latter doesn't.

`no_conjoined_id`/`no_universal_in_id` check the **id token**, not the
prose statement — e.g. rejecting `id: "cancel_and_refund"` structurally,
while a compound *statement* in prose still isn't caught (correctly, since
that would require judging prose). This is narrower than
`specification-evaluation-diagnostician`'s "Compound Requirements" finding,
which also reads sentence structure — worth flagging as a known gap this
checker doesn't close, rather than one it silently claims to.

No new `specodelic.md` gaps surfaced this time — this is the first check
whose constraints all trace cleanly to invariants already on the list.
