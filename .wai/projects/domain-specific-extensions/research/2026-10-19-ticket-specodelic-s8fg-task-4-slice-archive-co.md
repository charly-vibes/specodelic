---
tags: [pipeline-run:epic-orchestrator-2026-10-19-delta-sidecar-task4, pipeline-step:select-ticket]
---

TICKET: specodelic-s8fg (task 4 slice — archive-companion redesign). s8fg already claimed in_progress (commit 5aa84df), so no re-claim. tasks.md 1-3 verified landed via git log (8901bc1/378058b/73b698e); ticked their checkboxes for consistency. This run implements task 4 (4.1-4.3): RED tests in tests/cli archive_companion suite, then rework src/archive_companion.rs for openspec apply + layer re-attach with fail-closed guards. Remaining tasks 5-7 stay for later runs.
