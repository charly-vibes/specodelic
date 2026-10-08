---
tags: [pipeline-run:tdd-ro5-2026-10-08-specodelic-gre-6-per-file-state-diagrams-tasks-1-7-render-2-3-corpus-2-4, pipeline-step:green]
---

GREEN evidence: python3 -m unittest discover -s scripts -p test_graph_views.py → 37/37 OK; cargo test --test cli → 250/250 OK. Structural ratchet forced a split: pretender script-role caps (file ≤300 lines, fn ≤59) — graph_views.py is now the thin CLI entry (dispatch + labeled refusals + remediation hints), pure renderers live in scripts/schema_view.py and scripts/state_view.py with shared failure classes in scripts/view_common.py; the consumer suite split into schema_view_cases/state_view_cases/state_view_scope_cases re-imported by test_graph_views.py (meter command still runs all 37). parse_misc.rs additions moved verbatim to tests/cli/graph_views.rs (parse_misc was 21 lines under its 2300 cap — my block pushed it over); main.rs gains one mod line.
