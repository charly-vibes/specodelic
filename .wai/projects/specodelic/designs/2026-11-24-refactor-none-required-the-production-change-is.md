---
tags: [pipeline-run:tdd-ro5-2026-11-24-4yve-just-archive-change-accepts-the-id-id-invocation-form, pipeline-step:refactor]
---

REFACTOR: none required — the production change is two edits (one 20-line shell script, one recipe line plus an explanatory comment) and the test file mirrors the repo's existing drift-guard pattern (tests/ci_wiring.rs). Names are already the domain's ('change_id', 'archive-companion'), no duplication to extract (the two test helpers differ: script-only vs whole-recipe-with-stub-cargo), and no behavior was expanded. Post-refactor check: cargo test --test archive_change_recipe -> 7 passed / 0 failed; cargo fmt --check on the new test file -> clean (exit 0).
