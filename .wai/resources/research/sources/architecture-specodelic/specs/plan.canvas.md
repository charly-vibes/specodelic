---
id: plan.canvas
kind: intent
statement: "THE mobile semantic sketch canvas capability SHALL satisfy typed structural consistency, provenance, reversible human correction and independently testable behavior as defined below."
---

# Mobile semantic sketch canvas

Provide a finger-first, offline-capable sketching interface whose drawings can be progressively converted into and edited as architectural geometry.

## Constraints

| id | kind | expr | traces_to |
|---|---|---|---|
| inkfirst | invariant | `Users SHALL draw ordinary strokes without choosing architectural object types first.` | [[plan.canvas]] |
| gestures | invariant | `Pan/zoom/undo SHALL coexist with selection, precision handles and finger or stylus input; gesture recognition SHALL not silently alter model geometry.` | [[plan.canvas]] |
| proposals | invariant | `Straightening, snapping and wall recognition SHALL show preview/accept/reject controls; users SHALL be able to disable automatic suggestions.` | [[plan.canvas]] |
| transactions | invariant | `Drag, dimension entry, split, connect, opening placement and deletion SHALL dispatch semantic commands, not directly mutate render objects.` | [[plan.canvas]] |
| errors | invariant | `When a command violates constraints the UI SHALL show the conflict and retain the last valid model.` | [[plan.canvas]] |
| undo | invariant | `Undo/redo SHALL restore model state, accepted interpretations and references without changing immutable raw evidence.` | [[plan.canvas]] |
| mobile | invariant | `Core draw/convert/edit flow SHALL work on a narrow touch display without a hardware keyboard or network.` | [[plan.canvas]] |
| access | invariant | `Controls SHALL offer accessible labels and target sizing; precision editing SHALL not require pixel-perfect tapping.` | [[plan.canvas]] |

## Model

### States

- `ink`
- `candidate`
- `editing`
- `conflict`

### Transitions

| id | from | to | guard |
|---|---|---|---|
| advance_candidate | ink | candidate | `preconditions for candidate hold` ∧ [[plan.canvas.inkfirst]] |
| advance_editing | candidate | editing | `preconditions for editing hold` ∧ [[plan.canvas.inkfirst]] |
| advance_conflict | editing | conflict | `preconditions for conflict hold` ∧ [[plan.canvas.inkfirst]] |

## Properties

| id | kind | derives_from | generator | predicate |
|---|---|---|---|---|
| inkfirst_holds | unit | [[plan.canvas.inkfirst]] | `fixture_inkfirst()` | `Users SHALL draw ordinary strokes without choosing architectural object types first.` |
| gestures_holds | unit | [[plan.canvas.gestures]] | `fixture_gestures()` | `Pan/zoom/undo SHALL coexist with selection, precision handles and finger or stylus input; gesture recognition SHALL not silently alter model geometry.` |
| proposals_holds | unit | [[plan.canvas.proposals]] | `fixture_proposals()` | `Straightening, snapping and wall recognition SHALL show preview/accept/reject controls; users SHALL be able to disable automatic suggestions.` |
| transactions_holds | unit | [[plan.canvas.transactions]] | `fixture_transactions()` | `Drag, dimension entry, split, connect, opening placement and deletion SHALL dispatch semantic commands, not directly mutate render objects.` |
| errors_holds | unit | [[plan.canvas.errors]] | `fixture_errors()` | `When a command violates constraints the UI SHALL show the conflict and retain the last valid model.` |
| undo_holds | unit | [[plan.canvas.undo]] | `fixture_undo()` | `Undo/redo SHALL restore model state, accepted interpretations and references without changing immutable raw evidence.` |
| mobile_holds | unit | [[plan.canvas.mobile]] | `fixture_mobile()` | `Core draw/convert/edit flow SHALL work on a narrow touch display without a hardware keyboard or network.` |
| access_holds | unit | [[plan.canvas.access]] | `fixture_access()` | `Controls SHALL offer accessible labels and target sizing; precision editing SHALL not require pixel-perfect tapping.` |
