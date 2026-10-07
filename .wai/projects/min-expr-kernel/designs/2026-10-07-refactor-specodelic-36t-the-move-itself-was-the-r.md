---
tags: [pipeline-run:tdd-ro5-2026-10-07-specodelic-36t-ss4-3-shared-fixture-module-named-case-law-promotion-path, pipeline-step:refactor]
---

REFACTOR specodelic-36t: the move itself was the refactor — duplication removed (no corpus consts remain in kernel_grounding.rs / kernel_status.rs; only their id string literals in assertions, which keep meaning). Tests green after refactor: cargo test --test kernel_grounding --test kernel_status --test kernel_agreement = 18 passed 0 failed; just test-smart exit 0 (testaruda adapter, 12 results ingested; the '180 matched no known item' warning is a pre-existing lib-unit-test ingest quirk, unrelated to this diff). No behavior expansion, no unrelated cleanup.
