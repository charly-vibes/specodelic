---
id: kinds
kind: intent
statement: "THE kinds spec SHALL define the field set, sub-kind values, and shape-validity rule for each of the five schema objects (Intent, Constraint, State, Transition, Property) that every specodelic file instantiates."
---

# Kinds

The five objects of `𝒦` — `Intent`, `Constraint`, `State`, `Transition`,
`Property` — are referenced constantly throughout this repo but have never
been specified *as subjects* in their own right (`STATUS.md` §4, P0). This
file is that specification: for each kind, its exact field set and which
values its own internal `kind` column (where it has one) may take. Every
later checker — `linter.frontmatter` through `linter.coverage` — assumes a
row has already been correctly bucketed into one of these five before it
runs its own check; this file is what makes that bucketing itself a
checkable fact instead of an unstated assumption.

## Constraints

| id                    | kind      | expr                                                                                                                                            | traces_to | satisfies |
|------------------------|-----------|------------------------------------------------------------------------------------------------------------------------------------------------------|----------------------|
| kind_enum_closed        | invariant | `the five kinds are exactly {Intent, Constraint, State, Transition, Property}, one per top-level section of a spec file (frontmatter, Constraints table, Model/States, Model/Transitions, Properties table)` | [[kinds]] |          |
| intent_row_shape        | invariant | `an Intent row is the file's frontmatter block; its fields are exactly {id, kind, statement}, with kind == "intent"`                                    | [[kinds]] |          |
| constraint_row_shape    | invariant | `a Constraint row is one row of the Constraints table; its base fields are exactly {id, kind, expr, traces_to}, optionally followed by typed reference columns — each such column must be declared in `specodelic.md`'s Reference Typing table (currently `satisfies`, `observes`) and carries no meaning beyond that typing; kind ∈ {invariant, advisory, effect, extension_point}`   | [[kinds]] |          |
| state_row_shape         | invariant | `a State row is one bullet under Model/States; its fields are exactly {id, emits?} and it has no kind column of its own — states are named variants, never a typed column; `emits` is optional and, when present, must resolve to a Constraint with kind == "effect" (see `specodelic.md`'s Reference Typing table) — a state with no `emits` is a bare automaton state, not a Moore state, and both are well-formed` | [[kinds]] |          |
| transition_row_shape    | invariant | `a Transition row is one row of the Model/Transitions table; its fields are exactly {id, from, to, guard}, with no kind column of its own`              | [[kinds]] |          |
| property_row_shape      | invariant | `a Property row is one row of the Properties table; its fields are exactly {id, kind, derives_from, generator, predicate}, with kind ∈ {unit, law}`     | [[kinds]] |          |
| kind_field_extensible   | invariant | `a Constraint or Property row's own kind value-set is one instance of [[specodelic.append_only_variants]] — it grows only under a new Revision heading in this file, never silently` | [[kinds]] |          |
| kind_assignment_failure | effect | `kinds.kind_assignment_failure(detail)` | [[kinds]] | [[errors.envelope_error_kind]] ∧ [[errors.exit_code_mapping]] ∧ [[errors.remediation_hint_present]] |
| shape_check_failure | effect | `kinds.shape_check_failure(detail)` | [[kinds]] | [[errors.envelope_error_kind]] ∧ [[errors.exit_code_mapping]] ∧ [[errors.remediation_hint_present]] |

## Model

### States
- `unclassified`
- `kind_assigned`
- `shape_checked`
- `passed`
- `kind_failed` (emits: `[[kinds.kind_assignment_failure]]`)
- `shape_failed` (emits: `[[kinds.shape_check_failure]]`)

### Transitions

| id           | from           | to             | guard                                                                                                                                                                                                                                                                       |
|--------------|----------------|----------------|------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| begin        | unclassified   | kind_assigned  | `row belongs to exactly one of the five sections (frontmatter / Constraints table / Model States / Model Transitions / Properties table)`                                                                                                                                    |
| assign_ok    | kind_assigned  | shape_checked  | [[kinds.kind_enum_closed]]                                                                                                                                                                                                                                                    |
| assign_fail | kind_assigned | kind_failed | `¬([[kinds.kind_enum_closed]])` |
| accept       | shape_checked  | passed         | `(row.kind==Intent ∧ [[kinds.intent_row_shape]]) ∨ (row.kind==Constraint ∧ [[kinds.constraint_row_shape]]) ∨ (row.kind==State ∧ [[kinds.state_row_shape]]) ∨ (row.kind==Transition ∧ [[kinds.transition_row_shape]]) ∨ (row.kind==Property ∧ [[kinds.property_row_shape]])` |
| reject | shape_checked | shape_failed | `¬([[kinds.intent_row_shape]] ∧ [[kinds.constraint_row_shape]] ∧ [[kinds.state_row_shape]] ∧ [[kinds.transition_row_shape]] ∧ [[kinds.property_row_shape]])` |


## Properties

| id                                    | kind | derives_from                       | generator                                                      | predicate                                                                                                                                                                                             |
|-----------------------------------------|------|-----------------------------------------|---------------------------------------------------------------------|------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| unknown_kind_rejected                    | unit | [[kinds.kind_enum_closed]]             | `row_belonging_to_a_sixth_section("Actions")`                        | `check(row) == failed`                                                                                                                                                                                |
| intent_missing_statement_rejected        | unit | [[kinds.intent_row_shape]]             | `intent_row_without("statement")`                                    | `check(row) == failed`                                                                                                                                                                                |
| constraint_row_extra_field_rejected      | unit | [[kinds.constraint_row_shape]]         | `constraint_row_with(extra_field: "guard")`                          | `check(row) == failed`                                                                                                                                                                                |
| state_row_with_extra_field_rejected      | unit | [[kinds.state_row_shape]]              | `state_row_with(extra_field: "kind")`                                | `check(row) == failed`                                                                                                                                                                                |
| transition_missing_guard_rejected        | unit | [[kinds.transition_row_shape]]         | `transition_row_without("guard")`                                    | `check(row) == failed`                                                                                                                                                                                |
| property_row_bad_subkind_rejected        | unit | [[kinds.property_row_shape]]           | `property_row_with(kind: "audit")`                                   | `check(row) == failed`                                                                                                                                                                                |
| subkind_addition_without_revision_rejected | unit | [[kinds.kind_field_extensible]]      | `diff_adding_subkind_value_with_no_new_Revision_heading()`           | `check(diff) == failed`                                                                                                                                                                               |
| constraint_row_advisory_accepted         | unit | [[kinds.constraint_row_shape]]         | `constraint_row_with(kind: "advisory")`                              | `check(row) == passed`                                                                                                                                                                                |
| constraint_row_effect_accepted           | unit | [[kinds.constraint_row_shape]]         | `constraint_row_with(kind: "effect")`                                | `check(row) == passed`                                                                                                                                                                                |
| state_row_emits_accepted                 | unit | [[kinds.state_row_shape]]              | `state_row_with(emits: an_effect_kind_constraint_id)`                | `check(row) == passed`                                                                                                                                                                                |
| state_row_without_emits_still_passes     | unit | [[kinds.state_row_shape]]              | `state_row_with(fields: {id})` — no `emits`                          | `check(row) == passed` — `emits` is optional, not a second required field                                                                                                                            |
| state_row_emits_wrong_kind_rejected      | unit | [[kinds.state_row_shape]]              | `state_row_with(emits: an_invariant_kind_constraint_id)`             | `check(row) == failed`                                                                                                                                                                                |
| constraint_row_extension_point_accepted  | unit | [[kinds.constraint_row_shape]]         | `constraint_row_with(kind: "extension_point")`                       | `check(row) == passed`                                                                                                                                                                                |
| well_formed_row_passes                   | unit | [[kinds.kind_enum_closed]]             | `arbitrary_well_formed_row_of_one_kind()`                            | `check(row) == passed`                                                                                                                                                                                |
| kind_shape_naturality                    | law  | [[kinds.kind_enum_closed]]             | `arbitrary_row_of_one_kind(), arbitrary_id_rename()`                 | **identity:** `assigned_kind(rename(row,a,a)) == assigned_kind(row)`  **associativity:** `assigned_kind(rename(rename(row,a,b),b,c)) == assigned_kind(rename(row,a,c))` — renaming a row's id never changes which of the five kinds it belongs to |
| kind_assignment_failure_label_asserted | unit | [[kinds.kind_assignment_failure]] | `kind_assignment_failure_raised()` | `error_label == "kinds.kind_assignment_failure"` — renaming the label touches the error Constraint, this property, and its note together (EDGE-002) |
| shape_check_failure_label_asserted | unit | [[kinds.shape_check_failure]] | `shape_check_failure_raised()` | `error_label == "kinds.shape_check_failure"` — renaming the label touches the error Constraint, this property, and its note together (EDGE-002) |
## Notes

**Two different things are called "kind" in this repo, and they must not be
conflated.** `𝒦`'s five objects (`Intent`, `Constraint`, `State`,
`Transition`, `Property`) are what this file calls *kinds* — which section
of a spec file a row lives in. Separately, two of those kinds carry their
*own* `kind` column as one of their fields: a Constraint row's `kind`
(currently only `invariant`) and a Property row's `kind` (`unit` or
`law`). These are sub-classifications *within* an already-assigned 𝒦-kind,
not alternate values of it — a Property row with `kind: "law"` is still a
`Property` in `𝒦`'s sense, never a sixth object. Flagging this as
**CLAR-002** (same shape as `STATUS.md`'s open `CLAR-001`, the
`coverage`/`linter.coverage` collision): worth an explicit rename in a
future revision (e.g. `subkind` for the column, reserving `kind` for the
𝒦-level classification alone) rather than leaving two meanings on one
word. Not resolved here, since renaming a live column is exactly the kind
of change that should go through `rename_naturality`, not be done by hand.

`property_row_shape`'s `kind ∈ {unit, law}` is the authoritative home for
the enum `linter-schema_shape.md`'s `EXCL-001` wants a third member added
to (`kind = "audit"`, for claims like `no_prose_field_parsed` that are
about the parser's implementation rather than about spec-file content).
When `EXCL-001` is actioned, this file's `property_row_shape` constraint
is what gets a new Revision, and `property_row_bad_subkind_rejected`'s
generator would need to change from a rejection case to an acceptance
case for `kind: "audit"` specifically.

**A gap surfaced writing this file, folded into `specodelic.md` as
Revision 4 (same session, per `AGENTS.md`):** nothing before this file
required a Constraint row's `kind` or a Property row's `kind` to come from
a closed set at all — `frontmatter_valid` only ever checked the file-level
`kind == "intent"`. A Property row with `kind: "made_up"` had no
constraint anywhere rejecting it. `constraint_row_shape` and
`property_row_shape` above close that, and `specodelic.md` now carries
matching top-level invariants (`constraint_kind_closed`,
`property_kind_closed`) tracing back to this file. **Follow-up closed the
same session:** `linter-schema_shape.md` now owns and enforces both — see
its `kind_ok` transition and the two `..._kind_invalid_rejected`
properties.

## Revision 2

`constraint_row_shape`'s `kind` set widens from `{invariant}` to
`{invariant, advisory}` — `kind_field_extensible` exercised exactly as it
was designed to be, one Revision heading per widening, nothing silent.
An `advisory` Constraint is still fully a row of the Constraints table
(same four fields, same table); the only place its meaning differs is
outside this file entirely, in `specodelic.md`'s Reference Typing table,
which now types `guard`'s target as `Constraint, kind == invariant` —
this file owns *row shape*, not *what other rows may reference a row*, so
that cross-row rule correctly lives there and not here. `property_kind_closed`
is untouched: `EXCL-001`'s pending `audit` value is a separate, still-open
question and this Revision takes no position on it.

## Revision 3

Two changes, both from `specodelic.md` Revision 6, folded in here the
same session per `AGENTS.md`:

- **`constraint_row_shape`'s `kind` set widens again**, from
  `{invariant, advisory}` to `{invariant, advisory, effect}`. Same pattern
  as Revision 2 — one more `kind_field_extensible`-governed widening, row
  shape itself untouched (still `{id, kind, expr, traces_to}`). What an
  `effect`-kind Constraint means to other rows is again a cross-row fact
  that lives in `specodelic.md`'s Reference Typing table, not here: only
  `emits` may target one, the same way only `guard` may target an
  `invariant` one.
- **`state_row_shape` gains an optional field, `emits`.** This *is* a
  row-shape change, unlike the two `kind` widenings above — State's field
  set moves from `{id}` to `{id, emits?}`, which is why it's stated as its
  own bullet rather than folded into the `kind_field_extensible` framing.
  `emits` gives a State the output half of a Moore machine (states,
  transitions, and a function from state to output — only the first two
  had anywhere to live before this). Making it optional rather than
  required means a plain automaton state and a Moore state are both
  well-formed `State` rows; nothing about existing specs (none of which
  use `emits`) becomes invalid by this widening, so `variant_set_grows_only`-
  style backward compatibility holds automatically. See `USAGE.md` for the
  worked pattern (a reducer's three phases, each with its own `emits`).

- **`kind_field_extensible`'s wording was also corrected**, independent of
  either change above: it used to restate, in its own words, the same
  "grows only via Revision" rule `specodelic.md`'s `append_only_variants`
  already stated for variant tables — the two had drifted slightly out of
  sync (compare Revision 5's wording of `append_only_variants` against
  this file's `kind_field_extensible` at the time). `specodelic.md`
  Revision 6 generalized `append_only_variants` to cover kind value-sets
  and the Reference Typing table too; this file's row now just cites that
  one rule instead of keeping a second copy of the logic.

## Revision 4

`constraint_row_shape`'s `kind` set widens a third time, from
`{invariant, advisory, effect}` to `{invariant, advisory, effect,
extension_point}`, folded in from `specodelic.md` Revision 7 the same
session per `AGENTS.md`. Same pattern as Revisions 2 and 3 — row shape
itself is untouched (still exactly `{id, kind, expr, traces_to}`); only
the closed value-set `kind_field_extensible` governs grows by one member.
An `extension_point` row states a contract a *consumer's* code must
satisfy (a method signature, a behavioral expectation) rather than an
invariant this file's own author enforces — what makes a Constraint
reference *that* row meaningful lives, as with `effect`/`emits`, entirely
outside this file, in `specodelic.md`'s Reference Typing table, which now
types a new field, `satisfies`, at `Constraint, kind == extension_point
only`. `constraint_row_extension_point_accepted` above is the matching
acceptance case, added the same way `constraint_row_advisory_accepted`
and `constraint_row_effect_accepted` were.

## Revision 5

`constraint_row_shape` stated its fields as *exactly*
`{id, kind, expr, traces_to}` — but the format already carries optional
typed reference columns beside `traces_to`: Revision 7 of
`specodelic.md` added `satisfies` (USAGE §2.6's five-column example), and
Revision 9 adds `observes`. The "exactly four fields" reading and the
Reference Typing table contradicted each other — HITL ticket
`specodelic-mp1` row 9 asked which governs; this Revision answers: the
table governs.

- `constraint_row_shape` now states the general rule: base fields are
  exactly `{id, kind, expr, traces_to}`, plus optional typed reference
  columns — each such column must be declared in `specodelic.md`'s
  Reference Typing table, and carries no meaning beyond that typing (the
  table row is the whole contract). No `observes` special case — the
  next field costs one typing row, zero shape edits.
- No existing file changes meaning: every optional column in use
  (`satisfies`) was already valid under the table-driven reading; the
  narrowing "exactly four" was the only reading it violated, and no
  checker enforced it.

Row shape is otherwise untouched — the same
general-rule-instead-of-special-case move as `kind_field_extensible`.

## Revision 6

`EXCL-001` is resolved by dissolution — `property_row_shape`'s kind set
stays `{unit, law}` and no `audit` member is added (HITL ticket
`specodelic-mp1` row 5, user-approved 2026-09-30, advised by a typed Jev
evaluation, jev-1.13.0, confidence 1.00).

The only instance that ever motivated an `audit` kind was
`linter-schema_shape.md`'s `no_prose_field_parsed` — a claim about the
parser's implementation, framed as "verified by code audit, not by a
runnable PBT case". That framing was wrong: the claim is mechanically
testable by a planted-prose differential parse (generate a spec with
prose in `rationale`/`description`, parse, assert no parsed row contains
it). An enum member whose definition is "not machine-verified" would
crack the format's promise that Properties are runnable, and
`kind_field_extensible` should be spent when the shape genuinely needs
it — not on a singleton. The rewrite of that property to a runnable
`unit` row lands with the corpus-reconciliation work (`specodelic-cxq`);
a genuinely unrunnable implementation claim remains a possible future
widening, on demand and under a Revision heading, as always.
