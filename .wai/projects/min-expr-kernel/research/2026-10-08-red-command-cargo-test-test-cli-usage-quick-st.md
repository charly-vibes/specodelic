---
tags: [pipeline-run:tdd-ro5-2026-10-08-specodelic-gch-quick-start-kernel-examples-produce-expected-claim-evidence, pipeline-step:red]
---

RED: command='cargo test --test cli usage_quick_start_kernel_claims_verify' result=FAILED 1/244-filtered; failure is the intended missing-claim-evidence one: extraction test pulls order.cancel live from specs/USAGE.md, lints+compiles+model-checks it; at HEAD the example carries zero kernel claims — persisted report has claims:[] and unchecked_claim_ids [refund_bounded, refund_timely], outcome exploration_only (assert_eq left String(exploration_only) right no_counterexample). Setup is not broken: lint/compile stages pass; the doc example itself lints clean (check_doc_examples 2/2). Baseline at HEAD: just lint-doc-examples 2 examples clean, just lint-specs ok:true issues:[], cargo test --test cli 243 passed.
