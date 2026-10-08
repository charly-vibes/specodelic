---
tags: [pipeline-run:tdd-ro5-2026-10-08-specodelic-0zk-graph-projection-exit-code-semantics-f3-f8, pipeline-step:refactor]
---

REFACTOR: extracted named_roots(paths) helper in src/main.rs — the named-roots computation was duplicated between collect_specs and cmd_graph_projection's F3 readability check; single definition now. All suites stay green (just test, clippy -D warnings, fmt).
