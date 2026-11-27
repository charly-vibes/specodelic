---
tags: [pipeline-run:tdd-ro5-2026-11-27-trust-ledger-derivation-slices-1-3, pipeline-step:fix-review]
---

RO5U-FIXES: applied both medium findings from 2026-11-27-ro5u-trust-ledger-slices.md. M1: human.rs trust_base_lines empty-ledger message reworded 'the corpus declares' -> 'the checked scope declares' (derivation scope is the verify invocation's file set, not the whole corpus). M2: trust_base.rs dedup_by now carries the explicit total-sort precondition comment (consecutive-equal drops are safe because the sort is total on (kind, source, id)); multi-satisfies duplicates collapse to one member. Tests: human trust_base tests + trust_base module tests.
