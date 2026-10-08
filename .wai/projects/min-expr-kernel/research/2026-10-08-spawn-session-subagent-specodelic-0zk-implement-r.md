---
tags: [pipeline-run:epic-orchestrator-2026-10-08-bug-fix-sweep-run-5-specodelic-0zk-graph-projection-exit-code-semantics-bad-path-silent-success-violations-exit-0-in-text-modes-f3-f8, pipeline-step:spawn-subagent]
---

SPAWN: session=subagent:specodelic-0zk:implement report=F3 unresolved root -> exit 2 labeled refusal to stderr (projection stdout stays raw text channel; zero-spec-files-on-real-path keeps exit 0); F8 typing violations -> exit 1 in text modes, annotation rows ride along per D3. Commit 5b2f5fe (+f660e9a beads note), CHANGELOG #121, new tests/cli/graph_projection.rs (6 tests), 4 stale exit-0 pins updated. just test 836 passed. Meter discrepancy: fixture carries 7 violations, not six as ticket meter said. Follow-up flagged: dangling refs + supersedes cycles still exit 0 in text modes (JSON exits 1) — NOT widened silently.
