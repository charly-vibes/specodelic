---
tags: [pipeline-run:tdd-ro5-2026-11-13-specodelic-lf3-migrate-compile-properties, pipeline-step:red]
---

RED: command=cargo test --test compile_fixtures expected failure=the corpus-correspondence test fails before the migration — the compile file's property table named generator names with no fixture in the registry ('ghost' was missing), and every fixture except the two deliberately-lint-failing ones had to lint clean (the effect-row fragment fixture tripped linter.ref_kind_compatible, the guard-fragment fixture tripped linter.single_root_reachable). The second RED was the properties gate itself: before the migration, 34 of the 35 compiled blocks of openspec/specs/compile/spec.md failed (only the one demonstration row, proptest_block_per_property, carried an executable **rust:** fragment); the rest still emitted todo_predicate! placeholders. The narrow command that exposed both is cargo test --test compile_fixtures plus spk verify --json openspec/specs/compile/spec.md.
