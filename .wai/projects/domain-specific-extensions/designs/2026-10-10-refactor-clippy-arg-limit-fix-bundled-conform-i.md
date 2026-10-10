---
tags: [pipeline-run:tdd-ro5-2026-10-10-specodelic-eczv-4-conform-phase-4-cli-wiring, pipeline-step:refactor]
---

REFACTOR: clippy arg-limit fix — bundled conform invocation params into a ConformTarget struct (CheckTarget precedent); tasks 4.1-4.3 checked off; just lint + just pretender-check (main.rs and tests/cli/* all green) + openspec validate --all --strict (29 passed) clean; ah check reports only the pre-existing sequenced no-toml findings for the conform spec (contract authoring is the deferred follow-up, not this ticket)
