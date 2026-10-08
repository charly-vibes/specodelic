---
tags: [pipeline-run:tdd-ro5-2026-10-08-specodelic-m6k-stale-compile-demotes-malformed-kernel-claim-to-unchecked-f2, pipeline-step:red]
---

RED: command='cargo test --test cli stale_compile_with_kernel_grammar_broken_cell_fails_labeled' expected failure=exit 0 + no_counterexample (silent demotion), got exactly that — assertion 'left: Some(0), right: Some(1)'. Boundary test prose_only_cell_edit_stays_unchecked_not_labeled passes (never-required anti-goal already correct).
