---
tags: [pipeline-run:epic-orchestrator-2026-10-19-delta-sidecar-task5, pipeline-step:select-ticket]
---

TICKET: specodelic-s8fg (task 5 slice — corpus migration). s8fg already in_progress, no re-claim. Tasks 1-4 landed (head c0df241); this run does tasks.md 5.1-5.3: convert add-graph-views, add-py-fragment-emission, add-ts-fragment-emission to plain deltas + layer.md sidecars (layer content = existing delta frontmatter/tables verbatim; requirement text byte-preserved from current mirror; scenario headers verbatim for ah bindings), check in the /tmp/os1x 1.14.2 shape fixture (CONFIRMED still present at /tmp/os1x with cap-a deployed dual spec + cap-b plain delta + sidecar layer), then 1.14.x strict validate 51-52 items + spk lint 0 findings + just ci green. Tasks 6-7 remain for later runs. Global openspec CLI is 0.19.0; 1.14.2 runs via npx (see scripts/spec_validate.py for the pinned pattern).
