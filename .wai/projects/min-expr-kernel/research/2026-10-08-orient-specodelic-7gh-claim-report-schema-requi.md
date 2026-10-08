---
tags: [pipeline-run:tdd-ro5-2026-10-08-specodelic-7gh-claim-records-carry-reason-when-not-verified-f4-f6, pipeline-step:orient]
---

ORIENT: specodelic-7gh — claim_report_schema requires reason where not verified; native kernel path returns Unknown with reason None (evaluate_corpus_claims native arm line ~1129). Fix: thread (status, reason) tuples through KernelEnv eval (mirrors citation_corpus::evaluate Kleene-with-reasons pattern) and attach reasons in evaluate_corpus_claims. Counterexample records: citation path attaches reasons on unknown only; decide consistent fix within kernel.rs scope.
