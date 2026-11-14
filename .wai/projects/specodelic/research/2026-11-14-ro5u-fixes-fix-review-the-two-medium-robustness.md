---
tags: [pipeline-run:tdd-ro5-2026-11-14-specodelic-oeuz-gate-the-format-summary-prose-across-docs-copies, pipeline-step:fix-review]
---

RO5U-FIXES (fix-review): the two MEDIUM robustness fixes are applied and verified in this step — content-based link exemption with trailing-colon acceptance, and wrapped-block joining for the llms/llm twin check; four new tests (10 total). python3 -m unittest scripts.test_check_format_summary 10 passed; python3 scripts/check_format_summary.py . exit 0; pretender check exit 0. No open findings.
