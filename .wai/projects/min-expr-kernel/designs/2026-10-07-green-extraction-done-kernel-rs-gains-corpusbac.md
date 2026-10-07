---
tags: [pipeline-run:tdd-ro5-2026-10-07-specodelic-k3h-ss3-8-claim-identity-and-command-parity-during-cleanup, pipeline-step:green]
---

GREEN: extraction done — kernel.rs gains CorpusBackend::from_tlc_presence(bool) and merge_corpus_statuses(&mut RunReport, &[ResolvedStatus], &[CorpusClaimStatus]) -> Vec<Value> (the ONE identity/merge helper: kernel-claim push into persisted statuses + reasons map + command-output entries, claim reasons win for shared ids matching historical chain order). Both call sites (src/commands/model_check.rs, src/orchestrate.rs) now call the shared helpers — duplicated ~20-line merge blocks and inline backend selection removed (net shrink on both). src/model_check.rs and citation_corpus.rs untouched. Narrow: cargo test --test claim_parity 3/3 green. Clippy -D warnings clean; pretender check exit 0 (kernel.rs atomic() red is pre-existing at-ceiling, untouched); just test-smart ingested 15 results, 0 failed.
