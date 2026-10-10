---
tags: [pipeline-run:tdd-ro5-2026-10-10-specodelic-eczv-4-conform-phase-4-cli-wiring, pipeline-step:green]
---

GREEN: added Conform verb to Commands (READ-ONLY help text), dispatch arm, and cmd_conform in src/main.rs — invocation errors (unreadable oracle, unparsed/zero/multi-spec batch) exit 2 with zero records; conform::run drives gate→classify→report; exit 1 flips envelope to error kind via commands::as_failure (specs/errors.md envelope_error_kind) rendering conform::human_view, exit 0 rides conform::emit_report; exit code = forbidden+unsupported counts > 0. Narrow command 'cargo test --test cli conform' — 6 passed
