---
tags: [pipeline-run:epic-orchestrator-2026-10-08-bug-fix-sweep-run-4-specodelic-efb-acset-instance-builder-panic-on-member-path-label-qualified-link-targets, pipeline-step:spawn-subagent]
---

SPAWN: session=subagent:specodelic-efb:implement report=intern canonical_id(target) at instance.rs:377 + graph::build records canonical target (parity); resolve() pinned contract untouched; typing still consults raw spelling (member-path keeps resolved-clean semantics). Commit ffb3e87, CHANGELOG #120. just ci green. Repro exits 0 with canonical edge v.mem -> v.row; dangling member paths stay labeled exit 1. Deviation: adjacent pre-existing panic on [[model.state]] anchor links NOT fixed (scope) — filed specodelic-39w with fixture + fix directions.
