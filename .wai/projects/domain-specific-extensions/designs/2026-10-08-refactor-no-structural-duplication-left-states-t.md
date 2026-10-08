---
tags: [pipeline-run:tdd-ro5-2026-10-08-specodelic-gre-7-file-level-traceability-view-task-2-5, pipeline-step:refactor]
---

REFACTOR: no structural duplication left (states/trace share artifact_cli; scope gate, TSV parse, mermaid helpers all reused from state_view). Fixed pre-existing ruff F541 in graph_views.py (file touched this session). pretender check exit 0 — all touched script files green; tests/reference_oracle.rs cognitive-31 red is pre-existing at baseline and advisory. Remaining ruff F401/F811 findings live in gre.6-owned files (state_view*.py) outside this ticket's allowed file list — left alone. Meter still 56 OK.
