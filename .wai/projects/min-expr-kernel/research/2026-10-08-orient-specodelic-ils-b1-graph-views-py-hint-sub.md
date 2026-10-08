---
tags: [pipeline-run:tdd-ro5-2026-10-08-specodelic-ils-view-scripts-b1-hint-routing-b2-mermaid-escaping-b4-docstring, pipeline-step:orient]
---

ORIENT: specodelic-ils B1 (graph_views.py hint substring routing — ArtifactInvalid 'lint envelope carries no issues list' contains 'lint' and wrongly gets LINT_HINT; fix by exception type), B2 (view_common.mermaid_escape only escapes quotes; need --> / # / %% / leading-end label escaping; check Rust mirror src/graph.rs), B4 (tests/kernel_grammar.rs:5 docstring says marker-free whole-cell opt-in; spec kernel_expr_opt_in requires **kernel:** marker; comment-only). Scope: scripts/graph_views.py, scripts/view_common.py, scripts/test_*.py, src/graph.rs escape mirror only, tests/kernel_grammar.rs comment-only. NOT openspec/specs/, specs/, B3, B5.
