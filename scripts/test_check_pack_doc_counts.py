#!/usr/bin/env python3
"""Tests for scripts/check_pack_doc_counts.py (stdlib unittest).

specodelic-typ0: pack-count prose is hardcoded in six user-facing
surfaces and has drifted twice (README said "four" while commands.md
said "three"; same day — fd2fbe0 fixed it by hand). This guard makes
the drift a labeled CI failure:

  1. any count phrase about packs ("N standard packs", "N packs ship
     in-repo") on an audited surface must equal the count derived from
     packs/*.md frontmatter (total, or total minus the pilot ids);
  2. an "enumerate" surface must name every pack id (backticked id or
     packs/<stem>.md link); a "standard-only" surface may omit the
     pilot ids (bioimage.data is the D6 pilot — a historical artifact,
     NOT structurally derivable: quant.finance also carries cross-pack
     Requires deps but is a standard pack);
  3. fail-closed: a missing/unreadable packs dir or a pack file without
     a parsable frontmatter id is a finding, never a silent pass;
  4. llm.txt freshness (EDGE-002): the committed llm.txt must equal
     stamp_llms.narrative_llm(llms.txt, docs/src/SUMMARY.md) — the
     generated narrative drifting from its source is the same failure
     class as a hand-edited count.
"""

import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))

import check_pack_doc_counts as check  # noqa: E402


def write_pack(root: Path, stem: str, pack_id: str, kind: str = "profile") -> None:
    packs = root / "packs"
    packs.mkdir(exist_ok=True)
    (packs / f"{stem}.md").write_text(
        f"---\nid: {pack_id}\nkind: {kind}\n"
        f'checked_against_core: clear\nstatement: "THE format SHALL x."\n'
        f"---\n\n# {pack_id}\n"
    )


def write_page(root: Path, rel: str, text: str) -> None:
    path = root / rel
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(text)


GREEN_SURFACES = {
    "README.md": (
        "Domain packs ship in-repo. The two standard packs and the\n"
        "gamma D6 pilot:\n"
        "[`packs/alpha-one.md`](packs/alpha-one.md) and\n"
        "[`packs/beta-two.md`](packs/beta-two.md),\n"
        "[`packs/gamma-pilot.md`](packs/gamma-pilot.md).\n"
    ),
    "docs/src/index.md": (
        "Three packs ship in-repo — the two standard packs (`alpha.one`,\n"
        "`beta.two`) plus the gamma pilot (`gamma.pilot`).\n"
    ),
    "docs/src/faq.md": (
        "Three packs ship in-repo — two standard packs (`alpha.one`,\n"
        "`beta.two`) plus the first pilot (`gamma.pilot`).\n"
    ),
    "docs/src/commands.md": (
        "The two in-repo standard packs live in `packs/` (`alpha.one`,\n"
        "`beta.two`).\n"
    ),
    "llms.txt": (
        "# specodelic\n\n"
        "Three ship in-repo — the two standard packs (`alpha.one`,\n"
        "`beta.two`) and the `gamma.pilot` pilot.\n"
    ),
}

SUMMARY = "# Summary\n\n- [Index](index.md)\n"


class PackDocCountsTest(unittest.TestCase):
    def setUp(self):
        self._pilot = check.PILOT_PACK_IDS
        check.PILOT_PACK_IDS = frozenset({"gamma.pilot"})

    def tearDown(self):
        check.PILOT_PACK_IDS = self._pilot

    def _green_root(self, tmp) -> Path:
        root = tmp / "repo"
        root.mkdir()
        write_pack(root, "alpha-one", "alpha.one")
        write_pack(root, "beta-two", "beta.two")
        write_pack(root, "gamma-pilot", "gamma.pilot")
        for rel, text in GREEN_SURFACES.items():
            write_page(root, rel, text)
        write_page(root, "docs/src/SUMMARY.md", SUMMARY)
        (root / "llm.txt").write_text(
            __import__("stamp_llms").narrative_llm(
                GREEN_SURFACES["llms.txt"], SUMMARY
            )
        )
        return root

    def test_clean_tree_passes(self):
        import tempfile

        with tempfile.TemporaryDirectory() as d:
            findings = check.check(self._green_root(Path(d)))
        self.assertEqual([], findings)

    def test_stale_standard_count_fails(self):
        import tempfile

        with tempfile.TemporaryDirectory() as d:
            root = self._green_root(Path(d))
            write_page(root, "docs/src/faq.md", GREEN_SURFACES["docs/src/faq.md"].replace("two standard packs", "three standard packs"))
            findings = check.check(root)
        self.assertTrue(any("docs/src/faq.md" in f and "three" in f for f in findings), findings)

    def test_stale_total_count_fails(self):
        import tempfile

        with tempfile.TemporaryDirectory() as d:
            root = self._green_root(Path(d))
            write_page(root, "docs/src/index.md", GREEN_SURFACES["docs/src/index.md"].replace("Three packs ship in-repo", "Five packs ship in-repo"))
            findings = check.check(root)
        self.assertTrue(any("docs/src/index.md" in f and "Five" in f for f in findings), findings)

    def test_missing_enumerated_id_fails(self):
        import tempfile

        with tempfile.TemporaryDirectory() as d:
            root = self._green_root(Path(d))
            write_page(root, "docs/src/index.md", GREEN_SURFACES["docs/src/index.md"].replace("`beta.two`) plus", ") plus"))
            findings = check.check(root)
        self.assertTrue(any("beta.two" in f for f in findings), findings)

    def test_pilot_omission_allowed_on_standard_only(self):
        import tempfile

        with tempfile.TemporaryDirectory() as d:
            findings = check.check(self._green_root(Path(d)))
        # commands.md (standard-only) never names gamma.pilot — no finding.
        self.assertFalse(any("gamma.pilot" in f and "commands.md" in f for f in findings), findings)

    def test_pilot_missing_on_enumerate_page_fails(self):
        import tempfile

        with tempfile.TemporaryDirectory() as d:
            root = self._green_root(Path(d))
            write_page(root, "llms.txt", GREEN_SURFACES["llms.txt"].replace("`gamma.pilot` pilot", "pilot pack"))
            findings = check.check(root)
        self.assertTrue(any("gamma.pilot" in f and "llms.txt" in f for f in findings), findings)

    def test_non_profile_file_ignored(self):
        import tempfile

        with tempfile.TemporaryDirectory() as d:
            root = self._green_root(Path(d))
            write_pack(root, "extra", "extra.thing", kind="intent")
            findings = check.check(root)
        self.assertFalse(any("extra.thing" in f for f in findings), findings)

    def test_fail_closed_on_missing_packs_dir(self):
        import tempfile

        with tempfile.TemporaryDirectory() as d:
            root = Path(d) / "repo"
            root.mkdir()
            findings = check.check(root)
        self.assertEqual(1, len(findings), findings)
        self.assertIn("packs", findings[0])

    def test_llm_txt_stale_fails(self):
        import tempfile

        with tempfile.TemporaryDirectory() as d:
            root = self._green_root(Path(d))
            (root / "llm.txt").write_text("# specodelic\n\nstale body\n")
            findings = check.check(root)
        self.assertTrue(any("llm.txt" in f for f in findings), findings)

    def test_findings_carry_file_and_hint(self):
        import tempfile

        with tempfile.TemporaryDirectory() as d:
            root = self._green_root(Path(d))
            write_page(root, "README.md", GREEN_SURFACES["README.md"].replace("packs/beta-two.md", "packs/gone.md"))
            findings = check.check(root)
        self.assertTrue(all("remediation" in f or "hint" in f or "->" in f or "packs/" in f for f in findings), findings)


if __name__ == "__main__":
    unittest.main()
