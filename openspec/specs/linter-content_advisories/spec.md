---
id: linter.content_advisories
kind: intent
statement: "WHEN a spec corpus is linted, THE linter SHALL warn — advisory, exit 0 — on structured content cells that are known placeholders or trivially-true predicates, and SHALL never turn those warnings into gating findings."
---

# Linter: Content Advisories

The format's structured cells are data, not prose: an Intent has a
`statement`, a Constraint has an `expr`, a Property has a `predicate`.
Lint never judges their meaning, but it can detect two shapes that carry
no meaning at all — a cell that was never filled in, and a predicate that
cannot fail. Both are emitted on the success envelope's warnings channel
(exit 0); neither ever joins the issues channel. This keeps the
fail-closed gating contract intact while making the two most common
AI-author filler shapes visible instead of silent.

## Purpose

Close the "plausible filler" gap recorded in specodelic-xci: an untouched
scaffold and a tautological predicate (`true`, `x == x`) currently lint with
zero findings. Two generators produce that filler, and the signature must
recognise both:

- `spk new` — the stub statement `THE system SHALL …` and `TODO — …` cells;
- `spk migrate` — a sidecar whose ids are `scaffold_constraint`,
  `scaffold_transition`, and `scaffold_property` with bare `true` cells,
  beside a prose statement that is not the stub. That file's only scaffold
  evidence is a tautology, which is why a tautological content cell counts
  as the signature.

The advisory is deliberately non-gating: whether these shapes should ever
block a lifecycle transition is a separate decision that needs
dogfood-friction evidence, exactly as `linter-observability.md`'s advisory
was kept out of the `linted` gate.

## Constraints

| id                          | kind      | expr                                                                                                                                                                                                                                                                                                                                                 | traces_to                    |
|-----------------------------|-----------|------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|------------------------------|
| placeholder_classes         | invariant | `a structured content cell is a placeholder when its trimmed text is TODO, TBD, or FIXME (optionally followed by a separator and a note), a row id begins with scaffold_ in a file that still carries the scaffold signature (its intent statement is a placeholder, or a constraint expr or property predicate is itself a placeholder or a tautology), or the intent statement is a placeholder (the spk new stub "THE system SHALL …" or a whole-cell placeholder word) — the real producers are the spk new stub/TODO cells and spk migrate's scaffold_constraint/scaffold_transition/scaffold_property sidecar rows whose cells are bare true; detection is whole-cell or scaffold-signature-gated row-id-prefix, never a substring search inside an executable fragment`                                                                              | [[linter.content_advisories]] |
| tautology_class             | invariant | `a property predicate or invariant constraint expr is tautological when, after stripping its surrounding code span and any **rust:** fragment marker, its trimmed text is exactly true or ⊤, or it is one identifier compared equal to itself (`x == x`) — an effect or advisory constraint expr is never judged (it is a label, not a claim that must be able to fail); no arithmetic, calls, or compound expressions are judged`                                                               | [[linter.content_advisories]] |
| advisory_severity           | invariant | `every content advisory is emitted on the success envelope's warnings channel with exit 0 — never an Issue, never a nonzero exit, and never a change to the hard-findings count of an otherwise clean corpus`                                                                                                                                            | [[linter.content_advisories]] |
| fragment_interior_not_scanned | invariant | `the checker never inspects the interior of a **rust:** fragment for placeholder words — a legitimate fragment such as todo_predicate! or a string containing TODO stays silent unless the whole cell is one of the placeholder forms above`                                                                                                            | [[linter.content_advisories]] |

## Model

### States

- `scanning`
- `clean`
- `warned`

### Transitions

| id          | from     | to       | guard                                                                     |
|-------------|----------|----------|---------------------------------------------------------------------------|
| scan_clean  | scanning | clean    | `no structured cell is a whole-cell placeholder and no predicate or invariant expr is a tautology` |
| scan_warned | scanning | warned   | [[linter.content_advisories.placeholder_classes]] ∨ [[linter.content_advisories.tautology_class]] |

## Properties

| id                        | kind | derives_from                                          | generator                            | predicate                                                                                              |
|---------------------------|------|-------------------------------------------------------|--------------------------------------|--------------------------------------------------------------------------------------------------------|
| placeholder_warned        | unit | [[linter.content_advisories.placeholder_classes]]     | `spec_with_todo_expr()`              | `exit 0 ∧ a warnings-channel entry names the row and rule linter.content_advisories.placeholder_content` |
| tautology_warned          | unit | [[linter.content_advisories.tautology_class]]         | `spec_with_true_predicate()`         | `exit 0 ∧ a warnings-channel entry names the row and rule linter.content_advisories.tautological_predicate` |
| warning_never_gates       | unit | [[linter.content_advisories.advisory_severity]]       | `corpus_with_placeholder_cells()`    | `exit code 0 ∧ zero Issue findings attributable to this checker`                                        |
| fragment_interior_silent  | unit | [[linter.content_advisories.fragment_interior_not_scanned]] | `predicate_with_todo_predicate_macro()` | `no warning — a legitimate executable fragment is never treated as a placeholder`                      |

## Requirements

### Requirement: Placeholder and tautology cells are warned, never gated

Content advisories SHALL be emitted on the warnings channel with exit 0,
covering whole-cell placeholders, scaffold-signature row ids, stub
statements, and tautological property predicates or invariant constraint
exprs — never effect/advisory exprs, statements, or fragment interiors.
The scaffold signature SHALL recognise both real producers: `spk new`'s
stub/TODO cells and `spk migrate`'s sidecar (`scaffold_*` ids with `true`
cells).

#### Scenario: placeholder_warned

- **WHEN** `spec_with_todo_expr()`
- **THEN** `exit 0 ∧ a warnings-channel entry names the row and rule linter.content_advisories.placeholder_content`
- **VERIFIES** [[linter.content_advisories.placeholder_warned]]

#### Scenario: tautology_warned

- **WHEN** `spec_with_true_predicate()`
- **THEN** `exit 0 ∧ a warnings-channel entry names the row and rule linter.content_advisories.tautological_predicate`
- **VERIFIES** [[linter.content_advisories.tautology_warned]]

#### Scenario: warning_never_gates

- **WHEN** `corpus_with_placeholder_cells()`
- **THEN** `exit code 0 ∧ zero Issue findings attributable to this checker`
- **VERIFIES** [[linter.content_advisories.warning_never_gates]]

#### Scenario: fragment_interior_silent

- **WHEN** `predicate_with_todo_predicate_macro()`
- **THEN** `no warning — a legitimate executable fragment is never treated as a placeholder`
- **VERIFIES** [[linter.content_advisories.fragment_interior_silent]]