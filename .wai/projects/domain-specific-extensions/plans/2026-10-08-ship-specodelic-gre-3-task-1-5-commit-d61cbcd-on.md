---
tags: [pipeline-run:tdd-ro5-2026-10-08-specodelic-gre-3-native-dot-mermaid-projections-task-1-5, pipeline-step:ship-close]
---

SHIP specodelic-gre.3 task 1.5: commit d61cbcd on gre/dot-mermaid (4 files: src/graph.rs renderers + ProjectionRow.violation_from, src/main.rs GraphFormat::{Dot,Mermaid} + cmd_graph_projection, tests/cli/parse_misc.rs 13 RED-first cases, tasks.md checkbox 1.5). Gates at commit time: just ci green (ah check --run-tests 167/0), openspec validate --all --strict 25 passed, pretender check exit 0. Not pushed, wai close not run — orchestrator owns both per brief. Remaining known risks: mermaid label escaping covers quotes only; dot labels carry full field.column (documented jq-parity delta); violation linkStyle index pinned to the deterministic link sort.
