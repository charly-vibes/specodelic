"""Mermaid label-escaping unit tests (specodelic-ils B2, design D8).

Labels embed arbitrary spec-cell text, so `mermaid_escape` must neutralize
everything that can break out of or alter the quoted-label context: the
double quote (string terminator), `-->` (arrow grammar), `#` (mermaid's
`#nn;` entity codes), `%%` (mermaid comments), a label line starting
`end` (subgraph terminator — reachable only through a newline, which the
escaper flattens), `&` (entity smuggling) and `<`/`>` (HTML tags).
Already-safe label text must stay byte-identical (D8: deterministic text
output, no double-escaping drift). The Rust mirror in src/graph.rs pins
the same cases in its own test module — the two must agree byte-for-byte.
"""

import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))

from view_common import mermaid_escape  # noqa: E402


class TestMermaidEscape(unittest.TestCase):
    """The quoted-label context is closed against spec-cell text (B2)."""

    def test_quote_breakout_escaped(self):
        self.assertEqual(mermaid_escape('a "b"'), "a &quot;b&quot;")

    def test_arrow_grammar_neutralized(self):
        self.assertEqual(mermaid_escape("a --> b"), "a --&gt; b")

    def test_hash_entity_syntax_neutralized(self):
        self.assertEqual(mermaid_escape("a # b"), "a &num; b")

    def test_percent_comment_syntax_neutralized(self):
        self.assertEqual(mermaid_escape("a %% b"), "a &percnt;&percnt; b")

    def test_entity_smuggling_neutralized(self):
        self.assertEqual(mermaid_escape("a & b"), "a &amp; b")

    def test_newline_flattened_so_no_line_starts_with_end(self):
        # A multi-line label would put attacker text at the start of a
        # diagram line (a line starting `end` closes the subgraph).
        escaped = mermaid_escape("before\nend")
        self.assertEqual(escaped, "before end")

    def test_already_safe_label_is_byte_identical(self):
        self.assertEqual(mermaid_escape("two.c1 (fan-in 2)"), "two.c1 (fan-in 2)")

    def test_escape_deterministic(self):
        self.assertEqual(mermaid_escape("a --> # %%"), mermaid_escape("a --> # %%"))


if __name__ == "__main__":
    unittest.main()