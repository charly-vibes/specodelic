---
tags: [pipeline-run:tdd-ro5-2026-10-08-specodelic-gre-6-per-file-state-diagrams-tasks-1-7-render-2-3-corpus-2-4, pipeline-step:orient]
---

ORIENT: specodelic-gre.6 — per-file state-machine view in scripts/graph_views.py consuming spk graph --format edges TSV + graph --json envelope (scope: data.files>=1) + optional lint --json envelope (invariant issues refuse). States/transition/guard nodes from transitions.from/to/guard edges, grouped by owning file, guard dashed (D8), fan-in counts distinct source transitions (D2), violations annotated red-dashed, no_transitions labeled empty view, out_of_scope_refused legs (intentless, lint-dirty) before output. Fixtures: zero_file (exists), single_intent (exists), typing_violations (exists), lint_dirty (new). Rust side: parse_misc.rs pins fixture-corpus TSV contracts.
