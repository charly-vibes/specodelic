---
tags: [pipeline-run:tdd-ro5-2026-10-07-specodelic-k3h-ss3-8-claim-identity-and-command-parity-during-cleanup, pipeline-step:fix-review]
---

RO5U-FIXES: no critical/high. M1 accepted (kernel.rs is the sanctioned home; citation_corpus.rs out of allowed scope). M2 deferred with reason + tracked as beads specodelic-dzn (P3: unify ResolvedStatus/CorpusClaimStatus shape mirror — needs citation_corpus.rs edits). L1/L2 tracked, not cheap enough to warrant churn on a behavior-pinned ticket. No behavior-impacting fixes → no new tests; suite re-run: cargo test --test claim_parity 3/3, clippy clean.
