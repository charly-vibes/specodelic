---
tags: [pipeline-run:tdd-ro5-2026-11-11-specodelic-8ebr-pack-check-rows, pipeline-step:refactor]
---

REFACTOR: intentionally deferred — the visible duplication (count-map construction duplicated between build_report and with_pack_rows; row-emission variance across row kinds) IS task 2.3 'unify row-emission code across row kinds', a separate ticket (specodelic-qqgn) per the brief's scope guard: 'Do NOT start 2.3'. No tidy commit — zero-change refactor step; tests re-confirmed green (cargo test --lib audit 12/12).
