---
tags: [pipeline-run:tdd-ro5-2026-11-12-specodelic-d8c-enforce-reference-typing-ref-kind-compatible-as-a-lint-rule, pipeline-step:ro5u-review]
---

REFACTOR: no separate tidy commit needed — lint_ref_kinds was decomposed at birth (TypedEndpoints::build/resolve + row_endpoint helper + single walker, no per-field match arms); fmt/clippy already clean; pretender unchanged vs HEAD (11=11 pre-existing grandfathered reds, exit 0). Tests green after refactor: just test exit 0.
