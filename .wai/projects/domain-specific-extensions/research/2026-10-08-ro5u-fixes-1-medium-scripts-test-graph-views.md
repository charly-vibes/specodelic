---
tags: [pipeline-run:tdd-ro5-2026-10-08-specodelic-gre-5-guide-schema-json-and-schema-diagrams-tasks-2-1-2-3, pipeline-step:fix-review]
---

RO5U-FIXES: (1) MEDIUM — scripts/test_graph_views.py run_cli now uses tempfile.NamedTemporaryFile (no repo pollution, no parallel collision); (2) HIGH — guide.rs module Rationale updated for the new contract (value-set payload + schema_export responsibilities); (3) LOW (cheap) — added CLI-level unknown-version refusal subprocess case. Tests after fixes: cargo test --lib guide:: 21 passed; python meter 15/15 OK; fmt/clippy clean.
