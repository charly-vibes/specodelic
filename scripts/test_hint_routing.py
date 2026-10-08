"""CLI hint-routing tests (specodelic-ils B1).

The remediation hint on `artifact_invalid`/`out_of_scope_refused` stderr
refusals must be routed by the exception TYPE, not by substring: an
`ArtifactInvalid` raised while parsing the lint envelope itself ("lint
envelope carries no issues list") contains the word "lint" but is an
artifact-contract breach — it gets the regenerate-the-artifact hint, not
the fix-your-invariants hint. Controls pin the three real routes:
lint-dirty refusals keep LINT_HINT, intentless refusals keep
INTENTLESS_HINT, artifact breaches keep ARTIFACT_HINT.
"""

import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))

from state_view_cases import (  # noqa: E402
    GRAPH_OK,
    GRAPH_ZERO_FILES,
    LINT_DIRTY,
    run_states_cli,
)

# A successful lint envelope whose data carries no issues list — the
# artifact-contract breach the substring router mis-hints (B1).
LINT_NO_ISSUES_LIST = {
    "ok": True,
    "envelope_version": "0.1",
    "data": {"files_linted": 1, "warnings": []},
}

ARTIFACT_HINT_MARK = "matching tool version"
LINT_HINT_MARK = "fix the reported invariants"
INTENTLESS_HINT_MARK = "zero spec files"


class TestHintRouting(unittest.TestCase):
    """Hints ride the exception type (artifact breach vs corpus refusal)."""

    def test_lint_envelope_without_issues_list_gets_artifact_hint(self):
        proc = run_states_cli(lint=LINT_NO_ISSUES_LIST)
        self.assertNotEqual(proc.returncode, 0)
        self.assertIn("artifact_invalid", proc.stderr)
        self.assertIn(ARTIFACT_HINT_MARK, proc.stderr,
                      "the lint envelope's artifact breach routes ARTIFACT_HINT")
        self.assertNotIn(LINT_HINT_MARK, proc.stderr,
                         "a broken envelope is not a fix-your-invariants case")

    def test_lint_dirty_refusal_keeps_the_fix_invariants_hint(self):
        proc = run_states_cli(lint=LINT_DIRTY)
        self.assertNotEqual(proc.returncode, 0)
        self.assertIn("out_of_scope_refused", proc.stderr)
        self.assertIn(LINT_HINT_MARK, proc.stderr)

    def test_intentless_refusal_keeps_the_zero_files_hint(self):
        proc = run_states_cli(graph=GRAPH_ZERO_FILES)
        self.assertNotEqual(proc.returncode, 0)
        self.assertIn("out_of_scope_refused", proc.stderr)
        self.assertIn(INTENTLESS_HINT_MARK, proc.stderr)

    def test_short_tsv_row_gets_artifact_hint(self):
        proc = run_states_cli(tsv="two.t\tTransition\ttransitions.from\ttwo.s1\t\n",
                              graph=GRAPH_OK)
        self.assertNotEqual(proc.returncode, 0)
        self.assertIn("artifact_invalid", proc.stderr)
        self.assertIn(ARTIFACT_HINT_MARK, proc.stderr)
        self.assertNotIn(LINT_HINT_MARK, proc.stderr)


if __name__ == "__main__":
    unittest.main()