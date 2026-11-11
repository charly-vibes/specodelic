---
tags: [pipeline-run:tdd-ro5-2026-11-11-specodelic-qqgn-unify-row-emission, pipeline-step:refactor]
---

REFACTOR: the extraction IS this ticket's refactor step (combined green+refactor per the gheu/8ebr precedent) — audit_row/recount/sort_rows landed in ea17b6b; the step's tidy pass found no further in-scope simplification (pack_check_row's shown/violated sort+dedup pairs are two terse call sites; extracting them would be churn). Found + left alone: pre-existing empty-line-after-doc-comment in pack_row_tests — fires only under clippy --all-targets (just ci clippy is lib+bins only), unrelated to 2.3, tracked for ro5u. rustfmt normalization of the new pins committed separately (437abd1, 8a9c6a3 precedent). Tests re-run green after every step: cargo test --lib audit 16/16.
