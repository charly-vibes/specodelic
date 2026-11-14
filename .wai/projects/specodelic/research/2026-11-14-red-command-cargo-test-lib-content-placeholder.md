---
tags: [pipeline-run:tdd-ro5-2026-11-13-specodelic-xci-advisory-warnings-for-placeholder-and-tautology-content, pipeline-step:red]
---

RED: command=cargo test --lib content_placeholder_and_tautology_warn_never_fail; cargo test --test cli lint_warns_on_scaffold_placeholder_and_tautology_but_exits_zero. Result: both FAIL for the expected missing behavior — the lib test panics on unwrap_p(None) because Report.warnings is empty, the CLI test exits 0 with empty warnings and no placeholder/tautology entries (assert at tests/cli/lint.rs:1427). Fixtures were corrected to cite their own invariant in the transition guard so single_root_reachable no longer masks the red (first run failed on that fixture defect; corrected and re-run). No production code touched.
