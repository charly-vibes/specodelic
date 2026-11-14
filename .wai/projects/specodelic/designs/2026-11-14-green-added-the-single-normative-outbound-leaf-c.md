---
tags: [pipeline-run:tdd-ro5-2026-11-14-specodelic-mlc-consolidate-the-outbound-leaf-carve-out-into-one-normative-statement, pipeline-step:green]
---

GREEN: added the single normative 'Outbound-leaf carve-out (Revision 22)' paragraph under specodelic.md's Reference Typing; added ## Revision 22 — 2026-11-14; bumped FORMAT_REVISION 21->22 and migrate LATEST_REVISION/registry (no step) + dated-vector test; added the CHANGELOG Revision 22 entry; refreshed the managed AGENTS block via spk init --force (adds the xci rules too); rewrote the ten pack/observability/linter files to at most one pointer line each (identifier spellings untouched); added tests/outbound_single_source.rs enforcing one home. Verified: cargo test --test outbound_single_source 1 passed; cargo test --lib 437 passed; guide/migrate lib tests pass; just lint-specs 49 files 0 issues; openspec validate --all --strict 54/54.
