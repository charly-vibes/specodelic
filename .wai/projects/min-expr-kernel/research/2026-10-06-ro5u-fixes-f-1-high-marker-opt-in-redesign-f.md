---
tags: [pipeline-run:tdd-ro5-2026-10-06-add-min-expr-kernel-phase-3-specodelic-7ga-tier-b-kernel-core, pipeline-step:fix-review]
---

RO5U-FIXES: F-1 (HIGH) marker opt-in redesign + F-2 (MEDIUM) dot-splitting were fixed during GREEN (commits feat(kernel) grammar + feat(kernel) grounding); stage-4 edge gaps pinned in the RO5U fixes commit (tests/kernel_grammar.rs row_typing_edge_cases_fail_labeled + tests/kernel_compile_edge.rs, 4 tests); 2 low perf notes deferred of record (linear Proj lookup, per-call canonical schema rebuild — termination holds).
