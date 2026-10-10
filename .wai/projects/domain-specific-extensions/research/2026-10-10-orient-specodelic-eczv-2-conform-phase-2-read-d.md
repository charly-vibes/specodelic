---
tags: [pipeline-run:tdd-ro5-2026-10-10-specodelic-eczv-2-conform-phase-2-report-schema-and-scope-digest, pipeline-step:orient]
---

ORIENT: specodelic-eczv.2 conform phase 2 — read design.md (D4 report reuse, D6 evidence_scope as data, digest binding structured content + corpus bytes), tasks.md 2.1-2.4, delta spec verdict_report_schema; phase-1 conform.rs exposes Verdict/VerdictRecord/classify_traces/parse_corpus; digest machinery = verify::scope_digest (public, model_check write path via orchestrate:430); emit pattern = main.rs emit_report + genesis Output::emit with human_text override; conform digest plan: compose verify::scope_digest(structured content) + corpus-bytes sha256 under a conform-local REPORT_SCHEMA_VERSION payload — model_check contract untouched
