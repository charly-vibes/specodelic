---
tags: [pipeline-run:tdd-ro5-2026-10-08-specodelic-54v-mixed-delta-migrate-mirror-fails-lint-added-modified, pipeline-step:orient]
---

ORIENT: specodelic-54v — mixed-delta (ADDED+MODIFIED) migrate mirror drops MODIFIED body (D6 ADDED-only slice), guaranteed requirement_drift lint failure post-CHANGELOG #93. Fix: option 1 (aggregate ADDED+MODIFIED per-requirement); labeled refusal for unfaithfully-aggregatable cases (duplicate requirement heading across sections). Read src/migrate.rs D6, lint/rules.rs requirement_drift (per-req, all carried sections), openspec/specs/migrate/spec.md (mirror_byte_identical constraint — semantics change needs openspec change overlay, openspec/specs is read-only).
