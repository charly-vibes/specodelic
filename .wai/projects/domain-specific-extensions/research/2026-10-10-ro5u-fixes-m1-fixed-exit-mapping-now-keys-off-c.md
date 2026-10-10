---
tags: [pipeline-run:tdd-ro5-2026-10-10-specodelic-eczv-4-conform-phase-4-cli-wiring, pipeline-step:fix-review]
---

RO5U-FIXES: M1 fixed — exit mapping now keys off conform::Verdict::{Forbidden,Unsupported}.as_str() via a violation_count closure (no magic strings). M3 fixed — new CLI test conform_empty_corpus_yields_valid_empty_report_exit_0 (zero-line corpus: ok:true, records [], verdict_counts {}, evidence_scope intact, exit 0). M2 deferred with reason: single-rendering-path fix requires extending conform.rs emit_report beyond this ticket's hard scope ('conform.rs = tiny CLI helper only'); no observable drift today, tracked as follow-up note. L1-L3: no action (accurate as-is). Tests: cargo test --test cli conform 7 passed; just lint, just pretender-check (own files green), just test all clean
