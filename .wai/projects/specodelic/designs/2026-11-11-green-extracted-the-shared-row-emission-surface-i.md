---
tags: [pipeline-run:tdd-ro5-2026-11-11-specodelic-qqgn-unify-row-emission, pipeline-step:green]
---

GREEN: extracted the shared row-emission surface in src/audit.rs (commit ea17b6b after red 1a8788d): audit_row() — single construction site, class always AuditClass::of(verdict) by construction, consumed by build_report's oracle map and pack_check_row's two branches; recount() — single count-map builder replacing the duplicated loops in build_report + with_pack_rows (the duplication 8ebr explicitly deferred here); sort_rows() — single key-sort discipline replacing 3 copies. diff_with_prior's class-only recount is a diff concern, left alone. Narrow: cargo test --lib audit 16/16; full cargo test green; ZERO-BEHAVIOR PROOF: 5 fixed-fixture CLI invocations at /var/tmp/qqgn-proof (fresh/human/baseline/stale-diff/digest-refusal, exits 1/1/0/1/2) byte-identical before-vs-after via diff.
