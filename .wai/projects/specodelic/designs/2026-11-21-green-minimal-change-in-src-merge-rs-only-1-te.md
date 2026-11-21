---
tags: [pipeline-run:tdd-ro5-2026-11-21-specodelic-1izc-merge-add-add-order-asymmetry-fix, pipeline-step:green]
---

GREEN: minimal change in src/merge.rs only. (1) textual_conflict_findings now computes contested paths at RAW-tree level (a∩b, differing text, base absent OR both-changed-from-base) — covers add/add including unparsable-side holes the parsed-tip version missed — and flags kind=textual_conflict per path, returning BTreeSet<String> contested. (2) relint_findings takes contested set and skips those paths (a contested file has no true merged content; the textual_conflict flag already fails the verdict) + guards on merged_specs. (3) run() threads the set; union_tree untouched (its pick is now unobservable); doc comments updated. Narrow: cargo test --test hegel_properties merge_ → 7/7 ok (RED exhibit + 5 laws + guard). Suite: just test-smart → 1589 selected, all pass. Exhibits: merge_verdict_symmetry_add_add ok; no pinned law regressed.
