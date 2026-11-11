---
tags: [pipeline-run:tdd-ro5-2026-11-11-specodelic-gheu-extract-report-diff-into-testable-unit, pipeline-step:green]
---

GREEN: no production change — the 6 diff-unit pins pass against the current code (diff_with_prior + parse_prior_report have been standalone, envelope-decoupled, module-level pub since a435157), which is the definition of green for a refactor ticket per the brief ('watch it pass, move the code, watch it still pass'). Narrow: cargo test --lib audit::diff_tests 6/6; suite: cargo test --test audit_oracle_rows 14/14. The MOVE (overlay_class + ensure_diffable extraction out of diff_with_prior's body) is deliberately the next step's refactor commit, keeping red→green→refactor commits separate.
