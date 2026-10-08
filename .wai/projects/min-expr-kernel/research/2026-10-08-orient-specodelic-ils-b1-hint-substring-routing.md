---
tags: [pipeline-run:tdd-ro5-2026-10-08-specodelic-ils-view-scripts-b1-hint-routing-b2-mermaid-escaping-b4-docstring, pipeline-step:orient]
---

ORIENT: specodelic-ils — B1 hint substring routing bug (graph_views.py:140 ArtifactInvalid 'lint envelope carries no issues list' wrongly gets LINT_HINT), B2 incomplete mermaid_escape (view_common.py:184 + src/graph.rs:603 only replace '"' — labels embed arbitrary spec text, --> / # / %% / 'end' break diagrams), B4 docstring contradiction (tests/kernel_grammar.rs:5 'marker-free whole-cell opt-in' vs normative kernel_expr_opt_in marker-based opt-in in specs/compile.md). Out of scope: B3 (moved to design ticket), B5 (moved to specodelic-s64, don't touch specs/CHANGELOG.md).
