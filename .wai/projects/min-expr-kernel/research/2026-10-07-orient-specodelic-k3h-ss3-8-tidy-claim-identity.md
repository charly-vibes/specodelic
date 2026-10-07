---
tags: [pipeline-run:tdd-ro5-2026-10-07-specodelic-k3h-ss3-8-claim-identity-and-command-parity-during-cleanup, pipeline-step:orient]
---

ORIENT specodelic-k3h §3.8 TIDY (claim identity + command parity): duplication confirmed in src/commands/model_check.rs ~111-170 and src/orchestrate.rs ~367-440 — (a) kernel backend selection, (b) kernel-claim push into invariant_statuses, (c) ~20-line reasons-map + statuses_json merge block duplicated verbatim. Identity shapes: citation_corpus::ResolvedStatus and kernel::CorpusClaimStatus mirror each other (id/status/reason + invariant_status/output_json). citation_corpus.rs NOT in allowed files → shared helpers go in src/kernel.rs (allowed), call sites slim down. Plan: characterization parity tests first (tests/claim_parity.rs, must pass pre-refactor), then extract merge_corpus_statuses + backend selection into kernel.rs, behavior-preserving. pretender: file_lines_max 2300 not at risk (kernel.rs 1243). src/model_check.rs untouched.
