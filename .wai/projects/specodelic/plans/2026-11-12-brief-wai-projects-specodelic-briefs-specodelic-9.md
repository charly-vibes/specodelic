---
tags: [pipeline-step:write-brief, pipeline-run:epic-orchestrator-2026-11-12-specodelic-ez9a-2-migrate-namespace-bug, pipeline-run:epic-orchestrator-2026-11-12-specodelic-ez9a-2-migrate-namespace-bug]
---

BRIEF: .wai/projects/specodelic/briefs/specodelic-ez9a.2.md - reproduce end-to-end in a scratch clone (wrap delta -> scaffolded sidecar with an error constraint labeled by its own id -> archive -> lint fails), then fix the namespace derivation in src/migrate.rs scaffold and/or src/archive_companion.rs rekey_sidecar (rekey bare error-label occurrences, not just wikilinks), regression tests first; keep byte-plain delta + fail-closed layer contracts; targeted tests only.
