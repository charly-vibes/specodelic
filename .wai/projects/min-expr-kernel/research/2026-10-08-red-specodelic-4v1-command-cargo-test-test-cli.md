---
tags: [pipeline-run:tdd-ro5-2026-10-08-specodelic-4v1-failures-ride-success-shaped-envelopes-f1, pipeline-step:red]
---

RED specodelic-4v1: command='cargo test --test cli -- refuted_kernel_claim_exits_1 kernel_grammar_failure verify_blocked_is_an_error orchestrate_failed_overall model_check_without_compiled_artifact model_check_rejects_a_spec' expected failure=7 failed / 0 passed — refuted claim exits 0 not 1; compile/verify/orchestrate labeled failures emit ok:true envelopes; stale/missing-artifact labeled failures emit ok:true. New tests: tests/cli/compile.rs (new module), tests/cli/model_check.rs, tests/cli/orchestrate_verify.rs. Counter-tests (success stays ok:true exit 0) pass as expected.
