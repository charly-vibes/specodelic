---
tags: [pipeline-run:tdd-ro5-2026-10-07-specodelic-36t-ss4-3-shared-fixture-module-named-case-law-promotion-path, pipeline-step:red]
---

RED specodelic-36t: command='cargo test --test kernel_grounding --test kernel_status' → both crates fail E0432 unresolved import common::kernel_corpora — the expected missing shared module, not setup breakage (baseline 18 tests were green pre-move). kernel_grounding.rs and kernel_status.rs now import their corpus constants from common::kernel_corpora; fixtures not yet moved.
