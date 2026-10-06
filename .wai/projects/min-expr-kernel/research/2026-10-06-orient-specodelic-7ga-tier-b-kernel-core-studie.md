---
tags: [pipeline-run:tdd-ro5-2026-10-06-add-min-expr-kernel-phase-3-specodelic-7ga-tier-b-kernel-core, pipeline-step:orient]
---

ORIENT: specodelic-7ga Tier B kernel core — studied design.md D1/D8, tasks.md §3, kernel+compile delta specs, substrate (src/acset mod/instance/query/schema, graph traversal via query.cyclic_nodes/forward_closure), slice-1 precedent in src/compile.rs (parse_citation_expr whole-cell parse, ThreeValued, sd1 fragment-position discipline, unknown_tag_markers), pretender ratchets (file_lines 2300, model_check.rs 2282 pinned shrink-only). Grammar decision: kernel expressions recognized marker-free in expr cells by whole-cell parse (citation precedent), fragment-position rule carries over; closed atomic set {==, <, <=, >, >=, bounded ∀/∃, ∧/¬, resolves, unique, acyclic, reachable}.
