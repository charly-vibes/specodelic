---
tags: [pipeline-run:tdd-ro5-2026-11-11-specodelic-xch1-oracle-row-classification-before-diff, pipeline-step:fix-review]
---

RO5U-FIXES: M1 fixed — diff_with_prior now refuses prior reports whose report_schema_version differs from AUDIT_SCHEMA_VERSION (labeled before_diff, message names schema versions + remediation), test before_schema_version_mismatch_refuses (RED observed: failed on missing guard, GREEN after). L1 fixed — parse_prior_report refuses duplicate scenario ids in the prior (the diff keys rows by id; a duplicate would let .find() pick silently), test before_parse_refuses_duplicate_prior_rows (RED→GREEN). L2 deferred with reason: audit_json helper recompiling per call is a test-speed nicety, zero observable drift — not worth the churn in this slice. Suite: 14/14 --test audit_oracle_rows; just test-smart exit 0; just lint clean.
