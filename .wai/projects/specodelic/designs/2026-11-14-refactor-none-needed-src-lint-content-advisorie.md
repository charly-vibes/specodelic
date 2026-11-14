---
tags: [pipeline-run:tdd-ro5-2026-11-13-specodelic-xci-advisory-warnings-for-placeholder-and-tautology-content, pipeline-step:refactor]
---

REFACTOR: none needed — src/lint/content_advisories.rs is already decomposed into single-purpose helpers (strip_code_span, strip_rust_marker, is_placeholder, is_tautology, is_self_comparison, carries_scaffold_signature); the four new tests were moved to src/lint/tests/content_advisories.rs to stay under the 2300-line src/lint/mod.rs ratchet (mod.rs 2177). No behavior change. Verified: cargo test --lib lint 102 passed; cargo test --test cli lint 53 passed; pretender check exit 0 (the pre-existing lint_checklists cognitive-34 red is grandfathered, not introduced).
