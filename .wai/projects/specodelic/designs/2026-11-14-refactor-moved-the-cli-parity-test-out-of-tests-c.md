---
tags: [pipeline-run:tdd-ro5-2026-11-14-specodelic-3a8-decouple-dual-format-metadata-into-an-openspec-adapter, pipeline-step:refactor]
---

REFACTOR: moved the CLI parity test out of tests/cli/parse_misc.rs (pretender file_lines 2327 > 2300) into its own tests/cli/parse_dual.rs module; parse_misc is back under the ratchet and the test still passes. Adapter/parser code is already single-purpose; no further structural change needed.
