---
tags: [pipeline-run:tdd-ro5-2026-11-12-specodelic-d8c-enforce-reference-typing-ref-kind-compatible-as-a-lint-rule, pipeline-step:orient]
---

ORIENT: specodelic-d8c task = enforce ref_kind_compatible as a lint rule. Read bd ticket, specodelic.md Reference Typing table, linter-referential_integrity/spec.md, src/lint/* (rule table + graph.rs), src/acset/schema.rs canonical(), src/graph.rs kind_index/resolve_link/typing_violation, tests/reference_oracle.rs oracle.ref_kind_mismatch. Baseline probes: graph violations over corpus = 0; lint = 0 issues / 53 warnings; corpus lint pin test = files_linted 48, issues empty. Decision: ENFORCE (not demote) — schema-as-data decision reused from graph (typing_table_is_data).
