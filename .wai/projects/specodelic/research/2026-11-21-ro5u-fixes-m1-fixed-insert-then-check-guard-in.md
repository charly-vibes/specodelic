---
tags: [pipeline-run:tdd-ro5-2026-11-21-specodelic-1izc-merge-add-add-order-asymmetry-fix, pipeline-step:fix-review]
---

RO5U-FIXES: M1 FIXED — insert-then-check guard in textual_conflict_findings ((add_add||both_edited) && contested.insert(path)) so a path repeated within one tree slice flags once; new unit test merge::textual_conflict_dedup::duplicate_path_in_tree_flags_textual_conflict_once RED first (2 flags observed) then GREEN (1). fmt applied (test line wrap). M2 DEFERRED with reason: parsed-tip base lookup misclassifying both-edited-from-unparsable-base is a pre-existing tip-model limitation, prose-only (message wording), no verdict/kind/contested effect; recorded on ticket closure. L1 accepted (file-style O(n·m)); L2 n/a. Post-fix verification: textual_conflict_dedup 1/1 ok; hegel merge_ 7/7 ok; fmt clean; clippy -D warnings clean.
