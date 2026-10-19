---
id: python.properties.layer
kind: intent
statement: "THE toolchain SHALL execute Python unit properties with complete attributable results."
---

## Constraints

| id | kind | expr | traces_to |
|----|------|------|-----------|
| process_lifecycle | invariant | `the capability advances through its declared lifecycle states under the repo's change process — each stage transition fires only when its stage gate holds` | [[python.properties.layer]] |
| python_execution | invariant | Python unit predicates compile into actual executed tests | [[python.properties.layer]] |
| all_blocks | invariant | every language block contributes to the final gate | [[python.properties.layer]] |
| runner_honesty | invariant | timeouts and missing dependencies fail labeled and shrinking is accurately reported | [[python.properties.layer]] |
| agreement | invariant | Python and Rust agree over the shared finite kernel fixtures | [[python.properties.layer]] |

## Model

### States
- proposed
- approved
- implemented
- deployed

### Transitions

| id | from | to | guard |
|----|------|----|-------|
| approve | proposed | approved | [[python.properties.layer.process_lifecycle]] ∧ maintainer approves the proposal  |
| implement | approved | implemented | [[python.properties.layer.process_lifecycle]] ∧ every specified behavior has passing evidence  |
| deploy | implemented | deployed | [[python.properties.layer.process_lifecycle]] ∧ validation passes and the dual-format archive preserves all requirements  |

## Properties

| id | kind | derives_from | generator | predicate |
|----|------|--------------|-----------|-----------|
| process_lifecycle_checked | unit | [[python.properties.layer.process_lifecycle]] | `lifecycle_model_present()` | `check(file) == passed` |
| python_execution_checked | unit | [[python.properties.layer.python_execution]] | python_unit_fixture() | expected property IDs execute and return pass or failure |
| all_blocks_checked | unit | [[python.properties.layer.all_blocks]] | mixed_language_partial_failure() | no missing or failing block can be hidden by other passes |
| runner_honesty_checked | unit | [[python.properties.layer.runner_honesty]] | runner_failure_fixtures() | timeouts block and unshrunk cases are labeled |
| agreement_checked | unit | [[python.properties.layer.agreement]] | shared_kernel_and_workflow_fixtures() | statuses agree and the broken application example fails its bound test |

