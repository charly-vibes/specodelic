---
tags: [pipeline-run:tdd-ro5-2026-11-14-specodelic-oeuz-gate-the-format-summary-prose-across-docs-copies, pipeline-step:red]
---

RED: wrote scripts/test_check_format_summary.py (six tests: clean copies pass; llms frontmatter layer; index 'Intent (frontmatter)' row; book description missing Properties; FAQ sentence missing layers; missing target fails closed) importing check_format_summary.check(root). Command python3 -m unittest scripts.test_check_format_summary fails with ModuleNotFoundError: No module named 'check_format_summary' — expected red for the absent gate. No docs or justfile edits yet.
