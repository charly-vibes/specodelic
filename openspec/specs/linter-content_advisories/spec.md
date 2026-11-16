---
id: linter.content_advisories
kind: intent
statement: "WHEN a spec corpus is linted, THE linter SHALL warn — advisory, exit 0 — on whole-cell placeholder or tautological content and on invariant constraint exprs or non-law Property predicates whose executable fragment cites no declared model element, and on invariants that forbid a state the Model never declares, and SHALL never gate on any of these findings or judge reachability."
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

| id                            | kind      | expr                                                                                                                                                                                                                                                                                                                                                 | traces_to                            |
|-------------------------------|-----------|------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|--------------------------------------|
| placeholder_classes           | invariant | `a structured content cell is a placeholder when its trimmed text is TODO, TBD, or FIXME (optionally followed by a separator and a note), a row id begins with scaffold_ in a file that still carries the scaffold signature (its intent statement is a placeholder, or a constraint expr or property predicate is itself a placeholder or a tautology), or the intent statement is a placeholder (the spk new stub "THE system SHALL …" or a whole-cell placeholder word) — the real producers are the spk new stub/TODO cells and spk migrate's scaffold_constraint/scaffold_transition/scaffold_property sidecar rows whose cells are bare true; detection is whole-cell or scaffold-signature-gated row-id-prefix, never a substring search inside an executable fragment` | [[linter.content_advisories]] |
| tautology_class               | invariant | `a property predicate or invariant constraint expr is tautological when, after stripping its surrounding code span and any **rust:** fragment marker, its trimmed text is exactly true or ⊤, or it is one identifier compared equal to itself (`x == x`) — an effect or advisory constraint expr is never judged (it is a label, not a claim that must be able to fail); no arithmetic, calls, or compound expressions are judged`                                                               | [[linter.content_advisories]] |
| advisory_severity             | invariant | `every content advisory is emitted on the success envelope's warnings channel with exit 0 — never an Issue, never a nonzero exit, and never a change to the hard-findings count of an otherwise clean corpus`                                                                                                                                            | [[linter.content_advisories]] |
| fragment_interior_not_scanned | invariant | `the checker never inspects the interior of a **rust:** fragment for placeholder words — a legitimate fragment such as todo_predicate! or a string containing TODO stays silent unless the whole cell is one of the placeholder forms above`                                                                                                            | [[linter.content_advisories]] |
| fragment_citation_required    | invariant | `an invariant Constraint expr or a non-law Property predicate that carries an executable **rust:** fragment must cite at least one declared model element — a Model state id, a transition id, or a Constraint/Property row id; the intent id is the file's own identity and is never a model element`                                                     | [[linter.content_advisories]] |
| citation_whole_token          | invariant | `citation is a whole-token match over [A-Za-z0-9_]+ runs, never substring — unlocked does not cite lock and beheld does not cite held — and a dot is a token boundary, so a dotted demo.spec.held reference cites held through its segments`                                                                                                             | [[linter.content_advisories]] |
| declared_citation_sufficient  | invariant | `one declared citation is sufficient: a declared state, transition, constraint, or property that is never exercised is never flagged, because the check compares the fragment's citation text against the declared id set and never judges reachability or coverage of declared ids`                                                                       | [[linter.content_advisories]] |
| law_predicate_exempt          | invariant | `a law-kind Property predicate is outside the fragment surface — its law-case labels are the law-case grammar, never fragment markers, so the vacuity check skips law rows entirely`                                                                                                                                                                    | [[linter.content_advisories]] |
| forbidden_state_declared      | invariant | `an invariant fragment's state != "literal" comparison, or its mirror literal != state, must name a state declared under Model/States — an undeclared forbidden state cannot exist, so the invariant is structurally vacuous and is reported with the literal it forbids`                                                                                 | [[linter.content_advisories]] |
| declared_phase_state_silent   | invariant | `a state declared under Model/States is silent even when no transition enters it — a declared phase its correct runs never reach is legitimate defensive design pressure, and the rule never judges reachability`                                                                                                                                        | [[linter.content_advisories]] |
| narrow_state_equality         | invariant | `only the bare state binder's string inequality is judged — other binders, ==, arithmetic, calls, and escaped or raw literals stay silent, and my_state, model.state, and state_extra are other names, never the binder`                                                                                                                                  | [[linter.content_advisories]] |
| advisory_channel_only         | invariant | `both vacuity findings ride the success envelope's warnings channel with exit 0 — never an Issue, never a nonzero exit, and never a change to the failures count of an otherwise clean corpus`                                                                                                                                                             | [[linter.content_advisories]] |
| advisory_class_partition      | invariant | `each vacuity rule carries its own id and remedy and never restates a placeholder or tautology finding: one finding per fragment for the citation rule, one per undeclared forbidden literal for the invariant rule, and the whole-cell rules never restate a vacuity finding`                                                                             | [[linter.content_advisories]] |

## Model

### States

- `scanning`
- `clean`
- `warned`

### Transitions

| id          | from     | to     | guard                                                                                                                                                                                                                                                   |
|-------------|----------|--------|---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| scan_clean  | scanning | clean  | `no structured cell is a whole-cell placeholder, no predicate or invariant expr is a tautology, and no vacuity rule fires`                                                                                                                              |
| scan_warned | scanning | warned | [[linter.content_advisories.placeholder_classes]] ∨ [[linter.content_advisories.tautology_class]] ∨ [[linter.content_advisories.fragment_citation_required]] ∨ [[linter.content_advisories.forbidden_state_declared]]           |

## Properties

| id                            | kind | derives_from                                                        | generator                                        | predicate                                                                                                                                                       |
|-------------------------------|------|---------------------------------------------------------------------|--------------------------------------------------|-----------------------------------------------------------------------------------------------------------------------------------------------------------------|
| placeholder_warned            | unit | [[linter.content_advisories.placeholder_classes]]             | `spec_with_todo_expr()`                          | `exit 0 ∧ a warnings-channel entry names the row and rule linter.content_advisories.placeholder_content`                                                          |
| tautology_warned              | unit | [[linter.content_advisories.tautology_class]]                 | `spec_with_true_predicate()`                     | `exit 0 ∧ a warnings-channel entry names the row and rule linter.content_advisories.tautological_predicate`                                                       |
| warning_never_gates           | unit | [[linter.content_advisories.advisory_severity]]               | `corpus_with_placeholder_cells()`                | `exit code 0 ∧ zero Issue findings attributable to this checker`                                                                                                  |
| fragment_interior_silent      | unit | [[linter.content_advisories.fragment_interior_not_scanned]]   | `predicate_with_todo_predicate_macro()`          | `no warning — a legitimate executable fragment is never treated as a placeholder`                                                                                |
| vacuous_fragment_warns        | unit | [[linter.content_advisories.fragment_citation_required]]      | `spec_with(invariant_expr: flag != 0)`           | `exit 0 ∧ a warnings entry names the row and rule linter.fragment_binds_nothing ∧ the issues channel carries no matching finding`                                 |
| whole_token_citation          | unit | [[linter.content_advisories.citation_whole_token]]            | `fragment_naming(unlocked) over a declared lock` | `no citation and a warning, while a dotted demo.spec.held reference cites held and stays silent`                                                                   |
| declared_element_sufficiency  | unit | [[linter.content_advisories.declared_citation_sufficient]]    | `fragment_citing(a_state_no_transition_enters)`  | `no fragment_binds_nothing warning`                                                                                                                               |
| law_predicate_not_judged      | unit | [[linter.content_advisories.law_predicate_exempt]]            | `law_property(predicate_with_case_labels_only)`  | `no fragment_binds_nothing warning for the law row`                                                                                                               |
| undeclared_forbidden_state_warns | unit | [[linter.content_advisories.forbidden_state_declared]]     | `invariant(fragment: state != "aborted", states: running, done)` | `a warnings entry names the row and the literal aborted`                                                                                            |
| phase_state_silent            | unit | [[linter.content_advisories.declared_phase_state_silent]]     | `invariant(fragment: state != "halted", states include halted)` | `no vacuous_invariant warning`                                                                                                                         |
| narrow_state_surface          | unit | [[linter.content_advisories.narrow_state_equality]]           | `invariant(fragment: state == "halted", or another binder)` | `no vacuous_invariant warning`                                                                                                                        |
| vacuity_never_gates           | unit | [[linter.content_advisories.advisory_channel_only]]           | `corpus_with(vacuous_fragment, undeclared_forbidden_state)` | `exit code 0 ∧ zero issue findings under either vacuity rule`                                                                                          |
| family_partition              | unit | [[linter.content_advisories.advisory_class_partition]]        | `cells_with(placeholder_expr, tautological_predicate, vacuous_fragment)` | `each finding carries its owning rule id ∧ no subject receives two findings from the same rule`                                                                   |

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

### Requirement: Fragment citation advisory

A `**rust:**` fragment on an invariant Constraint's `expr` or a non-law
Property's `predicate` SHALL cite at least one declared model element — a
Model state id, a transition id, or a Constraint/Property row id, never the
intent id — matched whole-token with `.` as a token boundary, and a fragment
citing none SHALL be reported as `linter.fragment_binds_nothing` on the
warnings channel at exit 0, naming the row and the file; one declared citation
SHALL be sufficient.

#### Scenario: vacuous_fragment_warns

- **WHEN** an invariant Constraint's expr is `**rust:** flag != 0` and the
  file declares a state, a transition, that constraint, and a unit property
- **THEN** lint reports `linter.fragment_binds_nothing` for that row on the
  warnings channel, and the issues channel carries no finding under that rule
  (exit 0)
- **VERIFIES** [[linter.content_advisories.vacuous_fragment_warns]]

#### Scenario: whole_token_citation

- **WHEN** a fragment reads `**rust:** unlocked == false` and the file
  declares a `lock` state
- **THEN** the fragment does not cite `lock` and warns, while a fragment
  naming `demo.spec.held` cites a declared `held` and stays silent
- **VERIFIES** [[linter.content_advisories.whole_token_citation]]

#### Scenario: declared_element_sufficiency

- **WHEN** a fragment cites a declared state that no transition ever enters
- **THEN** no `linter.fragment_binds_nothing` finding is emitted for that row —
  citing one declared element is sufficient, and reachability is never judged
- **VERIFIES** [[linter.content_advisories.declared_element_sufficiency]]

#### Scenario: law_predicate_not_judged

- **WHEN** a law-kind Property's predicate carries `**identity:**` and
  `**associativity:**` case labels and no `**rust:**` fragment
- **THEN** no `linter.fragment_binds_nothing` finding is emitted for that row
- **VERIFIES** [[linter.content_advisories.law_predicate_not_judged]]

### Requirement: Forbidden-state declaration advisory

An invariant Constraint's `**rust:**` fragment comparing the bare `state`
binder for string inequality SHALL name a state declared under `Model/States`,
and each undeclared literal SHALL be reported as `linter.vacuous_invariant`
on the warnings channel, naming the constrained row and the literal, with the
run still exiting 0; a declared state SHALL be silent regardless of
reachability, and other binders, `==`, arithmetic, calls, and escaped or raw
literals SHALL be outside the surface.

#### Scenario: undeclared_forbidden_state_warns

- **WHEN** an invariant expr is `**rust:** state != "aborted"` and
  `Model/States` declares only `running` and `done`
- **THEN** lint reports `linter.vacuous_invariant` naming the row and
  `aborted`, on the warnings channel (exit 0)
- **VERIFIES** [[linter.content_advisories.undeclared_forbidden_state_warns]]

#### Scenario: phase_state_silent

- **WHEN** an invariant expr is `**rust:** state != "halted"` and
  `Model/States` declares `halted` even though the correct runs never enter it
- **THEN** no `linter.vacuous_invariant` finding is emitted — the check is
  citation versus declaration, never reachability
- **VERIFIES** [[linter.content_advisories.phase_state_silent]]

#### Scenario: narrow_state_surface

- **WHEN** an invariant expr uses `state == "halted"`, a different binder
  (`my_state` or `model.state`), or a raw/escaped literal
- **THEN** no `linter.vacuous_invariant` finding is emitted — only the bare
  `state` binder's `!=` string comparison is judged
- **VERIFIES** [[linter.content_advisories.narrow_state_surface]]

### Requirement: Vacuity advisory severity and family partition

Both vacuity rules SHALL ride the lint success envelope's warnings channel
with exit 0, SHALL never push a finding to the issues channel, SHALL emit at
most one finding per subject under their own rule id and remedy, and SHALL
NOT restate a `linter.placeholder_content` or `linter.tautological_predicate`
finding; the whole-cell rules SHALL NOT restate a vacuity finding.

#### Scenario: vacuity_never_gates

- **WHEN** a corpus contains a fragment citing no declared element and an
  invariant forbidding an undeclared state
- **THEN** the run exits 0, both findings appear on the warnings channel, and
  the issues channel carries neither rule id
- **VERIFIES** [[linter.content_advisories.vacuity_never_gates]]

#### Scenario: family_partition

- **WHEN** one corpus carries a whole-cell placeholder, a tautological
  predicate, and a vacuous fragment
- **THEN** each finding carries its owning rule id, no rule emits twice for
  one subject, and no finding is re-issued under another rule's id
- **VERIFIES** [[linter.content_advisories.family_partition]]
