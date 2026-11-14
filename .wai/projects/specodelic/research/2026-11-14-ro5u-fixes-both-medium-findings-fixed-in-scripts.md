---
tags: [pipeline-run:tdd-ro5-2026-11-14-specodelic-oeuz-gate-the-format-summary-prose-across-docs-copies, pipeline-step:ro5u-review]
---

RO5U-FIXES: both MEDIUM findings fixed in scripts/check_format_summary.py + tests. Link exemption is now content-based: a  bullet is skipped only when it names none of the four canonical layers (and no frontmatter), and LINK_ENTRY accepts a trailing colon; an enumerating link is checked.  joins continuation lines into blocks before the four-layer check, so wrapped bullets pass and wrapped-missing-layer still fails. Added four tests (10 total). Verified: python3 -m unittest scripts.test_check_format_summary 10 passed; python3 scripts/check_format_summary.py . exit 0; pretender check exit 0 (both new files all green).
