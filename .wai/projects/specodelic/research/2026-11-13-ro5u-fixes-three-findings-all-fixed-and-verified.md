---
tags: [pipeline-run:tdd-ro5-2026-11-13-specodelic-3632-lint-revision-seams-for-the-four-blocked-rows, pipeline-step:fix-review]
---

RO5U-FIXES: three findings, all fixed and verified by the test suite. (1) MEDIUM — src/lint/mod.rs: demoted 'pub mod row_shape;' to 'mod row_shape;' so the crate's public surface stays the revision seam (the only seam the property fragments call). (2) LOW — src/lint/revision.rs: extracted has_new_revision_heading(prior, current) and used it both in diff_id_sets and in check_pair's Reference Typing branch, removing the duplicated computation. (3) LOW — src/lint/row_shape.rs: removed the dangling empty doc line at the end of allowed_fields' doc comment. After the fixes: cargo fmt clean, cargo clippy -- -D warnings clean, cargo test --lib 428 passed, tests/specodelic_fixtures.rs 28 passed, tests/conformance_matrix.rs 1 passed.
