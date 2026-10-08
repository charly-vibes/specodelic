---
tags: [pipeline-run:epic-orchestrator-2026-10-08-bug-fix-sweep-run-5-specodelic-0zk-graph-projection-exit-code-semantics-bad-path-silent-success-violations-exit-0-in-text-modes-f3-f8, pipeline-step:spawn-subagent]
---

VERIFY specodelic-0zk (orchestrator, isolated worktree at 4ac6cf5): initial just ci at f660e9a FAILED (sync-sections-test views_purity corpus expected exit 0 on violations) — fixed forward via bounded spawn subagent:specodelic-0zk:bounded-fix commit 4ac6cf5 (scripts/views_purity_cases.py only, expect (1,) for violation-bearing corpus; test assertions on contents untouched); full just ci green after. Meter: nonexistent path --format edges exit 2 labeled; typing violations --format edges exit 1 with 7 annotation rows riding along; JSON mode exit 1 unchanged. Ticket meter said six rows — actual 7 (JSON data.violations confirms 7; ticket miscounted). Follow-up filed: specodelic-nkg (dangling refs/cycles text-mode exits). PASS.
