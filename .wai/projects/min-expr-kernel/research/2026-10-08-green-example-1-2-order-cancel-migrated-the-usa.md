---
tags: [pipeline-run:tdd-ro5-2026-10-08-specodelic-gch-quick-start-kernel-examples-produce-expected-claim-evidence, pipeline-step:red]
---

GREEN example 1/2 (order.cancel): migrated the USAGE.md quick-start example — Constraints gains refund_traces_resolve (**kernel:** resolves(traces_to)) and refund_guard_reaches (**kernel:** reachable(order.cancel.refund, order.cancel.refund_bounded, guard)), Properties gains the two coverage rows; short prose note explains the executable slice and why domain-data cells stay informal strings. Same test now passes: usage_quick_start_kernel_claims_verify ok — lint clean, outcome no_counterexample, both kernel claims verified in JSON and persisted report. just lint-doc-examples 2/2 clean; zero exemptions added. Domain-field cells (refund_amount==paid_amount, days-between) deliberately stay prose: kernel semantics never outruns the acset instance (Eq over non-row ids would be an honest counterexample — migration would be wrong, not incomplete).
