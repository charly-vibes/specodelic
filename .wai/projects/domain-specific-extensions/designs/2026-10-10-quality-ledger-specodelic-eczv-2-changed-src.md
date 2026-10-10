---
tags: [pipeline-run:tdd-ro5-2026-10-10-specodelic-eczv-2-conform-phase-2-report-schema-and-scope-digest, pipeline-step:quality-ledger]
---

QUALITY-LEDGER specodelic-eczv.2:

Changed:
- src/conform.rs — persisted-report layer: REPORT_SCHEMA_VERSION (conform-local), EVIDENCE_SCOPE const, ConformReport (schema version, evidence_scope, scope_sha256, closed_world header flag, verdict_counts, records sorted by scenario id, no timestamps), build_report over the phase-1 engine, scope_digest composing verify::scope_digest (structured content, model_check path reused unchanged) with model_check::artifact_sha256 over the consumed corpus bytes, human_view + emit_report through genesis Output::emit (JSON envelope default, human view with evidence_scope and counts). VerdictRecord gained Eq.
- tests/conform_report.rs (new) — report_schema_roundtrip, evidence_scope_present_in_both_views, rerun_byte_identical, digest_binds_scenarios.
- openspec/changes/add-conform/tasks.md — tasks 2.1–2.4 checked off (checkboxes only).

Verified:
- TMPDIR=/var/tmp/specodelic-eczv cargo test --test conform_report → 4/4 green
- cargo test --test conform_classification → 13/13 green (phase-1 untouched)
- just test-smart → 495 selected, exit 0
- just lint → clean (clippy -D warnings; known foreign errors only under --all-targets, specodelic-9b1h)
- just pretender-check → exit 0, no new reds, no thresholds raised
- src/model_check.rs untouched (2282 lines), src/verify.rs untouched — model_check digest contract byte-identical

Review:
- RO5U converged at stage 4, verdict pass; zero critical/high/medium; both lows fixed in-tidy.

Risks:
- conform digest binds structured content + corpus bytes per the delta constraint; artifact-hash binding was NOT added (delta does not require it; phase-3 artifact gate may revisit).
