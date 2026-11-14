---
tags: [pipeline-run:tdd-ro5-2026-11-14-specodelic-9ilm-resolves-over-an-empty-morphism-population-is-vacuous-decide-and-document, pipeline-step:plan]
---

PLAN specodelic-9ilm: Desired behavior — document and ratify the vacuous-truth reading: resolves(m) over an empty morphism population is Verified, not Unknown/labeled. Tests: add tests/kernel_status.rs::resolves_over_an_empty_morphism_population_is_vacuously_verified, using the CLEAN corpus (no uses edges) and parse_kernel_str("resolves(uses)"), asserting ThreeValued::Verified; also assert the non-vacuous BROKEN corpus still yields Counterexample through its existing test (no change). Narrow command: cargo test --test kernel_status. Full: just ci. Files: src/kernel.rs comments only if needed, openspec/specs/kernel/spec.md Notes, openspec/decisions/2026-11-14-resolves-empty-population.md (new), tests/kernel_status.rs. Out of scope: no semantic change, no new lint rule.
