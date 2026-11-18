---
id: plan.interpretation
kind: intent
statement: "THE sketch-to-architecture interpretation capability SHALL satisfy typed structural consistency, provenance, reversible human correction and independently testable behavior as defined below."
---

# Sketch-to-architecture interpretation

Represent uncertainty explicitly when converting ink or photo features into candidate architectural entities.

## Constraints

| id | kind | expr | traces_to |
|---|---|---|---|
| alternatives | invariant | `A stroke or image feature MAY support zero, one or multiple typed hypotheses; candidates SHALL be distinguishable from confirmed entities.` | [[plan.interpretation]] |
| no_automatic | invariant | `Raw ink SHALL never become a confirmed wall solely from an inference score.` | [[plan.interpretation]] |
| reversible | invariant | `Candidate acceptance, rejection and correction SHALL be undoable while source ink remains unchanged.` | [[plan.interpretation]] |
| trace | invariant | `Every confirmed entity derived from an observation SHALL retain links to its source evidence and interpreting action.` | [[plan.interpretation]] |
| ambiguity | invariant | `Conflicting interpretations SHALL be surfaced rather than silently merged.` | [[plan.interpretation]] |
| scans | invariant | `Scanned sketch extraction SHALL preserve paper image, rectification transform and recognized annotation uncertainty.` | [[plan.interpretation]] |
| no_metric_from_ink | invariant | `Pixel lengths and freehand dimensions SHALL not be promoted to metric measurements absent a verified scale anchor.` | [[plan.interpretation]] |

## Model

### States

- `raw`
- `proposed`
- `confirmed`
- `rejected`

### Transitions

| id | from | to | guard |
|---|---|---|---|
| advance_proposed | raw | proposed | `preconditions for proposed hold` ∧ [[plan.interpretation.alternatives]] |
| advance_confirmed | proposed | confirmed | `preconditions for confirmed hold` ∧ [[plan.interpretation.alternatives]] |
| advance_rejected | confirmed | rejected | `preconditions for rejected hold` ∧ [[plan.interpretation.alternatives]] |

## Properties

| id | kind | derives_from | generator | predicate |
|---|---|---|---|---|
| alternatives_holds | unit | [[plan.interpretation.alternatives]] | `fixture_alternatives()` | `A stroke or image feature MAY support zero, one or multiple typed hypotheses; candidates SHALL be distinguishable from confirmed entities.` |
| no_automatic_holds | unit | [[plan.interpretation.no_automatic]] | `fixture_no_automatic()` | `Raw ink SHALL never become a confirmed wall solely from an inference score.` |
| reversible_holds | unit | [[plan.interpretation.reversible]] | `fixture_reversible()` | `Candidate acceptance, rejection and correction SHALL be undoable while source ink remains unchanged.` |
| trace_holds | unit | [[plan.interpretation.trace]] | `fixture_trace()` | `Every confirmed entity derived from an observation SHALL retain links to its source evidence and interpreting action.` |
| ambiguity_holds | unit | [[plan.interpretation.ambiguity]] | `fixture_ambiguity()` | `Conflicting interpretations SHALL be surfaced rather than silently merged.` |
| scans_holds | unit | [[plan.interpretation.scans]] | `fixture_scans()` | `Scanned sketch extraction SHALL preserve paper image, rectification transform and recognized annotation uncertainty.` |
| no_metric_from_ink_holds | unit | [[plan.interpretation.no_metric_from_ink]] | `fixture_no_metric_from_ink()` | `Pixel lengths and freehand dimensions SHALL not be promoted to metric measurements absent a verified scale anchor.` |
