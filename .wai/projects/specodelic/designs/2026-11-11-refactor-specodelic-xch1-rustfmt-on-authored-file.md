---
tags: [pipeline-run:tdd-ro5-2026-11-11-specodelic-xch1-oracle-row-classification-before-diff, pipeline-step:refactor]
---

REFACTOR specodelic-xch1: rustfmt on authored files only (src/audit.rs, tests/audit_oracle_rows.rs — repo-wide fmt skipped: known rustfmt drift at HEAD on committed foreign files, eczv.5 ledger); tests/audit_oracle_rows.rs envelope assertion corrected during GREEN (failure message rides the warnings channel, not an 'error' field — genesis failure convention); no 1.5 refactor (report-diff extraction is ticket gheu's scope); tasks.md 1.1-1.4 ticked. Narrow suite 12/12 green; just lint clean.
