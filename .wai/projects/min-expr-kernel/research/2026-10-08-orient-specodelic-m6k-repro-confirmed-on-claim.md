---
tags: [pipeline-run:tdd-ro5-2026-10-08-specodelic-m6k-stale-compile-demotes-malformed-kernel-claim-to-unchecked-f2, pipeline-step:orient]
---

ORIENT: specodelic-m6k — repro confirmed on claim fixture: compile ok, sed kernel cell to bogusfn(), standalone model-check reports no_counterexample/ok:true/exit 0 — claim silently demoted to unchecked. Cause: claim_classification (src/verify.rs:695) falls through when extract_model_ir has no guard_kernel entry for the edited cell; artifact-consistency check (src/model_check.rs:1169) covers states/transitions/edges/output/invariant fragments but not constraint cells.
