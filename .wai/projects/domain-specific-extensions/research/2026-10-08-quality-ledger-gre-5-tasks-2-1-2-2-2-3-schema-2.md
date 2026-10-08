---
tags: [pipeline-run:tdd-ro5-2026-10-08-specodelic-gre-5-guide-schema-json-and-schema-diagrams-tasks-2-1-2-3, pipeline-step:quality-ledger]
---

QUALITY LEDGER (gre.5, tasks 2.1/2.2/2.3-schema/2.6).

Changed:
- src/guide.rs — value_set_payload() + schema_export(&Schema) pure exporters, Rationale updated, 6 new unit tests incl. the Rust→Python producer↔consumer delta test
- src/main.rs — Guide { --schema } subcommand wired through genesis Output::emit (cmd_guide)
- scripts/graph_views.py (new) — schema view: validates the versioned export (unsuccessful envelope, missing fields, unknown version, duplicate identities, absent endpoints → schema_export_invalid before output), renders revision-labeled deterministic Mermaid
- scripts/test_graph_views.py (new) — 15 consumer tests: rendering, 1-morphism perturbation, refinement labels, determinism, 8 malformed refusals, CLI-level fail-before-output
- openspec/changes/add-graph-views/tasks.md — 2.1/2.2/2.3(schema portion, annotated)/2.6 ticked only

Verified:
- just ci → exit 0 (incl. ah check --run-tests: 167 passed, 0 findings)
- python3 -m unittest discover -s scripts -p test_graph_views.py → 15/15 OK
- ah check → no issues, 0 findings; openspec validate --all --strict → 25 passed, 0 failed
- pretender check → exit 0 (one yellow advisory: _validated_morphisms cyclomatic 12/15)
- CLI verified: spk guide --json and spk guide --schema --json emit; real export renders through scripts/graph_views.py

Review:
- RO5U done; Critical/High/Medium fixed (tempfile in CLI tests, Rationale update, extra CLI refusal case); remaining: none Critical/High; one Low known — script yellow advisory within cap

Risks:
- none known beyond pretender yellow advisory (advisory only, cap 15)

Next:
- commit the five authored files; orchestrator verifies and pushes
