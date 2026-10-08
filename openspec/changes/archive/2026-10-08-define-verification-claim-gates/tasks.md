# Tasks: define verification claim gates

Reviewed and ticketed at the user’s request; implementation has not begun.
Integrate add-min-expr-kernel 3.6–3.8 first. Each RED→GREEN cycle and each
separate TIDY ticket is mapped below; keep refactoring commits separate. Preserve current source-file intent headers.

## 1. Mixed claims govern the verdict
- [x] 1.1 RED: command fixtures for false/unknown/missing/all-true claims;
      assert both report and exit behavior, including false negated citation.
- [x] 1.2 GREEN: implement required-claim classification and shared aggregate
      rules; retain explicitly unchecked prose and bounded-run semantics.
- [x] 1.3 TIDY: separate ticket/commit consolidates aggregate rendering only
      after the command fixtures pass unchanged.

## 2. Evidence belongs to the current scope
- [x] 2.1 RED: old report, missing/duplicate claim, reversed file order,
      cross-file edit, missing input, and artifact mismatch fixtures. Two
      dual-format files with identical local claim IDs but opposite outcomes
      fail combined invocation as isolated_scope_required; separate runs
      preserve outcomes, differ in digest and reject swapped reports. Pin
      the same preflight in CLI/orchestrate and preserve multi-file lint.
- [x] 2.2 GREEN: versioned report, canonical input digest, live required-set
      comparison, and rerun hints. No hashes synthesized for old reports.
- [x] 2.3 TIDY: separate ticket/commit shares scope encoding across commands.

## 3. Users can interpret results
- [x] 3.1 RED: assert JSON/persisted/human claim counts agree; fail the docs
      check when version or capability status conflicts with the release.
- [x] 3.2 GREEN: show blockers/unchecked claims and synchronize README,
      docs/src/status.md, openspec/project.md, specs/STATUS.md, and embedded
      guide. Document the distinction from application-test execution.
- [x] 3.3 TIDY: separate documentation cleanup commit removes stale restatements.

## 4. Release discipline
- [x] 4.1 Update domain specs under their Revision rules and regenerate affected
      artifacts; preserve deployed requirements and scenario identities.
      → 93fb322 — claim-gated verification lands in the two capability
      files that already own the gates' precise meanings, per AGENTS.md
      rule 3a (widen, don't restate): model_check.md gains
      required_claims_classified, claim_aggregate_governs,
      claim_report_schema, dual_format_isolated_scope (+ finish_clean
      guard citing the aggregate; 9 deriving properties);
      verify.md gains required_claims_govern_acceptance,
      evidence_scope_bound, assurance_views_agree (accept/reject
      transitions cite all three; 7 deriving properties). NO
      specodelic.md edit → NO Revision 18: no governed id-set is
      touched, `no_counterexample`'s expr is unchanged, and the
      guide-drift guard stays green at Revision 17 (decision recorded
      in both files' Notes). CHANGELOG #116, STATUS inventory rows;
      artifacts regenerated (spk compile specs / model-check specs:
      22 checked, 0 failed; lint specs: 0 issues).
- [x] 4.2 Author genuine scenario contracts bound to the new tests; update
      docs SUMMARY for the new capability; run ah check --changes
      define-verification-claim-gates and executed checks when supported.
      → c00e1ec — 10 scenario contracts staged change-scoped
      (.espectacular/changes/define-verification-claim-gates/
      verification-claims/), one per deployed scenario, 31 cargo test
      bindings across tests/cli/{model_check,orchestrate_verify,
      citation_resolution}.rs; ah check --changes
      define-verification-claim-gates --run-tests: 243 passed,
      0 findings (0 orphans, 0 missing bindings). SUMMARY entry lands
      with the archive commit (bxk 91ae76b precedent —
      summary-completeness requires openspec/specs/verification-claims/
      spec.md to exist first).
- [x] 4.3 Run strict OpenSpec validation, section sync, corpus lint and just ci;
      archive only via the dual-format recipe after implementation approval.
      → 9b622b5 (+staged-promotion commits) — just ci green pre-archive
      (214 contract tests, 0 findings); archive via
      `just archive-change define-verification-claim-gates`
      (openspec archive --skip-specs via archive-companion + verbatim
      deploy; dry-run first: restore plan =
      openspec/specs/verification-claims/spec.md). Verbatim parity
      verified by diff (archive delta == deployed spec, byte-identical):
      4 requirements, 10 scenarios, all identities preserved — 0
      deploy-time repairs needed. Contracts promoted via `ah archive`
      (10 → .espectacular/verification-claims/). SUMMARY entry added
      (summary-completeness: 0 missing). Post-archive gates: just ci
      green; ah check --run-tests 243 passed, 0 findings;
      openspec validate --all --strict green (25/25); lint specs
      0 issues. Mid-task, 60123fc (specodelic-75a, parallel gre.7
      session) repaired the 4.1 row's traces_to to [[model_check]] —
      artifacts regenerated after the repair (9b622b5).

## Implementation ticket map

Created at the user’s request after the proposal review fixes. Checkboxes
remain unchecked until the linked behavior is implemented and verified.
Dependencies are recorded in beads; this table maps scope, not completion.

| Tasks | Ticket | Outcome |
|-------|--------|---------|
| 1.1, 1.2 | `specodelic-68m.1` | False or unknown required claims block verification |
| 1.3 | `specodelic-68m.2` | Keep aggregate verdicts consistent across report views |
| 2.1, 2.2 | `specodelic-68m.3` | Verification rejects evidence from another input scope |
| 2.3 | `specodelic-68m.4` | Keep freshness decisions identical across commands |
| 3.1, 3.2 | `specodelic-68m.5` | Users see the same claim blockers in every report |
| 3.3 | `specodelic-68m.6` | Keep assurance guidance consistent after documentation cleanup |
| 4.1, 4.2, 4.3 | `specodelic-68m.7` | Strict verification policy ships with current contracts and guidance |
