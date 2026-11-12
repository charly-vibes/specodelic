---
tags: [pipeline-step:land-gates, pipeline-run:epic-orchestrator-2026-11-12-specodelic-ez9a-2-migrate-namespace-bug, pipeline-run:epic-orchestrator-2026-11-12-specodelic-ez9a-2-migrate-namespace-bug]
---

GATES: none apply - bugfix at P3, no sign-off; the repro is reconstructible from the measured scaffold+rekey behavior (no stored repro procedure beyond repro_hash f29f4cf1f143d662). Fix must keep the wrap/rekey and archive-companion contracts intact (byte-plain deltas, fail-closed layer validation).
