---
tags: [pipeline-run:tdd-ro5-2026-10-15-specodelic-nvs9-migrate-wraps-modified-only-deltas-gh-16, pipeline-step:red]
---

RED: command=cargo test --test cli -- migrate_modified_only migrate_no_delta_section; expected failure=both new tests fail for the missing behavior — MODIFIED-only delta refused with 'no ## ADDED Requirements section found — nothing to wrap' (code 2), and refusal message does not name both sections. Verified live; 294 other tests filtered out unaffected. Found: stale remediation hint 'a delta must carry an ## ADDED Requirements section' at src/commands/manage.rs:659 (outside allowed files — will flag as deviation).
