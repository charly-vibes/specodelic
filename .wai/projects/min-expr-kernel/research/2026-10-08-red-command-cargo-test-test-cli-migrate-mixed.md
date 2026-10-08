---
tags: [pipeline-run:tdd-ro5-2026-10-08-specodelic-54v-mixed-delta-migrate-mirror-fails-lint-added-modified, pipeline-step:red]
---

RED: command='cargo test --test cli migrate_mixed_delta' — migrate_mixed_delta_lints_clean fails because spk lint exits 1 with linter.requirement_drift ('Two from ## MODIFIED Requirements is missing from ## Requirements') after spk migrate exits 0 — the literal ticket repro. Unit tests in src/migrate.rs additionally fail to compile on missing MigrateError::ConflictingDelta variant (new labeled refusal API).
