---
tags: [pipeline-run:tdd-ro5-2026-11-12-specodelic-q7uo-compile-pipe-unescape, pipeline-step:ro5u-review]
---

# RO5U findings — specodelic-q7uo diff

## Critical
None.

## High
None.

## Medium
None.

## Low
1. `src/compile.rs::unescape_cell_pipes` — a fragment string literal containing an intentional backslash-pair before a pipe (`\\|`) loses one backslash. Evidence: literal two-char replacement. Fix: none required — documented in the helper doc; identical accepted semantics to packs.rs's ESC protection; no expression-position Rust begins with a backslash, so the intent is unreachable in practice.
2. `src/compile.rs::extract_model_ir` guard_citations / emits_values — those cells keep raw escaped text. Evidence: citation grammar uses ∧/∨/¬, never pipes; ticket scopes fragment emission only. Fix: none; a future guard-cell pipe requirement is a different ticket.

