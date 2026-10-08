---
id: python.properties
kind: intent
statement: "THE toolchain SHALL execute Python unit properties with complete attributable results."
---

# Python properties

## Purpose
THE toolchain SHALL execute Python unit properties with complete attributable results.

## Constraints

| id | kind | expr | traces_to |
|----|------|------|-----------|
| process_lifecycle | invariant | `the capability advances through its declared lifecycle states under the repo's change process — each stage transition fires only when its stage gate holds` | [[python.properties]] |
| python_execution | invariant | Python unit predicates compile into actual executed tests | [[python.properties]] |
| all_blocks | invariant | every language block contributes to the final gate | [[python.properties]] |
| runner_honesty | invariant | timeouts and missing dependencies fail labeled and shrinking is accurately reported | [[python.properties]] |
| agreement | invariant | Python and Rust agree over the shared finite kernel fixtures | [[python.properties]] |

## Model

### States
- proposed
- approved
- implemented
- deployed

### Transitions

| id | from | to | guard |
|----|------|----|-------|
| approve | proposed | approved | [[python.properties.process_lifecycle]] ∧ maintainer approves the proposal  |
| implement | approved | implemented | [[python.properties.process_lifecycle]] ∧ every specified behavior has passing evidence  |
| deploy | implemented | deployed | [[python.properties.process_lifecycle]] ∧ validation passes and the dual-format archive preserves all requirements  |

## Properties

| id | kind | derives_from | generator | predicate |
|----|------|--------------|-----------|-----------|
| process_lifecycle_checked | unit | [[python.properties.process_lifecycle]] | `lifecycle_model_present()` | `check(file) == passed` |
| python_execution_checked | unit | [[python.properties.python_execution]] | python_unit_fixture() | expected property IDs execute and return pass or failure |
| all_blocks_checked | unit | [[python.properties.all_blocks]] | mixed_language_partial_failure() | no missing or failing block can be hidden by other passes |
| runner_honesty_checked | unit | [[python.properties.runner_honesty]] | runner_failure_fixtures() | timeouts block and unshrunk cases are labeled |
| agreement_checked | unit | [[python.properties.agreement]] | shared_kernel_and_workflow_fixtures() | statuses agree and the broken application example fails its bound test |

## ADDED Requirements

### Requirement: Python unit properties compile and execute
Property predicates marked **py:** SHALL compile to deterministic pytest and
Hypothesis artifacts and execute through a PropertiesRunner adapter.
Each unit row SHALL retain its qualified identity and generated inputs.
Python invariant fragments and executable law fragments SHALL remain labeled
unsupported positions. Unknown language tags SHALL remain rejected.

#### Scenario: A Python predicate runs over generated values
- **WHEN** a unit Property uses a py predicate and a marked integer generator
- **THEN** compile emits a Python test and verify reports that exact property's executed result

### Requirement: Every expected language block contributes
A manifest SHALL enumerate every expected property block across Rust and
Python artifacts. Verify SHALL execute every block exactly once for result
accounting and SHALL block properties_pass on a missing, failed, skipped,
duplicate, stale, or inadequate block, irrespective of other languages' results.

#### Scenario: One language cannot hide another language failure
- **WHEN** Rust blocks pass but one Python property fails or is skipped
- **THEN** verify exits 1 and identifies the Python property blocker

### Requirement: Runner failures and shrinking are honest
The Python adapter SHALL use an explicit executable and bounded subprocess
execution, SHALL detect missing pytest or Hypothesis without installing them,
and SHALL clean up its process tree on timeout. Failing generated properties
SHALL report a native-shrunk counterexample or explicitly mark it unshrunk.

#### Scenario: Missing dependencies are actionable
- **WHEN** the selected Python lacks a required runner dependency
- **THEN** verify fails labeled with installation guidance and does not install packages

#### Scenario: Timeout and unfinished shrinking are not clean
- **WHEN** execution times out or shrinking cannot finish
- **THEN** timeout blocks verification and any retained unshrunk example is labeled accordingly

### Requirement: Kernel agreement and workflow scope are demonstrated
The Python reference evaluator SHALL run the same finite kernel fixtures as
the Rust evaluator in mandatory CI with identical expected statuses. The
worked workflow SHALL distinguish generated-property evidence from external
application-test binding and demonstrate a failing implementation test.

#### Scenario: Python activation enables agreement checks
- **WHEN** this emitter change lands with the shared kernel fixtures
- **THEN** CI executes both evaluators without silently skipping Python and their per-claim statuses agree

#### Scenario: A broken application behavior is visible
- **WHEN** the walkthrough's application mutation is enabled
- **THEN** its bound application test fails and the walkthrough identifies that stage without calling spec-only checks an application proof

## Requirements

### Requirement: Python unit properties compile and execute
Property predicates marked **py:** SHALL compile to deterministic pytest and
Hypothesis artifacts and execute through a PropertiesRunner adapter.
Each unit row SHALL retain its qualified identity and generated inputs.
Python invariant fragments and executable law fragments SHALL remain labeled
unsupported positions. Unknown language tags SHALL remain rejected.

#### Scenario: A Python predicate runs over generated values
- **WHEN** a unit Property uses a py predicate and a marked integer generator
- **THEN** compile emits a Python test and verify reports that exact property's executed result

### Requirement: Every expected language block contributes
A manifest SHALL enumerate every expected property block across Rust and
Python artifacts. Verify SHALL execute every block exactly once for result
accounting and SHALL block properties_pass on a missing, failed, skipped,
duplicate, stale, or inadequate block, irrespective of other languages' results.

#### Scenario: One language cannot hide another language failure
- **WHEN** Rust blocks pass but one Python property fails or is skipped
- **THEN** verify exits 1 and identifies the Python property blocker

### Requirement: Runner failures and shrinking are honest
The Python adapter SHALL use an explicit executable and bounded subprocess
execution, SHALL detect missing pytest or Hypothesis without installing them,
and SHALL clean up its process tree on timeout. Failing generated properties
SHALL report a native-shrunk counterexample or explicitly mark it unshrunk.

#### Scenario: Missing dependencies are actionable
- **WHEN** the selected Python lacks a required runner dependency
- **THEN** verify fails labeled with installation guidance and does not install packages

#### Scenario: Timeout and unfinished shrinking are not clean
- **WHEN** execution times out or shrinking cannot finish
- **THEN** timeout blocks verification and any retained unshrunk example is labeled accordingly

### Requirement: Kernel agreement and workflow scope are demonstrated
The Python reference evaluator SHALL run the same finite kernel fixtures as
the Rust evaluator in mandatory CI with identical expected statuses. The
worked workflow SHALL distinguish generated-property evidence from external
application-test binding and demonstrate a failing implementation test.

#### Scenario: Python activation enables agreement checks
- **WHEN** this emitter change lands with the shared kernel fixtures
- **THEN** CI executes both evaluators without silently skipping Python and their per-claim statuses agree

#### Scenario: A broken application behavior is visible
- **WHEN** the walkthrough's application mutation is enabled
- **THEN** its bound application test fails and the walkthrough identifies that stage without calling spec-only checks an application proof
