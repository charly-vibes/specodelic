---
id: plan.views
kind: intent
statement: "THE derived spatial views and export capability SHALL satisfy typed structural consistency, provenance, reversible human correction and independently testable behavior as defined below."
---

# Derived spatial views and export

Generate synchronized 2D, 3D and CAD outputs from one canonical model.

## Constraints

| id | kind | expr | traces_to |
|---|---|---|---|
| canonical | invariant | `The architectural graph and accepted metric constraints SHALL be the single authoritative project state; renderers SHALL be pure or explicitly cached projections.` | [[plan.views]] |
| edits | invariant | `View interactions SHALL be translated into validated semantic commands; 2D and 3D view state SHALL NOT become competing models.` | [[plan.views]] |
| match | invariant | `When section/projection conventions agree, lengths and shared wall incidences visible in 2D and 3D SHALL match within numeric tolerance.` | [[plan.views]] |
| extrude | invariant | `3D extrusion SHALL use known wall heights and thicknesses; unknown values SHALL be marked as assumed in previews.` | [[plan.views]] |
| cad | invariant | `CAD export SHALL retain stable entity IDs, unit system, reference frame, provenance sidecar and clearly labeled uncertified values.` | [[plan.views]] |
| idempotent | invariant | `Repeated exports of the same revision and settings SHALL produce equivalent canonical geometric contents and consistent identifiers.` | [[plan.views]] |
| no_backflow | invariant | `External CAD modifications SHALL not silently overwrite the canonical model; any import-back requires explicit reconciliation.` | [[plan.views]] |

## Model

### States

- `ready`
- `rendered`
- `exported`

### Transitions

| id | from | to | guard |
|---|---|---|---|
| advance_rendered | ready | rendered | `preconditions for rendered hold` ∧ [[plan.views.canonical]] |
| advance_exported | rendered | exported | `preconditions for exported hold` ∧ [[plan.views.canonical]] |

## Properties

| id | kind | derives_from | generator | predicate |
|---|---|---|---|---|
| canonical_holds | unit | [[plan.views.canonical]] | `fixture_canonical()` | `The architectural graph and accepted metric constraints SHALL be the single authoritative project state; renderers SHALL be pure or explicitly cached projections.` |
| edits_holds | unit | [[plan.views.edits]] | `fixture_edits()` | `View interactions SHALL be translated into validated semantic commands; 2D and 3D view state SHALL NOT become competing models.` |
| match_holds | unit | [[plan.views.match]] | `fixture_match()` | `When section/projection conventions agree, lengths and shared wall incidences visible in 2D and 3D SHALL match within numeric tolerance.` |
| extrude_holds | unit | [[plan.views.extrude]] | `fixture_extrude()` | `3D extrusion SHALL use known wall heights and thicknesses; unknown values SHALL be marked as assumed in previews.` |
| cad_holds | unit | [[plan.views.cad]] | `fixture_cad()` | `CAD export SHALL retain stable entity IDs, unit system, reference frame, provenance sidecar and clearly labeled uncertified values.` |
| idempotent_holds | unit | [[plan.views.idempotent]] | `fixture_idempotent()` | `Repeated exports of the same revision and settings SHALL produce equivalent canonical geometric contents and consistent identifiers.` |
| no_backflow_holds | unit | [[plan.views.no_backflow]] | `fixture_no_backflow()` | `External CAD modifications SHALL not silently overwrite the canonical model; any import-back requires explicit reconciliation.` |
