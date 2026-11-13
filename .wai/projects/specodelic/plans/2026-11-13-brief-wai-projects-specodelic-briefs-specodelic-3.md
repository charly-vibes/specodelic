---
tags: [pipeline-step:write-brief, pipeline-run:epic-orchestrator-2026-11-13-specodelic-u7kc-verify-runner-seams, pipeline-run:epic-orchestrator-2026-11-13-specodelic-u7kc-verify-runner-seams, pipeline-step:write-brief]
---

BRIEF: .wai/projects/specodelic/briefs/specodelic-u7kc.md - three seams spec-first (change add-verify-runner-seams with design.md settling P1 path dep on the lib, P2 minimal fixture builder API (no format Revision), P3 properties-only verdict that never weakens both_gates_required); RED tests (unresolved crate today, fixture materialization, verdict exactness), GREEN in src/verify.rs (+lib fixtures module, compile.rs only if emission changes), ONE demonstration compile property row migrated to a **rust:** fragment calling the real tool with a real strategy via the change/archive; no re-implemented checkers, no Just(name)/true scaffolds; cargo fmt; targeted tests only.
