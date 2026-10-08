---
tags: [pipeline-run:tdd-ro5-2026-10-08-specodelic-gre-8-derived-diagrams-stable-under-prose-perturbation-task-2-7, pipeline-step:red]
---

RED(characterization): command=python3 -m unittest discover -s scripts -p test_graph_views.py. Expected outcome per brief: the new views_from_artifact_only test PASSES on current code — it pins existing correct prose-independence, not pending behavior (RED was the missing test itself, now added in scripts/views_purity_cases.py). Observed: 58 tests OK including test_views_from_artifact_only. Verified en route: spk graph --json over the violation-bearing fixture exits 1 (diagnostic) with ok:true envelope — test accepts exits (0,1) and asserts the envelope contract; TSV bytes and envelope data asserted equal; perturbation no-op guard included. Pre-verified manually: perturbed corpus TSV + data identical to original.
