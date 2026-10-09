#!/usr/bin/env python3
"""Unit tests for check_quickstart_outputs.py — fence pairing + manifest.

Stdlib unittest only (repo pattern: scripts run in CI without deps).
Run: python3 -m unittest discover -s scripts -p 'test_*.py'

The manifest tests run against the real docs in this repo: every
documented captured output must stay resolvable (fence present, unique
first_line match), so doc reformatting fails the test suite, not just
the gate.
"""

import unittest
from pathlib import Path

from check_quickstart_outputs import (
    extract_pairs,
    extract_text_blocks,
    normalize,
    pair_index,
    resolve_expected,
)
from quickstart_manifest import (
    FINAL_SPEC,
    INSTALLATION,
    ROOT,
    WORKED,
    break_c1_link,
    scenario_table,
    spk_lines,
)

PAIRED_DOC = """\
prose

```sh
# comment
spk lint file.md
```

```text
lint: ok
```

```sh
spk compile file.md
spk model-check file.md
```

```text
compile: ok
model-check: ok
```
"""

UNPAIRED_DOC = """\
```sh
spk lint file.md
```

prose between, no text fence follows

```json
{"outcome": "clean"}
```
"""


class TestExtractPairs(unittest.TestCase):
    def test_adjacent_sh_text_pair_is_extracted(self):
        pairs = extract_pairs(PAIRED_DOC)
        self.assertEqual(len(pairs), 2)
        self.assertEqual(pairs[0][0], "# comment\nspk lint file.md")
        self.assertEqual(pairs[0][1], "lint: ok")

    def test_multi_command_fence_keeps_full_body(self):
        pairs = extract_pairs(PAIRED_DOC)
        self.assertEqual(
            pairs[1][0], "spk compile file.md\nspk model-check file.md")

    def test_fence_without_text_partner_is_skipped(self):
        self.assertEqual(extract_pairs(UNPAIRED_DOC), [])

    def test_non_sh_fences_are_never_paired(self):
        doc = "```text\njust text\n```\n"
        self.assertEqual(extract_pairs(doc), [])

    def test_text_blocks_listed_in_document_order(self):
        blocks = extract_text_blocks(PAIRED_DOC)
        self.assertEqual(blocks, ["lint: ok", "compile: ok\nmodel-check: ok"])


class TestManifestHelpers(unittest.TestCase):
    def test_pair_index_occurrences_are_in_doc_order(self):
        idx = pair_index([("cmd", "a"), ("cmd", "b"), ("other", "c")])
        self.assertEqual(idx["cmd"], ["a", "b"])
        self.assertEqual(idx["other"], ["c"])

    def test_spk_lines_filters_comments_and_setup(self):
        body = "# hint\ncp a b\ncd somewhere\nspk lint x.md\n"
        self.assertEqual(spk_lines(body), ["spk lint x.md"])

    def test_spk_lines_of_multi_command_fence(self):
        self.assertEqual(
            spk_lines("spk compile x\nspk model-check x"),
            ["spk compile x", "spk model-check x"])

    def test_normalize_rstrips_and_drops_trailing_blanks(self):
        self.assertEqual(normalize("a  \n b \n\n\n"), ["a", " b"])

    def test_break_c1_link_replaces_only_the_traces_to_cell(self):
        text = "| t1 | initial | initial | [[order.cancel.c1]] |\n" \
               "| c1 | invariant | TODO | [[order.cancel]] |\n"
        out = break_c1_link(text)
        self.assertIn("| the file intent |", out)
        self.assertNotIn("[[order.cancel]] |", out)
        self.assertIn("[[order.cancel.c1]]", out)


class TestManifestResolvesAgainstRealDocs(unittest.TestCase):
    """The gate's contract with the live docs: every scenario must resolve."""

    def setUp(self):
        self.unresolved = resolve_expected(ROOT, scenario_table())

    def test_every_scenario_resolves(self):
        self.assertIsNone(self.unresolved)

    def test_scenario_count_covers_both_docs(self):
        table = scenario_table()
        self.assertEqual(len(table), 18)
        self.assertEqual({sc["doc"] for sc in table},
                         {INSTALLATION, WORKED})

    def test_final_spec_scenarios_reference_the_checked_in_example(self):
        table = scenario_table()
        final_users = [sc for sc in table if sc["fixture"] == FINAL_SPEC]
        self.assertGreaterEqual(len(final_users), 4)
        self.assertTrue((ROOT / FINAL_SPEC).is_file())


if __name__ == "__main__":
    unittest.main()
