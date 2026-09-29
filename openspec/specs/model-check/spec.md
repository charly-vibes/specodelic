---
id: spec
kind: intent
statement: "THE model-check step SHALL run a model-check backend against each file's compiled artifact and report the outcome, the backend identity, the applied bound, and the invariants actually checked over the envelope."
---

# model-check Specification

## Purpose
Run the compiled model artifact through a model-check backend
(stateright, embedded, default) without re-compiling, and report an
honest outcome: exhaustive exploration within the stated bound, budget
exhaustion as `timed_out`, and never a fabricated counterexample or an
implied semantic check that did not run.

## Constraints

| id                | kind      | expr                                                                                                                                                | traces_to       |
|-------------------|-----------|------------------------------------------------------------------------------------------------------------------------------------------------------|-----------------|
| honest_outcome    | invariant | `every reported outcome is one of no_counterexample / counterexample_found / timed_out, and timed_out is never collapsed into no_counterexample`    | [[spec]]        |
| backend_identified | invariant | `a run report names the backend engine and version that produced it, and the stated bound is restated`                                              | [[spec.honest_outcome]] |
| provenance        | invariant | `each stored run report carries the SHA-256 of the compiled .tla module it consumed, so verify can detect stale clean results`                      | [[spec.honest_outcome]] |
| no_fabrication    | advisory  | `prose invariants produce no executable predicate; invariants_checked stays empty rather than implying a semantic check that never ran`             | [[spec]] — prose pointer: specs/model_check.md |

## Model

### States
- `pending`
- `running`
- `no_counterexample`
- `counterexample_found`
- `timed_out`

### Transitions

| id               | from     | to                   | guard                                                        |
|------------------|----------|----------------------|--------------------------------------------------------------|
| start            | pending  | running              | `the compiled .tla artifact exists`                          |
| finish_clean     | running  | no_counterexample    | [[spec.honest_outcome]]                                      |
| finish_violation | running  | counterexample_found | [[spec.honest_outcome]]                                      |
| finish_timeout   | running  | timed_out            | [[spec.honest_outcome]]                                      |

## Properties

| id              | kind | derives_from                  | generator                                   | predicate                                                        |
|-----------------|------|-------------------------------|---------------------------------------------|------------------------------------------------------------------|
| backend_named   | unit | [[spec.backend_identified]]   | `run_report_from_any_backend()`             | `report.backend == a named engine and version`                   |
| stale_detected  | unit | [[spec.provenance]]           | `(clean_run, model_edited_afterward_with_no_rerun)` | `verify treats the stored clean result as stale — not clean`     |
| timeout_distinct | unit | [[spec.honest_outcome]]      | `run_budget_exhausted_before_exhaustion()`  | `outcome == timed_out, never no_counterexample`                  |
| prose_unchecked  | unit | [[spec.no_fabrication]]      | `spec_with_prose_invariant_constraints()`   | `report.invariants_checked == [] — no semantic check implied`    |

## Requirements
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
  bound and no invariant is violated
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
  expressions are prose (no executable predicate language exists)
- **THEN** a native-backend run reports `no_counterexample` only from
  the exhaustive automaton exploration within the bound, with
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