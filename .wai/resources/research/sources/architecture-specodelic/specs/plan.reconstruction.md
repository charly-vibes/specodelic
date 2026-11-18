---
id: plan.reconstruction
kind: intent
statement: "THE hybrid reconstruction and human verification capability SHALL satisfy typed structural consistency, provenance, reversible human correction and independently testable behavior as defined below."
---

# Hybrid reconstruction and human verification

Fuse photographs, scans, sketch interpretation and selective physical measurements without inventing unavailable certainty.

## Constraints

| id | kind | expr | traces_to |
|---|---|---|---|
| modes | invariant | `The workflow SHALL operate with manual drawing alone; optical reconstruction, LiDAR and learning backends SHALL be optional.` | [[plan.reconstruction]] |
| proposals | invariant | `Photogrammetry, plane detection and learned interpretation SHALL produce evidence-tagged proposals, not confirmed geometry.` | [[plan.reconstruction]] |
| coverage | invariant | `Insufficient image overlap, calibration, occlusion and featureless walls SHALL yield failure or limited-confidence outputs, never fabricated measurements.` | [[plan.reconstruction]] |
| measurement | invariant | `Where the model is underdetermined the system SHALL suggest a physical measurement whose expected information gain is stated, with a fallback manual choice.` | [[plan.reconstruction]] |
| conflicts | invariant | `Inconsistent sketch, photo and measured constraints SHALL be surfaced with competing explanations and editable decisions.` | [[plan.reconstruction]] |
| metrics | invariant | `Evaluation SHALL separately measure plan topology, wall/corner metric error, opening placement, calibration, human correction time and total task time.` | [[plan.reconstruction]] |
| baseline | invariant | `Compare against manual sketch tracing and manual dimensional input, not only against fully automatic reconstruction.` | [[plan.reconstruction]] |
| scope | invariant | `A claimed accuracy or certification target SHALL require independent ground truth, a declared protocol and reported error distribution.` | [[plan.reconstruction]] |

## Model

### States

- `unobserved`
- `proposed`
- `reconciled`
- `validated`

### Transitions

| id | from | to | guard |
|---|---|---|---|
| advance_proposed | unobserved | proposed | `preconditions for proposed hold` ∧ [[plan.reconstruction.modes]] |
| advance_reconciled | proposed | reconciled | `preconditions for reconciled hold` ∧ [[plan.reconstruction.modes]] |
| advance_validated | reconciled | validated | `preconditions for validated hold` ∧ [[plan.reconstruction.modes]] |

## Properties

| id | kind | derives_from | generator | predicate |
|---|---|---|---|---|
| modes_holds | unit | [[plan.reconstruction.modes]] | `fixture_modes()` | `The workflow SHALL operate with manual drawing alone; optical reconstruction, LiDAR and learning backends SHALL be optional.` |
| proposals_holds | unit | [[plan.reconstruction.proposals]] | `fixture_proposals()` | `Photogrammetry, plane detection and learned interpretation SHALL produce evidence-tagged proposals, not confirmed geometry.` |
| coverage_holds | unit | [[plan.reconstruction.coverage]] | `fixture_coverage()` | `Insufficient image overlap, calibration, occlusion and featureless walls SHALL yield failure or limited-confidence outputs, never fabricated measurements.` |
| measurement_holds | unit | [[plan.reconstruction.measurement]] | `fixture_measurement()` | `Where the model is underdetermined the system SHALL suggest a physical measurement whose expected information gain is stated, with a fallback manual choice.` |
| conflicts_holds | unit | [[plan.reconstruction.conflicts]] | `fixture_conflicts()` | `Inconsistent sketch, photo and measured constraints SHALL be surfaced with competing explanations and editable decisions.` |
| metrics_holds | unit | [[plan.reconstruction.metrics]] | `fixture_metrics()` | `Evaluation SHALL separately measure plan topology, wall/corner metric error, opening placement, calibration, human correction time and total task time.` |
| baseline_holds | unit | [[plan.reconstruction.baseline]] | `fixture_baseline()` | `Compare against manual sketch tracing and manual dimensional input, not only against fully automatic reconstruction.` |
| scope_holds | unit | [[plan.reconstruction.scope]] | `fixture_scope()` | `A claimed accuracy or certification target SHALL require independent ground truth, a declared protocol and reported error distribution.` |
