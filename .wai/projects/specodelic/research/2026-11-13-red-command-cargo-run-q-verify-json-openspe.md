---
tags: [pipeline-run:tdd-ro5-2026-11-13-specodelic-lf3-migrate-packs-properties, pipeline-step:red]
---

RED: command=cargo run -q -- verify --json openspec/specs/packs/spec.md expected failure=properties_failed — 16 of 16 compiled blocks failed (every block still todo_predicate!; the scratch crate compiles and runs, so the failure is the missing executable predicate, not setup breakage). The new fixture harness is proven by cargo test --test packs_fixtures (9 passed): the packs generator names all resolve to pack fixtures, the corpus harness discovers packs order-independently, activates a pack for a vocabulary consumer, reports the labeled orphan, reports the skew advisory on the warnings channel, rejects a narrowing pack, and types a fiber kind only while its pack is active.
