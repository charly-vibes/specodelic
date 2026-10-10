---
tags: [pipeline-run:epic-orchestrator-2026-10-10-specodelic-eczv-4-conform-phase-4-cli-wiring-and-exit-code-contract]
---

ORIENT: specodelic-eczv.4 — conform phase 4 CLI wiring. Read proposal What Changes, design D2/D6, tasks 4.1-4.3, conform delta spec. Study Commands enum + exit-code contract in src/main.rs, src/commands/, tests/cli/, genesis CLI helpers, completions workflow. Wire Conform command + exit codes: 0=no forbidden/unsupported; 1=any forbidden/unsupported; unknown/underspecified counts only; 2=invocation error/gate refusal zero records. RED first in tests/cli/ (report envelope, --closed-world recorded, open_world_never_forbidden). GREEN Conform variant. TIDY completions, explain untouched.
