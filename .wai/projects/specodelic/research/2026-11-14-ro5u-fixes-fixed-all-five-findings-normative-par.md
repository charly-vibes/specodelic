---
tags: [pipeline-run:tdd-ro5-2026-11-14-specodelic-mlc-consolidate-the-outbound-leaf-carve-out-into-one-normative-statement, pipeline-step:ro5u-review]
---

RO5U-FIXES: fixed all five findings. Normative paragraph now states cross-file reachability and the separate acyclic edge set (guard stays in it); field list scoped to cross-file guard + satisfies/observes/uses + pack reference fields resolving to declared rows (opaque carriers excepted). Test now also rejects the restatement phrases (no reachability path / no acyclic edge set / never reachability path); six requirement bodies rewritten to pointers; packs and linter-graph_shape lines de-restated; observability requirement carries the leaf pointer. Regenerated specodelic/packs.toml. Verified: cargo test --test outbound_single_source 1 passed; cargo test --lib 437 passed; cargo test --test cli 334 passed; just lint-specs 0 issues.
