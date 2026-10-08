---
tags: [pipeline-run:tdd-ro5-2026-10-08-specodelic-7gh-claim-records-carry-reason-when-not-verified-f4-f6, pipeline-step:red]
---

RED: command='cargo test --test cli model_check::dangling_kernel_endpoint' + unknown_kernel_claim — expected failure confirmed: unknown claim records carry no reason ({id,status} only), panic messages show the bare records. Setup intact (compile+lint pass, outcome exploration_only as expected).
