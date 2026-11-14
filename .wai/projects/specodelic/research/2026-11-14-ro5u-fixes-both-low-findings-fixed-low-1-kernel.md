---
tags: [pipeline-run:tdd-ro5-2026-11-14-specodelic-9ilm-resolves-over-an-empty-morphism-population-is-vacuous-decide-and-document, pipeline-step:fix-review]
---

RO5U-FIXES: both LOW findings fixed. LOW-1: kernel.md Notes and the decision note now list only real unknown causes — an id literal naming no declared id, a reachable seed/target not in the instance, an unsupported backend; the impossible absent-morphism clause is gone. LOW-2: the pin now asserts the CLEAN fixture has no uses edges via env.instance().edges(), so it cannot silently stop exercising an empty population. Verified: cargo test --test kernel_status 8 passed; cargo fmt clean.
