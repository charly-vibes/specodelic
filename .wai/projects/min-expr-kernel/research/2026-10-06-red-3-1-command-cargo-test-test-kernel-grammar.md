---
tags: [pipeline-run:tdd-ro5-2026-10-06-add-min-expr-kernel-phase-3-specodelic-7ga-tier-b-kernel-core, pipeline-step:red]
---

RED 3.1: command='cargo test --test kernel_grammar' → compile failure E0432 (unresolved import specodelic::kernel) + E0609 (no field guard_kernel on ModelIr) — exactly the missing Tier B kernel behavior, not setup breakage. Test file tests/kernel_grammar.rs: 11 tests (closed set parse, non-member labeled naming atomic+set, malformed labeled, prose stays prose, citations not stolen, mid-span mention, byte-identical golden toml, extraction unchanged, kernel IR seam, ThreeValued reuse). Goldens captured from pre-phase-3 compiler.
