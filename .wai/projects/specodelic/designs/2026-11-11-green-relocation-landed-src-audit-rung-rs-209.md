---
tags: [pipeline-run:tdd-ro5-2026-11-11-specodelic-6hou-slice-4-gate-pass, pipeline-step:green]
---

GREEN: relocation landed — src/audit/rung.rs (209 lines: file header + use serde + use super::AuditReport + crate::model_check + crate::spec::Spec + the verbatim 587-764 block) and src/audit.rs 2294→2122 (mod rung; + pub use rung::{EvidenceMix,RungAggregate,rung_from_evidence,with_derived_rung} at the cut site; unused 'use crate::model_check;' removed with the block). NET -172 lines, ratchet 2294/2300 → 2122/2300. Narrow: just test-smart exit 0; full cargo test 994 passed 0 failed — BASELINE verified identical at clean HEAD via stash dance (994/0 both sides, zero tests lost; the 37hl note's 997 was an aggregated-suite count). Byte-identity proof: /var/tmp/6hou-proof before/ vs after/ cmp IDENTICAL on all three envelopes (audit --json rung-3, audit --human, audit --json rung-2 no-check), exit codes 0 all. cargo fmt --check clean; cargo build + clippy -D warnings clean.
