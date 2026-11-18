---
id: plan.geometry
kind: intent
statement: "THE geometric realization and observability capability SHALL satisfy typed structural consistency, provenance, reversible human correction and independently testable behavior as defined below."
---

# Geometric realization and observability

Associate architectural topology with 2D/3D surfaces, frames and constraint evidence while detecting undetermined geometry.

## Constraints

| id | kind | expr | traces_to |
|---|---|---|---|
| realization | invariant | `Vertices SHALL have optional world positions; wall centerlines SHALL derive from vertices; 3D wall solids SHALL derive from bounded faces, height and known thickness.` | [[plan.geometry]] |
| surfaces | invariant | `Straight walls SHALL use bounded planar faces by default; curved/nonplanar extensions SHALL be optional parametrized patches, not a prerequisite for MVP.` | [[plan.geometry]] |
| gauge | invariant | `Observability and uncertainty reports SHALL distinguish coordinate gauge freedoms from physically undetermined dimensions.` | [[plan.geometry]] |
| hardsoft | invariant | `Hard topology constraints and uncertain measurement/photo/sketch evidence SHALL be represented separately.` | [[plan.geometry]] |
| weights | invariant | `Optimization SHALL report residuals, assumptions and outliers and SHALL never silently discard conflicting measurements.` | [[plan.geometry]] |
| order | invariant | `Equivalent constraint sets in different insertion orders SHALL converge to numerically equivalent results within declared tolerance, or report solver instability.` | [[plan.geometry]] |
| cert | invariant | `No dimension derived only from sketch appearance, photographs or assumptions SHALL be labelled certified; certification SHALL follow the measured-first policy and a documented empirical uncertainty basis.` | [[plan.geometry]] |
| alternatives | invariant | `Where discrete topology or continuous geometry is ambiguous, multiple candidates or explicit unknown values SHALL be preserved.` | [[plan.geometry]] |
| plans | invariant | `Floor plans SHALL declare horizontal section height and projection conventions, and 2D/3D dimensional comparisons SHALL use consistent frames.` | [[plan.geometry]] |

## Model

### States

- `underdetermined`
- `solved`
- `inconsistent`
- `verified`

### Transitions

| id | from | to | guard |
|---|---|---|---|
| advance_solved | underdetermined | solved | `preconditions for solved hold` ∧ [[plan.geometry.realization]] |
| advance_inconsistent | solved | inconsistent | `preconditions for inconsistent hold` ∧ [[plan.geometry.realization]] |
| advance_verified | inconsistent | verified | `preconditions for verified hold` ∧ [[plan.geometry.realization]] |

## Properties

| id | kind | derives_from | generator | predicate |
|---|---|---|---|---|
| realization_holds | unit | [[plan.geometry.realization]] | `fixture_realization()` | `Vertices SHALL have optional world positions; wall centerlines SHALL derive from vertices; 3D wall solids SHALL derive from bounded faces, height and known thickness.` |
| surfaces_holds | unit | [[plan.geometry.surfaces]] | `fixture_surfaces()` | `Straight walls SHALL use bounded planar faces by default; curved/nonplanar extensions SHALL be optional parametrized patches, not a prerequisite for MVP.` |
| gauge_holds | unit | [[plan.geometry.gauge]] | `fixture_gauge()` | `Observability and uncertainty reports SHALL distinguish coordinate gauge freedoms from physically undetermined dimensions.` |
| hardsoft_holds | unit | [[plan.geometry.hardsoft]] | `fixture_hardsoft()` | `Hard topology constraints and uncertain measurement/photo/sketch evidence SHALL be represented separately.` |
| weights_holds | unit | [[plan.geometry.weights]] | `fixture_weights()` | `Optimization SHALL report residuals, assumptions and outliers and SHALL never silently discard conflicting measurements.` |
| order_holds | unit | [[plan.geometry.order]] | `fixture_order()` | `Equivalent constraint sets in different insertion orders SHALL converge to numerically equivalent results within declared tolerance, or report solver instability.` |
| cert_holds | unit | [[plan.geometry.cert]] | `fixture_cert()` | `No dimension derived only from sketch appearance, photographs or assumptions SHALL be labelled certified; certification SHALL follow the measured-first policy and a documented empirical uncertainty basis.` |
| alternatives_holds | unit | [[plan.geometry.alternatives]] | `fixture_alternatives()` | `Where discrete topology or continuous geometry is ambiguous, multiple candidates or explicit unknown values SHALL be preserved.` |
| plans_holds | unit | [[plan.geometry.plans]] | `fixture_plans()` | `Floor plans SHALL declare horizontal section height and projection conventions, and 2D/3D dimensional comparisons SHALL use consistent frames.` |
