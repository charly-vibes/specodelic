---
tags: [pipeline-run:tdd-ro5-2026-11-11-specodelic-37hl-rung-aggregate, pipeline-step:orient]
---

ORIENT: specodelic-37hl slice 4 (tasks 4.1-4.2) — audit ladder-rung aggregate, derived not stored (design D3): rung = highest rung with current evidence at report time; lint-clean floor 1, model-present 2, current model-check (no_counterexample/exploration_only/timed_out) 3, verify-pass 4 (no persisted verify output in v1 — unreachable through audit inputs, documented), current counterexample_found 5; unchecked/binds_nothing rows never lift; AuditReport gains a derived summary field (schema stays 1, serde default keeps old priors parseable); diff_with_prior never reads prior's rung
