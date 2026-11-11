---
tags: [pipeline-run:tdd-ro5-2026-11-11-specodelic-6hou-slice-4-gate-pass, pipeline-step:orient]
---

ORIENT: specodelic-6hou slice 4 tasks.md 4.3 (gate pass + refactor) — just ci proven green at HEAD 57b9c62 first; then ONE net-line-negative tidy: relocate audit.rs rung aggregate block (RungAggregate/outcome_name/EvidenceMix/rung_from_evidence/current_check_evidence/with_derived_rung, ~lines 598-773) into new src/audit/rung.rs with pub use re-export (paths preserved — tests/audit_rung.rs imports specodelic::audit::{...} untouched); bare 'use crate::model_check;' leaves audit.rs with the block (line 207 uses crate::-qualified path); lib.rs needs NO edit (mod audit already declared). Ratchet: 2294/2300 → expect ~2122. Proof: frozen fixture dir under /var/tmp/6hou-proof (SPEC+PROPS+CORPUS from tests/audit_rung.rs, compile + model-check artifacts) — audit --json/--human before vs after cmp byte-identical.
