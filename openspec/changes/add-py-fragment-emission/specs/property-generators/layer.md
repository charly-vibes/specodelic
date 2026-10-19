---
id: property.generators.layer
kind: intent
statement: "THE compiler SHALL generate typed bounded values from explicitly declared generator data."
---

## Constraints

| id | kind | expr | traces_to |
|----|------|------|-----------|
| process_lifecycle | invariant | `the capability advances through its declared lifecycle states under the repo's change process — each stage transition fires only when its stage gate holds` | [[property.generators.layer]] |
| typed_values | invariant | explicit generator grammar maps to the same value domains in each emitter | [[property.generators.layer]] |
| registry_freshness | invariant | named definitions are explicit data and their changes invalidate evidence | [[property.generators.layer]] |
| adequate_inputs | invariant | legacy constant placeholders cannot satisfy the property gate | [[property.generators.layer]] |

## Model

### States
- proposed
- approved
- implemented
- deployed

### Transitions

| id | from | to | guard |
|----|------|----|-------|
| approve | proposed | approved | [[property.generators.layer.process_lifecycle]] ∧ maintainer approves the proposal  |
| implement | approved | implemented | [[property.generators.layer.process_lifecycle]] ∧ every specified behavior has passing evidence  |
| deploy | implemented | deployed | [[property.generators.layer.process_lifecycle]] ∧ validation passes and the dual-format archive preserves all requirements  |

## Properties

| id | kind | derives_from | generator | predicate |
|----|------|--------------|-----------|-----------|
| process_lifecycle_checked | unit | [[property.generators.layer.process_lifecycle]] | `lifecycle_model_present()` | `check(file) == passed` |
| typed_values_checked | unit | [[property.generators.layer.typed_values]] | generator_domain_fixtures() | integer and collection boundaries agree across Rust and Python |
| registry_freshness_checked | unit | [[property.generators.layer.registry_freshness]] | registry_missing_cycle_or_edit() | bad definitions fail and edited definitions make reports stale |
| adequate_inputs_checked | unit | [[property.generators.layer.adequate_inputs]] | legacy_singleton_and_parameterless_cases() | only explicit singleton or parameterless inputs can pass |

