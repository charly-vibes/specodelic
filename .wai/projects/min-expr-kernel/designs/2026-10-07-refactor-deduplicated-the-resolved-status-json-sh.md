---
tags: [pipeline-run:tdd-ro5-2026-10-07-specodelic-hb4-add-min-expr-kernel-ss3-6-citation-resolution-in-commands, pipeline-step:refactor]
---

REFACTOR: deduplicated the resolved-status JSON shape into ResolvedStatus::output_json (commands + orchestrate now share it) and the scope-failure envelope into commands::scope_violation_output (one emission shape for model-check/verify/orchestrate). No behavior change; tests green (13/13 cli, pretender gate ok), fmt clean.
