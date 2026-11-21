---
tags: [pipeline-run:tdd-ro5-2026-11-21-specodelic-1izc-merge-add-add-order-asymmetry-fix, pipeline-step:orient]
---

ORIENT: specodelic-1izc — merge add/add order-asymmetry. Single task: union_tree in src/merge.rs reads base.texts.get(rel) (None for add/add paths) so A's version always survives; relint then judges the winner, making run(base,a,b) != run(base,b,a) finding multisets and flipping verdicts when only one side is broken. RED test exists ignored at tests/hegel_properties/main.rs:571 (merge_verdict_symmetry_add_add, specodelic-jnnf exhibit). Ticket claimed. No prior research; fresh fix. Blast radius: merge module only per C-6 grounding.
