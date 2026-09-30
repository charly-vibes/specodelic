#!/usr/bin/env python3
"""Unit tests for check_doc_examples.py — extraction and naming law.

Stdlib unittest only (repo pattern: scripts run in CI without deps).
Run: python3 -m unittest discover -s scripts -p 'test_*.py'
"""

import unittest

from check_doc_examples import extract_spec_examples, frontmatter_id, stem_for_id


FULL_EXAMPLE = """\
```markdown
---
id: order.cancel
kind: intent
statement: "WHEN a customer cancels THE system SHALL refund."
---

# Order Cancellation

Prose.

## Constraints

| id | kind | expr | traces_to |
|----|------|------|-----------|
| c1 | invariant | `x == y` | [[order.cancel]] |
```
"""

TABLE_FRAGMENT = """\
```markdown
| id | kind | expr | traces_to |
|----|------|------|-----------|
| c1 | invariant | `x == y` | [[canon]] |
```
"""


class TestExtractSpecExamples(unittest.TestCase):
    def test_full_example_with_frontmatter_is_extracted(self):
        found = extract_spec_examples(FULL_EXAMPLE)
        self.assertEqual(len(found), 1)
        self.assertTrue(found[0].lstrip().startswith("---"))
        self.assertIn("## Constraints", found[0])

    def test_table_fragment_without_frontmatter_is_skipped(self):
        self.assertEqual(extract_spec_examples(TABLE_FRAGMENT), [])

    def test_non_markdown_fences_are_skipped(self):
        text = "```bash\n---\nid: fake\n---\n```\n"
        self.assertEqual(extract_spec_examples(text), [])

    def test_example_and_fragment_coexist(self):
        text = FULL_EXAMPLE + "\nprose between fences\n\n" + TABLE_FRAGMENT
        self.assertEqual(len(extract_spec_examples(text)), 1)

    def test_multiple_examples_are_all_extracted(self):
        text = FULL_EXAMPLE + "\nmid-text\n\n" + FULL_EXAMPLE
        self.assertEqual(len(extract_spec_examples(text)), 2)

    def test_unclosed_fence_is_dropped_not_crashed(self):
        self.assertEqual(
            extract_spec_examples("```markdown\n---\nid: x\n---\n"), []
        )


class TestNamingLaw(unittest.TestCase):
    def test_id_to_stem_maps_dot_to_dash(self):
        self.assertEqual(stem_for_id("order.cancel"), "order-cancel")
        self.assertEqual(stem_for_id("linter.graph_shape"), "linter-graph_shape")

    def test_underscore_is_literal(self):
        self.assertEqual(stem_for_id("rename_naturality"), "rename_naturality")

    def test_frontmatter_id_parses_through_quotes(self):
        body = extract_spec_examples(FULL_EXAMPLE)[0]
        self.assertEqual(frontmatter_id(body), "order.cancel")


if __name__ == "__main__":
    unittest.main()
