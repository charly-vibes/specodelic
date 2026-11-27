# Tasks: add-verification-trust-ledger

Staged slices — each is an independent red→green→refactor cycle and is
shippable alone. No slice touches the specodelic format: there is no new
column, no new kind, and no `specodelic.md` Revision.

Scope anchors (spec rows): `trust_base_partitioned`, `trust_base_derived`,
`trust_base_versioned`, `run_limits_excluded`, `trust_base_views_agree`,
`empty_trust_base_meaning`, `trust_base_corpus_scoped`,
`trust_base_shared_derivation` — all in the change's
`specs/verification-claims/layer.md`.

## Slice 1 — the ledger model and its derivation (`src/verify.rs`)

- [x] 1.1 Red: a corpus with one prose-only invariant and one prose-only
      guard — the verdict carries `trust_base.unverified_claims` (the
      prose-only invariant, kind `prose_only_invariant`) and
      `trust_base.assumed` (the guard, kind `prose_only_guard`); a member
      never appears in both.
      → `trust_base_partitioned`
- [x] 1.2 Red: the unchecked set and the `prose_only_invariant` members are
      derived from one source — the two never disagree.
      → `trust_base_derived`
- [x] 1.3 Green: the ledger types (`TrustBase`, `TrustMember`, the two
      closed kind sets) and the derivation from parsed rows; always
      serialized, empty included.
- [x] 1.4 Refactor: extract the derivation into a pure function of the
      corpus rows (no CLI state, no filesystem access) so the sort order
      `(kind, source file id, row id)` is testable in isolation — the
      analogue of `report_claim_fields` (`src/verify.rs:810`).

## Slice 2 — the remaining kinds, and keeping run limits out

- [x] 2.1 Red: a row with `satisfies` pointing at a foreign
      `extension_point` — listed in `unverified_claims` with kind
      `foreign_extension_point`, never in `assumed`.
- [x] 2.2 Red: a file with an `advisory` Constraint — listed in
      `unverified_claims` with kind `advisory_constraint`.
- [x] 2.3 Red: a bounded model-check run — the bound and the artifact digest
      ride as run metadata and appear in neither array.
      → `run_limits_excluded`
- [x] 2.4 Red: a member coming from another file in the scope carries that
      file's id, and the ledger follows the corpus.
      → `trust_base_corpus_scoped`
- [x] 2.5 Green: extend the kind sets and the derivation; carry the bound
      and digest into the model-check report payload beside the verdict.
- [ ] 2.6 Refactor: one kind-set table shared by the derivation, the JSON
      serializer, and the human renderer — no per-view kind lists.

## Slice 3 — guards the citation algebra can resolve

- [x] 3.1 Red: a transition whose guard cites an evaluable constraint, or is
      a State "has reached state X" progress marker — absent from `assumed`.
      A guard that is prose and cites nothing evaluable is present.
      Ground the check on `ModelIr::guard_citations` / `guard_kernel` /
      `invariants` (`src/compile.rs:138-170`).
- [x] 3.2 Green: the guard classification reads the compiled model IR, not
      the guard's text.
- [x] 3.3 Refactor: share the guard classification with the model-check
      backend's citation algebra instead of re-deriving "evaluable" here.

## Slice 4 — version, scope binding, and the three views

- [ ] 4.1 Red: a stored report with **no** trust base — verify refuses it
      as stale with a rerun hint; it is never read as an empty ledger, and
      `claim_schema_version` stays 1.
      → `trust_base_versioned`
- [ ] 4.2 Red: a stored report whose `trust_base_schema_version` is
      unrecognized — refused as stale, same hint.
- [ ] 4.3 Red: a stored report whose trust base disagrees with the
      recomputation — refused as a ledger mismatch, with the same rerun
      hint the unchecked-set comparison uses (`src/verify.rs:495-506`); the
      stored report is left unmodified.
      → `trust_base_derived`
- [ ] 4.4 Red: the same corpus presented in a different file order — the
      recomputed ledger is identical member for member and the persisted
      report is byte-identical.
- [ ] 4.5 Red: CLI JSON, human output, and the saved report name the same
      members with the same kinds; no view reports an empty trust base while
      another names an entry.
      → `trust_base_views_agree`
- [ ] 4.6 Red: more than ten members — the human output names the count per
      kind and the first ten members in the deterministic order, and says the
      list is truncated; the JSON keeps every member.
- [ ] 4.7 Green: recompute-and-compare in `src/verify.rs`; the version gate
      beside the existing claim-schema gate (`src/verify.rs:369-379`); the
      ledger attached at the one write-side seam (`fresh_report_json`,
      `src/verify.rs:810-830`); human rendering in `src/human.rs`.
- [ ] 4.8 Red→Green: an empty trust base is serialized as an empty object,
      never omitted — an omitted ledger is not an empty one.
      → `empty_trust_base_meaning`
- [ ] 4.9 Refactor: fold the ledger into the existing
      `assurance_views_agree` comparison rather than adding a second
      agreement check; the model-check report and the verify verdict read the
      same derivation, never two.
      → `trust_base_shared_derivation`

## Slice 5 — docs and the layer sidecar

- [x] 5.1 The release assurance wording names the trust base: what a
      non-empty ledger reports, that it never blocks or grants `verified`,
      that an empty ledger says the corpus declares no unchecked claim and no
      machine-uninterpreted guard, and that neither says anything about the
      implementation's behaviour.
      → `visible_scope` (documentation clause)
- [ ] 5.2 `specs/verification-claims/layer.md` sidecar authored (done in
      this change) and validated fail-closed via
      `just archive-change id=add-verification-trust-ledger`.
- [ ] 5.3 `spk lint openspec/changes/add-verification-trust-ledger` is
      clean — the sidecar is a dual-format spec file and must pass the
      format's own linter.
- [ ] 5.4 `just ci` green.

## Out of scope (tracked separately)

- Checking the foreign extension point (a contract matcher — its own
  capability spec).
- Binding each lint rule to its owning corpus Constraint (`RULE_TABLE`).
- Any format-level trust annotation; any change to the required/unchecked
  claim semantics; any blocking behaviour on a non-empty ledger; any bump of
  the claim schema version.
