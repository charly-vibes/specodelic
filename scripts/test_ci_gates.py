#!/usr/bin/env python3
"""Tests for CI gate composition in the justfile (specodelic-12e).

The corpus artifacts shipped in specodelic/ were once re-keyed by hand
without a re-run (fabricated provenance). The gate that prevents silent
recurrence: `just ci` must include a corpus-wide model-check stage, and
that stage must actually invoke `spk model-check specs` (cargo run, never
a possibly-stale PATH binary).
"""

import re
import unittest
from pathlib import Path

JUSTFILE = Path(__file__).resolve().parent.parent / "justfile"


def recipe_body(text: str, name: str) -> str:
    """Return the body lines of a justfile recipe until the next recipe."""
    m = re.search(rf"(?m)^{re.escape(name)}(?:\s+\S+)*:\s*$", text)
    if not m:
        return ""
    rest = text[m.end():]
    lines = []
    for line in rest.splitlines():
        if line and not line[0].isspace():
            break
        lines.append(line)
    return "\n".join(lines)


class TestCorpusModelCheckGate(unittest.TestCase):
    def setUp(self):
        self.text = JUSTFILE.read_text()

    def test_ci_includes_model_check_specs(self):
        ci_recipe = re.search(rf"(?m)^{re.escape('ci')}:\s*\S.*$", self.text)
        self.assertIsNotNone(ci_recipe, "ci recipe missing")
        self.assertIn("model-check-specs", ci_recipe.group(0))

    def test_model_check_specs_recipe_invokes_spk(self):
        body = recipe_body(self.text, "model-check-specs")
        self.assertGreater(len(body.strip()), 0, "model-check-specs recipe missing")
        self.assertIn("model-check specs", body)

    def test_gate_uses_cargo_run_not_path_binary(self):
        body = recipe_body(self.text, "model-check-specs")
        self.assertIn("cargo run", body)


if __name__ == "__main__":
    unittest.main()
