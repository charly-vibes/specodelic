"""State-view scope and CLI refusal tests (add-graph-views 2.3, gre.6).

Pins the out_of_scope_refused legs (intentless corpora, lint-dirty
corpora — the failed gate named by rule id) and the artifact_invalid
refusals: all BEFORE any output is written, at the render level and
through the CLI. Fixtures are imported from state_view_cases.
Re-imported by test_graph_views.py.
"""

import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))

from state_view_cases import (  # noqa: E402
    GRAPH_INTENTLESS,
    GRAPH_OK,
    GRAPH_ZERO_FILES,
    LINT_CLEAN,
    LINT_DIRTY,
    LINT_FAILED_ENVELOPE,
    TWO_STATE_TSV,
    run_states_cli,
    states_view,
)
import graph_views  # noqa: E402

class TestStateViewScope(unittest.TestCase):
    """The scope gate refuses out-of-scope corpora BEFORE output (2.3)."""

    def test_intentless_zero_files_data_refused(self):
        with self.assertRaises(graph_views.OutOfScopeRefused) as ctx:
            states_view(graph=GRAPH_INTENTLESS)
        self.assertIn("intentless", str(ctx.exception))

    def test_unsuccessful_graph_envelope_refused_with_its_reason(self):
        with self.assertRaises(graph_views.OutOfScopeRefused) as ctx:
            states_view(graph=GRAPH_ZERO_FILES)
        self.assertIn("no spec files found", str(ctx.exception))

    def test_lint_dirty_corpus_refused_naming_the_failed_gate(self):
        with self.assertRaises(graph_views.OutOfScopeRefused) as ctx:
            states_view(lint=LINT_DIRTY)
        msg = str(ctx.exception)
        self.assertIn("lint", msg)
        self.assertIn("guard_required", msg, "the failed gate is named by rule id")

    def test_unsuccessful_lint_envelope_refused(self):
        with self.assertRaises(graph_views.OutOfScopeRefused):
            states_view(lint=LINT_FAILED_ENVELOPE)

    def test_clean_lint_envelope_renders(self):
        out = states_view(lint=LINT_CLEAN)
        self.assertIn("two.s1", out)

    def test_without_lint_input_the_lint_leg_is_skipped(self):
        out = states_view(lint=None)
        self.assertIn("two.s1", out)


class TestStateViewArtifactInvalid(unittest.TestCase):
    """Malformed TSV artifacts are refused (edges_tsv_invalid), like the
    schema view refuses malformed exports — before any output."""

    def test_short_row_refused(self):
        with self.assertRaises(graph_views.ArtifactInvalid):
            states_view(tsv="two.t\tTransition\ttransitions.from\ttwo.s1\t\n")

    def test_malformed_graph_envelope_refused_as_invalid(self):
        with self.assertRaises(graph_views.ArtifactInvalid):
            states_view(graph={"ok": True})  # envelope carries no data object


class TestStatesCliRefusals(unittest.TestCase):
    """The CLI writes no diagram and exits non-zero with the named failure."""

    def test_intentless_cli_fails_before_output(self):
        proc = run_states_cli(graph=GRAPH_ZERO_FILES)
        self.assertNotEqual(proc.returncode, 0)
        self.assertIn("out_of_scope_refused", proc.stderr)
        self.assertEqual(proc.stdout, "")

    def test_lint_dirty_cli_fails_before_output(self):
        proc = run_states_cli(lint=LINT_DIRTY)
        self.assertNotEqual(proc.returncode, 0)
        self.assertIn("out_of_scope_refused", proc.stderr)
        self.assertIn("guard_required", proc.stderr)
        self.assertEqual(proc.stdout, "")

    def test_valid_cli_writes_mermaid(self):
        proc = run_states_cli()
        self.assertEqual(proc.returncode, 0, proc.stderr)
        self.assertIn("flowchart", proc.stdout)
        self.assertIn("two.s1", proc.stdout)

    def test_single_intent_corpus_renders_labeled_empty_view(self):
        # tests/fixtures/single_intent: one intent, zero edges — in scope
        # (files=1) but no transition edges anywhere → the labeled
        # no_transitions view, exit 0 (files without transitions skip
        # cleanly; never silently clean).
        tsv = ""
        proc = run_states_cli(tsv=tsv, graph=GRAPH_OK)
        self.assertEqual(proc.returncode, 0, proc.stderr)
        self.assertIn("no_transitions", proc.stdout)



if __name__ == "__main__":
    import unittest

    unittest.main()
