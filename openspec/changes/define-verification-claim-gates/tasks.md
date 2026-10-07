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
- [ ] 2.1 RED: old report, missing/duplicate claim, reversed file order,
      cross-file edit, missing input, and artifact mismatch fixtures. Two
      dual-format files with identical local claim IDs but opposite outcomes
      fail combined invocation as isolated_scope_required; separate runs
      preserve outcomes, differ in digest and reject swapped reports. Pin
      the same preflight in CLI/orchestrate and preserve multi-file lint.
- [ ] 2.2 GREEN: versioned report, canonical input digest, live required-set
      comparison, and rerun hints. No hashes synthesized for old reports.
- [ ] 2.3 TIDY: separate ticket/commit shares scope encoding across commands.

## 3. Users can interpret results
- [ ] 3.1 RED: assert JSON/persisted/human claim counts agree; fail the docs
      check when version or capability status conflicts with the release.
- [ ] 3.2 GREEN: show blockers/unchecked claims and synchronize README,
      docs/src/status.md, openspec/project.md, specs/STATUS.md, and embedded
      guide. Document the distinction from application-test execution.
- [ ] 3.3 TIDY: separate documentation cleanup commit removes stale restatements.

## 4. Release discipline
- [ ] 4.1 Update domain specs under their Revision rules and regenerate affected
      artifacts; preserve deployed requirements and scenario identities.
- [ ] 4.2 Author genuine scenario contracts bound to the new tests; update
      docs SUMMARY for the new capability; run ah check --changes
      define-verification-claim-gates and executed checks when supported.
- [ ] 4.3 Run strict OpenSpec validation, section sync, corpus lint and just ci;
      archive only via the dual-format recipe after implementation approval.

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
