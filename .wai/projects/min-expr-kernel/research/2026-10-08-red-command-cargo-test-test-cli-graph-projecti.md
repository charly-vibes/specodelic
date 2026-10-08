---
tags: [pipeline-run:tdd-ro5-2026-10-08-specodelic-0zk-graph-projection-exit-code-semantics-f3-f8, pipeline-step:red]
---

RED: command='cargo test --test cli graph_projection' expected failure=F3 nonexistent-path projection exits 0 (want 2, both edges and dot/mermaid legs) and F8 violations projection exits 0 (want 1, edges + wiring views); guards pass (existing_empty_path exits 0, JSON mode still 1). Raw repro: 'spk graph /nonexistent --format edges' exit 0, 'spk graph tests/fixtures/typing_violations --format edges' exit 0.
