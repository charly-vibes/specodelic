---
tags: [pipeline-run:tdd-ro5-2026-11-13-specodelic-04c-kernel-vocabulary-pack, pipeline-step:red]
---

RED (specodelic-04c kernel vocabulary pack): tests/kernel_pack_vocabulary.rs written first; before the pack artifact existed the binding fixture failed with linter.orphan_vocabulary — the message named the candidate pack kernel.* and both remediations (enable/declare the pack, or fix the vocabulary); the corpus-discovery test failed because no kernel.pack was discovered. Narrow command: cargo test --test kernel_pack_vocabulary (TMPDIR=/var/tmp/specodelic-xch1) — 2 expected-red failures, no setup breakage.
