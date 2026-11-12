---
tags: [pipeline-run:tdd-ro5-2026-11-12-specodelic-ez9a-1-add-shared-fragment-helpers, pipeline-step:red]
---

RED: cargo test --test fragment_helpers → 8 failed for expected missing behavior (no inlining, no collision/hygiene/cycle/unknown-ref labeled failures, no dead-helper warning); 2 passed by design (byte-preservation pin vs PRE_CHANGE_PROPS, lint-opaque-section pin). Tasks 1.1-1.6 covered: 1.1 helper_inlines_at_each_use_site, 1.2 collision labeled, 1.3 absent_section_preserves_bytes pin, 1.4 hygiene labeled, 1.5 cycle+transitive, 1.6 lints clean. Plus requirement scenarios: unknown reference labeled, unused-def warns+success, direct properties_to_proptest path.
