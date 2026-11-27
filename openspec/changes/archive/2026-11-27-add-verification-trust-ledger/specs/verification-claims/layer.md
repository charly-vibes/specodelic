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
| trust_base_derived | invariant | `the trust base is recomputed from the current corpus and the current artifacts at report time and is never accepted from a stored report without that recomputation; a stored ledger that disagrees with the recomputation is refused with a rerun hint` | [[verification.claims.layer]] |
| trust_base_versioned | invariant | `the persisted report's trust base carries trust_base_schema_version 1, and a stored report whose trust base is absent or carries an unrecognized version is refused as stale — an absent trust base is never read as an empty one, and the claim schema version is left untouched` | [[verification.claims.layer]] |
| run_limits_excluded | invariant | `the model-check bound and the artifact digest are run metadata reported beside the verdict and appear in neither trust array — a ledger is a function of the corpus, never of the run's parameters` | [[verification.claims.layer]] |
| trust_base_views_agree | invariant | `CLI JSON, human output, and the persisted report name the same trust base members with the same kinds — no view reports an empty trust base while another names an entry` | [[verification.claims.layer]] |
| empty_trust_base_meaning | invariant | `an empty trust base means the corpus declares no unchecked claim and no machine-uninterpreted guard, and it is always serialized — the emptiness says nothing about a claim's verification status, which the verdict reports separately as checked, unknown, or refuted` | [[verification.claims.layer]] |
| trust_base_corpus_scoped | invariant | `the trust base is corpus-scoped: a member may come from any file in the scope and names its source file — so a foreign extension point appearing or disappearing changes the ledger of the files that cite it` | [[verification.claims.layer]] |
| trust_base_shared_derivation | invariant | `the trust base is derived once from the corpus rows and carried by both the model-check report and the verify verdict — one derivation, two carriers, never two implementations` | [[verification.claims.layer]] |

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
| deploy | implemented | deployed | [[verification.claims.layer.process_lifecycle]] ∧ required gates pass and the companion re-attachment preserves every requirement ∧ [[verification.claims.layer.trust_base_derived]] ∧ [[verification.claims.layer.trust_base_versioned]] ∧ [[verification.claims.layer.trust_base_views_agree]] |

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
| trust_base_versioned_checked | unit | [[verification.claims.layer.trust_base_versioned]] | `ledger_less_report_fixture()` | `a stored report without a trust base is stale, never an empty one, and the claim schema version is unchanged` |
| run_limits_excluded_checked | unit | [[verification.claims.layer.run_limits_excluded]] | `bounded_run_fixture()` | `the bound and the artifact digest appear in neither trust array` |
| trust_base_views_agree_checked | unit | [[verification.claims.layer.trust_base_views_agree]] | `three_view_fixture()` | `JSON, human, and persisted reports name the same members with the same kinds` |
| empty_trust_base_meaning_checked | unit | [[verification.claims.layer.empty_trust_base_meaning]] | `fully_checked_corpus()` | `an empty trust base ⇒ no unchecked claim and no machine-uninterpreted guard, the field is present, and the emptiness says nothing about a claim's verification status` |
| trust_base_corpus_scoped_checked | unit | [[verification.claims.layer.trust_base_corpus_scoped]] | `cross_file_extension_point_corpus()` | `a member from a foreign file appears with its source file, and the ledger follows the corpus` |
| trust_base_shared_derivation_checked | unit | [[verification.claims.layer.trust_base_shared_derivation]] | `two_surface_fixture()` | `the model-check report and the verify verdict carry the same members, kinds, and order` |

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
array. The ledger SHALL be corpus-scoped: a member may come from any file in
the scope and names its source file.

#### Scenario: The two kind sets are closed

- **WHEN** a verdict's trust base is read
- **THEN** every `unverified_claims` entry's kind is one of `prose_only_invariant`, `foreign_extension_point`, `advisory_constraint`, and every `assumed` entry's kind is `prose_only_guard`

#### Scenario: Prose-only invariant is an unverified claim

- **WHEN** a file declares a prose-only invariant Constraint
- **THEN** the verdict's `unverified_claims` lists that Constraint with kind `prose_only_invariant`, and it does not appear in `assumed`

#### Scenario: A prose-only guard is an assumption

- **WHEN** a transition's guard is prose and cites no evaluable constraint — no citation expression, no kernel binding, no executable invariant
- **THEN** the verdict's `assumed` lists that transition with kind `prose_only_guard`, and the guard's condition is absent from the model report's checked invariants, which name only the constraints actually checked

#### Scenario: A guard the citation algebra resolves is not an assumption

- **WHEN** a transition's guard cites an evaluable constraint, or is a State "has reached state X" progress marker
- **THEN** that transition is absent from `assumed` — only a guard with no resolvable condition is relied upon without checking

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
- **THEN** the corpus declares no unchecked claim and no machine-uninterpreted guard, and the empty ledger is still serialized — the emptiness says nothing about a claim's verification status, which the verdict reports separately as checked, unknown, or refuted

### Requirement: The trust base is derived, versioned, and scope-bound

The trust base SHALL be recomputed from the current corpus and artifacts at
report time and SHALL never be accepted from a stored report without that
recomputation. The persisted report SHALL carry `trust_base_schema_version`
1; a stored report whose trust base is absent or unrecognized SHALL be
refused as stale, never read as an empty trust base. Membership and order
SHALL be a pure function of the corpus rows, so the same corpus in a
different file order yields an identical ledger.

#### Scenario: A report without a trust base is stale

- **WHEN** a stored report carries no trust base, or carries a `trust_base_schema_version` the tool does not support
- **THEN** verify refuses the report as stale and names the rerun command — an absent trust base is never an empty one, and the claim schema version is left untouched

#### Scenario: A stale ledger is refused

- **WHEN** a stored report's trust base disagrees with the recomputation
- **THEN** verify refuses the report as a ledger mismatch, with the same rerun hint the unchecked-set comparison uses, and never silently re-scores the stored ledger

#### Scenario: Reordering does not change the ledger

- **WHEN** the same corpus is presented in a different file order
- **THEN** the recomputed trust base is identical member for member, and the persisted report is byte-identical

#### Scenario: The stored ledger is never the source of truth

- **WHEN** a stored report carries a trust base that no recomputation could have produced
- **THEN** the run fails with a rerun hint and the stored report is left unmodified

### Requirement: Both report surfaces carry the same trust base

The model-check report and the verify verdict SHALL carry the same trust
base, derived once from the same corpus rows. The two surfaces SHALL NOT
derive the ledger independently, and the write-side seam that attaches the
claim fields SHALL attach the trust base in the same place.

#### Scenario: The two surfaces agree

- **WHEN** a model-check report and the verify verdict for the same corpus are compared
- **THEN** their trust bases carry the same members with the same kinds and the same order

### Requirement: The trust base is visible in every view

CLI JSON, human output, and the persisted report SHALL agree on the trust
base's membership and kinds: no view may report an empty trust base while
another names an entry. The human view SHALL report the member count per
kind and the first ten members in the ledger's deterministic order, naming
the file each member comes from. Release documentation SHALL describe the
trust base as information about the corpus, never as a statement about
application correctness.

#### Scenario: Views agree on the ledger

- **WHEN** a run has one prose-only guard and one prose-only invariant
- **THEN** CLI JSON, human output, and the saved report name the same two entries with the same kinds

#### Scenario: The human view counts and truncates by a stated rule

- **WHEN** a run's trust base holds more than ten members
- **THEN** the human output names the count per kind and the first ten members in the deterministic order, and states that the list is truncated — the JSON keeps every member

#### Scenario: The ledger is not a verdict

- **WHEN** the release documentation describes the trust base
- **THEN** it states that a non-empty ledger reports what was not checked, that it never blocks or grants `verified`, and that neither an empty nor a non-empty ledger says anything about the implementation's behaviour

## Notes

This sidecar is the dual-format layer for the `verification-claims`
capability. The plain openspec delta in `spec.md` beside it is the
capability's openspec face; the archive companion re-attaches this layer
verbatim onto the deployed spec, and refuses the archive fail-closed when
the sidecar is missing or unrestorable. Before the change is archived, the
deployment path is `just archive-change id=add-verification-trust-ledger`.

The trust base is **information, never a verdict**: a non-empty ledger says
what was not checked, and an empty one says only that the corpus declares no
unchecked claim and no machine-uninterpreted guard. Neither says anything
about the implementation's behaviour, and neither changes the acceptance
conjunction (`both_gates_required`).
