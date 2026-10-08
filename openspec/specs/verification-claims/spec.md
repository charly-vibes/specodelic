---
id: spec
kind: intent
statement: "THE verifier SHALL accept only complete current evidence for every required invariant claim."
---

# Verification claims

## Purpose
THE verifier SHALL accept only complete current evidence for every required invariant claim.

## Constraints

| id | kind | expr | traces_to |
|----|------|------|-----------|
| required_set | invariant | required claims come from explicit executable or kernel or citation invariant opt-ins | [[spec]] |
| aggregate | invariant | nonempty required claims must all verify and property blocks must all pass | [[spec]] |
| freshness | invariant | reports bind a versioned claim set to current structured scope and artifacts | [[spec]] |
| visible_scope | invariant | all report views and release docs state the same assurance scope | [[spec]] |

## Model

### States
- proposed
- approved
- implemented
- deployed

### Transitions

| id | from | to | guard |
|----|------|----|-------|
| approve | proposed | approved | maintainer approves this proposal |
| implement | approved | implemented | every constraint has passing behavioral evidence |
| deploy | implemented | deployed | required gates pass and dual-format archive preserves every requirement |

## Properties

| id | kind | derives_from | generator | predicate |
|----|------|--------------|-----------|-----------|
| required_set_checked | unit | [[spec.required_set]] | mixed_claim_corpus() | required and unchecked sets equal the expected IDs |
| aggregate_checked | unit | [[spec.aggregate]] | mixed_claim_statuses() | false and unknown claims cannot yield verified |
| freshness_checked | unit | [[spec.freshness]] | changed_scope_or_legacy_report() | verify rejects stale or incomplete evidence with a rerun hint |
| visible_scope_checked | unit | [[spec.visible_scope]] | report_and_docs_fixture() | claim counts agree and pending features are not advertised as implemented |

## ADDED Requirements

### Requirement: Explicit required claim set
The toolchain SHALL derive the required claim set from invariant Constraints
opted into Rust, kernel, or citation evaluation. Prose-only invariant claims
SHALL be listed as unchecked and SHALL NOT be counted as verified.

#### Scenario: Prose is not executable evidence
- **WHEN** a file contains only prose invariant Constraints
- **THEN** its required set is empty, its unchecked set lists those Constraints, and verify cannot report verified

### Requirement: All required claims govern acceptance
A nonempty complete required set SHALL contain only verified claims before
model-check reports no_counterexample. A refuted claim SHALL yield
counterexample_found; absent refutation, exhausted bounds SHALL yield
timed_out; otherwise unknown, unsupported, or missing evidence SHALL prevent
clean acceptance. Verify SHALL also require every generated property block
to complete and pass, and SHALL reject invalid or duplicate claim records.

#### Scenario: Passing Rust does not hide a false kernel invariant
- **WHEN** one required Rust claim is verified and a kernel claim is counterexample
- **THEN** the aggregate is counterexample_found and verify exits 1 with the false claim ID

#### Scenario: Unknown and false citations block success
- **WHEN** a required citation is unknown or counterexample alongside a passing executable claim
- **THEN** verify exits 1 with the citation ID and reason, never verified

#### Scenario: Complete evidence passes both gates
- **WHEN** every required claim is verified within the bound and every generated property block passes
- **THEN** verify reports verified with the exact required and unchecked claim sets

### Requirement: Evidence is versioned and scope bound
A stored report SHALL declare claim schema version 1, canonical expected
claim IDs, actual statuses, and a deterministic digest of the structured
corpus and consumed artifacts. Verify SHALL recompute scope and expected
claims from current inputs. Unsupported or old reports, omitted or duplicate
records, changed contributing inputs, or artifact mismatch SHALL fail with a
rerun hint. Reordering CLI paths or changing prose alone SHALL preserve scope.
Ordinary corpus intent IDs SHALL be unique. A dual-format id: spec file
SHALL be the sole parsed input to model-check, verify or orchestrate;
combined input SHALL fail with isolated_scope_required and a hint to run
each file separately with separate artifact/report directories, before
compilation, evaluation or report writes. In that isolated scope, spec.row
claim IDs and references SHALL resolve only locally. Multi-file dual-format
lint SHALL remain supported. The digest SHALL bind structured content,
not filesystem paths, so opposite claims with identical names cannot
reuse each other's evidence.

#### Scenario: Cross-file changes invalidate a clean report
- **WHEN** a referenced file changes structurally or is omitted from the supplied corpus after the report was produced
- **THEN** verify rejects the report as stale or scope-mismatched and names the rerun command

#### Scenario: Dual-format commands isolate file-local identities
- **WHEN** two dual-format files share id: spec and local claim c1 but assert different invariant expressions
- **THEN** combined command evaluation fails as isolated_scope_required; independent runs in separate directories retain each claim's own status, produce different scope digests and reject swapped reports, while lint still accepts their file-local identities

#### Scenario: A dual-format file cannot join an ordinary corpus
- **WHEN** model-check, verify or orchestrate receives one id: spec file plus another parsed file
- **THEN** it refuses the combined scope before writing artifacts or reports and gives separate-invocation guidance

#### Scenario: Old reports cannot acquire new evidence implicitly
- **WHEN** a stored report has no claim schema version or required records
- **THEN** verify rejects it with a model-check rerun hint without modifying the report

### Requirement: Assurance scope is visible
CLI JSON, human output, and persisted reports SHALL agree on evaluated,
unchecked, and blocking claims. Release documentation SHALL distinguish
structural lint, bounded model exploration, verified declared claims, and
external application-test binding, and SHALL identify pending capabilities
without presenting them as implemented.

#### Scenario: Report views agree
- **WHEN** a run contains a required unknown claim and a prose-only invariant
- **THEN** JSON, human output, and the saved report identify the same blocker and unchecked claim

#### Scenario: Capability and release metadata stay current
- **WHEN** the documentation is built for a release
- **THEN** its version derives from Cargo metadata and its examples distinguish implemented behavior from pending proposals

## Requirements

### Requirement: Explicit required claim set
The toolchain SHALL derive the required claim set from invariant Constraints
opted into Rust, kernel, or citation evaluation. Prose-only invariant claims
SHALL be listed as unchecked and SHALL NOT be counted as verified.

#### Scenario: Prose is not executable evidence
- **WHEN** a file contains only prose invariant Constraints
- **THEN** its required set is empty, its unchecked set lists those Constraints, and verify cannot report verified

### Requirement: All required claims govern acceptance
A nonempty complete required set SHALL contain only verified claims before
model-check reports no_counterexample. A refuted claim SHALL yield
counterexample_found; absent refutation, exhausted bounds SHALL yield
timed_out; otherwise unknown, unsupported, or missing evidence SHALL prevent
clean acceptance. Verify SHALL also require every generated property block
to complete and pass, and SHALL reject invalid or duplicate claim records.

#### Scenario: Passing Rust does not hide a false kernel invariant
- **WHEN** one required Rust claim is verified and a kernel claim is counterexample
- **THEN** the aggregate is counterexample_found and verify exits 1 with the false claim ID

#### Scenario: Unknown and false citations block success
- **WHEN** a required citation is unknown or counterexample alongside a passing executable claim
- **THEN** verify exits 1 with the citation ID and reason, never verified

#### Scenario: Complete evidence passes both gates
- **WHEN** every required claim is verified within the bound and every generated property block passes
- **THEN** verify reports verified with the exact required and unchecked claim sets

### Requirement: Evidence is versioned and scope bound
A stored report SHALL declare claim schema version 1, canonical expected
claim IDs, actual statuses, and a deterministic digest of the structured
corpus and consumed artifacts. Verify SHALL recompute scope and expected
claims from current inputs. Unsupported or old reports, omitted or duplicate
records, changed contributing inputs, or artifact mismatch SHALL fail with a
rerun hint. Reordering CLI paths or changing prose alone SHALL preserve scope.
Ordinary corpus intent IDs SHALL be unique. A dual-format id: spec file
SHALL be the sole parsed input to model-check, verify or orchestrate;
combined input SHALL fail with isolated_scope_required and a hint to run
each file separately with separate artifact/report directories, before
compilation, evaluation or report writes. In that isolated scope, spec.row
claim IDs and references SHALL resolve only locally. Multi-file dual-format
lint SHALL remain supported. The digest SHALL bind structured content,
not filesystem paths, so opposite claims with identical names cannot
reuse each other's evidence.

#### Scenario: Cross-file changes invalidate a clean report
- **WHEN** a referenced file changes structurally or is omitted from the supplied corpus after the report was produced
- **THEN** verify rejects the report as stale or scope-mismatched and names the rerun command

#### Scenario: Dual-format commands isolate file-local identities
- **WHEN** two dual-format files share id: spec and local claim c1 but assert different invariant expressions
- **THEN** combined command evaluation fails as isolated_scope_required; independent runs in separate directories retain each claim's own status, produce different scope digests and reject swapped reports, while lint still accepts their file-local identities

#### Scenario: A dual-format file cannot join an ordinary corpus
- **WHEN** model-check, verify or orchestrate receives one id: spec file plus another parsed file
- **THEN** it refuses the combined scope before writing artifacts or reports and gives separate-invocation guidance

#### Scenario: Old reports cannot acquire new evidence implicitly
- **WHEN** a stored report has no claim schema version or required records
- **THEN** verify rejects it with a model-check rerun hint without modifying the report

### Requirement: Assurance scope is visible
CLI JSON, human output, and persisted reports SHALL agree on evaluated,
unchecked, and blocking claims. Release documentation SHALL distinguish
structural lint, bounded model exploration, verified declared claims, and
external application-test binding, and SHALL identify pending capabilities
without presenting them as implemented.

#### Scenario: Report views agree
- **WHEN** a run contains a required unknown claim and a prose-only invariant
- **THEN** JSON, human output, and the saved report identify the same blocker and unchecked claim

#### Scenario: Capability and release metadata stay current
- **WHEN** the documentation is built for a release
- **THEN** its version derives from Cargo metadata and its examples distinguish implemented behavior from pending proposals
