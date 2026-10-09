#!/usr/bin/env python3
"""Tests for scripts/stamp_llms.py + llms.txt drift guards (stdlib unittest).

specodelic-2m7: llms.txt is stamped at docs build time with the crate
version + build ref, and links the Release Status page — an agent writing
specs from the deployed site must be able to tell which format revision it
is reading before writing against an installed spk.

specodelic-cke: llm.txt vs llms.txt — llms.txt is the deploy target and
carries the merged Install & Links content. Originally the guard asserted
the undeployed repo-root llm.txt variant must stay gone; reversed per
specodelic-mvl (2026-10-05, espectacular family rule): llm.txt is a
repo-root narrative summary alongside llms.txt, and whether it deploys is
repo policy — it must exist at root and llms.txt keeps the merged content.

specodelic-lf4b.11: llm.txt is no longer hand-maintained — the stamp
pipeline (`stamp_llms.py root`) derives it from llms.txt + the book
SUMMARY.md, so the guards pin the derived content: capability surface
(graph-views, dual-format, hooks, archive-companion), the llms.txt
cross-link/disambiguation, and body-identity with llms.txt.

Run: python3 -m unittest discover -s scripts -p 'test_*.py'
"""

import os
import re
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

    def test_stamped_llms_disambiguates_llm_txt(self) -> None:
        # specodelic-lf4b.11: an agent reading the deployed llms.txt must
        # be able to tell it apart from the repo-root llm.txt summary.
        out = sl.stamped_llms("body", "0.3.1", "main", "abcdef123456")
        self.assertIn("llm.txt", out)

    def test_stamped_llms_preserves_source_body(self) -> None:
        body = "# specodelic\n\n## Docs\n\n- [kinds](https://example.com)\n"
        out = sl.stamped_llms(body, "1.0.0", "v1.0.0", "abcdef123456")
        self.assertIn(body, out)

    def test_release_page_has_version_and_no_hand_edit(self) -> None:
        out = sl.release_page("0.3.1", "main", "abcdef123456")
        self.assertIn("v0.3.1", out)
        self.assertIn("main @ abcdef1", out)
        self.assertIn("Do not hand-edit", out)

    def test_book_pages_maps_prose_pages_only(self) -> None:
        summary = (
            "[Introduction](index.md)\n"
            "[Release Status](./release.md)\n"
            "- [Graph views — diagrams](graph-views.md)\n"
            "- [core](specs/specodelic.md)\n"
            "- [hooks](openspec/hooks/spec.md)\n"
        )
        pages = sl.book_pages(summary)
        self.assertEqual(
            pages,
            [
                ("Introduction", "index.md"),
                ("Graph views — diagrams", "graph-views.md"),
            ],
        )

    def test_narrative_llm_has_body_and_header(self) -> None:
        # specodelic-lf4b.11: llm.txt is derived from llms.txt, so the two
        # cannot silently diverge; the header names both files.
        llms = "# specodelic\n\n## Command surface\n\n- dual-format skeleton\n"
        summary = "[Introduction](index.md)\n[graph-views](graph-views.md)\n"
        out = sl.narrative_llm(llms, summary)
        self.assertTrue(out.startswith("# specodelic\n"))
        self.assertIn("llms.txt", out)
        self.assertIn("do not hand-edit", out)
        self.assertIn("dual-format skeleton", out)  # llms.txt body carried over
        self.assertIn(
            "https://charly-vibes.github.io/specodelic/llms.txt", out
        )

    def test_narrative_llm_surfaces_graph_views(self) -> None:
        # The prose-page map comes from SUMMARY.md — graph-views must not
        # vanish again (the original hand-written llm.txt lacked it).
        summary = "[graph-views page](graph-views.md)\n"
        out = sl.narrative_llm("# specodelic\n\nbody\n", summary)
        self.assertIn("graph-views", out)
        self.assertIn("graph-views.html", out)

    def test_narrative_llm_single_h1(self) -> None:
        out = sl.narrative_llm("# specodelic\n\nbody\n", "[a](index.md)\n")
        self.assertEqual(out.count("# specodelic\n"), 1)
        self.assertEqual(len(re.findall(r"^# ", out, re.MULTILINE)), 1)
        # body survived the H1 strip
        self.assertIn("body", out)
        self.assertIn(
            "[a](https://charly-vibes.github.io/specodelic/index.html)", out
        )

    def test_narrative_llm_no_summary_yields_body_only(self) -> None:
        out = sl.narrative_llm("# specodelic\n\nbody\n", "")
        self.assertNotIn("prose pages", out)
        self.assertIn("body", out)
        self.assertIn("llms.txt", out)


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

    def test_env_ref_beats_git_fallback(self) -> None:
        # specodelic-lf4b.11: CI names the real ref/tag via env (the
        # module docstring's contract — release.yml stamps with the tag).
        old = dict(os.environ)
        try:
            os.environ["GITHUB_REF_NAME"] = "v9.9.9"
            os.environ["GITHUB_SHA"] = "0123456789abcdef"
            sl.main(["page"], root=self.tmp)
        finally:
            os.environ.clear()
            os.environ.update(old)
        page = (self.tmp / "docs" / "src" / "release.md").read_text()
        self.assertIn("v9.9.9 @ 0123456", page)

    def test_root_phase_regenerates_llm_txt(self) -> None:
        (self.tmp / "docs" / "src").mkdir(parents=True, exist_ok=True)
        (self.tmp / "docs" / "src" / "SUMMARY.md").write_text(
            "[graph-views page](graph-views.md)\n"
        )
        (self.tmp / "llms.txt").write_text(
            "# specodelic\n\n- spk archive-companion\n- dual-format\n"
        )
        sl.main(["root"], root=self.tmp)
        out = (self.tmp / "llm.txt").read_text()
        self.assertIn("archive-companion", out)
        self.assertIn("dual-format", out)
        self.assertIn("graph-views", out)
        self.assertIn("llms.txt", out)  # disambiguation header
        # the repo-source llms.txt must be untouched by the root phase
        self.assertEqual(
            (self.tmp / "llms.txt").read_text(),
            "# specodelic\n\n- spk archive-companion\n- dual-format\n",
        )


class DriftGuardTest(unittest.TestCase):
    """specodelic-cke: llm.txt must exist at repo root (specodelic-mvl).

    specodelic-lf4b.11: the guard also pins the regenerated narrative —
    llm.txt is derived from llms.txt at stamp time and must carry the
    capability surface (graph-views, dual-format, hooks, archive-companion)
    plus a disambiguating cross-link to the deployed llms.txt.
    """

    def test_repo_root_has_llm_txt_narrative_summary(self) -> None:
        text = (REPO / "llm.txt").read_text()
        self.assertTrue(text.startswith("# specodelic"))
        self.assertIn("## Installation", text)
        self.assertIn("cargo install specodelic", text)

    def test_llm_txt_is_regenerated_not_hand_written(self) -> None:
        text = (REPO / "llm.txt").read_text()
        self.assertIn("regenerated by", text)
        self.assertIn("scripts/stamp_llms.py", text)

    def test_llm_txt_carries_capability_surface(self) -> None:
        # specodelic-lf4b.11 meter: the regenerated narrative must cover
        # what the original hand-written copy was missing.
        text = (REPO / "llm.txt").read_text()
        for needle in (
            "graph-views",
            "dual-format",
            "archive-companion",
            "hooks install",
        ):
            self.assertIn(needle, text)

    def test_llm_txt_disambiguates_llms_txt(self) -> None:
        # Meter: an agent grabbing either summary can tell them apart.
        text = (REPO / "llm.txt").read_text()
        self.assertIn("llms.txt", text)
        self.assertIn(
            "https://charly-vibes.github.io/specodelic/llms.txt", text
        )
        self.assertIn("do not hand-edit", text)

    def test_llm_txt_tracks_llms_txt_body(self) -> None:
        # Divergence-proofing: the whole llms.txt body (after its H1) must
        # appear in the derived llm.txt (modulo the appended page map).
        llms = (REPO / "llms.txt").read_text()
        llm = (REPO / "llm.txt").read_text()
        body = llms.split("\n", 1)[1].lstrip("\n").rstrip("\n")
        self.assertIn(body, llm)

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

    def test_release_page_matches_crate_version(self) -> None:
        # Meter: the checked-in release.md must name the crate version.
        page = (REPO / "docs" / "src" / "release.md").read_text()
        cargo = (REPO / "Cargo.toml").read_text()
        m = re.search(r'^version\s*=\s*"([^"]+)"', cargo, re.MULTILINE)
        assert m
        self.assertIn(f"v{m.group(1)}", page)


if __name__ == "__main__":
    unittest.main()
