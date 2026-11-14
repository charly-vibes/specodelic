---
tags: [pipeline-run:tdd-ro5-2026-11-14-specodelic-oeuz-gate-the-format-summary-prose-across-docs-copies, pipeline-step:green]
---

GREEN: implemented scripts/check_format_summary.py (stdlib, fail-closed; index layer table in order, FAQ four-layer sentence, llms.txt + generated llm.txt four-layers lines, book.toml description; forbids 'frontmatter' as the Intent layer; link-title bullets exempt as page references); wrote scripts/test_check_format_summary.py 6 tests. Fixed the copies: index 'Intent (frontmatter)' -> 'Intent'; FAQ sentence names Intent/Constraints/Model/Properties; llms.txt both enumerations now Intent; book.toml description names all four; regenerated llm.txt via just stamp-artifacts (reverted the unrelated release.md stamp); added justfile format-summary recipe and ci wiring. Verified: python3 -m unittest scripts.test_check_format_summary 6 passed; python3 scripts/check_format_summary.py . exit 0.
