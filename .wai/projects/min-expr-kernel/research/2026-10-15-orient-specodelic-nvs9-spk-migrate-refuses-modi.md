---
tags: [pipeline-run:tdd-ro5-2026-10-15-specodelic-nvs9-migrate-wraps-modified-only-deltas-gh-16, pipeline-step:orient]
---

ORIENT: specodelic-nvs9 — spk migrate refuses MODIFIED-only deltas (NoDeltaSection at src/migrate.rs:192 triage + :54 msg). Fix: extend wrap to accept ## MODIFIED Requirements section, mirror holds MODIFIED body in modified form (reuse 54v per-requirement aggregation); NoDeltaSection message updated to name both sections; REMOVED-only stays refused labeled. Files: src/migrate.rs, tests/cli/migrate.rs.
