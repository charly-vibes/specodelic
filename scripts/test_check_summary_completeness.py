#!/usr/bin/env python3
"""Tests for scripts/check_summary_completeness.py (stdlib unittest, no deps).

Covers the SUMMARY.md completeness gate (specodelic-b3p): every corpus
spec file (frontmatter-bearing specs/*.md) and every capability spec
(openspec/specs/*/spec.md) MUST be linked from docs/src/SUMMARY.md —
the hand-written SUMMARY can silently miss new spec files, and mdbook
only renders what SUMMARY lists.

Run: python3 -m unittest discover -s scripts -p 'test_*.py'
"""

import sys
import tempfile
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))
import check_summary_completeness as csc  # noqa: E402

FRONTMATTER = "---\nid: {id}\nkind: intent\nstatement: \"SHALL x.\"\n---\n\ntext\n"


class SummaryCompletenessTest(unittest.TestCase):
    def write(self, tree: dict[str, str]) -> Path:
        tmp = tempfile.mkdtemp()
        root = Path(tmp)
        for rel, text in tree.items():
            p = root / rel
            p.parent.mkdir(parents=True, exist_ok=True)
            p.write_text(text, encoding="utf-8")
        return root

    def summary(self, *lines: str) -> str:
        return "# Summary\n\n" + "\n".join(lines) + "\n"

    def test_missing_corpus_spec_is_flagged(self):
        root = self.write({
            "docs/src/SUMMARY.md": self.summary("- [a](specs/a.md)"),
            "specs/a.md": FRONTMATTER.format(id="a"),
            "specs/b.md": FRONTMATTER.format(id="b"),
        })
        problems = csc.check(root)
        self.assertEqual(len(problems), 1)
        self.assertIn("specs/b.md", problems[0])
        self.assertIn("SUMMARY.md", problems[0])

    def test_missing_capability_spec_is_flagged(self):
        root = self.write({
            "docs/src/SUMMARY.md": self.summary("- [c](openspec/c/spec.md)"),
            "openspec/specs/c/spec.md": FRONTMATTER.format(id="spec"),
            "openspec/specs/d/spec.md": FRONTMATTER.format(id="spec"),
        })
        problems = csc.check(root)
        self.assertEqual(len(problems), 1)
        self.assertIn("openspec/specs/d/spec.md", problems[0])

    def test_frontmatterless_corpus_files_are_exempt(self):
        root = self.write({
            "docs/src/SUMMARY.md": self.summary("- [a](specs/a.md)"),
            "specs/a.md": FRONTMATTER.format(id="a"),
            "specs/AGENTS.md": "# governance, no frontmatter\n",
            "specs/STATUS.md": "# status, no frontmatter\n",
            "specs/USAGE.md": "# usage, no frontmatter\n",
            "specs/foo.checklist.md": "# checklist, no frontmatter\n",
        })
        self.assertEqual(csc.check(root), [])

    def test_non_spec_md_without_frontmatter_is_exempt(self):
        root = self.write({
            "docs/src/SUMMARY.md": self.summary("- [a](specs/a.md)"),
            "specs/a.md": FRONTMATTER.format(id="a"),
            "specs/theory.md": "# theory essay, no frontmatter\n",
        })
        self.assertEqual(csc.check(root), [])

    def test_fully_listed_tree_is_clean(self):
        root = self.write({
            "docs/src/SUMMARY.md": self.summary(
                "- [a](specs/a.md)",
                "- [c](openspec/c/spec.md)",
            ),
            "specs/a.md": FRONTMATTER.format(id="a"),
            "openspec/specs/c/spec.md": FRONTMATTER.format(id="spec"),
        })
        self.assertEqual(csc.check(root), [])

    def test_archived_change_deltas_are_not_required(self):
        root = self.write({
            "docs/src/SUMMARY.md": self.summary("- [a](specs/a.md)"),
            "specs/a.md": FRONTMATTER.format(id="a"),
            "openspec/changes/archive/2026-09-29-x/specs/c/spec.md":
                FRONTMATTER.format(id="spec"),
        })
        self.assertEqual(csc.check(root), [])

    def test_missing_summary_file_fails_closed(self):
        root = self.write({
            "specs/a.md": FRONTMATTER.format(id="a"),
        })
        problems = csc.check(root)
        self.assertEqual(len(problems), 1)
        self.assertIn("docs/src/SUMMARY.md", problems[0])

    def test_absolute_url_link_in_summary_is_flagged(self):
        # specodelic-j0m: mdbook 0.5 materializes absolute-URL SUMMARY
        # entries as literal src/https:/... directories. External links
        # belong in a page's prose, never in SUMMARY.md.
        root = self.write({
            "docs/src/SUMMARY.md": self.summary(
                "- [a](specs/a.md)",
                "- [Ecosystem](https://example.com/ecosystem-map.html)",
            ),
            "specs/a.md": FRONTMATTER.format(id="a"),
        })
        problems = csc.check(root)
        self.assertEqual(len(problems), 1)
        self.assertIn("https://example.com/ecosystem-map.html", problems[0])
        self.assertIn("index.md", problems[0])

    def test_relative_markdown_link_is_not_flagged_as_absolute(self):
        root = self.write({
            "docs/src/SUMMARY.md": self.summary("- [a](specs/a.md)"),
            "specs/a.md": FRONTMATTER.format(id="a"),
        })
        self.assertEqual(csc.check(root), [])

if __name__ == "__main__":
    unittest.main()