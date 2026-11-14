---
reviews: 2026-11-14-green-fix-the-revision-22-bump-moved-the-current.md
verdict: needs-work
severity: {critical: 0, high: 2, medium: 2, low: 1}
tags: [pipeline-run:tdd-ro5-2026-11-14-specodelic-mlc-consolidate-the-outbound-leaf-carve-out-into-one-normative-statement, pipeline-step:ro5u-review]
---

RO5U: two HIGH defects. H1 the one-home meter only counted standalone outbound lines, so six requirement bodies kept the restatement after the word was removed; fixed by adding the restatement-phrase check and rewriting the bodies. H2 the normative paragraph misstated the graph rule: guard edges are in the acyclic set and the traces_to/derives_from definition does not fit Transition rows; rewritten to cross-file reachability + the exact acyclic edge set. M1 regenerated specodelic/packs.toml after the packs expr edit. M2 scoped pack fields to resolving fields and named produces/emits correctly. L1 restored the observability requirement leaf typing. Verified: outbound test 1 passed; lib 437; cli 334; lint 0 issues; packs.toml regenerated.
