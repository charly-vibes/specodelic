---
reviews: 2026-11-12-ro5u-review-q7uo-diff-critical-high-medium-non.md
verdict: pass
tags: [pipeline-run:tdd-ro5-2026-11-12-specodelic-q7uo-compile-pipe-unescape, pipeline-step:ro5u-review]
---

# RO5U findings — specodelic-q7uo diff

## Critical / High / Medium
None.

## Low
1. src/compile.rs::unescape_cell_pipes — fragment string literal with an intentional backslash-pair before a pipe loses one backslash; documented in the helper doc, same accepted semantics as packs.rs's ESC protection, unreachable at expression position. Fix: none required.
2. IR guard_citations / emits_values keep raw text — citation grammar uses ∧/∨/¬ never pipes; ticket scopes fragment emission only. Fix: none; different ticket if it ever arises.
