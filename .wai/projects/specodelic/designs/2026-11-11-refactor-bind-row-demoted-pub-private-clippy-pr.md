---
tags: [pipeline-run:tdd-ro5-2026-11-11-specodelic-edrv-binds-nothing, pipeline-step:refactor]
---

REFACTOR: bind_row demoted pub->private (clippy private_interfaces — pub fn exposed the private EvidenceUnit; bind_rows is the public surface, mirrors pack_check_row/PackCheck being paired). Tidied the amendment pin's string continuation into one line. cargo fmt normalization. PRE-EXISTING gate fix (not authored this session, required by the gate): removed one blank line after the pack_row_tests doc comment at HEAD line 1097 — newer clippy's empty-line-after-doc-comments fails -D warnings; verified present at HEAD via git show. Justification: just ci must pass to report done; one-character tidy adjacent to touched code. Tests green after refactor: lib audit 21, audit_binds_nothing 1, audit_oracle_rows 14, clippy --all-targets -D warnings exit 0, fmt --check clean.
