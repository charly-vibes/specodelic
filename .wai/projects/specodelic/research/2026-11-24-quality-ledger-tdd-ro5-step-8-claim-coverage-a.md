---
tags: [pipeline-run:tdd-ro5-2026-11-24-claim-coverage, pipeline-step:quality-ledger]
---

QUALITY LEDGER (tdd-ro5 step 8, claim coverage / add-check-claim-coverage scope A).

Changed:
- src/claim_coverage.rs (NEW) — the claim-coverage section: declared invariant claims, the placement rule (direct naming or one-hop property->derives_from), the strongest-class fold over the full closed set (contradicted > stale > current > new > binds_nothing > unchecked), the residual bucket, class counts, machine-readable scope (SCOPE_ID 'this_run_producers' + SCOPE_STATEMENT).
- src/check.rs — CheckPayload{#[serde(flatten)] report, claim_coverage}; check::payload/human_view; emit_judged/emit_payload render the payload through one path; AuditReport untouched; exit-2 refusals unchanged (zero records, no section).
- src/audit.rs — human_view_labelled delegates to new human_view_labelled_extended(label, report, verdict_summary, section_lines); spk audit output byte-identical (empty additions).
- src/lib.rs — claim_coverage module registered.
- tests/check_claim_coverage.rs (NEW) — 12 tests pinning placement, residuals, fold, file-qualified keys, near-miss non-placement, scope field, envelope shape + prior-parse interop, verdict-line counts, closed vocabulary, byte-stability, non-gating.
- docs/src/graded-evidence.md — new section 'The claim-coverage section' (task 6.1, incl. the safe-by-default residual property for future producers = EXCL-001).
- openspec/changes/add-check-claim-coverage/{design.md (D9 amendment + rationale), specs/check/spec.md (fold order + rewrap), specs/check/layer.md (claim_coverage_listed expr), tasks.md (7.1 five-requirements text)}.
- scripts/check_evidence_axes.py: passes unchanged (task 6.2, no new value row).

Verified:
- cargo test --test check_claim_coverage — RED first (compile failure: unresolved claim_coverage, missing check::payload/human_view), then GREEN 12/12.
- cargo test --lib (460) + check_rows/check_matrix/check_contract_exit/check_contract_oracle/check_contract_verify + audit_* suites — all green.
- cargo clippy --all-targets -- -D warnings — exit 0 (one collapsible_if fixed in refactor step). cargo fmt --check clean.
- openspec validate add-check-claim-coverage --strict — valid (re-run after the CLAR-001 rewrap).
- cargo run -q -- lint openspec/specs — 50 files, 0 issues (warnings channel: pre-existing advisories only).
- python3 scripts/check_evidence_axes.py — OK: 6 verdicts, 6 audit classes, 6 of() arms vs 16 table rows.
- just ci — exit 0, incl. ah check --run-tests: 742 passed, 0 findings.
- ah check --changes add-check-claim-coverage — 14 no-toml findings (EXPECTED: the delta's 14 scenarios awaiting the sequenced per-scenario contract authoring follow-up per the 2026-10-01 espectacular adoption decision; matches the count recorded at authoring time in the prior session).

Review:
- RO5U (reviews/2026-11-24-ro5u-claim-coverage-scope-a-tdd-ro5-step-6.md): READY — 0 CRITICAL, 0 HIGH, 2 LOW; CONVERGED at stage 2. CLAR-001 fixed (rewrap); EXCL-001 deferred to task 6.1 (done in this run); EXCL-002 no action. TypeSafe pass: zero findings at/above VERIFY_SEVERITIES -> zero verification questions, all findings UNVERIFIED-below-severities by rule.

Risks:
- specodelic-qbox (0 of 50 corpus capability files judgeable; src/check.rs:242 corpus-less conform::gate) — untouched here by design (ruling D-5); the section cannot be exercised on the real corpus until it lands (tasks 6.5 self-hosting stays blocked).
- The 14 no-toml findings above — tracked by the sequenced espectacular contract-authoring follow-up, not this change.
- Archive (tasks 7.x) NOT run this session — the change stays in openspec/changes/ implemented-but-unarchived (repo precedent: add-evidence-record-axes completed-unarchived); archiving deploys the capability and is kept as an explicit step.

Next:
- ship-close: bd update specodelic-d5oa with evidence + close; bd export -> commit .beads/issues.jsonl; commit the diff atomically (only this session's files); git pull --rebase && git push; wai close; then wai pipeline next to complete the run.
