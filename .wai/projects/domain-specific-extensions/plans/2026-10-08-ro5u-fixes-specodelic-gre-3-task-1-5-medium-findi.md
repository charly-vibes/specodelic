---
tags: [pipeline-run:tdd-ro5-2026-10-08-specodelic-gre-3-native-dot-mermaid-projections-task-1-5, pipeline-step:fix-review]
---

RO5U-FIXES specodelic-gre.3 task 1.5: medium finding fixed — render_mermaid switched from names[&row.to] indexing to names.get() with skip-on-missing (pure renderer must not panic on a degenerate row); behavior identical for all real rows (from-less violations still render as annotated nodes since the empty id is absent from the node map). Tests re-run: cargo test --test cli parse_misc → 64 passed 0 failed. Low findings: none cheap outstanding — mermaid label escaping limited to quotes is tracked in the report as a documented simplification (canonical ids and lint-clean reasons never carry quotes).
