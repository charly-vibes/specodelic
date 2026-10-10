---
tags: [pipeline-run:tdd-ro5-2026-10-10-specodelic-eczv-4-conform-phase-4-cli-wiring, pipeline-step:orient]
---

ORIENT: specodelic-eczv.4 — conform phase 4 CLI wiring. Read add-conform proposal/design D2 D6, tasks 4.1-4.3, delta spec; reuse phase 1-3 src/conform.rs (verdict engine, build_report/emit_report, gate, corpus validation); wire spk conform <file> --oracle scenarios.jsonl as Commands variant with exit-code contract (0 no forbidden/unsupported; 1 any forbidden/unsupported; unknown/underspecified never fail; 2 invocation error/gate refusal zero verdict records); --closed-world recorded in verdict records + report header; open_world_never_forbidden at CLI level; completions generated on demand via clap_complete (no tracked files); explain untouched.
