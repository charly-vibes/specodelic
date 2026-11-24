---
id: check.layer
kind: intent
statement: "WHEN a lint-clean spec file with current compiled artifacts is judged by one invocation, THE check capability SHALL compose the existing evidence producers — conform's gate and trace walk, verify's both-gates evaluation, audit's classification and ladder derivation — into one row layer carrying exactly one existing verdict and one existing audit class per row, one audit-shaped report, and one exit contract in which 0 and 1 mean a judgement was emitted and 2 means none was — and SHALL name every judged claim by its file-qualified claim id and SHALL list every declared invariant claim of the judged file with its evidence status, placed only by the declared placement rule, so an uncovered expected behaviour is countable rather than invisible."
---

# check — dual-format layer (sidecar)

## Purpose
Give the pipeline one judging verb. `check` composes the evidence producers
that already exist — conform's artifact gate and trace walk, verify's
both-gates evaluation, audit's classification, fold and ladder-rung
derivation — and emits one row layer, one vocabulary, one report and one exit
contract over the evidence a single spec revision actually has. It introduces
no contract runner, no adapter registry, and no fourth vocabulary: it is the
verb G13/G14 name, built by fusion rather than accretion.

It also names what it judged. Every claim a row carries is named by its
file-qualified claim id — the form the corpus already guarantees unique
corpus-wide — and every declared invariant claim of the judged file is listed
with its evidence status, so an expected behaviour with no evidence behind it
is countable in the report instead of invisible. A row places a claim only by
naming it or by naming a property that derives from it, and a claim placed by
several rows carries the strongest class among them. Nothing is matched by name,
slug, or similarity, and no binding outside the corpus is read: the coverage
section is a statement about the judged file in this run, not a claim about
anyone else's suite.

<!-- Error labels are file-id-namespaced (error-contract): an error
(effect) Constraint's expr reads `<owning-file-id>.<variant_head>(field, …)`
and the owning id is THIS file's own id — `check.layer.` — so a failure label
here reads `check.layer.<variant_head>(field, …)`. The archive companion re-keys
that namespace to the deployed spec id when it attaches the layer. -->

## Constraints

| id                  | kind      | expr                                                                                                                                                                                                                   | traces_to |
|---------------------|-----------|------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|-----------|
| gate_reuse          | invariant | `check admits only a lint-clean file with current compiled artifacts in the out-dir, through conform's and audit's existing artifact gate — it compiles nothing and never re-derives an artifact location of its own`    | [[check.layer]] |
| reuse_stages        | invariant | `one invocation composes verify's both-gates evaluation, conform's trace walk when an oracle is supplied, and audit's row assembly, classification, prior diff and rung derivation — no second evaluator, artifact gate, or report family is introduced` | [[check.layer]] |
| execution_bounded   | invariant | `the only execution check performs is verify's own compiled property blocks, under verify's wall-clock bound (default 600s, --timeout-secs 0 unbounded); a run killed at the bound is a labeled termination and produces no verdict` | [[check.layer]] |
| evidence_present    | invariant | `a run that produces zero evidence rows is a refusal with exit 2 — an evidence-less pass is impossible, and a row exists only for a producer that actually ran`                                                          | [[check.layer]] |
| one_vocabulary      | invariant | `every record carries exactly one verdict spelling from conform's existing closed set and exactly one audit class from audit's existing closed set; no fourth vocabulary and no new enum value is introduced, and inconclusive is never emitted while reserved` | [[check.layer]] |
| verdict_mapping     | invariant | `every producer outcome maps to the row the normative mapping table declares, and every outcome that is not a judgement (unavailable, stale, unparseable, terminated, scope-mismatched, or empty evidence) produces no row — a verdict is never fabricated in its place` | [[check.layer]] |
| rung_derived        | invariant | `the ladder rung is derived at report time from evidence actually present — this run's verify verdict included — never exceeds that evidence, and rung 4 requires both gates green in this run while rung 5 still outranks a verify pass` | [[check.layer]] |
| exit_contract       | invariant | `exit 0 iff a judgement was emitted with no contradicted row and no stale row under a declared expectation; exit 1 iff a judgement was emitted with a contradicted or declared-expectation-stale row; exit 2 iff no judgement was emitted — and exit 2 never reads as a pass or a contradiction` | [[check.layer]] |
| report_interop      | invariant | `the report is the audit report's shape — same schema version, field set, row keys and evidence-scope discipline — so either verb's report is accepted as the other's prior; identical inputs serialize byte-identically, and check writes no persisted evidence artifact` | [[check.layer]] |
| evidence_read_only  | invariant | `check writes no evidence artifact, never compiles, never rewrites the model-check report, and leaves the spec and the compiled artifacts byte-identical on every refusal`                                              | [[check.layer]] |
| no_foreign_authority | invariant | `check never executes a consumer's contract suite, instantiate an adapter or registry, read sidecar bindings, demote the corpus, or claim independence for agent-authored evidence — scope B stays demand-gated and untouched` | [[check.layer]] |
| claim_key_file_qualified | invariant | `every judged claim is named by its file-qualified claim id — <file id>.<claim id> — the form the corpus already guarantees unique corpus-wide; a report row never abbreviates a claim to its bare id, and no claim is matched by name, slug, or similarity` | [[check.layer]] |
| claim_placement_declared | invariant | `an evidence row places a declared claim only when it names that claim directly or names a property whose derives_from names that claim — the corpus's declared edge — and a row that places no claim belongs to the section's residual bucket, never to a claim` | [[check.layer]] |
| claim_coverage_listed | invariant | `every declared invariant claim of the judged file appears in the report's claim-coverage section carrying exactly one audit class from audit's existing closed set {current, stale, unchecked, contradicted, binds_nothing, new}; a claim with no placing row carries unchecked — knowledge absence in this run — and is never absent from the section, never reported as evidence it does not have, and never given a new vocabulary value; a claim placed by several rows carries the strongest class in the order contradicted > stale > current > new > binds_nothing > unchecked; the section is materialized in the check envelope and leaves the audit report's schema version and field set unchanged` | [[check.layer]] |
| claim_coverage_non_gating | invariant | `the claim-coverage section is countable and never gates the exit code — an uncovered claim is knowledge absence, not a contradiction, and its count is surfaced in every output view alongside the existing evidence counts and on the verdict line` | [[check.layer]] |
| evidence_unavailable | effect   | `check.layer.evidence_unavailable(detail) — the run could not obtain evidence to judge because detail; the refusal rides the errors capability's published contract (envelope kind, exit code, remediation hint)`             | [[check.layer]] | [[errors.envelope_error_kind]] ∧ [[errors.exit_code_mapping]] ∧ [[errors.remediation_hint_present]] |

## Model

### States
- `requested`
- `collected`
- `judged`
- `reported`
- `evidence_failed` (emits: `[[check.layer.evidence_unavailable]]`)

### Transitions

| id      | from      | to              | guard                                                                                                                                                                                                                  |
|---------|-----------|-----------------|--------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| collect | requested | collected       | [[check.layer.gate_reuse]] ∧ [[check.layer.reuse_stages]] ∧ [[check.layer.execution_bounded]] ∧ [[check.layer.evidence_present]] ∧ [[check.layer.no_foreign_authority]] |
| judge   | collected | judged          | [[check.layer.one_vocabulary]] ∧ [[check.layer.verdict_mapping]] ∧ [[check.layer.rung_derived]] |
| emit    | judged    | reported        | [[check.layer.exit_contract]] ∧ [[check.layer.report_interop]] ∧ [[check.layer.evidence_read_only]] |
| refuse  | requested | evidence_failed | `¬([[check.layer.gate_reuse]] ∧ [[check.layer.reuse_stages]] ∧ [[check.layer.execution_bounded]] ∧ [[check.layer.evidence_present]] ∧ [[check.layer.no_foreign_authority]])` |

## Properties

| id            | kind | derives_from                              | generator                          | predicate                                                                                                          |
|---------------|------|-------------------------------------------|------------------------------------|--------------------------------------------------------------------------------------------------------------------|
| p_gate        | unit | [[check.layer.gate_reuse]]                | `lint_dirty_or_stale_artifact()`   | `check(file) == failed ∧ zero records emitted ∧ the hint names the failing gate`                                    |
| p_stages      | unit | [[check.layer.reuse_stages]]              | `arbitrary_evidence_corpus()`      | `rows name only the declared producers ∧ oracle verdicts equal a conform reference run over the same inputs`        |
| p_bounded     | unit | [[check.layer.execution_bounded]]         | `hanging_property_block()`         | `run killed at the bound ⇒ labeled termination ∧ no verdict row`                                                   |
| p_present     | unit | [[check.layer.evidence_present]]          | `corpus_with_no_evidence_units()`  | `zero rows ⇒ exit 2 ∧ zero records`                                                                                |
| p_vocabulary  | unit | [[check.layer.one_vocabulary]]            | `arbitrary_check_report()`         | `every row carries exactly one Verdict ∧ one AuditClass ∧ the output never spells inconclusive`                     |
| p_mapping     | unit | [[check.layer.verdict_mapping]]           | `producer_outcome_matrix()`        | `each outcome maps to its declared table row ∧ every non-judgement outcome yields no row`                          |
| p_rung        | unit | [[check.layer.rung_derived]]              | `verified_run()`                   | `rung == 4 iff both gates were green in this run ∧ rung never exceeds the evidence present`                        |
| p_exit        | unit | [[check.layer.exit_contract]]             | `arbitrary_class_mix()`            | `exit 0 iff no contradicted row (and no declared-expectation stale) ∧ exit 1 iff one exists ∧ exit 2 iff no judgement` |
| p_interop     | unit | [[check.layer.report_interop]]            | `check_report_bytes()`             | `audit's prior-report parser accepts it ∧ rows diff by key ∧ rerun byte-identical`                                  |
| p_read_only   | unit | [[check.layer.evidence_read_only]]        | `refused_run()`                    | `spec and artifacts byte-identical ∧ zero records ∧ no artifact written`                                            |
| p_scope       | unit | [[check.layer.no_foreign_authority]]      | `foreign_suite_offered()`          | `no adapter or registry path exists ∧ no independence claim emitted`                                               |
| p_claim_key   | unit | [[check.layer.claim_key_file_qualified]]  | `two_files_declaring_the_same_claim_id()` | `the two rows carry distinct keys ∧ each key resolves to its own file's claim ∧ no key is derived from a name or slug` |
| p_placement   | unit | [[check.layer.claim_placement_declared]] | `row_naming_a_property_derives_from_a_claim()` | `the claim that property derives from is listed ∧ no other claim is listed for that row ∧ a row placing no claim lands in the residual bucket` |
| p_coverage_listed | unit | [[check.layer.claim_coverage_listed]] | `declared_claim_with_no_evidence_row()` | `the claim is listed with class unchecked ∧ the section counts it ∧ no row claims evidence for it ∧ no new vocabulary value appears ∧ the section states that its classes describe this run's producers ∧ the audit report's schema version and field set are unchanged` |
| p_coverage_non_gating | unit | [[check.layer.claim_coverage_non_gating]] | `run_with_an_unchecked_claim()`  | `the exit code is unchanged ∧ the unchecked-claim count is surfaced in every output view ∧ the count appears on the verdict line` |
| p_label       | unit | [[check.layer.evidence_unavailable]]      | `evidence_unavailable_raised()`    | `error_label == "check.evidence_unavailable"`                                                                      |

## Requirements

### Requirement: Layer sidecar contract

This sidecar is the dual-format layer for the `check` capability and SHALL be
deployed verbatim by the archive companion; the plain deltas in `spec.md`
beside it are its openspec face.

#### Scenario: Archive deploys the layer

- **WHEN** `just archive-change id=add-check-claim-coverage` runs after the
  change is implemented
- **THEN** the capability deploys with this layer attached fail-closed
