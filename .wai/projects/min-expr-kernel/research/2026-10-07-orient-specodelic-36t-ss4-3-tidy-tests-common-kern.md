---
tags: [pipeline-run:tdd-ro5-2026-10-07-specodelic-36t-ss4-3-shared-fixture-module-named-case-law-promotion-path, pipeline-step:orient]
---

ORIENT specodelic-36t §4.3 TIDY: tests/common/kernel_fixtures.rs (36n) already holds the agreement corpus + Backend registry; kernel_grounding.rs (CYCLIC/DANGLING/DUPLICATES/CHAIN, demo.cyc/dng/dup/chn ids) and kernel_status.rs (CLEAN/BROKEN/UNKNOWABLE/ABSORBING, demo.st ids) still carry private near-duplicate corpus constants. Plan: verbatim move of those constants into tests/common/kernel_corpora.rs (new shared module registered in common/mod.rs), both tests import via #[path=common/mod.rs] shim (existing convention in kernel_agreement.rs); ids unchanged so all assertions keep meaning; rust interim gate untouched (kernel_agreement.rs not edited). Promotion contract goes into add-py-fragment-emission/design.md near D5. tasks.md 4.3 checkbox already [x] — leave as-is.
