---
tags: [pipeline-run:epic-orchestrator-2026-10-08-bug-fix-sweep-run-2-specodelic-m6k-model-check-stale-compile-demotes-malformed-kernel-claim-to-unchecked-f2, pipeline-step:spawn-subagent]
---

SPAWN: session=subagent:specodelic-m6k:implement report=kernel::grammar_failures pre-run guard in cmd_model_check — stale compile never demotes malformed kernel claim; labeled kernel_grammar findings in .data.failed, exit 1; prose-boundary stays unchecked. Commit 7d3250a (+CHANGELOG #118, docs sentence). just test + just ci + pretender green.
