---
tags: [pipeline-run:tdd-ro5-2026-11-11-specodelic-xch1-oracle-row-classification-before-diff, pipeline-step:red]
---

RED: command='TMPDIR=/var/tmp/specodelic-xch1 cargo test --test audit_oracle_rows' expected failure=compile E0432 — could not find audit in specodelic (tests/audit_oracle_rows.rs imports specodelic::audit::{AuditClass, AuditRecord, AuditReport, build_report, diff_with_prior, exit_code, run, AUDIT_SCHEMA_VERSION}); exactly the missing slice-1 surface (classification + --before diff + exit contract), no setup breakage.
