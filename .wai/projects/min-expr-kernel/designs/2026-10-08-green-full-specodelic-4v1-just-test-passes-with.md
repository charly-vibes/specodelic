---
tags: [pipeline-run:tdd-ro5-2026-10-08-specodelic-4v1-failures-ride-success-shaped-envelopes-f1, pipeline-step:green]
---

GREEN (full) specodelic-4v1: just test passes with the change set — full suite green incl. tests/claim_parity.rs (2 fixtures updated: cli_code 0→1 where the CLI aggregate is not clean, now agreeing with orchestrate's stage verdict) and tests/kernel_corpus.rs (4 fixtures). Note: pretender_gate::head_passes_gate fails in the main tree only because of the foreign docs-session artifact vendor/mermaid.min.js (untracked, gitignored, fetched by just docs-mermaid) being disk-scanned by the gate; verified my changes pass the gate in an isolated HEAD worktree and via just test under the advisory gate-drill lock with the artifact temporarily aside (restored after).
