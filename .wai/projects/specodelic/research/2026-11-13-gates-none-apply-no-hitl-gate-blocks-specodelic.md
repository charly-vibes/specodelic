---
tags: [pipeline-run:epic-orchestrator-2026-11-13-specodelic-8mta-resolve-check-kernel-id-literals-e2-f1-vacuous-verified, pipeline-step:land-gates]
---

GATES: none apply — no HITL gate blocks specodelic-8mta and no openspec change exists for it. Design decision recorded instead of a sign-off: enforce the already-declared guarantee (kernel.md kernel_status_three_valued / kernel_decidable) by resolve-checking id literals at evaluation time -> unknown with a labeled reason; the lint-advisory candidate is rejected as insufficient (a warning does not stop verified-vacuous claims). Scope: src/kernel.rs term_value/atomic/eval + docs + tests. Corpus blast radius checked: the four kernel claims in specodelic.md use only bound vars/projections/object cardinalities, no bare id literals, so their verdicts must not change.
