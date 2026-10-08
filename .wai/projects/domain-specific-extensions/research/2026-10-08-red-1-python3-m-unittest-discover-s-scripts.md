---
tags: [pipeline-run:tdd-ro5-2026-10-08-specodelic-gre-5-guide-schema-json-and-schema-diagrams-tasks-2-1-2-3, pipeline-step:red]
---

RED: (1) python3 -m unittest discover -s scripts -p test_graph_views.py → FAILED errors=1 (ImportError: graph_views module absent — 13 tests across rendering/perturbation/malformed/CLI classes). (2) cargo test --lib guide:: → compile failure E0425 cannot find value_set_payload / schema_export (4 new tests). (3) CLI: 'spk guide --json' and 'guide --schema --json' exit 2 unrecognized subcommand. All failures are for the expected missing behavior.
