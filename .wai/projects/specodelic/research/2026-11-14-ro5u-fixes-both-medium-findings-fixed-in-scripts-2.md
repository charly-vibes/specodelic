---
tags: [pipeline-run:tdd-ro5-2026-11-14-specodelic-oeuz-gate-the-format-summary-prose-across-docs-copies, pipeline-step:ro5u-review]
---

RO5U-FIXES: both MEDIUM findings fixed in scripts/check_format_summary.py and tests. Link exemption is content-based now: a markdown link bullet is skipped only when it names none of the four canonical layers (and no frontmatter), and the link regex accepts a trailing colon; an enumerating link is checked. The llm twin check joins continuation lines into blocks before the four-layer check, so wrapped bullets pass and wrapped-missing-layer still fails. Added four tests (10 total). Verified: python3 -m unittest scripts.test_check_format_summary 10 passed; python3 scripts/check_format_summary.py . exit 0; pretender check exit 0.
