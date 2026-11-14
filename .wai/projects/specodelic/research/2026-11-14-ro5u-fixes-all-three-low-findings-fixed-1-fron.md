---
tags: [pipeline-run:tdd-ro5-2026-11-14-specodelic-3a8-decouple-dual-format-metadata-into-an-openspec-adapter, pipeline-step:fix-review]
---

RO5U-FIXES: all three LOW findings fixed. (1) frontmatter_lines is now in src/spec.rs and shared by parse_frontmatter and the adapter, removing the duplicated offset logic; the adapter calls crate::spec::frontmatter_lines. (2) tests strengthened: distinct exact bodies for ADDED/Requirements/MODIFIED and a plain-spec CLI assertion that the six keys are false/empty. (3) stale dual-section comment reworded. Verified: cargo test --lib 437 passed; cargo test --test spec_parse 10 passed; CLI parse_json_keeps_dual_format_fields passed; pretender check exit 0.
