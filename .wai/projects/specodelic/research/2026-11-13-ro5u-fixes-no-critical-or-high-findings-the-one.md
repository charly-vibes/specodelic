---
tags: [pipeline-run:tdd-ro5-2026-11-13-specodelic-lf3-migrate-specodelic-kinds-properties, pipeline-step:fix-review]
---

RO5U-FIXES: no critical or high findings. The one medium finding (four rows without an honest tool seam) is fixed in the change itself: their predicates keep the honest todo_predicate! placeholder and tests/specodelic_fixtures.rs names the exact set (the_unmigrated_rows_are_the_documented_four), so the deferral is explicit and on the record rather than silent. Low findings are tracked, not fixed: the unknown_kind_rejected fragment decides the closed-set claim through the tool's own schema checker (a deliberate, documented choice) and the corpus-wide scope_sha256 move is the expected consequence of a corpus change — all 22 specodelic/*.check.json reports are committed together so the evidence stays consistent. Tests after the fixes: cargo test --test specodelic_fixtures — 22 passed, 0 failed; cargo test — 50 suites green; just ci exits 0.
