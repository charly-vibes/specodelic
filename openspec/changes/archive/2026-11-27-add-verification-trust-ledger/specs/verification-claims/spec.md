# verification-claims Specification

## Purpose
THE verifier SHALL accept only complete current evidence for every required invariant claim.

## ADDED Requirements

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
