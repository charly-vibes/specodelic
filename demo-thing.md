---
id: demo.thing
kind: intent
statement: "THE system SHALL …"
---

# thing

One paragraph of prose: why this exists, what it's not.

<!-- Intent layer: the statement must match one of the five EARS patterns
(Ubiquitous / Event-Driven / State-Driven / Unwanted-Behavior /
Optional-Feature) and carry an imperative SHALL. Learn them with:
spk explain ears -->

## Constraints

<!-- kind: one of invariant | advisory | effect | extension_point. `invariant` is the only kind a
transition guard may cite; `effect` is the only kind a state's emits may
reference. Every constraint needs a deriving property (coverage). See:
spk explain kinds -->

| id | kind | expr | traces_to |
|----|------|------|-----------|

## Model

### States

<!-- `emits: [[<effect-constraint>]]` may follow a state that outputs. -->

- `initial`

### Transitions

<!-- guard: must cite an invariant Constraint by [[id]] — typed, not
conventional (advisory constraints can never gate a transition). Every
state must appear as a from or to of at least one transition. -->

| id | from | to | guard |
|----|------|----|-------|
| t1 | initial | initial | TODO — cite a Constraint by [[id]] |

## Properties

<!-- kind: one of unit | law. Every property must cite a constraint
in derives_from; a `law` predicate requires **identity:** and
**associativity:** case labels (law_requires_cases). -->

| id | kind | derives_from | generator | predicate |
|----|------|--------------|-----------|-----------|

<!-- Delete each guidance comment as you fill the layer in.
Full format guide: spk explain -->
