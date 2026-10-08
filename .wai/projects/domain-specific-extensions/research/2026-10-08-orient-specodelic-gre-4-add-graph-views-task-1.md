---
tags: [pipeline-run:tdd-ro5-2026-10-08-specodelic-gre-4-wiring-view-task-1-6, pipeline-step:orient]
---

ORIENT: specodelic-gre.4 = add-graph-views task 1.6 only — --view wiring, file-level constraints.satisfies projection, self-loops dropped, no_wiring label when zero satisfies edges, composes with --format edges|dot|mermaid per 5qj decision; reference shapes in research samples/specodelic-wiring.dot (consumer->producer arrow direction, 'n satisfies' labels, aggregated counts); implementation in src/graph.rs + src/main.rs, tests in tests/cli/parse_misc.rs
