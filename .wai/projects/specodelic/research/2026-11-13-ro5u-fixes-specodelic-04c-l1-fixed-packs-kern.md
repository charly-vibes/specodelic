---
tags: [pipeline-run:tdd-ro5-2026-11-13-specodelic-04c-kernel-vocabulary-pack, pipeline-step:fix-review]
---

RO5U-FIXES (specodelic-04c): L1 fixed — packs/kernel-pack.md ## Properties table pipes realigned (the predicates_stay_external row now aligns with the header). M1 (no companion corpus capability spec) and M2 (spk doctor does not surface packs/) deferred with reasons and concrete follow-ups recorded in the RO5U findings file (2026-11-13-ro5u-findings-specodelic-04c-kernel-vocab-pack.md): M1 because the corpus tree holds capability specs only (the reference oracle requires kind intent there) and the kernel capability is already documented by openspec/specs/kernel/spec.md; M2 because widening the doctor's scan touches src/**, which this ticket forbids. No behavior-impacting fix, so no new test. Verified after the fix: cargo run -- lint packs/kernel-pack.md zero issues; cargo test --test kernel_pack_vocabulary 4 passed.
