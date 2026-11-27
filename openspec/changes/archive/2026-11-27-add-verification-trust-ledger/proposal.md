# Change: Verification trust ledger — a verdict that names its trust base

> Tracks bd `specodelic-rzwg`.

**What this does:** reports what the verdict rests on.
**What it does not:** check any of it, change the specodelic format, or
block or grant `verified`.

## Why

`verification-claims` already partitions the corpus's *claims*: the required
set (checked) and the unchecked set (prose-only invariants the tool never
evaluated). What no surface names is the rest of what a verdict rests on.

Concretely, a `verified` verdict today can coexist with:

- a **prose-only guard** — the transition fires on a condition no engine can
  interpret (`model_check.md`'s `invariants_checked: []`, exploration-only);
- a **foreign `extension_point`** conformance (`satisfies`) — asserted by the
  corpus, checked by nobody, anywhere;
- an **advisory Constraint** — declared, linted for shape, enforced by no gate.

Each of those is a different animal — the first is *relied upon*, the other
two are *asserted* — and none of them appears in the verdict. A reader of the
report cannot tell "every claim was machine-checked" from "one claim was
prose and we did not look".

Verus's own answer to the same question is the *guarantees* chapter and the
`external_body` / `external_trait_specification` soundness warnings: a
verification verdict is only meaningful together with the list of things it
assumed without checking. specodelic has the vocabulary for that
(`unchecked`, `exploration_only`, `evidence_scope`) but computes it nowhere
uniformly — the same gap `add-evidence-audit` closed for evidence rows, now
closed for the trust base.

This change adds the trust base to the verdict itself, partitioned so that
"asserted but unchecked" and "relied upon but unchecked" never share a
bucket — the distinction the Rule-of-5 review of the source memo
(2026-11-24) found missing, and the distinction the Rule-of-5 review of
this proposal re-checked against the code.

## What Changes

- Every `verify` and `model-check` verdict carries a `trust_base` object with
  exactly **two disjoint arrays**:
  - `unverified_claims[]` — claims the corpus asserts that no engine checked.
    Closed kind set: `prose_only_invariant`, `foreign_extension_point`,
    `advisory_constraint`.
  - `assumed[]` — facts the tool relied on without checking. Closed kind
    set: `prose_only_guard` (a guard that is prose and cites no evaluable
    constraint).
  - Each entry names its id, its kind, and its source file; an entry appears
    in at most one array; the ledger is corpus-scoped.
- **How the three existing sets compose** — checked claims are the required
  set; unchecked claims are the unchecked set plus foreign extension points
  plus advisory constraints; assumptions are the prose-only guards. The
  ledger adds members the other two do not carry.
- **Run limits are metadata, not trust**: the model-check bound and the
  artifact digest ride beside the verdict as run metadata and appear in
  neither array — a bound is a parameter of the run, not a fact the corpus
  asserts.
- The trust base is **derived once** from the corpus rows and carried by
  both report surfaces; it is **recomputed at report time** and never
  accepted from a stored report without that recomputation.
- **Versioned separately**: the persisted report carries
  `trust_base_schema_version: 1`. A stored report whose trust base is
  absent or carries an unrecognized version is **stale** — never an empty
  ledger. The claim schema version (`claim_schema_version`) is **not**
  touched, so no other evidence is invalidated by the addition.
- **All three views agree**: CLI JSON, human output, and the persisted
  report name the same members with the same kinds. The human view reports
  the count per kind and the first ten members in the deterministic order.
- An empty trust base is a **positive statement**: the corpus declares no
  unchecked claim and no machine-uninterpreted guard. It says nothing about
  a claim's verification status — unknown and refuted claims are the
  verdict's own vocabulary.
- **No specodelic format change**: no new column, no new kind, no
  `specodelic.md` Revision. The **report** schema grows additively; a
  report produced before this change is stale (re-run it), never empty.

## Impact

- Affected specs: `verification-claims` (ADDED requirements; the existing
  five requirements and their scenarios are unmodified — the claim schema
  version stays 1). `verify` and `model_check` keep their current
  requirements — the ledger is additional payload on the same surfaces their
  `assurance_views_agree` / `evidence_scope_bound` constraints already
  govern.
- Affected code:
  - `src/verify.rs` — the ledger derivation, the recompute-and-refuse
    comparison (alongside the existing unchecked-set comparison at
    `src/verify.rs:495-506`), the version gate
    (`src/verify.rs:369-379`), and the write-side seam
    (`fresh_report_json`, `src/verify.rs:810-830`).
  - `src/model_check.rs` — the run report payload (`RunReport`,
    `src/model_check.rs:120-155`).
  - `src/human.rs` — the human view (counts per kind, first ten members).
  - `src/commands/model_check.rs` — `cmd_verify` (`src/commands/model_check.rs:375`).
- Affected docs: the release assurance wording — `verification-claims`'s
  `visible_scope` already requires the release documentation to distinguish
  what was verified from what was not; the ledger names the difference
  machine-readably.
- Out of scope (separate changes): contract matching for `satisfies` (the
  `foreign_extension_point` kind is *reported* here, not *checked*); the
  rule↔constraint binding in `RULE_TABLE`; any format-level trust
  annotation; any change to the required/unchecked claim semantics; any
  blocking behaviour on a non-empty ledger.
- Provenance: Verus guide — `guarantees`, `external_trait_specifications`,
  `call-from-unverified-code` (the trust-base discipline); plus the
  Rule-of-5 reviews of the source memo and of this proposal (2026-11-24),
  whose findings split the ledger into two disjoint arrays, moved the bound
  out of both, and pinned the report's version story.
