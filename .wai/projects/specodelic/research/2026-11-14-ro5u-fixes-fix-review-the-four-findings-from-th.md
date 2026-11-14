---
tags: [pipeline-run:tdd-ro5-2026-11-14-specodelic-cbh-kernel-grammar-gate-placement-derive-typed-reference-columns-from-schema, pipeline-step:fix-review]
---

RO5U-FIXES (fix-review): the four findings from the cbh review are applied and verified in this step — kernel.md/decision note now name compile + model-check's kernel_grammar pre-check (verify never re-compiles); the mermaid classDef assertion is restored; the coverage assertion is exempt.is_subset(schema_cols) plus the exact-set pin; the uses exemption cites graph.md omission + packs.md uses_is_outbound_leaf. Tests: cargo test --lib graph 20 passed; cargo test --test acset_parity 18 passed; cargo fmt --check 0; spk lint corpus 0 issues. No open findings.
