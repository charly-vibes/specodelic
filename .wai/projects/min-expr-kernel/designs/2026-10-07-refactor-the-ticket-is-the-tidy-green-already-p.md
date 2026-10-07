---
tags: [pipeline-run:tdd-ro5-2026-10-07-specodelic-k3h-ss3-8-claim-identity-and-command-parity-during-cleanup, pipeline-step:refactor]
---

REFACTOR: the ticket IS the tidy — GREEN already performed the extraction (duplication removed: two ~20-line merge blocks → kernel::merge_corpus_statuses; inline backend selection → CorpusBackend::from_tlc_presence; both call sites net-shrunk). No further tidy within scope guard (citation_corpus.rs and src/model_check.rs untouchable; behavior expansion forbidden). Tests re-run after refactoring: cargo test --test claim_parity 3/3, clippy -D warnings clean, just test-smart 0 failed, pretender check exit 0.
