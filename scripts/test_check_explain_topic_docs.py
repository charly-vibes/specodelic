#!/usr/bin/env python3
"""Tests for scripts/check_explain_topic_docs.py (stdlib unittest, no deps).

Covers the anti-drift guard against src/guide.rs `TOPICS` as source of
truth (specodelic-lf4b.1): pages stating a topic count must state the
count the binary serves, must enumerate every served topic id, and any
digit/word count phrase must match the served count.

Run: python3 -m unittest discover -s scripts -p 'test_*.py'
"""

import sys
import tempfile
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))
import check_explain_topic_docs as cetd  # noqa: E402


ALL_IDS = [
    "format", "ears", "kinds", "references", "lifecycle",
    "lint-rules", "dual-format", "packs", "graph-views",
]

LISTING = (" (`format`, `ears`, `kinds`, `references`, `lifecycle`, "
           "`lint-rules`, `dual-format`, `packs`, `graph-views`)")


# Convenience accessors: PAGES is a per-page policy map of
# (relative_path, policy) tuples.
README = cetd.PAGES[0][0]
INDEX = cetd.PAGES[1][0]
INSTALL = cetd.PAGES[2][0]


def topics_block(ids: list[str]) -> str:
    entries = ",\n".join(
        f'    (\n        "{t}",\n        "Title for {t}",\n    )' for t in ids
    )
    return f"pub const TOPICS: &[(&str, &str)] = &[\n{entries},\n];\n"


class ExplainTopicDocsTest(unittest.TestCase):
    def write(self, tree: dict[str, str]) -> Path:
        tmp = tempfile.mkdtemp()
        root = Path(tmp)
        for rel, text in tree.items():
            p = root / rel
            p.parent.mkdir(parents=True, exist_ok=True)
            p.write_text(text, encoding="utf-8")
        return root

    def base_tree(self, count: str, id_list: str, with_ids: bool) -> dict[str, str]:
        pages = {}
        for rel, _policy in cetd.PAGES:
            listing = id_list if with_ids else ""
            pages[rel] = (
                f"Guide page.\n\n`spk explain` serves {count} topics"
                f"{listing}. Nothing more to say.\n"
            )
        return {"src/guide.rs": topics_block(ALL_IDS), **pages}

    def test_nine_topics_plus_all_ids_passes(self):
        root = self.write(self.base_tree("nine", LISTING, True))
        self.assertEqual(cetd.check(root), [])

    def test_digit_count_nine_topics_passes(self):
        root = self.write(self.base_tree("9", LISTING, True))
        self.assertEqual(cetd.check(root), [])

    def test_nine_topics_with_full_slash_list_passes(self):
        # README's post-fix shape: count phrase + adjacent slash-list of
        # every served id.
        tree = self.base_tree("nine", LISTING, True)
        tree[README] = (
            "- ✅ `explain` — embedded AIX guide (nine topics: "
            "format/ears/kinds/references/lifecycle/lint-rules/"
            "dual-format/packs/graph-views) plus extras.\n"
        )
        root = self.write(tree)
        self.assertEqual(cetd.check(root), [])

    def test_stale_inline_slash_enumeration_is_flagged(self):
        # README's original drift shape: a slash-list naming six served
        # ids right before "topics" while the binary serves nine — the
        # page-level missing-id rule can miss this when the omitted ids
        # appear elsewhere on the page.
        tree = self.base_tree("nine", LISTING, True)
        tree[README] = (
            "- ✅ `explain` — embedded AIX guide (format/ears/kinds/"
            "references/lifecycle/lint-rules topics) plus `dual-format`, "
            "`packs`, `graph-views` elsewhere.\n"
        )
        root = self.write(tree)
        problems = cetd.check(root)
        self.assertEqual(len(problems), 1, problems)
        self.assertIn("omits served topic(s) dual-format, graph-views, packs",
                      problems[0])

    def test_backticked_list_after_count_missing_id_is_flagged(self):
        # index.md's drift shape: "eight topics (`format`, ..., `packs`)"
        # — the adjacent backticked list omits graph-views.
        tree = self.base_tree("eight", LISTING, True)
        tree[INDEX] = (
            "`spk explain` serves eight topics (`format`, `ears`, `kinds`, "
            "`references`, `lifecycle`, `lint-rules`, `dual-format`, "
            "`packs`). Nothing more.\n"
        )
        root = self.write(tree)
        problems = cetd.check(root)
        self.assertTrue(any("omits served topic(s) graph-views" in p
                            for p in problems), problems)

    def test_non_topic_enumeration_is_ignored(self):
        tree = self.base_tree("nine", LISTING, True)
        tree[INSTALL] = (
            "Guide page. `spk explain` serves nine topics" + LISTING +
            ". It also mentions migrations/notes topics unrelated to "
            "the guide.\n"
        )
        root = self.write(tree)
        self.assertEqual(cetd.check(root), [])

    def test_seven_topics_is_flagged(self):
        root = self.write(self.base_tree("seven", LISTING, True))
        problems = cetd.check(root)
        self.assertEqual(len(problems), len(cetd.PAGES), problems)
        for p in problems:
            self.assertIn("'seven topics' != the 9 topics", p)

    def test_eight_topics_is_flagged(self):
        root = self.write(self.base_tree("eight", LISTING, True))
        problems = cetd.check(root)
        for p in problems:
            self.assertIn("'eight topics' != the 9 topics", p)

    def test_six_topics_is_flagged(self):
        # README's original drift shape: a word count one below served.
        root = self.write(self.base_tree("six", LISTING, True))
        problems = cetd.check(root)
        for p in problems:
            self.assertIn("'six topics' != the 9 topics", p)

    def test_stale_digit_count_is_flagged(self):
        # The word-count fallback includes digits: "6 topics" against a
        # binary serving 9 is drift, not a rename of the phrase.
        root = self.write(self.base_tree("6", LISTING, True))
        problems = cetd.check(root)
        for p in problems:
            self.assertIn("'6 topics' != the 9 topics", p)

    def test_missing_id_is_flagged(self):
        tree = self.base_tree("nine", LISTING, True)
        tree[INDEX] = (
            "Guide page.\n\n`spk explain` serves nine topics "
            "(`format`, `ears`, `kinds`, `references`, `lifecycle`, "
            "`lint-rules`, `dual-format`, `packs`). Nothing more.\n"
        )
        root = self.write(tree)
        problems = cetd.check(root)
        # Rule (b) page-level + rule 4 adjacency both fire on the same
        # omission — the page names eight ids and omits graph-views.
        self.assertEqual(len(problems), 2, problems)
        self.assertTrue(any("missing served explain topics graph-views" in p
                            for p in problems), problems)
        self.assertTrue(any("omits served topic(s) graph-views" in p
                            for p in problems), problems)

    def test_count_only_page_with_no_ids_passes(self):
        # installation.md is COUNT-ONLY: a correct count with NO topic
        # enumeration must pass (cross-reference page — count-only).
        tree = self.base_tree("nine", LISTING, True)
        tree[INSTALL] = (
            "Guide page.\n\n`spk explain` serves nine topics straight "
            "from the binary. Nothing more.\n"
        )
        root = self.write(tree)
        self.assertEqual(cetd.check(root), [])

    def test_count_only_page_with_wrong_count_fails(self):
        # COUNT-ONLY pages still cannot state a stale count.
        tree = self.base_tree("nine", LISTING, True)
        tree[INSTALL] = (
            "Guide page.\n\n`spk explain` serves eight topics straight "
            "from the binary. Nothing more.\n"
        )
        root = self.write(tree)
        problems = cetd.check(root)
        self.assertEqual(len(problems), 1, problems)
        self.assertIn("'eight topics' != the 9 topics", problems[0])

    def test_count_only_page_is_not_required_to_enumerate_ids(self):
        # The missing-id rule must not fire for count-only pages even
        # when zero served ids appear in the text.
        tree = self.base_tree("nine", LISTING, True)
        tree[INSTALL] = "Guide page. Nine topics served, ids omitted.\n"
        root = self.write(tree)
        self.assertEqual(cetd.check(root), [])

    def test_parse_topics_reads_append_only_const(self):
        self.assertEqual(cetd.parse_topics(topics_block(ALL_IDS)), ALL_IDS)

    def test_parse_topics_is_fail_closed_on_missing_block(self):
        with self.assertRaises(ValueError):
            cetd.parse_topics("pub const SOMETHING: &str = \"x\";\n")

    def test_parse_topics_is_fail_closed_on_empty_block(self):
        with self.assertRaises(ValueError):
            cetd.parse_topics("pub const TOPICS: &[(&str, &str)] = &[];\n")

    def test_lint_rules_never_matches_count_phrase(self):
        # "lint-rules topics" is a topic id before "topics", not a count —
        # the count regex must not read it as stale prose.
        self.assertFalse(cetd.COUNT_PHRASE_RE.search("lifecycle/lint-rules topics"))


if __name__ == "__main__":
    unittest.main()
