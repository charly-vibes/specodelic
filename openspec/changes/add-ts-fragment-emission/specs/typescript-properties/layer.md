---
id: typescript.properties.layer
kind: intent
statement: "THE toolchain SHALL execute TypeScript unit properties using the shared generator and evidence contracts."
---

## Constraints

| id | kind | expr | traces_to |
|----|------|------|-----------|
| process_lifecycle | invariant | `the capability advances through its declared lifecycle states under the repo's change process — each stage transition fires only when its stage gate holds` | [[typescript.properties.layer]] |
| domains | invariant | TypeScript strategies preserve the established typed generator domains | [[typescript.properties.layer]] |
| complete_results | invariant | installed project tools report every expected block without downloads | [[typescript.properties.layer]] |
| bounded_shrinking | invariant | execution is bounded and counterexample shrinking is honestly attributed | [[typescript.properties.layer]] |

## Model

### States
- proposed
- approved
- implemented
- deployed

### Transitions

| id | from | to | guard |
|----|------|----|-------|
| approve | proposed | approved | [[typescript.properties.layer.process_lifecycle]] ∧ maintainer approves the proposal  |
| implement | approved | implemented | [[typescript.properties.layer.process_lifecycle]] ∧ every specified behavior has passing evidence  |
| deploy | implemented | deployed | [[typescript.properties.layer.process_lifecycle]] ∧ validation passes and the dual-format archive preserves all requirements  |

## Properties

| id | kind | derives_from | generator | predicate |
|----|------|--------------|-----------|-----------|
| process_lifecycle_checked | unit | [[typescript.properties.layer.process_lifecycle]] | `lifecycle_model_present()` | `check(file) == passed` |
| domains_checked | unit | [[typescript.properties.layer.domains]] | cross_language_domain_fixtures() | bounds and Unicode scalar semantics agree |
| complete_results_checked | unit | [[typescript.properties.layer.complete_results]] | missing_tool_or_partial_result() | missing and partial execution fail labeled |
| bounded_shrinking_checked | unit | [[typescript.properties.layer.bounded_shrinking]] | typescript_timeout_and_failure() | timeout blocks and unshrunk examples are labeled |

