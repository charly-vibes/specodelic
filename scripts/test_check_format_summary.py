#!/usr/bin/env python3
"""Tests for scripts/check_format_summary.py (stdlib unittest, no deps).

The format summary has hand-maintained copies (homepage, FAQ, llms.txt and
its generated llm.txt mirror, book.toml). They drifted twice — "four
tables" listing three, and "Frontmatter" standing in for the Intent layer.
The gate asserts every copy names the canonical four layers in order and
never labels Intent by its location.

Run: python3 -m unittest discover -s scripts -p 'test_*.py'
"""

import sys
import tempfile
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))
import check_format_summary as cfs  # noqa: E402


CLEAN_WORLD = {
    "docs/src/index.md": (
        "# specodelic\n\n"
        "## The four layers\n\n"
        "| Layer | Carries |\n"
        "|-------|---------|\n"
        "| Intent | `id`, `kind`, `statement` |\n"
        "| Constraints | invariants and effects |\n"
        "| Model | states + transitions |\n"
        "| Properties | generators + predicates |\n\n"
        "## Next\n\nprose\n"
    ),
    "docs/src/faq.md": (
        "# FAQ\n\nA specification format for AI agents: specs are plain "
        "markdown with a closed four-layer schema — Intent, Constraints, "
        "Model, Properties — checked by a linter that never reads prose.\n"
    ),
    "llms.txt": (
        "# specodelic\n\n"
        "- **Four layers** — Intent, Constraints, Model, Properties.\n"
    ),
    "llm.txt": (
        "# specodelic\n\n"
        "- **Four layers** — Intent, Constraints, Model, Properties.\n"
    ),
    "book.toml": (
        '[book]\ntitle = "specodelic"\n'
        'description = "A specification format for AI agents — Intent, '
        'Constraints, Model, and Properties in plain markdown."\n'
    ),
}


class FormatSummaryTest(unittest.TestCase):
    def write(self, tree):
        root = Path(tempfile.mkdtemp())
        for rel, text in tree.items():
            p = root / rel
            p.parent.mkdir(parents=True, exist_ok=True)
            p.write_text(text, encoding="utf-8")
        return root

    def test_clean_copies_pass(self):
        self.assertEqual(cfs.check(self.write(CLEAN_WORLD)), [])

    def test_llms_frontmatter_layer_is_flagged(self):
        world = dict(CLEAN_WORLD)
        world["llms.txt"] = (
            "# specodelic\n\n"
            "- **Four layers** — frontmatter, Constraints, Model, Properties.\n"
        )
        problems = cfs.check(self.write(world))
        self.assertTrue(
            any("llms.txt" in p and "Intent" in p for p in problems), problems
        )

    def test_index_layer_row_location_is_flagged(self):
        world = dict(CLEAN_WORLD)
        world["docs/src/index.md"] = world["docs/src/index.md"].replace(
            "| Intent |", "| Intent (frontmatter) |"
        )
        problems = cfs.check(self.write(world))
        self.assertTrue(
            any("index.md" in p and "frontmatter" in p for p in problems),
            problems,
        )

    def test_book_description_missing_a_layer_is_flagged(self):
        world = dict(CLEAN_WORLD)
        world["book.toml"] = (
            '[book]\ntitle = "specodelic"\n'
            'description = "A specification format — Intent, Constraints, Model."\n'
        )
        problems = cfs.check(self.write(world))
        self.assertTrue(
            any("book.toml" in p and "Properties" in p for p in problems), problems
        )

    def test_faq_sentence_missing_layers_is_flagged(self):
        world = dict(CLEAN_WORLD)
        world["docs/src/faq.md"] = (
            "# FAQ\n\nA specification format for AI agents: specs are plain "
            "markdown with a closed four-layer schema.\n"
        )
        problems = cfs.check(self.write(world))
        self.assertTrue(any("faq.md" in p for p in problems), problems)

    def test_missing_target_fails_closed(self):
        world = dict(CLEAN_WORLD)
        del world["llms.txt"]
        problems = cfs.check(self.write(world))
        self.assertTrue(any("llms.txt" in p for p in problems), problems)

    def test_llm_enumeration_inside_a_link_is_flagged(self):
        world = dict(CLEAN_WORLD)
        world["llms.txt"] = (
            "# specodelic\n\n"
            "- **Four layers** — Intent, Constraints, Model, Properties.\n"
            "- [Four layers: Intent, Constraints, Model](https://example.test/x)\n"
        )
        problems = cfs.check(self.write(world))
        self.assertTrue(
            any("llms.txt" in p and "Properties" in p for p in problems), problems
        )

    def test_llm_page_reference_with_colon_is_exempt(self):
        world = dict(CLEAN_WORLD)
        world["llms.txt"] = (
            "# specodelic\n\n"
            "- **Four layers** — Intent, Constraints, Model, Properties.\n"
            "- [Architecture — the four layers and the lint pipeline](arch.html):\n"
        )
        self.assertEqual(cfs.check(self.write(world)), [])

    def test_wrapped_four_layers_bullet_passes(self):
        world = dict(CLEAN_WORLD)
        wrapped = (
            "# specodelic\n\n"
            "- **Four layers** — Intent, Constraints,\n"
            "  Model, Properties.\n"
        )
        world["llms.txt"] = wrapped
        world["llm.txt"] = wrapped
        self.assertEqual(cfs.check(self.write(world)), [])

    def test_wrapped_bullet_missing_a_layer_is_flagged(self):
        world = dict(CLEAN_WORLD)
        wrapped = (
            "# specodelic\n\n"
            "- **Four layers** — Intent, Constraints,\n"
            "  Model.\n"
        )
        world["llms.txt"] = wrapped
        world["llm.txt"] = wrapped
        problems = cfs.check(self.write(world))
        self.assertTrue(
            any("llms.txt" in p and "Properties" in p for p in problems), problems
        )


if __name__ == "__main__":
    unittest.main()