---
reviews: 2026-11-21-specodelic-1izc-fix-plan-desired-behavior-merge.md
verdict: pass
tags: [pipeline-run:tdd-ro5-2026-11-21-specodelic-1izc-merge-add-add-order-asymmetry-fix, pipeline-step:ro5u-review]
---

RO5U review of the 1izc diff (src/merge.rs + tests/hegel_properties/main.rs). CRITICAL: none — law exhibit green, 7/7 merge laws+guards, 1589 test-smart pass; post_merge_relint_required gates 'passed' and contested paths force textual_conflict→failed, so relint exclusion cannot create a false pass; no merge-spec scenario pins the old parsed-level condition. HIGH: none. MEDIUM M1: textual_conflict_findings over the raw slice lost dedup — duplicate path in one tree now double-flags (count wart, not an asymmetry; input contract says sets); fix: insert-then-check guard via the contested set. MEDIUM M2: base lookup via parsed tip misclassifies both-edited-from-unparsable-base as add/add — prose-only, pre-existing tip-model limitation (union_tree deletion loop shares it); record as known limitation at closure. LOW L1: O(n·m) find matches file style (union_tree) — accepted. LOW L2: test edits doc-only + un-ignore, no assertion drift. Verdict PASS; M1 to fix in fix-review step.
