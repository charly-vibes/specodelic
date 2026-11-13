---
tags: [pipeline-run:tdd-ro5-2026-11-13-specodelic-3632-lint-revision-seams-for-the-four-blocked-rows, pipeline-step:green]
---

GREEN: the row-shape rule (linter.row_shape_closed) reads each kind's allowed field set from the Schema-as-data's reference columns plus a small per-kind base-field table; the revision-pair seam (specodelic::lint::revision::check_pair) diffs the governed id-sets (State ids, Constraint/Property kind values, Reference Typing fields) across two revisions and reports Removed / GrownSilently. All four blocked property rows now carry executable **rust:** predicates that call those seams over real fixtures; the red tests in tests/specodelic_fixtures.rs pass (28 passed).
