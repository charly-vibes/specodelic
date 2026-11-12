---
tags: [pipeline-step:write-brief, pipeline-run:epic-orchestrator-2026-11-12-specodelic-75m-revision-upgrade-tooling, pipeline-run:epic-orchestrator-2026-11-12-specodelic-75m-revision-upgrade-tooling, pipeline-step:write-brief]
---

BRIEF: .wai/projects/specodelic/briefs/specodelic-75m.md - spec first: openspec change add-revision-upgrade (migrate delta + layer sidecar) defining a thin core revision registry and the upgrade contract (--to-revision, dry-run default, unknown revision labeled, byte-identical no-step files); then RED fixture corpus at an older real revision + byte pins + dry-run + unknown-revision failures, GREEN registry/upgrade in src/migrate.rs + main.rs wiring; archive via just archive-change, spk init --force, revert tools.toml churn; keep wrap/rekey untouched.
