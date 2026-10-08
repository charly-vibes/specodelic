---
tags: [pipeline-run:tdd-ro5-2026-10-08-specodelic-0zk-graph-projection-exit-code-semantics-f3-f8, pipeline-step:fix-review]
---

RO5U: none critical/high. MEDIUM — (1) named_roots inserted between collect_specs' doc comment and body during refactor (orphaned doc comment) — fixed by moving named_roots above it. (2) exit-1 trigger is violations only per ticket scope: dangling refs and supersedes cycles do not flip the projection exit though JSON mode exits 1 for them too — deferred to a future ticket, not silently widened. LOW — (1) graph::build runs twice per projection invocation (violations check + inside the projection fn); corpus-sized, acceptable. (2) readability check is fs::metadata on named roots — covers nonexistent paths and most unreadable cases; metadata-passing-unreadable files share JSON mode's blind spot shape. (3) refusal envelope routed to stderr via writer-swap with discard sink (projection stdout stays the raw text channel), documented inline.
