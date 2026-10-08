---
tags: [pipeline-run:tdd-ro5-2026-10-08-specodelic-4v1-failures-ride-success-shaped-envelopes-f1, pipeline-step:refactor]
---

REFACTOR specodelic-4v1: renamed the model-check aggregate flag refuted→not_clean (it covers every non-no_counterexample outcome, not only refuted claims); fixed the inline comment that claimed timed_out/exploration_only keep exit 0 (they do not — the broad rule tracks the orchestrate stage + verify gate); imported as_failure normally in corpus.rs; clippy-cleaned the new tests (is_some_and, no useless format!, removed an orphaned #[test]). No behavior change — full targeted suites green.
