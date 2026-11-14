---
tags: [pipeline-run:tdd-ro5-2026-11-14-specodelic-cbh-kernel-grammar-gate-placement-derive-typed-reference-columns-from-schema, pipeline-step:ro5u-review]
---

RO5U-FIXES: all four findings fixed. M1 corrected the kernel.md Notes + decision note: grammar is gated by spk compile and just ci's model-check kernel_grammar pre-check; spk verify never re-compiles and sees a kernel edit only as stale evidence. M2 restored the accidentally removed mermaid classDef assertion in render_wiring_labels_empty_views. L1 replaced the tautological coverage loop with exempt.is_subset(&schema_cols) plus the exact-set pin. L2 corrected the uses exemption citation to graph.md omission + packs.md uses_is_outbound_leaf. Verified: cargo test --lib graph 20 passed; cargo test --test acset_parity 18 passed; fmt clean; corpus lint 0 issues.
