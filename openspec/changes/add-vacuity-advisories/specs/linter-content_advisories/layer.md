---
id: linter.content_advisories.layer
kind: intent
statement: "WHEN a spec corpus is linted, THE linter SHALL warn — advisory, exit 0 — on whole-cell placeholder or tautological content and on invariant constraint exprs or non-law Property predicates whose executable fragment cites no declared model element, and on invariants that forbid a state the Model never declares, and SHALL never gate on any of these findings or judge reachability."
---

# linter-content_advisories — dual-format layer (sidecar)

## Purpose

The capability's full layer, re-attached fail-closed by the archive
companion. Two advisory classes share this file because they are one
family — findings a green corpus should still be told about, on the
warnings channel, exit 0:

- **No content.** specodelic-xci's gap: an untouched `spk new` scaffold, a
  `TODO` cell, or a predicate that cannot fail (`true`, `x == x`) lints with
  zero findings. Both real producers must be recognised — `spk new`'s
  stub/TODO cells, and `spk migrate`'s sidecar (`scaffold_constraint`,
  `scaffold_transition`, `scaffold_property` ids with bare `true` cells,
  beside a prose statement), which is why a tautological content cell counts
  as the scaffold signature.
- **No binding.** specodelic-h5ks's gap: content that exists but references
  nothing. An executable fragment that cites no declared model element is
  green yet vacuous (the bowling-rev1 shape — fragments that never called
  the scorer), and an invariant that forbids a state `Model/States` never
  declares cannot fail, because the bad state cannot exist.

Neither class may gate, and neither judges reachability: a declared element
that is simply never exercised — a phase state its correct runs never enter,
in particular — is legitimate design pressure and stays silent.

## Constraints

| id                            | kind      | expr                                                                                                                                                                                                                                                                                                                                                 | traces_to                            |
|-------------------------------|-----------|------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|--------------------------------------|
| placeholder_classes           | invariant | `a structured content cell is a placeholder when its trimmed text is TODO, TBD, or FIXME (optionally followed by a separator and a note), a row id begins with scaffold_ in a file that still carries the scaffold signature (its intent statement is a placeholder, or a constraint expr or property predicate is itself a placeholder or a tautology), or the intent statement is a placeholder (the spk new stub "THE system SHALL …" or a whole-cell placeholder word) — the real producers are the spk new stub/TODO cells and spk migrate's scaffold_constraint/scaffold_transition/scaffold_property sidecar rows whose cells are bare true; detection is whole-cell or scaffold-signature-gated row-id-prefix, never a substring search inside an executable fragment` | [[linter.content_advisories.layer]] |
| tautology_class               | invariant | `a property predicate or invariant constraint expr is tautological when, after stripping its surrounding code span and any **rust:** fragment marker, its trimmed text is exactly true or ⊤, or it is one identifier compared equal to itself (`x == x`) — an effect or advisory constraint expr is never judged (it is a label, not a claim that must be able to fail); no arithmetic, calls, or compound expressions are judged`                                                               | [[linter.content_advisories.layer]] |
| advisory_severity             | invariant | `every content advisory is emitted on the success envelope's warnings channel with exit 0 — never an Issue, never a nonzero exit, and never a change to the hard-findings count of an otherwise clean corpus`                                                                                                                                            | [[linter.content_advisories.layer]] |
| fragment_interior_not_scanned | invariant | `the checker never inspects the interior of a **rust:** fragment for placeholder words — a legitimate fragment such as todo_predicate! or a string containing TODO stays silent unless the whole cell is one of the placeholder forms above`                                                                                                            | [[linter.content_advisories.layer]] |
| fragment_citation_required    | invariant | `an invariant Constraint expr or a non-law Property predicate that carries an executable **rust:** fragment must cite at least one declared model element — a Model state id, a transition id, or a Constraint/Property row id; the intent id is the file's own identity and is never a model element`                                                     | [[linter.content_advisories.layer]] |
| citation_whole_token          | invariant | `citation is a whole-token match over [A-Za-z0-9_]+ runs, never substring — unlocked does not cite lock and beheld does not cite held — and a dot is a token boundary, so a dotted demo.spec.held reference cites held through its segments`                                                                                                             | [[linter.content_advisories.layer]] |
| declared_citation_sufficient  | invariant | `one declared citation is sufficient: a declared state, transition, constraint, or property that is never exercised is never flagged, because the check compares the fragment's citation text against the declared id set and never judges reachability or coverage of declared ids`                                                                       | [[linter.content_advisories.layer]] |
| law_predicate_exempt          | invariant | `a law-kind Property predicate is outside the fragment surface — its law-case labels are the law-case grammar, never fragment markers, so the vacuity check skips law rows entirely`                                                                                                                                                                    | [[linter.content_advisories.layer]] |
| forbidden_state_declared      | invariant | `an invariant fragment's state != "literal" comparison, or its mirror literal != state, must name a state declared under Model/States — an undeclared forbidden state cannot exist, so the invariant is structurally vacuous and is reported with the literal it forbids`                                                                                 | [[linter.content_advisories.layer]] |
| declared_phase_state_silent   | invariant | `a state declared under Model/States is silent even when no transition enters it — a declared phase its correct runs never reach is legitimate defensive design pressure, and the rule never judges reachability`                                                                                                                                        | [[linter.content_advisories.layer]] |
| narrow_state_equality         | invariant | `only the bare state binder's string inequality is judged — other binders, ==, arithmetic, calls, and escaped or raw literals stay silent, and my_state, model.state, and state_extra are other names, never the binder`                                                                                                                                  | [[linter.content_advisories.layer]] |
| advisory_channel_only         | invariant | `both vacuity findings ride the success envelope's warnings channel with exit 0 — never an Issue, never a nonzero exit, and never a change to the failures count of an otherwise clean corpus`                                                                                                                                                             | [[linter.content_advisories.layer]] |
| advisory_class_partition      | invariant | `each vacuity rule carries its own id and remedy and never restates a placeholder or tautology finding: one finding per fragment for the citation rule, one per undeclared forbidden literal for the invariant rule, and the whole-cell rules never restate a vacuity finding`                                                                             | [[linter.content_advisories.layer]] |

## Model

### States

- `scanning`
- `clean`
- `warned`

### Transitions

| id          | from     | to     | guard                                                                                                                                                                                                                                                   |
|-------------|----------|--------|---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| scan_clean  | scanning | clean  | `no structured cell is a whole-cell placeholder, no predicate or invariant expr is a tautology, and no vacuity rule fires`                                                                                                                              |
| scan_warned | scanning | warned | [[linter.content_advisories.layer.placeholder_classes]] ∨ [[linter.content_advisories.layer.tautology_class]] ∨ [[linter.content_advisories.layer.fragment_citation_required]] ∨ [[linter.content_advisories.layer.forbidden_state_declared]]           |

## Properties

| id                            | kind | derives_from                                                        | generator                                        | predicate                                                                                                                                                       |
|-------------------------------|------|---------------------------------------------------------------------|--------------------------------------------------|-----------------------------------------------------------------------------------------------------------------------------------------------------------------|
| placeholder_warned            | unit | [[linter.content_advisories.layer.placeholder_classes]]             | `spec_with_todo_expr()`                          | `exit 0 ∧ a warnings-channel entry names the row and rule linter.content_advisories.placeholder_content`                                                          |
| tautology_warned              | unit | [[linter.content_advisories.layer.tautology_class]]                 | `spec_with_true_predicate()`                     | `exit 0 ∧ a warnings-channel entry names the row and rule linter.content_advisories.tautological_predicate`                                                       |
| warning_never_gates           | unit | [[linter.content_advisories.layer.advisory_severity]]               | `corpus_with_placeholder_cells()`                | `exit code 0 ∧ zero Issue findings attributable to this checker`                                                                                                  |
| fragment_interior_silent      | unit | [[linter.content_advisories.layer.fragment_interior_not_scanned]]   | `predicate_with_todo_predicate_macro()`          | `no warning — a legitimate executable fragment is never treated as a placeholder`                                                                                |
| vacuous_fragment_warns        | unit | [[linter.content_advisories.layer.fragment_citation_required]]      | `spec_with(invariant_expr: flag != 0)`           | `exit 0 ∧ a warnings entry names the row and rule linter.fragment_binds_nothing ∧ the issues channel carries no matching finding`                                 |
| whole_token_citation          | unit | [[linter.content_advisories.layer.citation_whole_token]]            | `fragment_naming(unlocked) over a declared lock` | `no citation and a warning, while a dotted demo.spec.held reference cites held and stays silent`                                                                   |
| declared_element_sufficiency  | unit | [[linter.content_advisories.layer.declared_citation_sufficient]]    | `fragment_citing(a_state_no_transition_enters)`  | `no fragment_binds_nothing warning`                                                                                                                               |
| law_predicate_not_judged      | unit | [[linter.content_advisories.layer.law_predicate_exempt]]            | `law_property(predicate_with_case_labels_only)`  | `no fragment_binds_nothing warning for the law row`                                                                                                               |
| undeclared_forbidden_state_warns | unit | [[linter.content_advisories.layer.forbidden_state_declared]]     | `invariant(fragment: state != "aborted", states: running, done)` | `a warnings entry names the row and the literal aborted`                                                                                            |
| phase_state_silent            | unit | [[linter.content_advisories.layer.declared_phase_state_silent]]     | `invariant(fragment: state != "halted", states include halted)` | `no vacuous_invariant warning`                                                                                                                         |
| narrow_state_surface          | unit | [[linter.content_advisories.layer.narrow_state_equality]]           | `invariant(fragment: state == "halted", or another binder)` | `no vacuous_invariant warning`                                                                                                                        |
| vacuity_never_gates           | unit | [[linter.content_advisories.layer.advisory_channel_only]]           | `corpus_with(vacuous_fragment, undeclared_forbidden_state)` | `exit code 0 ∧ zero issue findings under either vacuity rule`                                                                                          |
| family_partition              | unit | [[linter.content_advisories.layer.advisory_class_partition]]        | `cells_with(placeholder_expr, tautological_predicate, vacuous_fragment)` | `each finding carries its owning rule id ∧ no subject receives two findings from the same rule`                                                                   |