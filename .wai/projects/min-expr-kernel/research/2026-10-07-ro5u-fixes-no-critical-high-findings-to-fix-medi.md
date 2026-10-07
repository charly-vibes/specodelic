---
tags: [pipeline-run:tdd-ro5-2026-10-07-specodelic-mui-ss3-7-kernel-claims-across-the-complete-invocation-corpus, pipeline-step:fix-review]
---

RO5U-FIXES: no critical/high findings to fix. Medium: none. Low: 3 recorded — the two duplication items (merged-status derivation in two call sites, CorpusClaimStatus mirroring ResolvedStatus) are deferred to §3.8 specodelic-k3h per the tasks.md amendment (TIDY is its own ticket); the inline unsupported label stays a string until a spec pins it. No code changes in this step; tests unchanged and green.
