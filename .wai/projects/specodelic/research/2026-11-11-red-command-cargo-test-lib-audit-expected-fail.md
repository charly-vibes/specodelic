---
tags: [pipeline-run:tdd-ro5-2026-11-11-specodelic-8ebr-pack-check-rows, pipeline-step:red]
---

RED: command=cargo test --lib audit; expected failure=compile errors on the not-yet-existing pack-row surface; actual=E0422/E0425 x19 — cannot find PackCheck, PackViolation, pack_check_rows, with_pack_rows in scope (src/audit.rs pack_row_tests only); lib itself compiles clean without the test target. Honest red = missing pack-row surface (slice-1 precedent f6aeabd).
