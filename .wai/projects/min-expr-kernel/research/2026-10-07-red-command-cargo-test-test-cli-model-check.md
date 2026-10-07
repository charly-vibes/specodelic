---
tags: [pipeline-run:tdd-ro5-2026-10-07-specodelic-68m-3-verification-rejects-evidence-from-another-input-scope, pipeline-step:red]
---

RED: command='cargo test --test cli model_check::' — 9 freshness fixtures fail for the intended missing behavior at HEAD: old_report/unrecognized_schema/missing_claim/duplicate_claim/dropped_input exit 0 (verify accepts stale evidence), cross_file_edit leaves scp_a verified, report_binds/reversed_order/dual_format panic on absent claim_schema_version+scope_sha256 (RED GREEN targets). Pins passing as expected: artifact_edit_after_report (existing staleness), combined dual-format isolated_scope_required, orchestrate preflight, multi-file lint. Also adapted verify_accepts_clean_report_with_current_artifacts to real kernel-claim evidence (forged-report fabrication is now the rejected class).
