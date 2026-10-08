---
tags: [pipeline-run:tdd-ro5-2026-10-08-specodelic-m6k-stale-compile-demotes-malformed-kernel-claim-to-unchecked-f2, pipeline-step:green]
---

REFACTOR: none beyond cargo fmt — the if-failed.is-empty guard wrapping the claim-gated pipeline is the minimal structure (pipeline must not run or write reports when the pre-check fails); cmd_model_check cyclomatic unchanged, pretender check 0 new violations.
