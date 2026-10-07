---
tags: [pipeline-run:tdd-ro5-2026-10-07-specodelic-bf5-ss5-1-5-2-opaque-kernel-binding-extraction-and-claim-carrier, pipeline-step:fix-review]
---

RO5U-FIXES: no critical/high findings. Medium#1 (kernel.binding vocab-orphan interplay) deferred out of ticket — filed bd issue (see bd log) for the §6/archive decision; CLI fixture demonstrates the sanctioned pack-enablement path. Lows #2-#5: no code change — #2/#3 are correct-by-precedent behavior (kernel_cell_content fence shaping, exact-kind scope), #4 is a markdown-table format constraint, #5 verified by no_semantic_drift on binding fixtures. No behavior-impacting fixes → no new tests. Suites re-run after review: kernel_binding 7/7, cli parse_misc::compile_cli ok.
