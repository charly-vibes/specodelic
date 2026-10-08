---
tags: [pipeline-run:tdd-ro5-2026-10-08-specodelic-gre-7-file-level-traceability-view-task-2-5, pipeline-step:plan]
---

gre.7 task 2.5 plan — file-level traceability view (trace subcommand).

Desired behavior: render_traceability(edges_tsv, graph_payload, lint_payload=None) — scope gate first (same out_of_scope_refused legs as states view: intentless corpus, lint-dirty corpus, unsuccessful envelopes); collapse every non-violation TSV row's endpoints to owning intents (graph JSON intents ∪ TSV Intent-kind endpoints authoritative); node set == intent set (exactly 1 node per intent, incl. single-intent corpus); dedupe to distinct cross-file (src,dst) pairs = file dependencies (intra-file collapse to self-loops, dropped as non-dependencies — documented, not silent cleaning of data: collapse semantics); fan-in per intent counts DISTINCT source intents (D2 multiplicity: duplicate TSV rows add 0 beyond first); violations stay annotated as red-dashed violation_i nodes dashed into their collapsed intent target (D3, gre.6 precedent), never contributing to fan-in; unattributable endpoint (collapses outside intent set) → artifact_invalid; zero cross-file deps → labeled no_dependencies note (never silently clean); deterministic Mermaid.

Out of scope: Rust changes (none needed — artifacts suffice), openspec/specs edits, prose-independence test (task 2.7), fan-out (open question in design).

Tests (scripts/traceability_view_cases.py, re-imported by test_graph_views.py): one node per intent; row-level ids never leak as endpoints; duplicate rows draw one edge + fan-in counts distinct sources (fan-in 2 not 3); violations annotated with reason text; single-intent corpus renders exactly 1 intent node + no_dependencies note; determinism; scope legs (intentless, lint-dirty) at render + CLI level; unattributable endpoint refused. CLI: trace subcommand writes Mermaid, refusals exit 1 with named failure before output.

Narrow test cmd: python3 -m unittest discover -s scripts -p test_graph_views.py
Full verification: just ci && pretender check && ah check && openspec validate --all --strict
Files: scripts/traceability_view.py (new), scripts/traceability_view_cases.py (new), scripts/test_graph_views.py (import line), scripts/graph_views.py (trace CLI + re-exports), openspec/changes/add-graph-views/tasks.md (checkbox 2.5 only).
