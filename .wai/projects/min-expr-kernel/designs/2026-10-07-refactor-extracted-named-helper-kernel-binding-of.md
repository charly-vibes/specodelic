---
tags: [pipeline-run:tdd-ro5-2026-10-07-specodelic-bf5-ss5-1-5-2-opaque-kernel-binding-extraction-and-claim-carrier, pipeline-step:refactor]
---

REFACTOR: extracted named helper kernel_binding_of_row(kind, cell) in src/compile.rs — the extraction rule reads as one named predicate instead of an inline then/flatten closure; doc comment carries the D4 rationale (verbatim, never interpreted, kernel_cell_content fence precedent, pure widening). Tests stay green (7/7 kernel_binding, CLI surface ok); cargo fmt clean; just lint (clippy -- -D warnings, the just ci gate) clean. Note: clippy --all-targets -D warnings is red on HEAD pre-existing (dead-code consts in tests/common/kernel_fixtures.rs — out of scope); the project gate uses plain 'cargo clippy --'.
