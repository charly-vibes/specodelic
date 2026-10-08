---
tags: [pipeline-run:tdd-ro5-2026-10-08-specodelic-ils-view-scripts-b1-hint-routing-b2-mermaid-escaping-b4-docstring, pipeline-step:orient]
---

ORIENT specodelic-ils: B1 hint substring routing in scripts/graph_views.py:140 (ArtifactInvalid 'lint envelope carries no issues list' wrongly gets LINT_HINT; use exception type discriminator), B2 mermaid_escape only replaces " in scripts/view_common.py:184 + mirror in src/graph.rs:603 (labels embed arbitrary spec-cell text; -->, #, %%, line-start end break diagram; D8), B4 tests/kernel_grammar.rs:5 docstring claims marker-free whole-cell opt-in contradicting kernel_expr_opt_in (marker required) and the tests themselves. Out of scope: B3, B5. Meter: python unittest escape+hint tests pass; grep marker-free = 0.
