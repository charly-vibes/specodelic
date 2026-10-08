---
tags: [pipeline-run:tdd-ro5-2026-10-08-specodelic-4v1-failures-ride-success-shaped-envelopes-f1, pipeline-step:green]
---

GREEN specodelic-4v1: as_failure(Output) helper in src/commands/mod.rs flips envelope kind to error keeping payload (is_error=true, message first warning, verbosity 0). cmd_model_check: failed_run = !failed.is_empty() || any checked outcome != no_counterexample (matches orchestrate stage + verify gate — no view drift); next_step hint distinguishes labeled failures vs refuted claims; exit 1. cmd_verify: blocked non-empty → as_failure + exit 1. cmd_compile (corpus.rs): failed non-empty → as_failure + exit 1. cmd_orchestrate: overall != succeeded → as_failure + exit 1. Updated 20 existing CLI fixtures that encoded the old 'verdict lives in the report' convention (tests/cli/{model_check,orchestrate_verify,citation_resolution,tlc_observability}.rs). CLI suite: 258 passed.
