---
tags: [pipeline-run:tdd-ro5-2026-10-10-specodelic-eczv-2-conform-phase-2-report-schema-and-scope-digest, pipeline-step:red]
---

RED: command='TMPDIR=/var/tmp/specodelic-eczv cargo test --test conform_report' expected failure=compile E0425/E0432 — conform has no report API (build_report/emit_report/scope_digest/ConformReport/EVIDENCE_SCOPE/REPORT_SCHEMA_VERSION absent), exactly the missing phase-2 behavior; 4 errors, all naming the missing report surface
