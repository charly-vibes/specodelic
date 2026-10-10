---
tags: [pipeline-run:tdd-ro5-2026-10-10-specodelic-eczv-4-conform-phase-4-cli-wiring, pipeline-step:orient]
---

ORIENT: specodelic-eczv.4 — conform phase 4 CLI wiring. Read proposal What Changes, design D2/D6, tasks 4.1-4.3, conform delta spec. Study Commands enum + exit codes in src/main.rs, tests/cli/, genesis CLI helpers, completions workflow. Wire Conform command + exit-code contract: 0=no forbidden/unsupported; 1=any forbidden/unsupported; unknown/underspecified counts only; 2=invocation error/gate refusal zero records. RED test first in tests/cli/ (report envelope, --closed-world recorded, open_world_never_forbidden). GREEN Conform variant in src/main.rs. TIDY completions, explain untouched.
