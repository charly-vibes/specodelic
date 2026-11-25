# Design: add-verification-trust-ledger

## Context

`verification-claims` already computes two sets — the required claim set and
the unchecked set — and `verify` refuses to report `verified` unless every
required claim is checked and every property block passes. What it never
computes is the set of things the verdict *rests on but did not check*:

- a transition whose guard is prose (machine-uninterpreted — the model
  explores with `invariants_checked: []`);
- a row that claims conformance to a foreign `extension_point` (`satisfies`),
  which `specodelic.md`'s Reference Typing deliberately leaves unverified;
- an `advisory` Constraint — declared, shape-checked, gating nothing.

The Verus guide's `guarantees` chapter names this the trust base of a
verification verdict, and `external_body` / `external_trait_specification`
are Verus's way of *marking* it. specodelic has no equivalent surface, so a
report's silence is indistinguishable from a clean bill of health.

## Goals / Non-Goals

- Goals: name the trust base on every verdict; keep "asserted but unchecked"
  and "relied upon but unchecked" distinguishable; keep the ledger derived,
  deterministic, and identical across the three report views; leave the
  format untouched.
- Non-Goals: **checking** any of the listed items (the foreign extension
  point stays an unchecked claim in v1 — a contract matcher is a separate
  change); changing the required/unchecked claim semantics; a format-level
  trust annotation; a new command; a "trust score".

## Decisions

- **Decision: two disjoint arrays, not one list with a kind tag.**
  `unverified_claims[]` holds claims the corpus asserts that no engine
  checked (`prose_only_invariant`, `foreign_extension_point`,
  `advisory_constraint`); `assumed[]` holds facts the tool relied on without
  checking (`prose_only_guard`).
  - Alternatives considered: a single `trust_base[]` with one kind set
    (rejected — a reader cannot tell an *asserted* foreign conformance from a
    *relied-upon* prose guard, and "assumed" would read as "we trust it");
    three arrays (rejected — `unverified_claims` is exactly the existing
    unchecked set plus one new kind, so it is one array with a closed kind
    set).
- **Decision: the ledger is recomputed, never trusted.** `verify` recomputes
  the trust base from the current corpus and current artifacts and compares
  it with the stored report's ledger; a mismatch is refused with a rerun
  hint, on the existing scope-digest discipline (`evidence_scope_bound`).
  - Alternatives considered: trusting the stored ledger (rejected — a stored
    ledger is a second source of truth that drifts exactly like a stale
    claim set); storing the ledger in a separate artifact (rejected — one
    more file to version, no new information).
- **Decision: run limits are metadata, not trust.** The model-check bound
  and the artifact digest ride beside the verdict as run metadata; they
  appear in neither array.
  - Alternatives considered: listing the bound as a trust entry (rejected —
    the ledger would become a function of run parameters, so the same corpus
    would yield different ledgers under different bounds and cross-run
    diffing would break; "bounded exploration" is already reported by
    `model_check`'s own outcome vocabulary).
- **Decision: advisory Constraints are unverified claims.** An advisory
  Constraint is a declared requirement that no gate enforces; listing it is
  the honest reading, and the linter's shape-check of it is not enforcement.
- **Decision: foreign extension points are reported, not checked.** The
  ledger names the conformance claim as unchecked so the reader sees it; a
  contract matcher (does the satisfying row's guard entail the extension
  point's precondition?) is a separate change with its own capability spec.
- **Decision: the ledger ships on both the model-check report and the
  verify verdict.** The model-check report is where the prose-only guards
  become visible (`invariants_checked: []` is currently the only hint); the
  verify verdict is where a release decision is read.
- **Decision: an empty ledger is serialized, never omitted.** An empty
  trust base is a positive statement ("everything was machine-checked"), so
  the fields are always present — unlike the optional vectors in
  `model_check`'s `RunReport`, which use `skip_serializing_if`.
- **Decision: deterministic order.** Members are sorted by
  `(kind, source file id, row id)`; no map-iteration order leaks into the
  output, so two runs over the same corpus produce byte-identical ledgers.

## Risks / Trade-offs

- Risk: the ledger reads as a failure list (a corpus with many advisory
  constraints looks bad) → the human view shows counts plus the first
  entries and names the file; the docs state that a non-empty ledger is
  information, not a verdict.
- Risk: an empty ledger is over-read as "the implementation is correct" →
  the release wording keeps the existing boundary explicit (the ledger is
  about the *corpus*, never about application behaviour).
- Risk: report-schema churn for consumers → the change is purely additive;
  no existing field changes name, type, or meaning.
- Risk: the ledger duplicates the unchecked set → the unchecked set *is* the
  `unverified_claims` array's `prose_only_invariant` members; the
  implementation derives both from one source so they cannot disagree.

## Open Questions

- Should the ledger feed `spk doctor --advice` (the parked
  `add-doctor-advice`)? Deferred — the advice profiler is a separate
  decision surface and the ledger's consumers are report surfaces.
- Should ledger members be ranked or weighted? Deferred — no ranking means
  no scoring semantics to keep honest; a rank would be a score in disguise.
- Should a non-empty ledger ever block `verified`? Deferred to a later
  change: v1 reports, and the acceptance conjunction (`both_gates_required`)
  stays exactly as it is.
