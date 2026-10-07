---
tags: [pipeline-run:tdd-ro5-2026-10-07-specodelic-mui-ss3-7-kernel-claims-across-the-complete-invocation-corpus, pipeline-step:red]
---

RED: command='cargo test --test kernel_corpus' — 5 of 6 failed, each at assert_exactly_once with invariant_statuses=[] — kernel claims omitted from CLI JSON, persisted .check.json, and orchestrate stage detail (the exact §3.7 gap); the empty-input pin passed (already-correct invocation error). GREEN: added kernel::evaluate_corpus_claims (one KernelEnv over ALL specs; TLC → labeled kernel_backend_unsupported unknowns), threaded through cmd_model_check and orchestrate run_model_check_stage; CLI statuses now derive from the fully merged report (exec + citations + kernel) per design D9's identical-statuses clause. Same command now 6/6 pass; tests/cli 193 pass, lib 360 pass.
