---
tags: [pipeline-run:tdd-ro5-2026-10-07-specodelic-k3h-ss3-8-claim-identity-and-command-parity-during-cleanup, pipeline-step:red]
---

RED (characterization-green, per cleanup-ticket convention): command=cargo test --test claim_parity; tests/claim_parity.rs (NEW, 3 tests) pins CLI vs orchestrate vs persisted parity — mixed rust/kernel/missing-citation fixture (CLI exit 0, orchestrate exit 1 with model_check stage passed — verify fails downstream, pinned as observed), clean corpus, two-file per-file merge identity incl. unknown-reason labels; persisted .check.json payloads byte-identical across paths. All 3 PASS pre-refactor. Deliberate-broken intermediate demonstrated: removing orchestrate's kernel-claim push → all 3 parity tests FAILED ('CLI and orchestrate entries' assertion) → recorded → restored (git diff clean, 3/3 green again).
