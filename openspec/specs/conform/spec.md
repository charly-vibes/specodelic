---
id: conform
kind: intent
statement: "WHEN conform runs against a lint-clean spec file with current compiled artifacts and an external scenario corpus, THE capability SHALL classify each oracle trace with a verdict from the closed taxonomy, SHALL forbid positively-contradicted traces in any invocation mode while reserving closed-world-declared forbidden for uncovered traces, SHALL keep underspecified, unknown, and unsupported distinct with reasons, and SHALL emit a scope-bound, deterministic report that never treats uninterpreted prose as checked or agreement on the corpus as behavioral equality."
---

# conform Specification

## Purpose

Check a spec against reality instead of against itself: classify traces
recorded from an external oracle (a legacy system, curated cases, a
reference implementation) against what the spec declares and can execute.
conform is the external-evidence leg the pipeline lacks — every other verb
evaluates the spec's own claims. Its discipline is the negative space: a
missing transition is underspecification, never prohibition; prose is never
silently checked; agreement on a finite corpus is never behavioral
equality. conform is read-only over specs and oracles alike — it evaluates,
it never generates traces, implementations, or verdicts beyond its
declared scope.

## Constraints

| id                            | kind      | expr                                                                                                                                                                                                                                                | traces_to    |
|-------------------------------|-----------|-----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|--------------|
| read_only_evaluation          | invariant | `conform consumes the parsed/linted/compiled spec artifacts and the scenario corpus as given; it never writes spec files, never generates or records traces, and never implements the domain system`                                                 | [[conform]]  |
| closed_world_gated            | invariant | `forbidden verdicts come from exactly two rules, both recorded in the verdict record: a trace that violates a declared executable claim is forbidden in any invocation mode (the same fact model_check reports as counterexample_found); a trace no declared Model element or claim covers is forbidden only under an explicitly declared closed-world mode (--closed-world) and is underspecified in the default open-world mode — absence of coverage alone is never forbidden without the declaration` | [[conform]]  |
| verdict_taxonomy_distinct     | invariant | `every trace receives exactly one verdict from the closed set {permitted, forbidden, underspecified, unknown, unsupported}: underspecified (no declared claim or Model element covers the trace) ≠ unknown (a claim covers it but is not interpretable) ≠ unsupported (the covering claim's evaluator kind is not executable in this run), each verdict record carrying its reason` | [[conform]]  |
| prose_never_checked           | invariant | `verdicts derive only from structural Model facts and executable claims (kernel, rust: fragment, citation); a prose-only claim covering a trace contributes unknown with reasons — prose wording, a constraint's name, or a deriving Property never implies a permitted/forbidden judgment, mirroring required_claims_classified` | [[conform]]  |
| verdict_report_schema         | invariant | `the run report declares its own conform-local report_schema_version, per-trace verdict records (scenario id, verdict, reason, evaluated claim ids, closed_world flag), an evidence_scope field stating agreement-on-corpus-not-equality, and a scope digest binding the parsed structured content and the consumed scenario corpus bytes — reordered CLI paths or prose edits preserve the digest, changed claims or scenarios never share one; re-running conform over unchanged inputs is byte-identical: verdict records sorted by scenario id, no timestamps, no iteration-order leakage` | [[conform]]  |
| exit_code_verdict_mapping     | invariant | `exit 0 iff no verdict in the report is forbidden or unsupported; exit 1 iff any verdict is forbidden or unsupported; unknown and underspecified verdicts never affect the exit code and are surfaced with counts in both report views; invocation errors and gate refusals remain exit 2 without verdict records` | [[conform]]  |
| artifact_gate                 | invariant | `conform runs only over a spec file that parses, lints clean, and has current compiled artifacts; an ungated invocation is refused with a remediation hint naming the failing stage — verdicts are never derived from stale or invalid artifacts`    | [[conform]]  |
| lifecycle_outside             | invariant | `conform is an evaluation verb outside the artifact lifecycle: it never advances draft→parsed→linted→compiled→model_checked→verified, is never part of orchestrate, and a conform run never gates or blocks a lifecycle transition`                  | [[conform]]  |

## Model

### States
- `input_read`
- `artifacts_gated`
- `claims_classified`
- `traces_classified`
- `reported`

### Transitions

| id               | from               | to                 | guard                                                                                                                        |
|------------------|--------------------|--------------------|------------------------------------------------------------------------------------------------------------------------------|
| gate_artifacts   | input_read         | artifacts_gated    | [[conform.artifact_gate]] ∧ `the target file parses, lints clean, and its compiled artifacts are current`                    |
| classify_claims  | artifacts_gated    | claims_classified  | [[conform.prose_never_checked]] ∧ `each invariant claim is classified executable or prose-only with its evaluator kind`       |
| classify_traces  | claims_classified  | traces_classified  | [[conform.verdict_taxonomy_distinct]] ∧ `every corpus trace has been assigned exactly one closed-taxonomy verdict with reason` |
| emit_report      | traces_classified  | reported           | [[conform.verdict_report_schema]] ∧ `the report carries per-trace records, evidence_scope, and the scope digest`              |

## Properties

| id                              | kind | derives_from                          | generator                                        | predicate                                                                                                            |
|---------------------------------|------|----------------------------------------|--------------------------------------------------|----------------------------------------------------------------------------------------------------------------------|
| consumes_never_writes           | unit | [[conform.read_only_evaluation]]       | `conform_run_over_fixture_corpus()`              | `git status clean and spec bytes unchanged after the run — no artifact outside the report was written`                |
| contradiction_forbidden_any_mode | unit | [[conform.closed_world_gated]]        | `contradicting_trace_without_closed_world()`     | `verdict == forbidden ∧ record.closed_world == false ∧ reason names the contradicted executable claim`                 |
| closed_world_forbidden_recorded | unit | [[conform.closed_world_gated]]         | `uncovered_trace_with_closed_world()`            | `verdict == forbidden ∧ record.closed_world == true ∧ reason names the closed-world exhaustiveness declaration`        |
| uncovered_trace_never_forbidden | unit | [[conform.closed_world_gated]]         | `uncovered_trace_without_closed_world()`         | `verdict == underspecified` — absence of coverage is never forbidden without the declaration                           |
| taxonomy_is_total_and_distinct  | unit | [[conform.verdict_taxonomy_distinct]]  | `corpus_with_all_five_verdicts()`                | `each trace has exactly one verdict; all five values are reachable; no trace maps to two`                              |
| prose_yields_unknown            | unit | [[conform.prose_never_checked]]        | `spec_with_prose_only_invariant()`               | `covering traces report unknown with reason naming the prose claim — never permitted/forbidden`                        |
| unsupported_kind_named          | unit | [[conform.prose_never_checked]]        | `claim_with_unsupported_evaluator_kind()`        | `verdict == unsupported ∧ reason names the evaluator kind`                                                             |
| report_schema_roundtrip         | unit | [[conform.verdict_report_schema]]      | `run_report_persisted_and_reread()`              | `report carries report_schema_version, per-trace records, evidence_scope, scope_sha256`                                |
| digest_binds_scenarios          | unit | [[conform.verdict_report_schema]]      | `two_runs_differing_only_in_scenarios()`         | `different scenario bytes → different scope digest; identical inputs → identical digest`                               |
| rerun_byte_identical            | unit | [[conform.verdict_report_schema]]      | `same_inputs_conformed_twice()`                  | `both reports byte-identical — records sorted by scenario id, no timestamps`                                           |
| exit_codes_match_verdicts       | unit | [[conform.exit_code_verdict_mapping]]  | `reports_across_all_verdict_classes()`           | `exit code matches the mapping for each verdict-class composition`                                                     |
| stale_artifacts_refused         | unit | [[conform.artifact_gate]]              | `spec_modified_after_compile()`                  | `invocation refused with remediation hint naming the stage — no verdict records emitted`                               |
| lint_dirty_refused              | unit | [[conform.artifact_gate]]              | `spec_with_lint_findings()`                      | `invocation refused with hint to run spk lint — no verdict records emitted`                                            |
| outside_lifecycle               | unit | [[conform.lifecycle_outside]]          | `orchestrate_run_with_conform_available()`       | `orchestrate never invokes conform; conform run leaves the artifact lifecycle stage unchanged`                         |

## ADDED Requirements

### Requirement: Oracle trace classification
The system SHALL provide `spk conform <spec-path> --oracle <scenarios.jsonl>`
classifying each trace in the corpus with exactly one verdict from the
closed taxonomy {permitted, forbidden, underspecified, unknown,
unsupported}, each verdict record carrying its reason and the ids of the
claims evaluated, deriving only from structural Model facts and executable
claims — never from prose, never from a constraint's name, and never from
the existence of a deriving Property.

#### Scenario: Executable claim contradicts a trace
- **WHEN** a corpus trace's observed transition violates an executable invariant claim (`**rust:**` fragment or kernel atom) over the compiled model
- **THEN** the trace is classified `forbidden` and the verdict record names the contradicted claim id in its reason

#### Scenario: Prose-only claim covers a trace
- **WHEN** the only claim covering a trace has a prose-only expr (no kernel, `**rust:**`, or citation marker)
- **THEN** the trace's verdict is `unknown` with a reason naming the prose claim — never `permitted` or `forbidden`

#### Scenario: No declared claim covers a trace
- **WHEN** a trace references a transition or observation no declared Model element or claim covers
- **THEN** the verdict is `underspecified` in the default open-world mode

### Requirement: Closed-world gate
The system SHALL separate the two prohibition evidence classes: a trace
that violates a declared, executable claim SHALL be `forbidden` in any
invocation mode; a trace no declared Model element or claim covers SHALL
be `forbidden` only when the invocation explicitly declared closed-world
mode via `--closed-world`; the closed-world declaration SHALL be recorded
in every verdict record and the report header; absence of coverage in the
default open-world mode SHALL yield `underspecified`, never `forbidden`.

#### Scenario: Contradiction forbidden in any mode
- **WHEN** a trace violates an executable claim and the run was invoked without `--closed-world`
- **THEN** the verdict is `forbidden` and its record carries `closed_world: false` with the contradicted claim id in the reason

#### Scenario: Uncovered trace forbidden under the declaration
- **WHEN** a trace no declared claim or Model element covers and the run was invoked with `--closed-world`
- **THEN** the verdict is `forbidden` and its record carries `closed_world: true` with the exhaustiveness declaration named in the reason

#### Scenario: Uncovered trace underspecified without the declaration
- **WHEN** the same uncovered trace occurs in a run without `--closed-world`
- **THEN** the verdict is `underspecified`, never `forbidden`

### Requirement: Verdict report schema and evidence scope
The system SHALL emit the verdict report through the standard envelope
conventions (JSON default for pipes, `--human` for TTYs), carrying a
conform-local `report_schema_version`, per-trace verdict records (scenario
id, verdict, reason, evaluated claim ids, closed_world flag), an
`evidence_scope` field stating that the report establishes agreement on
the supplied corpus and not behavioral equality, and a scope digest
binding the parsed structured content and the consumed scenario corpus
bytes. Re-running conform over unchanged inputs SHALL produce
byte-identical reports — verdict records sorted by scenario id, no
timestamps, no iteration-order leakage. A corpus with zero traces SHALL
yield a valid report with zero records and the evidence_scope intact.

#### Scenario: Scope digest binds inputs
- **WHEN** two runs consume identical spec content but scenario corpora differing by one byte
- **THEN** the two reports carry different scope digests; identical inputs yield identical digests

#### Scenario: Evidence scope present in both views
- **WHEN** the report is rendered as JSON and as `--human`
- **THEN** both views carry the same `evidence_scope` statement — no view can omit it

#### Scenario: Re-runs are byte-identical
- **WHEN** conform runs twice over unchanged spec and corpus inputs
- **THEN** both reports are byte-identical — records sorted by scenario id, no timestamps

#### Scenario: Empty corpus yields a valid empty report
- **WHEN** the `--oracle` file contains zero trace records
- **THEN** conform exits 0 with a valid report carrying zero verdict records and the evidence_scope intact

### Requirement: Exit-code contract
The system SHALL map exit codes to verdict classes: exit 0 when no verdict
in the report is `forbidden` or `unsupported`; exit 1 when any verdict is
`forbidden` or `unsupported`; `unknown` and `underspecified` verdicts
SHALL NOT affect the exit code and SHALL be surfaced with counts in both
report views. Invocation errors and gate refusals SHALL remain exit 2.

#### Scenario: Violation-class verdict fails the run
- **WHEN** the report contains at least one `forbidden` or `unsupported` verdict
- **THEN** conform exits 1

#### Scenario: Coverage gaps do not fail the run
- **WHEN** the report contains only `permitted`, `unknown`, and `underspecified` verdicts
- **THEN** conform exits 0 with the unknown and underspecified counts visible in both views

#### Scenario: Invocation error stays exit 2
- **WHEN** the spec path does not exist or the corpus is refused by the artifact gate
- **THEN** conform exits 2 without emitting verdict records

### Requirement: Artifact gate
The system SHALL refuse to run conform over a spec file that does not
parse, does not lint clean, or whose compiled artifacts are not current,
with a remediation hint naming the failing stage (`run spk lint` / `run
spk compile`), and SHALL NOT emit verdict records from such an invocation.

#### Scenario: Stale artifacts refused
- **WHEN** the spec file's structured content changed after its last compile
- **THEN** conform refuses with a hint naming compile, and no verdict records are emitted

#### Scenario: Lint-dirty file refused
- **WHEN** the target file carries lint findings
- **THEN** conform refuses with a hint naming lint, and no verdict records are emitted

### Requirement: Outside the artifact lifecycle
The system SHALL keep conform outside the artifact lifecycle: a conform run
SHALL NOT advance the spec artifact's lifecycle stage, SHALL NOT gate any
lifecycle transition, and SHALL NOT be part of `orchestrate` runs.

#### Scenario: Orchestrate never runs conform
- **WHEN** `spk orchestrate` runs over a spec file with conform available
- **THEN** the pipeline is lint → compile → model_check → verify as specified and conform is not invoked

#### Scenario: Conform does not advance lifecycle
- **WHEN** conform completes over a `model_checked` artifact
- **THEN** the artifact's lifecycle stage is unchanged after the run

## Requirements

### Requirement: Oracle trace classification
The system SHALL provide `spk conform <spec-path> --oracle <scenarios.jsonl>`
classifying each trace in the corpus with exactly one verdict from the
closed taxonomy {permitted, forbidden, underspecified, unknown,
unsupported}, each verdict record carrying its reason and the ids of the
claims evaluated, deriving only from structural Model facts and executable
claims — never from prose, never from a constraint's name, and never from
the existence of a deriving Property.

#### Scenario: Executable claim contradicts a trace
- **WHEN** a corpus trace's observed transition violates an executable invariant claim (`**rust:**` fragment or kernel atom) over the compiled model
- **THEN** the trace is classified `forbidden` and the verdict record names the contradicted claim id in its reason

#### Scenario: Prose-only claim covers a trace
- **WHEN** the only claim covering a trace has a prose-only expr (no kernel, `**rust:**`, or citation marker)
- **THEN** the trace's verdict is `unknown` with a reason naming the prose claim — never `permitted` or `forbidden`

#### Scenario: No declared claim covers a trace
- **WHEN** a trace references a transition or observation no declared Model element or claim covers
- **THEN** the verdict is `underspecified` in the default open-world mode

### Requirement: Closed-world gate
The system SHALL separate the two prohibition evidence classes: a trace
that violates a declared, executable claim SHALL be `forbidden` in any
invocation mode; a trace no declared Model element or claim covers SHALL
be `forbidden` only when the invocation explicitly declared closed-world
mode via `--closed-world`; the closed-world declaration SHALL be recorded
in every verdict record and the report header; absence of coverage in the
default open-world mode SHALL yield `underspecified`, never `forbidden`.

#### Scenario: Contradiction forbidden in any mode
- **WHEN** a trace violates an executable claim and the run was invoked without `--closed-world`
- **THEN** the verdict is `forbidden` and its record carries `closed_world: false` with the contradicted claim id in the reason

#### Scenario: Uncovered trace forbidden under the declaration
- **WHEN** a trace no declared claim or Model element covers and the run was invoked with `--closed-world`
- **THEN** the verdict is `forbidden` and its record carries `closed_world: true` with the exhaustiveness declaration named in the reason

#### Scenario: Uncovered trace underspecified without the declaration
- **WHEN** the same uncovered trace occurs in a run without `--closed-world`
- **THEN** the verdict is `underspecified`, never `forbidden`

### Requirement: Verdict report schema and evidence scope
The system SHALL emit the verdict report through the standard envelope
conventions (JSON default for pipes, `--human` for TTYs), carrying a
conform-local `report_schema_version`, per-trace verdict records (scenario
id, verdict, reason, evaluated claim ids, closed_world flag), an
`evidence_scope` field stating that the report establishes agreement on
the supplied corpus and not behavioral equality, and a scope digest
binding the parsed structured content and the consumed scenario corpus
bytes. Re-running conform over unchanged inputs SHALL produce
byte-identical reports — verdict records sorted by scenario id, no
timestamps, no iteration-order leakage. A corpus with zero traces SHALL
yield a valid report with zero records and the evidence_scope intact.

#### Scenario: Scope digest binds inputs
- **WHEN** two runs consume identical spec content but scenario corpora differing by one byte
- **THEN** the two reports carry different scope digests; identical inputs yield identical digests

#### Scenario: Evidence scope present in both views
- **WHEN** the report is rendered as JSON and as `--human`
- **THEN** both views carry the same `evidence_scope` statement — no view can omit it

#### Scenario: Re-runs are byte-identical
- **WHEN** conform runs twice over unchanged spec and corpus inputs
- **THEN** both reports are byte-identical — records sorted by scenario id, no timestamps

#### Scenario: Empty corpus yields a valid empty report
- **WHEN** the `--oracle` file contains zero trace records
- **THEN** conform exits 0 with a valid report carrying zero verdict records and the evidence_scope intact

### Requirement: Exit-code contract
The system SHALL map exit codes to verdict classes: exit 0 when no verdict
in the report is `forbidden` or `unsupported`; exit 1 when any verdict is
`forbidden` or `unsupported`; `unknown` and `underspecified` verdicts
SHALL NOT affect the exit code and SHALL be surfaced with counts in both
report views. Invocation errors and gate refusals SHALL remain exit 2.

#### Scenario: Violation-class verdict fails the run
- **WHEN** the report contains at least one `forbidden` or `unsupported` verdict
- **THEN** conform exits 1

#### Scenario: Coverage gaps do not fail the run
- **WHEN** the report contains only `permitted`, `unknown`, and `underspecified` verdicts
- **THEN** conform exits 0 with the unknown and underspecified counts visible in both views

#### Scenario: Invocation error stays exit 2
- **WHEN** the spec path does not exist or the corpus is refused by the artifact gate
- **THEN** conform exits 2 without emitting verdict records

### Requirement: Artifact gate
The system SHALL refuse to run conform over a spec file that does not
parse, does not lint clean, or whose compiled artifacts are not current,
with a remediation hint naming the failing stage (`run spk lint` / `run
spk compile`), and SHALL NOT emit verdict records from such an invocation.

#### Scenario: Stale artifacts refused
- **WHEN** the spec file's structured content changed after its last compile
- **THEN** conform refuses with a hint naming compile, and no verdict records are emitted

#### Scenario: Lint-dirty file refused
- **WHEN** the target file carries lint findings
- **THEN** conform refuses with a hint naming lint, and no verdict records are emitted

### Requirement: Outside the artifact lifecycle
The system SHALL keep conform outside the artifact lifecycle: a conform run
SHALL NOT advance the spec artifact's lifecycle stage, SHALL NOT gate any
lifecycle transition, and SHALL NOT be part of `orchestrate` runs.

#### Scenario: Orchestrate never runs conform
- **WHEN** `spk orchestrate` runs over a spec file with conform available
- **THEN** the pipeline is lint → compile → model_check → verify as specified and conform is not invoked

#### Scenario: Conform does not advance lifecycle
- **WHEN** conform completes over a `model_checked` artifact
- **THEN** the artifact's lifecycle stage is unchanged after the run