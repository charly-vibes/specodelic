# Tasks: add-verification-trust-ledger

Staged slices — each is an independent red→green→refactor cycle and is
shippable alone. No slice touches the spec format: there is no new column, no
new kind, and no `specodelic.md` Revision.

Scope anchors (spec rows): `trust_base_partitioned`, `trust_base_derived`,
`run_limits_excluded`, `trust_base_views_agree`,
`empty_trust_base_meaning` — all in the change's
`specs/verification-claims/layer.md`.

## Slice 1 — the ledger model and its derivation

- [ ] 1.1 Red: a corpus with one prose-only invariant and one prose-only
      guard — the verdict carries `trust_base.unverified_claims` (the
      prose-only invariant, kind `prose_only_invariant`) and
      `trust_base.assumed` (the guard, kind `prose_only_guard`); a member
      never appears in both.
      → `trust_base_partitioned`
- [ ] 1.2 Red: the unchecked set and the `prose_only_invariant` members are
      derived from one source — the two never disagree.
      → `trust_base_derived`
- [ ] 1.3 Green: `src/verify.rs` — the ledger types (`TrustBase`,
      `TrustMember`, the two closed kind sets) and the derivation from
      parsed rows; always serialized, empty included.
- [ ] 1.4 Refactor: extract the derivation into a pure function of the
      corpus rows (no CLI state, no filesystem access) so the sort order
      `(kind, source file id, row id)` is testable in isolation.

## Slice 2 — the remaining kinds, and keeping run limits out

- [ ] 2.1 Red: a row with `satisfies` pointing at a foreign
      `extension_point` — listed in `unverified_claims` with kind
      `foreign_extension_point`, never in `assumed`.
- [ ] 2.2 Red: a file with an `advisory` Constraint — listed in
      `unverified_claims` with kind `advisory_constraint`.
- [ ] 2.3 Red: a bounded model-check run — the bound and the artifact digest
      ride as run metadata and appear in neither array.
      → `run_limits_excluded`
- [ ] 2.4 Green: extend the kind sets and the derivation; carry the bound
      and digest into the model-check report payload beside the verdict.
- [ ] 2.5 Refactor: one kind-set table shared by the derivation, the JSON
      serializer, and the human renderer — no per-view kind lists.

## Slice 3 — scope binding and the three views

- [ ] 3.1 Red: a stored report whose trust base disagrees with the
      recomputation — verify refuses with a rerun hint and leaves the stored
      report unmodified.
      → `trust_base_derived`
- [ ] 3.2 Red: the same corpus presented in a different file order — the
      recomputed ledger is identical member for member and the persisted
      report is byte-identical.
- [ ] 3.3 Red: CLI JSON, human output, and the saved report name the same
      members with the same kinds; no view reports an empty trust base while
      another names an entry.
      → `trust_base_views_agree`
- [ ] 3.4 Green: recompute-and-compare in `src/verify.rs`; human rendering
      in `src/human.rs` (counts plus the first entries and the file they
      come from).
- [ ] 3.5 Red→Green: an empty trust base is serialized as an empty object,
      never omitted — an omitted ledger is not an empty one.
      → `empty_trust_base_meaning`
- [ ] 3.6 Refactor: fold the ledger into the existing
      `assurance_views_agree` comparison rather than adding a second
      agreement check.

## Slice 4 — docs and the layer sidecar

- [ ] 4.1 The release assurance wording names the trust base: what a
      non-empty ledger reports, that it never blocks or grants `verified`,
      and that neither an empty nor a non-empty ledger says anything about
      the implementation's behaviour.
      → `visible_scope` (documentation clause)
- [ ] 4.2 `specs/verification-claims/layer.md` sidecar authored (done in
      this change) and validated fail-closed via
      `just archive-change id=add-verification-trust-ledger`.
- [ ] 4.3 `spk lint openspec/changes/add-verification-trust-ledger` is
      clean — the sidecar is a dual-format spec file and must pass the
      format's own linter.
- [ ] 4.4 `just ci` green.

## Out of scope (tracked separately)

- Checking the foreign extension point (a contract matcher — its own
  capability spec).
- Binding each lint rule to its owning corpus Constraint (`RULE_TABLE`).
- Any format-level trust annotation; any change to the required/unchecked
  claim semantics; any blocking behaviour on a non-empty ledger.
