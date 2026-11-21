---
tags: [pipeline-run:tdd-ro5-2026-11-21-specodelic-1izc-merge-add-add-order-asymmetry-fix, pipeline-step:red]
---

RED: command=cargo test --test hegel_properties merge_verdict_symmetry_add_add; expected failure=add/add asymmetry; actual=assert kinds_ab==kinds_ba failed at main.rs:590 — left(ab)=[blast_radius_intersection + 13×id_collision], right(ba)=left + relint_failure; verdict assert (line ~586) passed (failed both orders at HEAD, per ticket); failure is the missing add/add conflict+relint-exclusion behavior, not setup breakage. Test un-ignored in same commit series (test-file-only change).
