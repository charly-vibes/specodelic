---
tags: [pipeline-run:tdd-ro5-2026-10-06-add-min-expr-kernel-phase-3-specodelic-7ga-tier-b-kernel-core, pipeline-step:refactor]
---

REFACTOR 3.5 (predicate-registry seam): tests/kernel_registry.rs (3 tests) — the shipped registry is EXACTLY the v0 closed set (no pack predicate registers, D8); the registration gate refuses NoGrounding (D1), Undecidable (D8 — stays pack-side via contract-TOML runners), Duplicate; a grounded+decidable registration widens PURELY (a battery of v0 expressions evaluates identically before/after) and does NOT widen this Revision's grammar (pack_only(x) still fails labeled — grammar-level widening rides the widening law in a later change). Registry seam code landed with GREEN 3.2 in src/kernel.rs (PredicateRegistry::{v0,grounding,names,try_register}, Registered, RegisterError). Gates: just test green, lint-specs 0, spk lint deltas 0, clippy/fmt clean.
