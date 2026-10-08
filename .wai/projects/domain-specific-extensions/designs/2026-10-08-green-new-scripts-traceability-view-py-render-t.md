---
tags: [pipeline-run:tdd-ro5-2026-10-08-specodelic-gre-7-file-level-traceability-view-task-2-5, pipeline-step:green]
---

GREEN: new scripts/traceability_view.py — render_traceability reuses state_view's _scope_gate/parse_edges_tsv/_owning_file/_mermaid_names/_mermaid_escape; collapses deps to distinct cross-file (src,dst) pairs over the envelope's intent set (unattributable endpoint → artifact_invalid), self-loops dropped (intra-file ≠ file dependency), fan-in over DISTINCT source intents (D2), violations annotated red-dashed into collapsed targets (D3, no fan-in contribution), labeled no_dependencies note when no cross-file deps, deterministic. graph_views.py: states_cli generalized to artifact_cli(rest, render) shared by states+trace CLIs; trace subcommand + usage + re-export. Meter: 56 tests OK (was 41 pass + 19 red).
