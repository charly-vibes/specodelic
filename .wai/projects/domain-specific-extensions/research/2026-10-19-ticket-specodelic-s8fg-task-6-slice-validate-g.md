---
tags: [pipeline-run:epic-orchestrator-2026-10-19-delta-sidecar-task6, pipeline-step:select-ticket]
---

TICKET: specodelic-s8fg (task 6 slice — validate-gate retirement). s8fg already in_progress, no re-claim. Tasks 1-5 landed (head 7c6a7d0). This run does tasks.md 6.1-6.2: collapse scripts/spec_validate.py to a plain 1.x invocation (no version guard, no 0.19 npx fallback), bump ci.yml's npm install pin @fission-ai/openspec@0.19.0 to the pinned 1.x version + document dev-machine install, update tests/ci_wiring.rs + tests/test_spec_validate.py (TDD: gate test names 1.x as validator first), decision record openspec/decisions/2026-10-DD-validate-pin-retired.md superseding 2026-10-19-openspec-validate-pin, GH-20 landing note. Task 7 (archive + close) stays for the final run.
