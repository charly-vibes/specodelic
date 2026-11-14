---
tags: [pipeline-run:tdd-ro5-2026-11-14-specodelic-9ilm-resolves-over-an-empty-morphism-population-is-vacuous-decide-and-document, pipeline-step:red]
---

RED: added tests/kernel_status.rs::resolves_over_an_empty_morphism_population_is_vacuously_verified using CLEAN (no uses edges) and resolves(uses), expecting ThreeValued::Verified. The behavior already matches the ratified decision, so this is a regression pin, not a behavior gap; the first run failed only on an attempted emptiness assertion that used a pub(crate) helper from an integration test, which was removed. Run: cargo test --test kernel_status -> 8 passed. Red evidence is the absent documentation/pin; no production behavior change is needed.
