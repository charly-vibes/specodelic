---
tags: [pipeline-run:tdd-ro5-2026-10-07-specodelic-hb4-add-min-expr-kernel-ss3-6-citation-resolution-in-commands, pipeline-step:orient]
---

ORIENT: specodelic-hb4 — D9 complete command-path evaluation. Read design D9, tasks 3.6, delta scenarios (model-check: file-local-identities, mixed-inputs, qualified-bare-parity, scope-and-dependency-order, property-execution-not-invented). Baseline divergence: run_executable keys citation outcomes by bare local id only, so qualified [[file.c1]] is unknown while bare [[c1]] verifies; fragment-less runs have empty evidence. Plan: new src/citation_corpus.rs (scope guard + cross-file resolution), wire into cmd_model_check/cmd_orchestrate/cmd_verify; keep src/model_check.rs untouched (ratchet). Single task specodelic-hb4.
