---
tags: [pipeline-run:tdd-ro5-2026-11-12-specodelic-q7uo-compile-pipe-unescape, pipeline-step:quality-ledger]
---

QUALITY LEDGER (specodelic-q7uo): Changed: src/compile.rs (unescape_cell_pipes helper + application at Fragment.text in fragment_of_strict and expr in constraints_doc — the only two fragment-text production sites), tests/cli/fragment_unescape.rs (new RED/GREEN + byte-stability pin), tests/cli/main.rs (mod registration). Verified: cargo test --test cli fragment_unescape (2/2, after 1 expected RED); cargo test --test cli (312 passed, 0 failed); cargo test --lib (439 passed). Review: RO5U pass, zero Critical/High/Medium; two documented Lows deferred. Risks: none known — replace is identity without the escape; byte-stability pinned to pre-change artifact bytes. Next: cargo fmt, pretender check, commit (no push — orchestrator owns).
