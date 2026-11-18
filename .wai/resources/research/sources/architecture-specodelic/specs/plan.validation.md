---
id: plan.validation
kind: intent
statement: "THE falsifiable evaluation program capability SHALL satisfy typed structural consistency, provenance, reversible human correction and independently testable behavior as defined below."
---

# Falsifiable evaluation program

Gate implementation choices with reproducible tests and control baseline.

## Constraints

| id | kind | expr | traces_to |
|---|---|---|---|
| dataset | invariant | `Evaluation SHALL include rectangular, skewed, occluded, open-plan, poor-light and underspecified cases, plus intentionally distorted hand sketches.` | [[plan.validation]] |
| gold | invariant | `Reference geometry SHALL be measured independently of the application under test.` | [[plan.validation]] |
| ablate | invariant | `Compare manual canvas, canvas+snapping, canvas+interpretation, and canvas+photo evidence under equivalent hardware and tasks.` | [[plan.validation]] |
| metrics | invariant | `Report median, tail errors, topology error, invalid-model rate, false certification rate, user time and failed-operation recovery.` | [[plan.validation]] |
| thresholds | invariant | `Acceptance thresholds SHALL be set before data collection; observed results SHALL not redefine passing thresholds after the fact.` | [[plan.validation]] |
| failure | invariant | `Every failed case SHALL retain source input, model revision, proposed commands, solver diagnostics and reproduction instructions.` | [[plan.validation]] |
| reuse | invariant | `ACSet generic core versus conventional graph SHALL use the same fixtures and record correctness, runtime, memory and maintainability evidence.` | [[plan.validation]] |

## Model

### States

- `planned`
- `executed`
- `adjudicated`

### Transitions

| id | from | to | guard |
|---|---|---|---|
| advance_executed | planned | executed | `preconditions for executed hold` ∧ [[plan.validation.dataset]] |
| advance_adjudicated | executed | adjudicated | `preconditions for adjudicated hold` ∧ [[plan.validation.dataset]] |

## Properties

| id | kind | derives_from | generator | predicate |
|---|---|---|---|---|
| dataset_holds | unit | [[plan.validation.dataset]] | `fixture_dataset()` | `Evaluation SHALL include rectangular, skewed, occluded, open-plan, poor-light and underspecified cases, plus intentionally distorted hand sketches.` |
| gold_holds | unit | [[plan.validation.gold]] | `fixture_gold()` | `Reference geometry SHALL be measured independently of the application under test.` |
| ablate_holds | unit | [[plan.validation.ablate]] | `fixture_ablate()` | `Compare manual canvas, canvas+snapping, canvas+interpretation, and canvas+photo evidence under equivalent hardware and tasks.` |
| metrics_holds | unit | [[plan.validation.metrics]] | `fixture_metrics()` | `Report median, tail errors, topology error, invalid-model rate, false certification rate, user time and failed-operation recovery.` |
| thresholds_holds | unit | [[plan.validation.thresholds]] | `fixture_thresholds()` | `Acceptance thresholds SHALL be set before data collection; observed results SHALL not redefine passing thresholds after the fact.` |
| failure_holds | unit | [[plan.validation.failure]] | `fixture_failure()` | `Every failed case SHALL retain source input, model revision, proposed commands, solver diagnostics and reproduction instructions.` |
| reuse_holds | unit | [[plan.validation.reuse]] | `fixture_reuse()` | `ACSet generic core versus conventional graph SHALL use the same fixtures and record correctness, runtime, memory and maintainability evidence.` |
