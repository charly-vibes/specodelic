---
tags: [pipeline-run:tdd-ro5-2026-10-08-specodelic-gre-7-file-level-traceability-view-task-2-5, pipeline-step:orient]
---

ORIENT: gre.7 task 2.5 — file-level traceability view. Studied tasks.md 2.5, design D2 (fan-in counts DISTINCT targets/sources; TSV keeps multiplicity, views collapse), D3 (violations ride along, never silently clean), D7 (scope gate), D8. Studied landed modules: graph_views.py (CLI surface), view_common.py (failure classes), state_view.py (scope gate, parse_edges_tsv, _owning_file, _fan_in, mermaid helpers — reuse), state_view_cases/scope_cases (consumer-test split, fixture envelopes). Spec scenario: node set == intent set, edges connect only intents, fan-in k over k distinct source intents. Plan: new scripts/traceability_view.py renderer + traceability_view_cases.py tests, graph_views.py gains trace subcommand reusing the states artifact CLI shape. RED first.
