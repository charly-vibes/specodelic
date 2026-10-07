# VERIFY: specodelic-36t (add-min-expr-kernel §4.3) — orchestrator verification

- git log 09b88d0..HEAD → single commit f563853, diff matches report exactly
  (5 files, all within the allowed list: tests/common/kernel_corpora.rs new,
  tests/common/mod.rs +1, grounding/status corpora moved verbatim,
  add-py-fragment-emission/design.md +26 promotion contract).
- `just test` → exit 0.
- `ah check` deployed-scope → 0 structural / 0 execution.
- `openspec validate --all --strict` → 25 passed, 0 failed.
- `just ci` → exit 0 (167 contract tests passed).
- Note: tasks.md §4.3 checkbox was pre-checked before this ticket ran
  (pre-existing inconsistency — bd ticket open while box ticked). The work
  is now actually done, so the checkbox is finally true; no edit was needed.
- Deviation accepted: grounding/status corpora moved to a sibling module
  (tests/common/kernel_corpora.rs) rather than merged into 36n's frozen
  agreement corpus — merging would change assertion meaning.
