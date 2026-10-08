---
tags: [pipeline-run:tdd-ro5-2026-10-08-specodelic-gre-6-per-file-state-diagrams-tasks-1-7-render-2-3-corpus-2-4, pipeline-step:fix-review]
---

RO5U-FIXES: both Medium findings fixed in-place during the review (not deferred): (1) _failure_message() now extracts refusal reasons from warnings[].message; GRAPH_ZERO_FILES/LINT_FAILED_ENVELOPE fixtures match the real genesis envelope shape and a new assertion pins 'no spec files found' in the refusal — scripts/state_view.py, scripts/state_view_cases.py, scripts/state_view_scope_cases.py. (2) GraphReport.intents (additive, sorted) in src/graph.rs consumed by the scope gate for owning-file grouping; pinned by fixture_single_intent_corpus_is_in_scope_with_wellformed_rows in tests/cli/graph_views.rs. Tests re-run after fixes: python3 -m unittest discover -s scripts -p test_graph_views.py → 37/37 OK; cargo test --test cli → 250/250 OK. Low findings: (1) documented as intentional in _fan_in docstring; (2)-(4) cosmetic/acceptable, no action.
