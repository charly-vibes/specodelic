---
tags: [pipeline-run:tdd-ro5-2026-10-10-specodelic-eczv-2-conform-phase-2-report-schema-and-scope-digest, pipeline-step:refactor]
---

REFACTOR specodelic-eczv.2 (2.4 tidy): assessment — the shared helper ALREADY exists (verify::scope_digest, the model_check computation path); conform reuses it unchanged and keeps only the corpus-byte binding conform-local, so no extraction is performed and model_check's digest contract stays byte-identical (src/verify.rs and src/model_check.rs untouched, model_check.rs still 2282 lines). Test-file tidy only: claim_schema_version cross-reference assertion uses the imported const in its message instead of a dead let. Gates: conform_report 4/4 green, clippy --all-targets shows ONLY the two known foreign errors (src/lint/mod.rs:1605, src/migrate.rs:377 — specodelic-9b1h), just lint clean, pretender-check exit 0
