# Sketch-to-architecture — Specodelic specification bundle

Status: proposed, unimplemented. 9 intent specs authored in the verified Specodelic document-table dialect.

## Scope

- `acset.generic`: optional schema-parametric incidence engine; **does not change** the existing closed `acset.core` in Specodelic.
- `plan.observation`: raw ink, photo, sketch, measurements and provenance.
- `plan.interpretation`: ambiguous semantic proposals and user confirmation.
- `plan.topology`: vertices, boundaries, rooms, wall faces, openings.
- `plan.geometry`: metric and differential surface realization, constraints, uncertainty, observability.
- `plan.canvas`: touch-first reversible sketch interactions.
- `plan.views`: 2D/3D consistency and CAD export.
- `plan.reconstruction`: hybrid photo/sketch/manual workflow.
- `plan.validation`: independent benchmarks and baselines.

**Compatibility:** These files are standalone intent documents rather than a change to the existing five-object Specodelic schema. The generic ACSet engine is an experiment, not an assumed prerequisite. The specs favor ordinary human-readable predicates; these are NOT automatically executable formal properties. Validation results, if available, are recorded separately.

## Suggested integration

Copy `specs/*.md` to a dedicated project spec folder; do not replace `openspec/specs/acset-core/spec.md`. Cross-cutting integration with existing `vision.space` and `space.plans` must retain their measured-first and certification rules. Implement a four-wall-room prototype before generalizing ACSet machinery.

## Sources and provenance

Authored 2026-10-19 by OpenAI GPT-6 from user discussion and uploaded archives `specodelic-main(2)(1).zip` and `vision-platform-bundle.zip`. Reviewed source files include `src/acset/{schema,instance,query,writer}.rs`, `openspec/specs/acset-core/spec.md`, and the vision platform `space-plans.md`, `vision-space.md`. No unsupported claim is made that the specs have been executed as runtime implementations.
