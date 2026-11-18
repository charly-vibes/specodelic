---
id: plan.observation
kind: intent
statement: "THE evidence and uncertainty capability SHALL satisfy typed structural consistency, provenance, reversible human correction and independently testable behavior as defined below."
---

# Evidence and uncertainty

Store strokes, scanned images, camera observations and physical measurements without conflating them with interpreted architectural facts.

## Constraints

| id | kind | expr | traces_to |
|---|---|---|---|
| immutable | invariant | `Raw strokes, images and original measurement entries SHALL be immutable and content-addressable or revision-addressable.` | [[plan.observation]] |
| sources | invariant | `Every observation SHALL have a stable ID, input modality, timestamp when known, coordinate frame, transformation history and provenance.` | [[plan.observation]] |
| units | invariant | `Physical measurements SHALL record units, endpoint semantics, reference height, instrument, stated uncertainty and measured/derived/assumed classification.` | [[plan.observation]] |
| frames | invariant | `Pixel, canvas, world and camera coordinate frames SHALL be distinct, and conversions SHALL record calibration and uncertainty.` | [[plan.observation]] |
| confidence | invariant | `Recognition scores SHALL NOT be represented as calibrated probabilities without calibration evidence.` | [[plan.observation]] |
| preserve | invariant | `Rejecting an interpretation SHALL NOT delete or alter source observations.` | [[plan.observation]] |
| privacy | invariant | `Locally imported photos and sketches SHALL remain local unless an explicit export or consented transfer is initiated.` | [[plan.observation]] |

## Model

### States

- `captured`
- `annotated`
- `linked`

### Transitions

| id | from | to | guard |
|---|---|---|---|
| advance_annotated | captured | annotated | `preconditions for annotated hold` ∧ [[plan.observation.immutable]] |
| advance_linked | annotated | linked | `preconditions for linked hold` ∧ [[plan.observation.immutable]] |

## Properties

| id | kind | derives_from | generator | predicate |
|---|---|---|---|---|
| immutable_holds | unit | [[plan.observation.immutable]] | `fixture_immutable()` | `Raw strokes, images and original measurement entries SHALL be immutable and content-addressable or revision-addressable.` |
| sources_holds | unit | [[plan.observation.sources]] | `fixture_sources()` | `Every observation SHALL have a stable ID, input modality, timestamp when known, coordinate frame, transformation history and provenance.` |
| units_holds | unit | [[plan.observation.units]] | `fixture_units()` | `Physical measurements SHALL record units, endpoint semantics, reference height, instrument, stated uncertainty and measured/derived/assumed classification.` |
| frames_holds | unit | [[plan.observation.frames]] | `fixture_frames()` | `Pixel, canvas, world and camera coordinate frames SHALL be distinct, and conversions SHALL record calibration and uncertainty.` |
| confidence_holds | unit | [[plan.observation.confidence]] | `fixture_confidence()` | `Recognition scores SHALL NOT be represented as calibrated probabilities without calibration evidence.` |
| preserve_holds | unit | [[plan.observation.preserve]] | `fixture_preserve()` | `Rejecting an interpretation SHALL NOT delete or alter source observations.` |
| privacy_holds | unit | [[plan.observation.privacy]] | `fixture_privacy()` | `Locally imported photos and sketches SHALL remain local unless an explicit export or consented transfer is initiated.` |
