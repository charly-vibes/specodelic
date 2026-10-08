# model-check Specification

## Purpose
Upgrade guard citations from inert TLA+ annotations to kernel-evaluated
claims — slice 1, the one-green-where-today-is-red increment — while
keeping the fragment_guard_rejected decision of record intact
(executable guard fragments stay rejected; the deferral's
data-carrying-state-space condition is not met by instance-grounded
expressions). Run reports persist backend, bound, and per-invariant
three-valued status, with the backend-agreement property running as a
CI property test over shared fixtures until the py backend lands.

## Constraints

| id                        | kind      | expr                                                                                                                                                                                                                                                                        | traces_to |
|---------------------------|-----------|-------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|-----------|
| guard_citation_evaluated  | invariant | `an invariant-kind Constraint's guard citations resolve against same-run invariant outcomes and compose under the citation algebra — conjunction and negation with an explicit three-valued status — the evaluated citation is an annotation upgrade and never a sole verdict unless executed` | [[model.check]]  |
| fragment_guard_rejected_stands | invariant | `a Transition's guard cell carrying a rust: fragment still fails labeled — the deferral of record (deferred to a future Revision alongside a data-carrying state space) is not met by instance-grounded expressions, so the decision of record stands unchanged`                  | [[model.check]]  |
| run_report_status         | invariant | `a model-check run persists backend, bound, and per-invariant status — verified, counterexample, or unknown — unknown is persisted honestly and never coerced, and a run whose invariants were not executed remains exploration_only and explicitly not clean`                      | [[model.check]]  |
| backend_agreement_ci      | invariant | `the same fixture corpus evaluates through every deployed backend with identical three-valued status per cell, asserted by a CI property test — the rust-only interim asserts rust status equals the fixture's expected status, and the cross-backend assertion activates with the py backend without being disabled` | [[model.check]]  |

| command_claims | invariant | every accepted kernel or citation invariant has one canonical status in CLI output and persisted reports over the explicit corpus | [[model.check]] |

## Model

### States
- `citation_inert`
- `citation_evaluated`
- `agreement_active`

### Transitions

| id        | from              | to               | guard                                                                              |
|-----------|-------------------|------------------|------------------------------------------------------------------------------------|
| upgrade   | citation_inert    | citation_evaluated | `citation algebra lands per [[model.check.guard_citation_evaluated]]: citations resolve and compose with three-valued status`  |
| activate  | citation_evaluated | agreement_active | `the py backend lands (l8l): the cross-backend assertion activates per [[model.check.backend_agreement_ci]]`      |

## Properties

| id                        | kind | derives_from                              | generator                              | predicate                                                                  |
|---------------------------|------|-------------------------------------------|----------------------------------------|-----------------------------------------------------------------------------|
| citation_composes         | unit | [[model.check.guard_citation_evaluated]]         | `artifact_with_composite_citation()`   | `citation resolves and composes with an explicit three-valued status`        |
| undischargable_is_unknown | unit | [[model.check.guard_citation_evaluated]]         | `artifact_with_undischargable_citation()` | `citation reports unknown, never pass`                                   |
| guard_fragment_still_rejected | unit | [[model.check.fragment_guard_rejected_stands]] | `transition_row_with_fragment_guard()` | `compile fails labeled (fragment_extraction stage), decision of record intact` |
| report_names_status       | unit | [[model.check.run_report_status]]                | `run_with_mixed_invariant_status()`    | `report persists backend, bound, and per-invariant status with unknown honest` |
| fixture_status_agrees     | unit | [[model.check.backend_agreement_ci]]             | `shared_fixture_corpus()`              | `every deployed backend yields identical status per cell on shared fixtures` |

| command_claims_checked | unit | [[model.check.command_claims]] | mixed_kernel_and_citation_cli_corpus() | reported claim IDs and statuses equal expected values in CLI and persisted JSON |

## ADDED Requirements

### Requirement: Guard citation semantics evaluated
Guard citations in invariant-kind Constraints SHALL resolve against
same-run invariant outcomes and compose under the citation algebra (conjunction
and negation) with an explicit three-valued status; the evaluated
citation remains an annotation upgrade and never a sole verdict unless
executed. Executable guard fragments SHALL remain rejected of record —
the `fragment_guard_rejected` deferral's condition (a data-carrying
state space) is not met by instance-grounded expressions.

#### Scenario: Citation algebra evaluates
- **WHEN** an artifact's invariant Constraint cites `[[a]] ∧ [[b]]` or a negated citation
- **THEN** the citation resolves against same-run invariant outcomes and composes with an explicit three-valued status

#### Scenario: Undischargable citation is honest
- **WHEN** a citation cannot be discharged over the artifact's instances
- **THEN** it reports unknown, never pass

#### Scenario: Executable guard fragments stay rejected
- **WHEN** a Transition's guard cell carries a `rust:` fragment
- **THEN** compile fails labeled (fragment_extraction stage) — the decision of record stands unchanged

### Requirement: Backend agreement property
The same fixture corpus SHALL evaluate through every deployed backend
with identical three-valued status per cell, asserted by a CI property
test: the rust-only interim asserts rust status equals the fixture's
expected status, and the cross-backend assertion activates with the py
backend without being disabled or commented out.

#### Scenario: Rust-only interim holds
- **WHEN** the shared fixture corpus evaluates through the rust backend
- **THEN** every cell's status equals the fixture's expected status

#### Scenario: Cross-backend agreement activates with the py backend
- **WHEN** the py backend lands (l8l) and the shared fixtures evaluate through both backends
- **THEN** the CI property asserts identical status per cell across backends

### Requirement: Status-bearing run reports
A model-check run SHALL persist backend, bound, and per-invariant
status — `verified`, `counterexample`, or `unknown` — with `unknown`
persisted honestly and never coerced; a run whose invariants were not
executed remains `exploration_only` and is explicitly not clean.

#### Scenario: Report names its status
- **WHEN** a run evaluates invariants with mixed outcomes
- **THEN** the report persists backend, bound, and per-invariant status, including honest unknown

#### Scenario: Unexecuted invariants are not clean
- **WHEN** a run compiles invariants but executes none
- **THEN** the outcome is exploration_only and explicitly not clean

### Requirement: Complete command-path claim evaluation
The model-check command and orchestrator SHALL evaluate every accepted
kernel invariant over the complete explicit input corpus and SHALL publish
one status per opted-in kernel or citation invariant in both CLI output
and persisted reports. Qualified citations SHALL resolve exact invariant
IDs; bare citations SHALL resolve within the citing file. Citation chains
SHALL resolve in dependency order. Cycles, missing targets, and property
rows without same-run evidence SHALL report unknown with a reason.
An incapable backend SHALL report labeled unsupported claims, never omit
them. This requirement supplies statuses; aggregate acceptance is specified
by the separate verification-claims capability.
Ordinary corpus intent IDs SHALL be unique; duplicates SHALL fail as
duplicate_corpus_identity. Dual-format id: spec files SHALL retain local
identity and SHALL run only as the sole parsed input to model-check,
verify or orchestrate. Combined scopes SHALL fail as isolated_scope_required
before compilation, evaluation or report writes, with a hint to run each
file separately using separate artifact/report directories. Single-file
spec.row citations SHALL resolve locally; multi-file dual-format lint SHALL
remain valid. Report freshness follows verification-claims.

#### Scenario: File-local identities do not merge into a corpus
- **WHEN** two dual-format files both define spec.c1 with opposite invariant outcomes, or one dual-format file is combined with an ordinary file
- **THEN** combined command evaluation refuses the scope before writes; the two dual-format files run independently with their own outcomes and multi-file lint retains file-local identity support

#### Scenario: Mixed inputs cannot hide a false kernel claim
- **WHEN** a two-state corpus contains a kernel claim asserting 999 states and a passing Rust invariant
- **THEN** both claim IDs appear in CLI and persisted output and the kernel claim is counterexample

#### Scenario: Qualified and bare citations agree locally
- **WHEN** a local passing invariant is cited by its local name and its qualified name
- **THEN** both citations evaluate to verified, without confusing an identically named invariant in another file

#### Scenario: Scope and dependency order are explicit
- **WHEN** the same multi-file corpus with a citation chain is supplied in either file order
- **THEN** the same statuses result from the full corpus and a cycle or absent target remains unknown with a reason

#### Scenario: Property execution is not invented
- **WHEN** a citation targets a Property row without same-run invariant evidence
- **THEN** its status is unknown and model-check does not invoke a property runner

### Requirement: Model check runs a real backend against the compiled artifact
The system SHALL provide a `spk model-check <files>` step that runs a
model-check backend — stateright (embedded, default) — against the
compiled model artifact of each file, without re-compiling, and SHALL
report the outcome over the envelope (`data.outcome`), where a missing
compiled artifact is a labeled ERROR with a `spk compile` hint, never a
silent success.

#### Scenario: Clean spec within bound
- **WHEN** `spk model-check <file> --json` runs on a linted, covered,
  compiled spec whose model explores exhaustively within the stated
  bound, at least one executable invariant is checked, and none is violated
- **THEN** the envelope reports `.data.outcome == "no_counterexample"`
  with the backend engine and version and the stated bound restated

#### Scenario: Missing compiled artifact is an error
- **WHEN** `spk model-check <file>` runs on a spec with no compiled
  artifact present
- **THEN** the command fails with a labeled error naming the missing
  artifact and a `spk compile` remediation hint, and never reports
  `no_counterexample`

### Requirement: The run names its backend, bound, and checked invariants
The system SHALL include in every run report the backend engine and
version, the applied bound (max depth, and optional state count and time
budget), and the set of invariants actually checked — so a report where
no user invariant was executable states `invariants_checked` as empty
rather than implying a semantic check that never ran.

#### Scenario: Prose invariants are not fabricated
- **WHEN** a spec's Constraints table contains invariant rows whose
  expressions are prose with no executable opt-in
- **THEN** a native-backend run reports `exploration_only` after
  exhaustive automaton exploration within the bound, with
  `invariants_checked` empty — never a fabricated counterexample or an
  implied semantic check

#### Scenario: Budget exhaustion is a distinct outcome
- **WHEN** a run's state/time budget expires before exhaustive
  exploration completes
- **THEN** the report outcome is `timed_out` — never collapsed into
  `no_counterexample` or `counterexample_found`

### Requirement: Run reports persist with artifact provenance
The system SHALL write each run's report as `<stem>.check.json` next to
the compiled artifacts, carrying the SHA-256 of the compiled `.tla`
module the run consumed, so downstream `verify` can determine whether a
stored clean result still applies to the current compiled artifact.

#### Scenario: Stale clean result is detectable
- **WHEN** a stored clean report's artifact hash differs from the hash
  of the current compiled `.tla` (the model changed since the run)
- **THEN** the stored result no longer satisfies
  `no_counterexample` — the report is stale until a re-run against the
  current artifact

## Requirements

### Requirement: Guard citation semantics evaluated
Guard citations in invariant-kind Constraints SHALL resolve against
same-run invariant outcomes and compose under the citation algebra (conjunction
and negation) with an explicit three-valued status; the evaluated
citation remains an annotation upgrade and never a sole verdict unless
executed. Executable guard fragments SHALL remain rejected of record —
the `fragment_guard_rejected` deferral's condition (a data-carrying
state space) is not met by instance-grounded expressions.

#### Scenario: Citation algebra evaluates
- **WHEN** an artifact's invariant Constraint cites `[[a]] ∧ [[b]]` or a negated citation
- **THEN** the citation resolves against same-run invariant outcomes and composes with an explicit three-valued status

#### Scenario: Undischargable citation is honest
- **WHEN** a citation cannot be discharged over the artifact's instances
- **THEN** it reports unknown, never pass

#### Scenario: Executable guard fragments stay rejected
- **WHEN** a Transition's guard cell carries a `rust:` fragment
- **THEN** compile fails labeled (fragment_extraction stage) — the decision of record stands unchanged

### Requirement: Backend agreement property
The same fixture corpus SHALL evaluate through every deployed backend
with identical three-valued status per cell, asserted by a CI property
test: the rust-only interim asserts rust status equals the fixture's
expected status, and the cross-backend assertion activates with the py
backend without being disabled or commented out.

#### Scenario: Rust-only interim holds
- **WHEN** the shared fixture corpus evaluates through the rust backend
- **THEN** every cell's status equals the fixture's expected status

#### Scenario: Cross-backend agreement activates with the py backend
- **WHEN** the py backend lands (l8l) and the shared fixtures evaluate through both backends
- **THEN** the CI property asserts identical status per cell across backends

### Requirement: Status-bearing run reports
A model-check run SHALL persist backend, bound, and per-invariant
status — `verified`, `counterexample`, or `unknown` — with `unknown`
persisted honestly and never coerced; a run whose invariants were not
executed remains `exploration_only` and is explicitly not clean.

#### Scenario: Report names its status
- **WHEN** a run evaluates invariants with mixed outcomes
- **THEN** the report persists backend, bound, and per-invariant status, including honest unknown

#### Scenario: Unexecuted invariants are not clean
- **WHEN** a run compiles invariants but executes none
- **THEN** the outcome is exploration_only and explicitly not clean

### Requirement: Complete command-path claim evaluation
The model-check command and orchestrator SHALL evaluate every accepted
kernel invariant over the complete explicit input corpus and SHALL publish
one status per opted-in kernel or citation invariant in both CLI output
and persisted reports. Qualified citations SHALL resolve exact invariant
IDs; bare citations SHALL resolve within the citing file. Citation chains
SHALL resolve in dependency order. Cycles, missing targets, and property
rows without same-run evidence SHALL report unknown with a reason.
An incapable backend SHALL report labeled unsupported claims, never omit
them. This requirement supplies statuses; aggregate acceptance is specified
by the separate verification-claims capability.
Ordinary corpus intent IDs SHALL be unique; duplicates SHALL fail as
duplicate_corpus_identity. Dual-format id: spec files SHALL retain local
identity and SHALL run only as the sole parsed input to model-check,
verify or orchestrate. Combined scopes SHALL fail as isolated_scope_required
before compilation, evaluation or report writes, with a hint to run each
file separately using separate artifact/report directories. Single-file
spec.row citations SHALL resolve locally; multi-file dual-format lint SHALL
remain valid. Report freshness follows verification-claims.

#### Scenario: File-local identities do not merge into a corpus
- **WHEN** two dual-format files both define spec.c1 with opposite invariant outcomes, or one dual-format file is combined with an ordinary file
- **THEN** combined command evaluation refuses the scope before writes; the two dual-format files run independently with their own outcomes and multi-file lint retains file-local identity support

#### Scenario: Mixed inputs cannot hide a false kernel claim
- **WHEN** a two-state corpus contains a kernel claim asserting 999 states and a passing Rust invariant
- **THEN** both claim IDs appear in CLI and persisted output and the kernel claim is counterexample

#### Scenario: Qualified and bare citations agree locally
- **WHEN** a local passing invariant is cited by its local name and its qualified name
- **THEN** both citations evaluate to verified, without confusing an identically named invariant in another file

#### Scenario: Scope and dependency order are explicit
- **WHEN** the same multi-file corpus with a citation chain is supplied in either file order
- **THEN** the same statuses result from the full corpus and a cycle or absent target remains unknown with a reason

#### Scenario: Property execution is not invented
- **WHEN** a citation targets a Property row without same-run invariant evidence
- **THEN** its status is unknown and model-check does not invoke a property runner

### Requirement: Model check runs a real backend against the compiled artifact
The system SHALL provide a `spk model-check <files>` step that runs a
model-check backend — stateright (embedded, default) — against the
compiled model artifact of each file, without re-compiling, and SHALL
report the outcome over the envelope (`data.outcome`), where a missing
compiled artifact is a labeled ERROR with a `spk compile` hint, never a
silent success.

#### Scenario: Clean spec within bound
- **WHEN** `spk model-check <file> --json` runs on a linted, covered,
  compiled spec whose model explores exhaustively within the stated
  bound, at least one executable invariant is checked, and none is violated
- **THEN** the envelope reports `.data.outcome == "no_counterexample"`
  with the backend engine and version and the stated bound restated

#### Scenario: Missing compiled artifact is an error
- **WHEN** `spk model-check <file>` runs on a spec with no compiled
  artifact present
- **THEN** the command fails with a labeled error naming the missing
  artifact and a `spk compile` remediation hint, and never reports
  `no_counterexample`

### Requirement: The run names its backend, bound, and checked invariants
The system SHALL include in every run report the backend engine and
version, the applied bound (max depth, and optional state count and time
budget), and the set of invariants actually checked — so a report where
no user invariant was executable states `invariants_checked` as empty
rather than implying a semantic check that never ran.

#### Scenario: Prose invariants are not fabricated
- **WHEN** a spec's Constraints table contains invariant rows whose
  expressions are prose with no executable opt-in
- **THEN** a native-backend run reports `exploration_only` after
  exhaustive automaton exploration within the bound, with
  `invariants_checked` empty — never a fabricated counterexample or an
  implied semantic check

#### Scenario: Budget exhaustion is a distinct outcome
- **WHEN** a run's state/time budget expires before exhaustive
  exploration completes
- **THEN** the report outcome is `timed_out` — never collapsed into
  `no_counterexample` or `counterexample_found`

### Requirement: Run reports persist with artifact provenance
The system SHALL write each run's report as `<stem>.check.json` next to
the compiled artifacts, carrying the SHA-256 of the compiled `.tla`
module the run consumed, so downstream `verify` can determine whether a
stored clean result still applies to the current compiled artifact.

#### Scenario: Stale clean result is detectable
- **WHEN** a stored clean report's artifact hash differs from the hash
  of the current compiled `.tla` (the model changed since the run)
- **THEN** the stored result no longer satisfies
  `no_counterexample` — the report is stale until a re-run against the
  current artifact
