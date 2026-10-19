#!/usr/bin/env python3
"""Tests for scripts/migrate_deployed_strip.py (stdlib unittest, no deps).

One-time migration (specodelic-s8fg Stage 1 follow-through): strip the
delta sections (## ADDED/MODIFIED/REMOVED/RENAMED Requirements) from
every deployed capability spec under openspec/specs/, so the corpus is
openspec-1.x-strict-valid at the deployed-spec level. Mirrors the
Rust-side strip in src/archive_companion.rs (fence-aware, byte-preserving
outside the delta sections); the two implementations are asserted
equivalent on shared fixtures in the cargo test suite is out of scope —
here the fixtures pin the behavior.

Run: python3 -m unittest discover -s scripts -p 'test_*.py'
"""

import sys
import tempfile
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))
import migrate_deployed_strip as mds  # noqa: E402


FULL = (
    "---\nid: c.a\nkind: intent\nstatement: \"WHEN x THE format SHALL y.\"\n---\n"
    "\n# c\n\n## Purpose\n\nLong enough purpose text for the tooling.\n\n"
    "## Constraints\n\n| id | kind | expr | traces_to |\n"
    "|----|------|------|-----------|\n| a | invariant | `x` | [[c.a]] |\n\n"
    "## ADDED Requirements\n\n"
    "### Requirement: One holds\nThe system SHALL hold one.\n\n"
    "#### Scenario: One\n- **WHEN** x\n- **THEN** y\n\n"
    "## Requirements\n\n"
    "### Requirement: One holds\nThe system SHALL hold one.\n\n"
    "#### Scenario: One\n- **WHEN** x\n- **THEN** y\n"
)

STRIPPED = (
    "---\nid: c.a\nkind: intent\nstatement: \"WHEN x THE format SHALL y.\"\n---\n"
    "\n# c\n\n## Purpose\n\nLong enough purpose text for the tooling.\n\n"
    "## Constraints\n\n| id | kind | expr | traces_to |\n"
    "|----|------|------|-----------|\n| a | invariant | `x` | [[c.a]] |\n\n"
    "## Requirements\n\n"
    "### Requirement: One holds\nThe system SHALL hold one.\n\n"
    "#### Scenario: One\n- **WHEN** x\n- **THEN** y\n"
)


class StripTest(unittest.TestCase):
    def test_strips_delta_sections_keeps_mirror(self):
        self.assertEqual(mds.strip_delta_sections(FULL), STRIPPED)

    def test_all_four_titles(self):
        delta = (
            "## ADDED Requirements\n\n### Requirement: A\nBody.\n\n"
            "## MODIFIED Requirements\n\n### Requirement: B\nBody.\n\n"
            "## REMOVED Requirements\n\n### Requirement: C\n\n"
            "## RENAMED Requirements\n\n### Requirement: D\n- FROM: D\n- TO: E\n\n"
            "## Requirements\n\n### Requirement: A\nBody.\n"
        )
        out = mds.strip_delta_sections(delta)
        for title in ("ADDED", "MODIFIED", "REMOVED", "RENAMED"):
            self.assertNotIn(f"## {title}", out)
        self.assertIn("### Requirement: A", out)

    def test_fenced_headers_are_content(self):
        delta = (
            "## Requirements\n\n### Requirement: One holds\nExample:\n\n"
            "```md\n## ADDED Requirements\n```\n"
        )
        out = mds.strip_delta_sections(delta)
        self.assertIn("## ADDED Requirements", out)
        # byte-preservation: the fence markers themselves survive
        self.assertIn("```md\n", out)
        self.assertEqual(out.count("```"), 2)

    def test_fenced_block_is_byte_identical(self):
        delta = (
            "## Requirements\n\n### Requirement: One holds\nExample:\n\n"
            "```md\n## ADDED Requirements\n| a | b |\n```\n\n"
            "Trailing prose.\n"
        )
        self.assertEqual(mds.strip_delta_sections(delta), delta)

    def test_idempotent(self):
        once = mds.strip_delta_sections(FULL)
        self.assertEqual(mds.strip_delta_sections(once), once)


class MigrationTest(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.root = Path(self.tmp.name)
        specs = self.root / "openspec" / "specs"
        (specs / "cap-a").mkdir(parents=True)
        (specs / "cap-b").mkdir(parents=True)
        (specs / "cap-a" / "spec.md").write_text(FULL)
        (specs / "cap-b" / "spec.md").write_text(STRIPPED)
        # a plain prose file must be left alone (no frontmatter)
        (specs / "STATUS.md").write_text("# status\n")

    def tearDown(self):
        self.tmp.cleanup()

    def test_migrates_only_dual_deployed_specs(self):
        changed = mds.migrate(self.root)
        self.assertEqual(
            changed,
            [self.root / "openspec/specs/cap-a/spec.md"],
        )
        self.assertEqual(
            (self.root / "openspec/specs/cap-a/spec.md").read_text(),
            STRIPPED,
        )
        # already-clean files are untouched (byte-identical)
        self.assertEqual(
            (self.root / "openspec/specs/cap-b/spec.md").read_text(),
            STRIPPED,
        )

    def test_idempotent_second_run_changes_nothing(self):
        mds.migrate(self.root)
        self.assertEqual(mds.migrate(self.root), [])

    def test_dry_run_reports_without_writing(self):
        changed = mds.migrate(self.root, dry_run=True)
        self.assertEqual(
            changed,
            [self.root / "openspec/specs/cap-a/spec.md"],
        )
        # nothing written
        self.assertEqual(
            (self.root / "openspec/specs/cap-a/spec.md").read_text(),
            FULL,
        )


if __name__ == "__main__":
    unittest.main()
