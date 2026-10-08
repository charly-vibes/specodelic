---
tags: [pipeline-run:tdd-ro5-2026-10-08-specodelic-0zk-graph-projection-exit-code-semantics-f3-f8, pipeline-step:orient]
---

ORIENT: specodelic-0zk graph projection exit codes — cmd_graph_projection (src/main.rs:411) returns 0 on empty specs incl. nonexistent path; JSON cmd_graph (src/commands/corpus.rs:127) exits 2 on empty, 1 on violations. Fix: projection mirrors — nonexistent/unreadable roots exit 2 labeled, typing violations exit 1, violations still ride along (D3). Errors contract: 0/1/2 mapping.
