---
tags: [pipeline-run:tdd-ro5-2026-10-09-specodelic-eczv-1-conform-phase-1-verdict-engine-core, pipeline-step:fix-review]
---

RO5U-FIXES: F1 (High) deny_unknown_fields on TraceStep + refusal test with remediation hint — FIXED; F2 (High) cited_claim_ids foreign-prefix misattribution — resolves local claims only when prefix == spec.intent.id or bare target; foreign citations skipped — FIXED, test added; F3 (Low) dead bool param on reason::uncovered_forbidden — FIXED (cheap). F4 deferred with reason: classify_traces double IR extraction (cheap, pure, kept for testable &Spec public API); non-string observation values deferred to phase-3 corpus validation (task 3.3). Tests: RED verified against pre-fix HEAD (2 failed), post-fix 13/13 pass; just lint clean; pretender-check exit 0.
