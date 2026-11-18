---
id: check
kind: intent
statement: "WHEN a lint-clean spec file with current compiled artifacts is judged by one invocation, THE check capability SHALL compose the existing evidence producers — conform's gate and trace walk, verify's both-gates evaluation, audit's classification and ladder derivation — into one row layer carrying exactly one existing verdict and one existing audit class per row, one audit-shaped report, and one exit contract in which 0 and 1 mean a judgement was emitted and 2 means none was."
---

# check Specification

## Purpose
Give the pipeline one judging verb. `check` composes the evidence producers
that already exist — conform's artifact gate and trace walk, verify's
both-gates evaluation, audit's classification, fold and ladder-rung
derivation — and emits one row layer, one vocabulary, one report and one exit
contract over the evidence a single spec revision actually has. It introduces
no contract runner, no adapter registry, and no fourth vocabulary: it is the
verb G13/G14 name, built by fusion rather than accretion.

## Constraints

| id                  | kind      | expr                                                                                                                                                                                                                   | traces_to |
|---------------------|-----------|------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|-----------|
| gate_reuse          | invariant | `check admits only a lint-clean file with current compiled artifacts in the out-dir, through conform's and audit's existing artifact gate — it compiles nothing and never re-derives an artifact location of its own`    | [[check]] |
| reuse_stages        | invariant | `one invocation composes verify's both-gates evaluation, conform's trace walk when an oracle is supplied, and audit's row assembly, classification, prior diff and rung derivation — no second evaluator, artifact gate, or report family is introduced` | [[check]] |
| execution_bounded   | invariant | `the only execution check performs is verify's own compiled property blocks, under verify's wall-clock bound (default 600s, --timeout-secs 0 unbounded); a run killed at the bound is a labeled termination and produces no verdict` | [[check]] |
| evidence_present    | invariant | `a run that produces zero evidence rows is a refusal with exit 2 — an evidence-less pass is impossible, and a row exists only for a producer that actually ran`                                                          | [[check]] |
| one_vocabulary      | invariant | `every record carries exactly one verdict spelling from conform's existing closed set and exactly one audit class from audit's existing closed set; no fourth vocabulary and no new enum value is introduced, and inconclusive is never emitted while reserved` | [[check]] |
| verdict_mapping     | invariant | `every producer outcome maps to the row the normative mapping table declares, and every outcome that is not a judgement (unavailable, stale, unparseable, terminated, scope-mismatched, or empty evidence) produces no row — a verdict is never fabricated in its place` | [[check]] |
| rung_derived        | invariant | `the ladder rung is derived at report time from evidence actually present — this run's verify verdict included — never exceeds that evidence, and rung 4 requires both gates green in this run while rung 5 still outranks a verify pass` | [[check]] |
| exit_contract       | invariant | `exit 0 iff a judgement was emitted with no contradicted row and no stale row under a declared expectation; exit 1 iff a judgement was emitted with a contradicted or declared-expectation-stale row; exit 2 iff no judgement was emitted — and exit 2 never reads as a pass or a contradiction` | [[check]] |
| report_interop      | invariant | `the report is the audit report's shape — same schema version, field set, row keys and evidence-scope discipline — so either verb's report is accepted as the other's prior; identical inputs serialize byte-identically, and check writes no persisted evidence artifact` | [[check]] |
| evidence_read_only  | invariant | `check writes no evidence artifact, never compiles, never rewrites the model-check report, and leaves the spec and the compiled artifacts byte-identical on every refusal`                                              | [[check]] |
| no_foreign_authority | invariant | `check never executes a consumer's contract suite, instantiate an adapter or registry, read sidecar bindings, demote the corpus, or claim independence for agent-authored evidence — scope B stays demand-gated and untouched` | [[check]] |
| evidence_unavailable | effect   | `check.evidence_unavailable(detail) — the run could not obtain evidence to judge because detail; the refusal rides the errors capability's published contract (envelope kind, exit code, remediation hint)`             | [[check]] | [[errors.envelope_error_kind]] ∧ [[errors.exit_code_mapping]] ∧ [[errors.remediation_hint_present]] |

## Model

### States
- `requested`
- `collected`
- `judged`
- `reported`
- `evidence_failed` (emits: `[[check.evidence_unavailable]]`)

### Transitions

| id      | from      | to              | guard                                                                                                                                                                                                                  |
|---------|-----------|-----------------|--------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| collect | requested | collected       | [[check.gate_reuse]] ∧ [[check.reuse_stages]] ∧ [[check.execution_bounded]] ∧ [[check.evidence_present]] ∧ [[check.no_foreign_authority]] |
| judge   | collected | judged          | [[check.one_vocabulary]] ∧ [[check.verdict_mapping]] ∧ [[check.rung_derived]] |
| emit    | judged    | reported        | [[check.exit_contract]] ∧ [[check.report_interop]] ∧ [[check.evidence_read_only]] |
| refuse  | requested | evidence_failed | `¬([[check.gate_reuse]] ∧ [[check.reuse_stages]] ∧ [[check.execution_bounded]] ∧ [[check.evidence_present]] ∧ [[check.no_foreign_authority]])` |

## Properties

| id            | kind | derives_from                              | generator                          | predicate                                                                                                          |
|---------------|------|-------------------------------------------|------------------------------------|--------------------------------------------------------------------------------------------------------------------|
| p_gate        | unit | [[check.gate_reuse]]                | `lint_dirty_or_stale_artifact()`   | `check(file) == failed ∧ zero records emitted ∧ the hint names the failing gate`                                    |
| p_stages      | unit | [[check.reuse_stages]]              | `arbitrary_evidence_corpus()`      | `rows name only the declared producers ∧ oracle verdicts equal a conform reference run over the same inputs`        |
| p_bounded     | unit | [[check.execution_bounded]]         | `hanging_property_block()`         | `run killed at the bound ⇒ labeled termination ∧ no verdict row`                                                   |
| p_present     | unit | [[check.evidence_present]]          | `corpus_with_no_evidence_units()`  | `zero rows ⇒ exit 2 ∧ zero records`                                                                                |
| p_vocabulary  | unit | [[check.one_vocabulary]]            | `arbitrary_check_report()`         | `every row carries exactly one Verdict ∧ one AuditClass ∧ the output never spells inconclusive`                     |
| p_mapping     | unit | [[check.verdict_mapping]]           | `producer_outcome_matrix()`        | `each outcome maps to its declared table row ∧ every non-judgement outcome yields no row`                          |
| p_rung        | unit | [[check.rung_derived]]              | `verified_run()`                   | `rung == 4 iff both gates were green in this run ∧ rung never exceeds the evidence present`                        |
| p_exit        | unit | [[check.exit_contract]]             | `arbitrary_class_mix()`            | `exit 0 iff no contradicted row (and no declared-expectation stale) ∧ exit 1 iff one exists ∧ exit 2 iff no judgement` |
| p_interop     | unit | [[check.report_interop]]            | `check_report_bytes()`             | `audit's prior-report parser accepts it ∧ rows diff by key ∧ rerun byte-identical`                                  |
| p_read_only   | unit | [[check.evidence_read_only]]        | `refused_run()`                    | `spec and artifacts byte-identical ∧ zero records ∧ no artifact written`                                            |
| p_scope       | unit | [[check.no_foreign_authority]]      | `foreign_suite_offered()`          | `no adapter or registry path exists ∧ no independence claim emitted`                                               |
| p_label       | unit | [[check.evidence_unavailable]]      | `evidence_unavailable_raised()`    | `error_label == "check.evidence_unavailable"`                                                                      |

## Requirements

### Requirement: Single judge verb over existing evidence

`spk check <spec-file>` SHALL compose the existing evidence producers in one
invocation — conform's artifact gate and trace walk, verify's both-gates
evaluation, and audit's row assembly, classification, diff and rung
derivation — and SHALL introduce no second evaluator, artifact gate, report
family, contract runner, or adapter registry.

#### Scenario: Verify-only run judges the spec's own evidence

- **WHEN** `spk check demo.md --out-dir out` runs with a current compiled
  artifact and a current model-check report, and no `--oracle`
- **THEN** the report carries the verify-stage rows (`verify:prop:*`,
  `verify:model`), a derived ladder rung, and the fixed evidence-scope
  statement naming that only the spec's own evidence was consumed

#### Scenario: Oracle run reuses conform's walk

- **WHEN** `spk check demo.md --oracle corpus.jsonl` runs over the same inputs
  as `spk conform demo.md --oracle corpus.jsonl`
- **THEN** every oracle row carries the verdict `spk conform` produced for that
  trace id, and no second trace evaluator is invoked

#### Scenario: Gate refusal names its cause

- **WHEN** the spec file has lint findings or its compiled artifacts are stale
- **THEN** the run refuses with a labeled error naming the failing gate and a
  remediation hint, emits zero records, and exits 2

### Requirement: One evidence vocabulary

Every record SHALL carry exactly one verdict spelling from conform's existing
closed set (`permitted`, `forbidden`, `underspecified`, `unknown`,
`unsupported`, `agree`) and exactly one audit class from the existing closed
set (`current`, `stale`, `unchecked`, `contradicted`, `binds_nothing`, `new`);
`check` SHALL introduce no fourth vocabulary, no new enum value, and SHALL NOT
emit `inconclusive` while that value remains reserved.

#### Scenario: Every row carries both layers

- **WHEN** any check report is emitted, in any output view
- **THEN** every record names both its verdict and its audit class, and every
  value is one the existing `Verdict` and `AuditClass` closed sets declare

#### Scenario: Inconclusive is never emitted

- **WHEN** a run cannot decide — a killed runner, a missing report, or an
  undecidable claim
- **THEN** the output spells no `inconclusive` verdict; the run either records
  the existing honest value (`unknown`, `unchecked`) or refuses with exit 2

#### Scenario: The evidence-axes table stays the single source

- **WHEN** the change lands
- **THEN** `scripts/check_evidence_axes.py` still passes with no new value row:
  the documented axes and the code enums are unchanged

### Requirement: Verify-stage outcome mapping

Every verify-stage outcome SHALL map to exactly one evidence row whose verdict
and audit class come from the existing closed sets, per the scenarios below.

#### Scenario: A passed property block is current evidence

- **WHEN** a compiled property block ran and passed
- **THEN** its row is keyed `verify:prop:<block-id>` with verdict `permitted`
  and class `current`, and the exit code is 0

#### Scenario: A failed property block is a contradiction

- **WHEN** a compiled property block fails
- **THEN** its row is keyed `verify:prop:<block-id>` with verdict `forbidden`
  and class `contradicted`, the reason carries the shrunk failing input, and
  the run exits 1

#### Scenario: A clean model report is current evidence

- **WHEN** the model report's outcome is `no_counterexample` and every required
  claim is `verified`
- **THEN** the row is keyed `verify:model` with verdict `permitted` and class
  `current`, and the exit code is 0

#### Scenario: A refuted claim is a contradiction

- **WHEN** the model report's outcome is `counterexample_found` or a required
  claim's status is `refuted`
- **THEN** the row is keyed `verify:model:<claim-id>` with verdict `forbidden`
  and class `contradicted`, and the run exits 1

#### Scenario: Knowledge absence is not a contradiction

- **WHEN** a required claim's status is `unknown`, or the outcome is
  `exploration_only` or `timed_out`
- **THEN** the row's verdict is `unknown`, its class `unchecked`, the warning
  is counted, and the run exits 0

### Requirement: Recorded and citation evidence mapping

Every oracle, pack, binding and prior-report outcome SHALL map to exactly one
evidence row per the scenarios below, reusing the existing verdicts and audit
classes unchanged.

#### Scenario: Oracle verdicts carry through

- **WHEN** conform's walk returns `permitted` or `forbidden` for a trace
- **THEN** the row keeps the corpus id and that verdict, with class `current`
  or `contradicted` respectively, and the exit code is 0 or 1

#### Scenario: Unchecked oracle verdicts never gate

- **WHEN** a trace's verdict is `underspecified`, `unknown`, `unsupported`, or
  `agree`
- **THEN** its row keeps the verdict, carries class `unchecked`, and the exit
  code is 0

#### Scenario: A pack violation is a contradiction

- **WHEN** a pack checker produces a violation over the judged file
- **THEN** its row is `forbidden` with `contradicted`, names the violated rule,
  and the run exits 1

#### Scenario: An empty checked-set is not a pass

- **WHEN** a pack checker produces an empty checked-set
- **THEN** its row is `agree` with class `unchecked`, countable in the summary,
  and the exit code is 0

#### Scenario: Vacuous evidence never gates

- **WHEN** an evidence unit cites no declared model element
- **THEN** its row is `binds_nothing` on the warnings channel, counted in the
  summary, and the exit code is unchanged

#### Scenario: Prior-report overlays are currency, not verdicts

- **WHEN** a row worsened since the prior report, or was absent from it
- **THEN** its class is `stale` or `new` and its verdict is unchanged; `stale`
  exits 1 only under a declared expectation, and `new` never affects the exit

### Requirement: Non-judgement outcomes produce no row

An outcome that is not a judgement — a gate refusal, unavailable, stale,
unparseable or scope-mismatched evidence, a terminated runner, a digest or
schema mismatch, or zero rows — SHALL produce no record and SHALL refuse with
exit 2; a verdict is never fabricated for evidence that was not obtained.

#### Scenario: A killed runner yields no verdict

- **WHEN** the properties runner is killed at its wall-clock bound
- **THEN** no row and no verdict are emitted, the run reports a labeled
  termination naming the bound, and exits 2

#### Scenario: A stale model report is refused

- **WHEN** `<stem>.check.json` is missing, stale, unparseable, or fails the
  scope check
- **THEN** the run exits 2 with a hint naming the model-check command — the
  stored report is never rewritten and no verdict is inferred

#### Scenario: Zero evidence rows are refused

- **WHEN** no producer yields a row for the judged file
- **THEN** the run emits zero records and exits 2 — an evidence-less pass is
  impossible

### Requirement: Exit contract

`spk check` SHALL exit 0 when a judgement was emitted and the report carries
no `contradicted` row (and no `stale` row under a declared expectation), 1
when a judgement was emitted and at least one such row exists (the
contradicted set is classified before the prior-report overlay), and 2 when
no judgement was emitted — with exit 2 never reading as a pass or a
contradiction.

#### Scenario: Honest unchecked run exits zero

- **WHEN** every row is `unchecked` or `binds_nothing`
- **THEN** the exit code is 0 and the counts are surfaced in both output views

#### Scenario: A contradiction fails the run

- **WHEN** any row is `contradicted`
- **THEN** the exit code is 1 in both output views

#### Scenario: Could not judge is exit two

- **WHEN** the run emits zero records because the evidence was unavailable
- **THEN** the exit code is 2, the cause and remediation hint are stated, and
  the output nowhere reads as a pass or a contradiction

#### Scenario: Declared-expectation staleness gates

- **WHEN** a row is `stale` and the prior report was recorded with
  `--declare-expectation`
- **THEN** the exit code is 1; without the declaration the same row is
  informational and the exit code is 0

### Requirement: Ladder rung derived from evidence present

The report SHALL carry the highest verification-ladder rung the evidence
actually supports, derived at report time — including this run's verify
verdict — and SHALL never exceed that evidence.

#### Scenario: A fully verified run reaches rung four

- **WHEN** every compiled property block passed and the current model-check
  report is clean with all required claims verified
- **THEN** the reported rung is 4 with the verify-pass basis stated

#### Scenario: A failed verify does not lift the rung

- **WHEN** the properties gate or the model gate fails
- **THEN** the rung is derived from the remaining current evidence and never
  reports 4, stating the basis it did use

#### Scenario: Better-design still outranks a verify pass

- **WHEN** the current model-check run reported `counterexample_found`
- **THEN** the rung is 5 — a refuted claim is the design lesson — and `check`
  does not override the ladder

### Requirement: Report interoperability and determinism

The check report SHALL be the audit report's shape — the same schema version,
field set, row keys and evidence-scope discipline — so a check report is
accepted as a prior report by the audit verb and vice versa, and identical
inputs SHALL yield a byte-identical report. `check` SHALL write no persisted
evidence artifact.

#### Scenario: A check report is a valid prior

- **WHEN** `spk audit demo.md --oracle corpus.jsonl --before check-report.json`
  consumes a check report
- **THEN** the prior parses and its rows diff per row key, with no schema
  divergence

#### Scenario: Re-runs are byte-identical

- **WHEN** the same inputs are judged twice
- **THEN** the two reports serialize identically — no timestamps, no
  iteration-order leakage, stable row order

#### Scenario: No oracle states its scope

- **WHEN** a run supplies no oracle corpus
- **THEN** the evidence-scope statement names the spec-internal evidence the
  run consumed, and the scope digest is the empty corpus's digest

### Requirement: Scope boundary

`check` SHALL NOT execute a consumer's contract suite, instantiate an adapter
or adapter registry, read sidecar bindings, demote the corpus, or claim
independence for agent-authored evidence; scope B remains demand-gated and
unchanged by this verb.

#### Scenario: No adapter surface exists

- **WHEN** the command surface is inspected
- **THEN** no adapter, registry, or suite-binding flag exists, and the only
  execution path is verify's own bounded property runner

#### Scenario: The claim of record is stated honestly

- **WHEN** any report or its documentation describes what was established
- **THEN** it states that drift is visible and graded, and claims neither a
  second authoritative copy nor independent evidence

#### Scenario: Existing verbs keep their contracts

- **WHEN** the change lands
- **THEN** `verify`, `conform`, and `audit` keep their status strings, report
  schemas and exit codes unchanged
