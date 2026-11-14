---
reviews: 2026-11-14-green-implemented-scripts-check-format-summary-py.md
verdict: needs-work
severity: {critical: 0, high: 0, medium: 2, low: 0}
tags: [pipeline-run:tdd-ro5-2026-11-14-specodelic-oeuz-gate-the-format-summary-prose-across-docs-copies, pipeline-step:ro5u-review]
---

RO5U: gate catches every required drift class and is wired into ci; two MEDIUM robustness defects. M1 link-title exemption was form-based — an enumerating link escaped while a colon-suffixed/wrapped reference false-failed. M2 the twin check validated a physical line, so a wrapped canonical bullet false-failed. Fixes: content-based link exemption (link bullet is skipped only when it names no canonical layer and no frontmatter), regex accepts trailing colon, and llm blocks join continuation lines before checking. Added tests for enumeration-in-link, colon reference, wrapped pass, wrapped missing layer. 10 unittest tests pass; real-tree gate exit 0; pretender check exit 0.
