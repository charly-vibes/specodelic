---
tags: [pipeline-run:tdd-ro5-2026-10-08-specodelic-gre-3-native-dot-mermaid-projections-task-1-5, pipeline-step:green]
---

GREEN specodelic-gre.3 task 1.5: cargo test --test cli parse_misc → 64 passed, 0 failed (all 13 new dot/mermaid cases green). Implementation: ProjectionRow.violation_from (TSV bytes unchanged — pinned edges tests still pass), render_dot/render_mermaid pure renderers + dot_projection/mermaid_projection in src/graph.rs (zero new deps), GraphFormat::{Dot,Mermaid} + cmd_graph_projection in src/main.rs. jq parity check (manual, scripts/graph_to_dot.jq over two-state envelope): identical structure — header, rankdir, ellipse node default, quoting, indentation, style attrs, unique dedup; two documented content deltas: labels carry the full field.column kind (D2 contract) vs jq's short kind, and no trailing blank line. cargo build clean after removing an unused struct.
