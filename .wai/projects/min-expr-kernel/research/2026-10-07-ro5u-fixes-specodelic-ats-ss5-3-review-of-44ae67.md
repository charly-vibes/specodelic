---
tags: [pipeline-run:tdd-ro5-2026-10-07-specodelic-ats-add-min-expr-kernel-ss5-3-external-claim-guidance-binding-path-coverage-lint-turn-ons-no-contract-tomls, pipeline-step:fix-review]
---

RO5U-FIXES: specodelic-ats §5.3 — review of 44ae671: 0 critical, 0 high. Medium 1 (checklist mapped_ids anchored to compile.constraint_table_to_toml rather than a kernel.binding-specific row) — deferred with reason: the kernel.binding requirement is change-phase (specodelic-bxk archive lands it) and linter-external_completeness resolves mapped_ids only inside the linted specs/ tree; re-anchor post-archive if it deploys as its own row. Low 1 (no specs/CHANGELOG.md entry) — tracked, not fixed: file outside the ticket's hard-scope allowed list; orchestrator may add a line at wrap. No behavior-impacting fixes → no test changes; gates re-verified after review (no new diff).
