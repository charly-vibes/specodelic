#!/usr/bin/env python3
"""Tests for scripts/stamp_llms.py + llms.txt drift guards (stdlib unittest).

specodelic-2m7: llms.txt is stamped at docs build time with the crate
version + build ref, and links the Release Status page — an agent writing
specs from the deployed site must be able to tell which format revision it
is reading before writing against an installed spk.

specodelic-cke: the repo-root llm.txt variant (install + Links sections)
was deployed nowhere and referenced by no build recipe — guard: it must
stay gone, and llms.txt must carry the merged Install & Links content.

Run: python3 -m unittest discover -s scripts -p 'test_*.py'
"""

import sys
import tempfile
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))
import stamp_llms as sl  # noqa: E402

REPO = Path(__file__).parent.parent


class StampTest(unittest.TestCase):
    """stamp_llms.py pure functions."""

    def test_stamped_llms_carries_version_and_ref(self) -> None:
        out = sl.stamped_llms("# specodelic\n\nbody\n", "0.3.1", "main", "abcdef123456")
        self.assertIn("v0.3.1", out)
        self.assertIn("main @ abcdef1", out)
        self.assertTrue(out.startswith("# specodelic\n"))

    def test_stamped_llms_links_release_status_page(self) -> None:
        out = sl.stamped_llms("body", "0.3.1", "main", "abcdef123456")
        self.assertIn(
            "https://charly-vibes.github.io/specodelic/release.html", out
        )

    def test_stamped_llms_preserves_source_body(self) -> None:
        body = "# specodelic\n\n## Docs\n\n- [kinds](https://example.com)\n"
        out = sl.stamped_llms(body, "1.0.0", "v1.0.0", "abcdef123456")
        self.assertIn(body, out)

    def test_release_page_has_version_and_no_hand_edit(self) -> None:
        out = sl.release_page("0.3.1", "main", "abcdef123456")
        self.assertIn("v0.3.1", out)
        self.assertIn("main @ abcdef1", out)
        self.assertIn("Do not hand-edit", out)


class StampCliTest(unittest.TestCase):
    """The build-time CLI phases against a fake repo root."""

    def setUp(self) -> None:
        self.tmp = Path(tempfile.mkdtemp())
        (self.tmp / "Cargo.toml").write_text(
            '[package]\nname = "specodelic"\nversion = "9.9.9"\n'
        )
        (self.tmp / "llms.txt").write_text("# specodelic\n\nbody\n")
        (self.tmp / "docs").mkdir()

    def test_page_phase_writes_release_md(self) -> None:
        sl.main(["page"], root=self.tmp)
        page = self.tmp / "docs" / "src" / "release.md"
        self.assertTrue(page.exists())
        self.assertIn("v9.9.9", page.read_text())

    def test_llms_phase_writes_stamped_copy(self) -> None:
        (self.tmp / "book").mkdir()
        sl.main(["llms"], root=self.tmp, tag="main", sha="abcdef123456")
        out = self.tmp / "book" / "llms.txt"
        self.assertTrue(out.exists())
        self.assertIn("v9.9.9", out.read_text())
        self.assertIn("release.html", out.read_text())

    def test_llms_phase_never_touches_repo_source(self) -> None:
        (self.tmp / "book").mkdir()
        src = self.tmp / "llms.txt"
        before = src.read_text()
        sl.main(["llms"], root=self.tmp, tag="main", sha="abcdef123456")
        self.assertEqual(before, src.read_text())


class DriftGuardTest(unittest.TestCase):
    """specodelic-cke: the undeployed llm.txt variant must stay gone."""

    def test_repo_root_has_no_undeployed_llm_txt(self) -> None:
        self.assertFalse((REPO / "llm.txt").exists())

    def test_llms_txt_carries_merged_install_content(self) -> None:
        text = (REPO / "llms.txt").read_text()
        self.assertIn("## Installation", text)
        self.assertIn("cargo install specodelic", text)
        self.assertIn("## Links", text)
        self.assertIn("https://github.com/charly-vibes/specodelic", text)

    def test_llms_txt_keeps_core_sections(self) -> None:
        text = (REPO / "llms.txt").read_text()
        for section in ("## What it is", "## Core concepts", "## Docs"):
            self.assertIn(section, text)


if __name__ == "__main__":
    unittest.main()