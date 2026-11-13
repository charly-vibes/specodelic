---
tags: [pipeline-run:tdd-ro5-2026-11-13-specodelic-lf3-migrate-specodelic-kinds-properties, pipeline-step:orient]
---

ORIENT: specodelic-lf3 run 4 — migrate the specodelic + kinds property rows to executable predicates. Read the ticket, both corpus files (30 + 17 property rows), the compile/packs precedents (861491e1, c34a864c, 55f058c0) and the Revision 21 law grammar (f1d5698f). Located the real seams: spec::parse_str, lint::lint_corpus, acset::schema, rename::run, compile::compile_spec, verify::evaluate_properties_gate, model_check. Established that the current lint path does NOT reject an extra Constraint column, an extra State field or a sixth section, so those claims have no honest seam. Task selected: migrate every row with an honest seam (43 of 47), leave the other four on todo_predicate! and report them.
