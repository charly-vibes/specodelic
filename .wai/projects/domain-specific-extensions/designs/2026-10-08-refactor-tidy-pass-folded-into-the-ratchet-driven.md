---
tags: [pipeline-run:tdd-ro5-2026-10-08-specodelic-gre-6-per-file-state-diagrams-tasks-1-7-render-2-3-corpus-2-4, pipeline-step:ro5u-review]
---

REFACTOR: tidy pass folded into the ratchet-driven split — pure renderers (schema_view.py, state_view.py) carry no CLI/IO; graph_views.py owns artifact reading, usage errors, labeled refusals with remediation hints; view_common.py holds the shared failure vocabulary. No behavior change: 37/37 python + 250/250 cli re-run green after the split.
