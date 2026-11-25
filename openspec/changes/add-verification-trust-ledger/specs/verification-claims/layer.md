---
id: verification.claims.layer
kind: intent
statement: "WHEN a verification or model-check verdict is reported, THE verifier SHALL name its trust base as two disjoint sets — unverified claims and assumed facts — and SHALL exclude run limits from both."
---

# Verification claims

## Purpose
THE verifier SHALL accept only complete current evidence for every required invariant claim.

## Constraints

| id | kind | expr | traces_to |
|----|------|------|-----------|
| process_lifecycle | invariant | `the capability advances through its declared lifecycle states under the repo's change process — each stage transition fires only when its stage gate holds` | [[verification.claims.layer]] |
| required_set | invariant | required claims come from explicit executable or kernel or citation invariant opt-ins | [[verification.claims.layer]] |
| aggregate | invariant | nonempty required claims must all verify and property blocks must all pass | [[verification.claims.layer]] |
| freshness | invariant | reports bind a versioned claim set to current structured scope and artifacts | [[verification.claims.layer]] |
| visible_scope | invariant | all report views and release docs state the same assurance scope | [[verification.claims.layer]] |
| trust_base_partitioned | invariant | `every verdict's trust base is partitioned into exactly two disjoint arrays — unverified_claims (kinds {prose_only_invariant, foreign_extension_point, advisory_constraint}) and assumed (kind {prose_only_guard}) — each entry naming its id, its kind, and its source file, and an entry appearing in at most one array` | [[verification.claims.layer]] |
| trust_base_derived | invariant | `the trust base is recomputed from the current corpus and the current artifacts at report time and is never accepted from a stored report; a stored ledger that disagrees with the recomputation is refused with a rerun hint` | [[verification.claims.layer]] |
| run_limits_excluded | invariant | `the model-check bound and the artifact digest are run metadata reported beside the verdict and appear in neither trust array — a ledger is a function of the corpus, never of the run's parameters` | [[verification.claims.layer]] |
| trust_base_views_agree | invariant | `CLI JSON, human output, and the persisted report name the same trust base members with the same kinds — no view reports an empty trust base while another names an entry` | [[verification.claims.layer]] |
| empty_trust_base_meaning | invariant | `an empty trust base means every claim was machine-checked and every guard machine-interpreted, and it is always serialized — an omitted ledger is never an empty one` | [[verification.claims.layer]] |

## Model

### States
- proposed
- approved
- implemented
- deployed

### Transitions

| id | from | to | guard |
|----|------|----|-------|
| approve | proposed | approved | [[verification.claims.layer.process_lifecycle]] ∧ maintainer approves this proposal  |
| implement | approved | implemented | [[verification.claims.layer.process_lifecycle]] ∧ every constraint has passing behavioral evidence  |
| deploy | implemented | deployed | [[verification.claims.layer.process_lifecycle]] ∧ required gates pass and the companion re-attachment preserves every requirement ∧ [[verification.claims.layer.trust_base_derived]] ∧ [[verification.claims.layer.trust_base_views_agree]] |

## Properties

| id | kind | derives_from | generator | predicate |
|----|------|--------------|-----------|-----------|
| process_lifecycle_checked | unit | [[verification.claims.layer.process_lifecycle]] | `lifecycle_model_present()` | `check(file) == passed` |
| required_set_checked | unit | [[verification.claims.layer.required_set]] | mixed_claim_corpus() | required and unchecked sets equal the expected IDs |
| aggregate_checked | unit | [[verification.claims.layer.aggregate]] | mixed_claim_statuses() | false and unknown claims cannot yield verified |
| freshness_checked | unit | [[verification.claims.layer.freshness]] | changed_scope_or_legacy_report() | verify rejects stale or incomplete evidence with a rerun hint |
| visible_scope_checked | unit | [[verification.claims.layer.visible_scope]] | report_and_docs_fixture() | claim counts agree and pending features are not advertised as implemented |
| trust_base_partitioned_checked | unit | [[verification.claims.layer.trust_base_partitioned]] | `mixed_trust_base_corpus()` | `every member appears in exactly one array with a kind from that array's closed set` |
| trust_base_derived_checked | unit | [[verification.claims.layer.trust_base_derived]] | `changed_scope_ledger_fixture()` | `a stored ledger that disagrees with the recomputation is refused with a rerun hint` |
| run_limits_excluded_checked | unit | [[verification.claims.layer.run_limits_excluded]] | `bounded_run_fixture()` | `the bound and the artifact digest appear in neither trust array` |
| trust_base_views_agree_checked | unit | [[verification.claims.layer.trust_base_views_agree]] | `three_view_fixture()` | `JSON, human, and persisted reports name the same members with the same kinds` |
| empty_trust_base_meaning_checked | unit | [[verification.claims.layer.empty_trust_base_meaning]] | `fully_checked_corpus()` | `an empty trust base ⇒ every claim machine-checked and every guard machine-interpreted, and the field is present` |

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
claims from current inputs. The digest SHALL bind structured content,
not filesystem paths, so opposite claims with identical names cannot
reuse each other's evidence.

#### Scenario: Cross-file changes invalidate a clean report
- **WHEN** a referenced file changes structurally or is omitted from the supplied corpus after the report was produced
- **THEN** verify rejects the report as stale or scope-mismatched and names the rerun command

### Requirement: Stale evidence is refused with a rerun hint
Stale evidence SHALL be refused with a rerun hint: unsupported or old
reports, omitted or duplicate records, changed contributing inputs, or
artifact mismatch SHALL fail with a rerun hint. Reordering CLI paths or
changing prose alone SHALL preserve scope.

#### Scenario: Old reports cannot acquire new evidence implicitly
- **WHEN** a stored report has no claim schema version or required records
- **THEN** verify rejects it with a model-check rerun hint without modifying the report

### Requirement: Intent ids of parsed inputs are unique
The intent IDs of a command's parsed inputs SHALL be unique: a combined
invocation whose inputs claim the same intent id SHALL fail with
duplicate_corpus_identity and a rename hint before compilation,
evaluation or report writes (Revision 18: dual-format files carry real
ids and compose like any corpus — the former `id: spec` isolated-scope
rule retired with `id: spec` itself).

#### Scenario: A same-id pair is refused before any writes
- **WHEN** two parsed inputs claim the same intent id (the transitional pair: an active change's delta and its deployed capability spec)
- **THEN** model-check, verify and orchestrate refuse the combined scope as duplicate_corpus_identity before writing artifacts or reports, with rename guidance; independent runs in separate directories retain each claim's own status, produce different scope digests and reject swapped reports, while lint accepts files with distinct ids together

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

### Requirement: A verdict names its trust base

Every verification or model-check verdict SHALL carry a `trust_base` object
partitioned into exactly two disjoint arrays: `unverified_claims` (asserted
but unchecked) and `assumed` (relied upon but unchecked). Each entry SHALL
name its id, its kind, and its source file, and SHALL appear in at most one
array. Run limits SHALL be run metadata beside the verdict, never trust
entries.

#### Scenario: The two kind sets are closed

- **WHEN** a verdict's trust base is read
- **THEN** every `unverified_claims` entry's kind is one of `prose_only_invariant`, `foreign_extension_point`, `advisory_constraint`, and every `assumed` entry's kind is `prose_only_guard`

#### Scenario: Prose-only invariant is an unverified claim

- **WHEN** a file declares a prose-only invariant Constraint
- **THEN** the verdict's `unverified_claims` lists that Constraint with kind `prose_only_invariant`, and it does not appear in `assumed`

#### Scenario: A prose-only guard is an assumption

- **WHEN** a transition's guard is prose and therefore machine-uninterpreted
- **THEN** the verdict's `assumed` lists that transition with kind `prose_only_guard`, and the model-check report's `invariants_checked` stays empty — the ledger names what the exploration relied upon

#### Scenario: A foreign extension point is an unverified claim

- **WHEN** a row claims conformance to a foreign `extension_point`
- **THEN** the verdict's `unverified_claims` lists that row with kind `foreign_extension_point`, never in `assumed` — nothing in the corpus checks the conformance, so it is asserted, not trusted

#### Scenario: An advisory constraint is an unverified claim

- **WHEN** a file declares an `advisory` Constraint, which no gate enforces
- **THEN** the verdict's `unverified_claims` lists it with kind `advisory_constraint`

#### Scenario: Run limits are metadata, not trust

- **WHEN** a model-check run used a depth cap, a state cap, or a timeout
- **THEN** the bound and the artifact digest appear as run metadata and appear in neither trust array

#### Scenario: An empty trust base is a positive statement

- **WHEN** both trust arrays are empty
- **THEN** every claim was machine-checked and every guard machine-interpreted — the empty ledger is a statement about the corpus, and it is still serialized

### Requirement: The trust base is derived and scope-bound

The trust base SHALL be recomputed from the current corpus and artifacts at
report time and SHALL never be accepted from a stored report without that
recomputation; a stored ledger that disagrees with the recomputation SHALL
be refused with a rerun hint. Membership and order SHALL be a pure function
of the corpus rows, so the same corpus presented in a different file order
SHALL yield an identical ledger.

#### Scenario: A stale ledger is refused

- **WHEN** a stored report's trust base names a row the current corpus no longer declares
- **THEN** verify refuses the report as scope-mismatched and names the rerun command, never silently re-scoring the stored ledger

#### Scenario: Reordering does not change the ledger

- **WHEN** the same corpus is presented in a different file order
- **THEN** the recomputed trust base is identical member for member, and the persisted report is byte-identical

#### Scenario: The stored ledger is never the source of truth

- **WHEN** a stored report carries a trust base that no recomputation could have produced
- **THEN** the run fails with a rerun hint and the stored report is left unmodified

### Requirement: The trust base is visible in every view

CLI JSON, human output, and the persisted report SHALL agree on the trust
base's membership and kinds: no view may report an empty trust base while
another names an entry. Release documentation SHALL describe the trust base
as information about the corpus, never as a statement about application
correctness.

#### Scenario: Views agree on the ledger

- **WHEN** a run has one prose-only guard and one prose-only invariant
- **THEN** CLI JSON, human output, and the saved report name the same two entries with the same kinds

#### Scenario: The ledger is not a verdict

- **WHEN** the release documentation describes the trust base
- **THEN** it states that a non-empty ledger reports what was not checked, that it never blocks or grants `verified`, and that neither an empty nor a non-empty ledger says anything about the implementation's behaviour

## Notes

This sidecar is the dual-format layer for the `verification-claims`
capability. The plain openspec delta in `spec.md` beside it is the
capability's openspec face; the archive companion re-attaches this layer
verbatim onto the deployed spec (`just archive-change
id=add-verification-trust-ledger`), and refuses the archive fail-closed
when the sidecar is missing or unrestorable.

The trust base is **information, never a verdict**: a non-empty ledger says
what was not checked, and an empty one says only that the corpus was fully
machine-checked. Neither says anything about the implementation's behaviour,
and neither changes the acceptance conjunction (`both_gates_required`).
