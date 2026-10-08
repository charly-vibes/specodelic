---
tags: [pipeline-run:tdd-ro5-2026-10-08-specodelic-m6k-stale-compile-demotes-malformed-kernel-claim-to-unchecked-f2, pipeline-step:quality-ledger]
---

QUALITY LEDGER specodelic-m6k: RED — cargo test --test cli stale_compile_with_kernel_grammar_broken_cell_fails_labeled failed for the right reason (exit 0/no_counterexample silent demotion). GREEN — same test passes after kernel::grammar_failures + cmd_model_check pre-check; all 47 model_check CLI tests green. FULL — just test exit 0; just ci exit 0 (fmt, clippy -D warnings, tests, release build); pretender check exit 0 with 0 new violations. Manual repro: compile ok → sed bogusfn → model-check exits 1, error envelope, stage kernel_grammar naming 'unknown atomic bogusfn'. Boundary: prose-only edit stays unchecked, no_counterexample, exit 0.
