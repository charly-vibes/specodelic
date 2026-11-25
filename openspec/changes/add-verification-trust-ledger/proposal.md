# Change: Verification trust ledger — a verdict that names its trust base

> Tracks bd `specodelic-rzwg`.

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
(2026-11-24) found missing.

## What Changes

- Every `verify` and `model-check` verdict carries a `trust_base` object with
  exactly **two disjoint arrays**:
  - `unverified_claims[]` — claims the corpus asserts that no engine checked.
    Closed kind set: `prose_only_invariant`, `foreign_extension_point`,
    `advisory_constraint`.
  - `assumed[]` — facts the tool relied on without checking. Closed kind
    set: `prose_only_guard`.
  - Each entry names its id, its kind, and the file it came from; an entry
    appears in at most one array.
- **Run limits are metadata, not trust**: the model-check bound
  (`max_depth` / `max_states` / `timeout_secs`) and the artifact digest ride
  beside the verdict as run metadata and appear in neither array — a bound
  is a parameter of the run, not a fact the corpus asserts.
- The trust base is **derived at report time** from the current corpus and
  the current artifacts; it is never read back from a stored report. A stored
  report whose ledger disagrees with the recomputed one is refused with a
  rerun hint, on the same scope-digest discipline as the claim set.
- **All three views agree**: CLI JSON, human output, and the persisted
  report name the same members with the same kinds.
- An empty trust base is a **positive statement**: every claim was
  machine-checked and every guard machine-interpreted.
- **No format change**: no new column, no new kind, no `specodelic.md`
  Revision. The ledger is a report-surface addition, so this change lands
  without touching the format's governed sets.

## Impact

- Affected specs: `verification-claims` (ADDED requirements; the existing
  five requirements and their scenarios are unmodified). `verify` and
  `model_check` keep their current requirements — the ledger is additional
  payload on the same surfaces their `assurance_views_agree` /
  `evidence_scope_bound` constraints already govern.
- Affected code: `src/verify.rs` (verdict assembly), `src/model_check.rs`
  (run report payload), `src/human.rs` (human view), the `verify` /
  `model-check` command modules (`src/commands/`).
- Affected docs: the release assurance wording — `verification-claims`'s
  `visible_scope` already requires the release documentation to distinguish
  what was verified from what was not; the ledger names the difference
  machine-readably.
- Out of scope (separate changes): contract matching for `satisfies`
  (the `foreign_extension_point` kind is *reported* here, not *checked*);
  rule↔constraint binding in `RULE_TABLE`; any format-level trust
  annotation; any change to the claim set's required/unchecked semantics.
- Provenance: Verus guide — `guarantees`, `external_trait_specifications`,
  `call-from-unverified-code` (the trust-base discipline); plus the
  Rule-of-5 review of the source memo (2026-11-24), whose `EDGE-001`
  finding split the ledger into two disjoint arrays and moved the bound out
  of both.
