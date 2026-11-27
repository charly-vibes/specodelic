# Design: add-verification-trust-ledger

## Context

`verification-claims` already computes two sets — the required claim set and
the unchecked set — and `verify` refuses to report `verified` unless every
required claim is checked and every property block passes. What it never
computes is the set of things the verdict *rests on but did not check*:

- a transition whose guard is prose and cites no evaluable constraint (the
  model explores with `invariants_checked: []`);
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
  versioned, deterministic, and identical across the three report views;
  leave the specodelic format untouched.
- Non-Goals: **checking** any of the listed items (the foreign extension
  point stays an unchecked claim in v1 — a contract matcher is a separate
  change); changing the required/unchecked claim semantics; a format-level
  trust annotation; a new command; a "trust score"; changing the claim
  schema version.

## Decisions

- **Decision: the ledger is derived from the corpus, never stored as truth.**
  One derivation over parsed rows produces it; it is recomputed at report
  time; a stored report's ledger is compared against the recomputation, and
  a disagreement is refused with a rerun hint.
  - Precedent: `verify` already does exactly this for the unchecked set —
    `src/verify.rs:495-506` compares `stored_unchecked` against the live
    classification and fails stale with "rerun model-check over the same
    file set". The ledger follows that pattern rather than inventing a
    mechanism.
  - Alternatives considered: trusting the stored ledger (rejected — a
    stored ledger is a second source of truth that drifts exactly like a
    stale claim set); storing the ledger in a separate artifact (rejected —
    one more file to version, no new information).
  - Consequence 1 (partition): the ledger has **two disjoint arrays** —
    `unverified_claims[]` (asserted but unchecked: `prose_only_invariant`,
    `foreign_extension_point`, `advisory_constraint`) and `assumed[]`
    (relied upon but unchecked: `prose_only_guard`). A single list with a
    kind tag was rejected: a reader cannot tell an *asserted* foreign
    conformance from a *relied-upon* prose guard, and "assumed" would read
    as "we trust it".
  - Consequence 2 (run limits): the model-check bound and the artifact
    digest are **run metadata** beside the verdict, in neither array — the
    ledger is a function of the corpus, never of the run's parameters, so
    the same corpus yields the same ledger under different bounds and
    cross-run diffing stays meaningful.
  - Consequence 3 (scope): the ledger is **corpus-scoped**: a member may come
    from any file in the scope and names its source file. A foreign
    extension point appearing or disappearing changes the ledger of the
    files that cite it — that is the scope, not instability.
- **Decision: the ledger rides with its own version; the claim schema
  version is untouched.** The persisted report carries
  `trust_base_schema_version: 1` inside the trust base. A stored report
  whose trust base is absent, or carries an unrecognized version, is
  **stale** — never an empty ledger.
  - Alternatives considered: bumping `CLAIM_SCHEMA_VERSION`
    (`src/verify.rs:657`) to 2 — rejected: it would invalidate every stored
    report corpus-wide, change every scope digest
    (`scope_digest` binds the claim schema version, `src/verify.rs:757-764`),
    and require MODIFYING `verification-claims`'s "claim schema version 1"
    requirement — a format-wide invalidation for a payload addition;
    treating an absent ledger as empty — rejected outright: it is the exact
    absent-vs-empty conflation this change exists to end.
  - Consequence: reports produced before this change are stale and must be
    re-run. That is loud, and it is the honest outcome.
- **Decision: one derivation, two carriers.** The model-check report and the
  verify verdict carry the same ledger, produced by the same function, and
  the write-side seam that attaches the claim fields attaches the ledger in
  the same place (`fresh_report_json`, `src/verify.rs:810-830` — "the ONE
  place these fields are attached to a persisted report").
  - Alternatives considered: deriving the ledger twice, per surface
    (rejected — two implementations is exactly the drift `visible_scope`
    exists to prevent); attaching it only to the verify verdict (rejected —
    the model report is where the prose-only guards first become visible,
    as `invariants_checked: []`).
- **Decision: an empty ledger is a positive statement, and it is not a
  claim about status.** An empty trust base means the corpus declares no
  unchecked claim and no machine-uninterpreted guard. It says nothing about
  a claim's verification status — `unknown` and refuted claims are reported
  by the verdict's own status vocabulary, and an unknown claim blocks
  acceptance without ever appearing in the ledger. The fields are always
  serialized (an omitted ledger is not an empty one), unlike the optional
  vectors in `model_check`'s `RunReport`.
- **Decision: the human view counts and truncates by a stated rule.** The
  human output names the member count per kind and the first ten members in
  the deterministic order `(kind, source file id, row id)`, and says the
  list is truncated; the JSON keeps every member. Advisory constraints live
  in the ledger forever, so a count and a cap are what keep the surface
  readable.
- **Decision: `prose_only_guard` means "no resolvable condition".** A
  transition's guard is an assumption only when it is prose and cites no
  evaluable constraint — no citation expression, no kernel binding, no
  executable invariant (`ModelIr::guard_citations` / `guard_kernel` /
  `invariants`, `src/compile.rs:138-170`). A guard citing an evaluable
  constraint, or a State "has reached state X" progress marker, is not
  listed: it is resolved, so nothing is being trusted unchecked.

### Shape (pinned)

```json
"trust_base": {
  "trust_base_schema_version": 1,
  "unverified_claims": [ { "id": "…", "kind": "prose_only_invariant", "source": "file.id" } ],
  "assumed":           [ { "id": "…", "kind": "prose_only_guard",     "source": "file.id" } ]
}
```

Members are sorted by `(kind, source file id, row id)`; both arrays are
always present, empty included.

## Risks / Trade-offs

- Risk: the ledger reads as a failure list (a corpus with many advisory
  constraints looks bad) → the human view's per-kind counts and cap, and
  the docs state that a non-empty ledger is information, not a verdict.
- Risk: an empty ledger is over-read as "the implementation is correct" →
  the release wording keeps the existing boundary explicit, and the
  constraint text says the emptiness says nothing about a claim's status.
- Risk: every stored report becomes stale at adoption → accepted: it is a
  one-time re-run, and the alternative (absent read as empty) would make the
  ledger lie on exactly the reports it is meant to qualify.
- Risk: report-schema churn for consumers → purely additive; no existing
  field changes name, type, or meaning, and the claim schema version does
  not move.

## Falsification tests

Each decision is stated with the evidence that would kill it:

- *Derived, not stored* — falsified if the unchecked set already carried
  every ledger member. It does not: `src/verify.rs:495-506` shows the
  unchecked set holds prose-only invariants alone, so the ledger adds
  `foreign_extension_point`, `advisory_constraint` and `prose_only_guard`
  members.
- *Own version, claim schema untouched* — falsified if the ledger had to be
  interpreted together with the claim records. It does not: the ledger is
  derived from rows, not from claim statuses.
- *One derivation, two carriers* — falsified if the two surfaces could
  legitimately differ. They cannot: both describe the same corpus.
- *Empty is positive* — falsified if an empty ledger implied every claim
  verified. It does not: an `unknown` claim blocks acceptance and never
  appears in the ledger.

## Open Questions

- Should the ledger feed `spk doctor --advice` (the parked
  `add-doctor-advice`)? Deferred — the advice profiler is a separate
  decision surface and the ledger's consumers are report surfaces.
- Should ledger members be ranked or weighted? Deferred — no ranking means
  no scoring semantics to keep honest; a rank would be a score in disguise.
- Should a non-empty ledger ever block `verified`? Deferred to a later
  change: v1 reports, and the acceptance conjunction (`both_gates_required`)
  stays exactly as it is.
