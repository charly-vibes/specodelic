---
tags: [pipeline-run:tdd-ro5-2026-10-08-specodelic-gre-3-native-dot-mermaid-projections-task-1-5, pipeline-step:orient]
---

ORIENT specodelic-gre.3 task 1.5 native dot/mermaid projections: reuse edge_projection/ProjectionRow/canonical_id from gre.2; extend ProjectionRow with violation_from (TSV bytes unchanged) so renderers draw red-dashed forbidden edges; report.dangling renders as red dashed annotated note nodes (no prose parsing). Grammar D8: solid=state machine, dashed=guard, bold=emits, dotted=traceability, red dashed=dangling/violations. Dot shape parity with retired scripts/graph_to_dot.jq. Tests in tests/cli/parse_misc.rs; flags in src/main.rs GraphFormat.
