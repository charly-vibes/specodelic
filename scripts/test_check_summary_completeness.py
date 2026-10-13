#!/usr/bin/env python3
"""Tests for scripts/check_summary_completeness.py (stdlib unittest, no deps).

Covers the SUMMARY.md completeness gate (specodelic-b3p): every
frontmatter-bearing spec in the single tree (openspec/specs/<cap>/spec.md,
specodelic-jf1) MUST be linked from docs/src/SUMMARY.md — the hand-written
SUMMARY can silently miss new spec files, and mdbook only renders what
SUMMARY lists.

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

    def test_missing_capability_spec_is_flagged(self):
        root = self.write({
            "docs/src/SUMMARY.md": self.summary("- [c](specs/c/spec.md)"),
            "openspec/specs/c/spec.md": FRONTMATTER.format(id="c"),
            "openspec/specs/d/spec.md": FRONTMATTER.format(id="d"),
        })
        problems = csc.check(root)
        self.assertEqual(len(problems), 1)
        self.assertIn("openspec/specs/d/spec.md", problems[0])
        self.assertIn("specs/d/spec.md", problems[0])

    def test_missing_corpus_spec_is_flagged(self):
        # A corpus spec is just a capability dir in the single tree; the
        # migrated specodelic core must be linked like any other.
        root = self.write({
            "docs/src/SUMMARY.md": self.summary("- [c](specs/c/spec.md)"),
            "openspec/specs/c/spec.md": FRONTMATTER.format(id="c"),
            "openspec/specs/specodelic/spec.md": FRONTMATTER.format(id="specodelic"),
        })
        problems = csc.check(root)
        self.assertEqual(len(problems), 1)
        self.assertIn("openspec/specs/specodelic/spec.md", problems[0])

    def test_frontmatterless_files_are_exempt(self):
        root = self.write({
            "docs/src/SUMMARY.md": self.summary("- [a](specs/a/spec.md)"),
            "openspec/specs/a/spec.md": FRONTMATTER.format(id="a"),
            "openspec/specs/AGENTS.md": "# governance, no frontmatter\n",
            "openspec/specs/claim-guidance.checklist.md": "# checklist, no frontmatter\n",
            "openspec/STATUS.md": "# status, no frontmatter\n",
            "openspec/USAGE.md": "# usage, no frontmatter\n",
        })
        self.assertEqual(csc.check(root), [])

    def test_fully_listed_tree_is_clean(self):
        root = self.write({
            "docs/src/SUMMARY.md": self.summary(
                "- [a](specs/a/spec.md)",
                "- [c](specs/c/spec.md)",
            ),
            "openspec/specs/a/spec.md": FRONTMATTER.format(id="a"),
            "openspec/specs/c/spec.md": FRONTMATTER.format(id="c"),
        })
        self.assertEqual(csc.check(root), [])

    def test_archived_change_deltas_are_not_required(self):
        root = self.write({
            "docs/src/SUMMARY.md": self.summary("- [a](specs/a/spec.md)"),
            "openspec/specs/a/spec.md": FRONTMATTER.format(id="a"),
            "openspec/changes/archive/2026-09-29-x/specs/c/spec.md":
                FRONTMATTER.format(id="spec"),
        })
        self.assertEqual(csc.check(root), [])

    def test_legacy_two_tree_corpus_is_not_scanned(self):
        # The specs/ tree is gone after the jf1 migration; a stale file
        # left behind must not satisfy or fail the gate.
        root = self.write({
            "docs/src/SUMMARY.md": self.summary("- [a](specs/a/spec.md)"),
            "openspec/specs/a/spec.md": FRONTMATTER.format(id="a"),
            "specs/a.md": FRONTMATTER.format(id="a"),
        })
        self.assertEqual(csc.check(root), [])

    def test_missing_summary_file_fails_closed(self):
        root = self.write({
            "openspec/specs/a/spec.md": FRONTMATTER.format(id="a"),
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
                "- [a](specs/a/spec.md)",
                "- [Ecosystem](https://example.com/ecosystem-map.html)",
            ),
            "openspec/specs/a/spec.md": FRONTMATTER.format(id="a"),
        })
        problems = csc.check(root)
        self.assertEqual(len(problems), 1)
        self.assertIn("https://example.com/ecosystem-map.html", problems[0])
        self.assertIn("index.md", problems[0])

    def test_relative_markdown_link_is_not_flagged_as_absolute(self):
        root = self.write({
            "docs/src/SUMMARY.md": self.summary("- [a](specs/a/spec.md)"),
            "openspec/specs/a/spec.md": FRONTMATTER.format(id="a"),
        })
        self.assertEqual(csc.check(root), [])

if __name__ == "__main__":
    unittest.main()
