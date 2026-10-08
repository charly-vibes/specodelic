---
reviews: 2026-10-08-refactor-extracted-shared-rendering-helpers-into.md
verdict: pass
tags: [pipeline-run:tdd-ro5-2026-10-08-specodelic-gre-8-derived-diagrams-stable-under-prose-perturbation-task-2-7, pipeline-step:fix-review]
---

RO5U review of specodelic-gre.8 (commits 42407ec, decbf45):

Critical: none.
High: none — byte-compat of the view_common extraction is proven by the 58-test suite pinning exact renders; CLI subprocess tests cover both views' exit/refusal contracts; tasks.md 2.7 ticked.
Medium: none.
Low 1: views_purity_cases depends on a prebuilt target/{debug,release}/specodelic — mitigated: fails loudly with a 'just build' hint, never silently skips.
Low 2: PERTURBATIONS anchors are tied to fixture wording — a future fixture edit fails the test loudly naming the anchor; acceptable for a characterization pin.
Fixes required: none.

