---
tags: [pipeline-run:tdd-ro5-2026-11-11-specodelic-gheu-extract-report-diff-into-testable-unit, pipeline-step:orient]
---

ORIENT: specodelic-gheu — brief read; design.md D1 (report-diff-not-files) + risks read; src/audit.rs read in full (671 lines, no in-module test module; diff_with_prior + parse_prior_report already standalone pub since a435157); tests/audit_oracle_rows.rs holds the 14 pin tests (11 library-level incl. diff unit coverage, 3 CLI-level); git tree clean of foreign staged files; ticket = tasks.md 1.5 extraction + focused in-module unit tests for the diff unit (same-class⇒current, worsened⇒stale, new⇒new, digest-mismatch⇒labeled refusal, empty-prior⇒all-new), zero behavior change with byte-identical envelope proof
