---
tags: [pipeline-run:tdd-ro5-2026-10-10-specodelic-eczv-3-conform-phase-3-input-gate, pipeline-step:orient]
---

ORIENT: specodelic-eczv.3 — read design.md D5/D3, tasks.md phase 3, conform delta spec; phases 1-2 landed (src/conform.rs verdict engine + report, tests/conform_classification.rs, tests/conform_report.rs). Gate = parse+lint-clean (reuse orchestrate lint-stage discipline) + artifact currency (stale .tla vs current spec refused, hint names spk compile); corpus validation refusals (malformed JSONL, missing id, duplicate ids, unknown fields); zero-line corpus = valid report, zero records, evidence_scope intact.
