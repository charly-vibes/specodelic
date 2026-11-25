# verification-claims Specification

## Purpose
THE verifier SHALL accept only complete current evidence for every required invariant claim.

## ADDED Requirements

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
