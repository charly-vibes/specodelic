---
tags: [pipeline-run:tdd-ro5-2026-10-08-specodelic-m6k-stale-compile-demotes-malformed-kernel-claim-to-unchecked-f2, pipeline-step:green]
---

GREEN: kernel::grammar_failures(spec) helper (mirrors compile's kernel_grammar_violation_labeled for every invariant row that opts in via **kernel:** and breaks the closed grammar) wired into cmd_model_check as a pre-run guard — labeled failures ride .data.failed (stage kernel_grammar) with exit 1 before any run/report write; the claim-gated pipeline only runs when the pre-check is clean. Both new tests green; all 47 model_check CLI tests green; manual repro now exits 1 with error envelope naming bogusfn.
