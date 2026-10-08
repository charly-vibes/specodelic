---
tags: [pipeline-run:epic-orchestrator-2026-10-08-specodelic-68m-5-claim-blockers-report-parity, pipeline-step:write-brief]
---

# Brief: specodelic-68m.5 — report view parity + docs assurance

Task: tasks.md 3.1 (RED) + 3.2 (GREEN). D2/D3 semantics untouched.

RED:
1. Parity fixtures in tests/cli/model_check.rs + orchestrate_verify.rs: on
   mixed fixtures (verified Rust + refuted kernel; verified Rust + unknown
   citation; prose-only unchecked) assert JSON envelope, persisted report,
   and human output agree on claim counts, blocker sets, unchecked sets.
2. Docs-consistency test: version literals in README.md / docs/src/status.md
   / openspec/project.md / specs/STATUS.md / guide.rs must not conflict with
   Cargo pkg version; capability-status claims must match implemented state.

GREEN:
1. Smallest changes in src/human.rs (+ model_check/verify/orchestrate emit
   paths) so all three views share the same claim fields (already largely
   shared after 68m.4 TIDY).
2. Sync docs with implemented capability status; document bounded model
   checking of opted-in invariant claims vs application-test execution.

Refactor step: NO-OP (68m.6 owns TIDY).

