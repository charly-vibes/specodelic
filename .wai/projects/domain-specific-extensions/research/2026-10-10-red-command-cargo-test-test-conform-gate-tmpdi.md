---
tags: [pipeline-run:tdd-ro5-2026-10-10-specodelic-eczv-3-conform-phase-3-input-gate, pipeline-step:red]
---

RED: command=cargo test --test conform_gate (TMPDIR=/var/tmp/specodelic-eczv) expected failure=missing behavior confirmed: no / in specodelic::conform (E0432) and orchestrate::run_lint_stage is private (E0603) — the gate API surface does not exist yet
