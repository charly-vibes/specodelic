# Python properties

## Purpose
THE toolchain SHALL execute Python unit properties with complete attributable results.

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
