---
tags: [pipeline-run:epic-orchestrator-2026-11-13-specodelic-8mta-resolve-check-kernel-id-literals-e2-f1-vacuous-verified, pipeline-step:select-ticket]
---

TICKET: specodelic-8mta — kernel id literals must resolve or evaluate unknown (E2 F1). Selected over 9o1h because F1 is self-contained and F2 (citation composition) needs a design note; over jwjj (P3). Drift check: NOT already implemented — src/kernel.rs:769 term_value(Term::Id) returns TermVal::Id(literal.clone()) with no resolution check, so an unqualified literal compares unequal and ¬∃ becomes verified vacuously (E2 report §3.2/§8 F1). Existing reachable() unknown-seed path already returns unknown with a labeled reason (forward_closure Err), so the gap is comparison literals (and unresolvable reachable targets). Claimed specodelic-8mta.
