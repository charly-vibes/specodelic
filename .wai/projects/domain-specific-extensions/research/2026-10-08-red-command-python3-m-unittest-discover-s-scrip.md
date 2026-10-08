---
tags: [pipeline-run:tdd-ro5-2026-10-08-specodelic-gre-7-file-level-traceability-view-task-2-5, pipeline-step:red]
---

RED: command=python3 -m unittest discover -s scripts -p test_graph_views.py. Result: 56 tests, failures=4 errors=15 — every traceability_view_cases test fails for the intended missing-behavior reason: render-level tests error with 'module graph_views has no attribute render_traceability' (view absent); CLI tests fail with returncode 2 + usage line (trace subcommand absent). Pre-existing schema/state tests still pass (41 green).
