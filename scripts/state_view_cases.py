"""State-view rendering consumer tests (add-graph-views 1.7/2.4, gre.6).

Feeds the six-column `spk graph --format edges` TSV plus the `spk graph
--json` envelope — the exact shapes the spk commands emit — to the
states renderer and pins: distinct state identities (two states stay
two), dashed guards (D8), distinct-source fan-in (D2 multiplicity),
violation annotation (D3), the labeled no_transitions view, and the
owning-file subgraph grouping. The scope/refusal classes live in
state_view_scope_cases. Re-imported by test_graph_views.py.
"""

import json
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))

import graph_views  # noqa: E402

TWO_STATE_TSV = (
    "two.c1\tConstraint\tconstraints.traces_to\ttwo\tIntent\t\n"
    "two.c1\tConstraint\tconstraints.traces_to\ttwo\tIntent\t\n"
    "two.t\tTransition\ttransitions.from\ttwo.s1\tState\t\n"
    "two.t\tTransition\ttransitions.guard\ttwo.c1\tConstraint\t\n"
    "two.t\tTransition\ttransitions.guard\ttwo.c1\tConstraint\t\n"
    "two.t\tTransition\ttransitions.to\ttwo.s2\tState\t\n"
)

GRAPH_OK = {
    "ok": True,
    "envelope_version": "0.1",
    "data": {"files": 1, "nodes": 5, "intents": ["two"]},
}
GRAPH_INTENTLESS = {
    "ok": True,
    "envelope_version": "0.1",
    "data": {"files": 0, "nodes": 0, "intents": []},
}
# What `spk graph --json` emits over a zero-spec-file directory: an
# unsuccessful envelope (exit 2) whose refusal reason rides warnings.
GRAPH_ZERO_FILES = {
    "ok": False,
    "envelope_version": "0.1",
    "envelope_kind": "error",
    "data": None,
    "warnings": [
        {"rule_name": "", "message": "no spec files found — nothing was graphed"}
    ],
}
LINT_CLEAN = {"ok": True, "envelope_version": "0.1", "data": {"files_linted": 1, "issues": [], "warnings": []}}
LINT_DIRTY = {
    "ok": True,
    "envelope_version": "0.1",
    "data": {
        "files_linted": 1,
        "issues": [
            {
                "rule_id": "linter.guard_required",
                "rule_semantics": "every transition must carry a non-null guard",
                "file": "dirty.t",
                "message": "transition `dirty.t` has no guard",
            }
        ],
        "warnings": [],
    },
}
LINT_FAILED_ENVELOPE = {
    "ok": False,
    "envelope_version": "0.1",
    "envelope_kind": "error",
    "data": None,
    "warnings": [
        {"rule_name": "", "message": "no spec files found — nothing was linted"}
    ],
}

VIOLATION_ROW = (
    "\t\tviolation:transitions.guard\ttwo.c1\tConstraint\t"
    "a transitions.guard edge must resolve to an invariant Constraint or "
    "a State — `two.c2_effect` is an effect Constraint\n"
)
TRACES_VIOLATION_ROW = (
    "\t\tviolation:constraints.traces_to\ttwo.c1\tConstraint\t"
    "a constraints.traces_to edge must resolve to an Intent — `two.c1` is "
    "a Constraint\n"
)


def states_view(tsv=TWO_STATE_TSV, graph=GRAPH_OK, lint=None):
    return graph_views.render_states(tsv, graph, lint)


def write_artifacts(tsv=TWO_STATE_TSV, graph=GRAPH_OK, lint=None):
    """Serialize the artifacts to temp files; return the CLI argv tail."""
    import tempfile

    paths = {}
    for name, content in (("tsv", tsv), ("graph", json.dumps(graph)),
                          ("lint", json.dumps(lint) if lint is not None else None)):
        if content is None:
            continue
        f = tempfile.NamedTemporaryFile(
            "w", suffix=f"_{name}", prefix="state_view_", delete=False
        )
        f.write(content)
        f.close()
        paths[name] = f.name
    return paths


def run_states_cli(tsv=TWO_STATE_TSV, graph=GRAPH_OK, lint=None):
    paths = write_artifacts(tsv, graph, lint)
    try:
        argv = [
            sys.executable, str(Path(__file__).parent / "graph_views.py"),
            "states", paths["tsv"], "--graph", paths["graph"],
        ]
        if lint is not None:
            argv += ["--lint", paths["lint"]]
        return subprocess.run(argv, capture_output=True, text=True)
    finally:
        for p in paths.values():
            Path(p).unlink(missing_ok=True)



class TestStateViewRendering(unittest.TestCase):
    """Per-file state machines keep distinct identities (task 1.7/2.4)."""

    def test_renders_exactly_two_distinct_states(self):
        out = states_view()
        self.assertIn("two.s1", out)
        self.assertIn("two.s2", out)
        # Two states, drawn as two distinct nodes — never collapsed to the
        # owning intent (only file-level views collapse, task 1.7).
        state_lines = [ln for ln in out.splitlines() if '"two.s' in ln]
        self.assertEqual(len(state_lines), 2, f"two states must render as two nodes:\n{out}")

    def test_renders_transition_from_to_and_guard(self):
        out = states_view()
        self.assertIn("transitions.from", out)
        self.assertIn("transitions.to", out)
        self.assertIn("transitions.guard", out)
        self.assertIn("two.t", out)

    def test_guard_edge_is_dashed_and_from_to_are_solid(self):
        out = states_view()
        guard = [ln for ln in out.splitlines() if "transitions.guard" in ln and "-.->" in ln]
        self.assertEqual(len(guard), 1, f"the guard edge renders dashed exactly once:\n{out}")
        from_to = [ln for ln in out.splitlines()
                   if ("transitions.from" in ln or "transitions.to" in ln)]
        self.assertTrue(from_to, "from/to edges render")
        for ln in from_to:
            self.assertIn("-->", ln, f"from/to edges are solid: {ln}")

    def test_duplicate_guard_rows_draw_one_edge_and_count_fan_in_once(self):
        # D2: the raw TSV keeps both duplicate instances; the view draws
        # the guard edge once and counts the guard target's fan-in over
        # DISTINCT sources.
        out = states_view()
        guard_edges = [ln for ln in out.splitlines()
                       if "transitions.guard" in ln and "two_c1" in ln and "-.->" in ln]
        self.assertEqual(len(guard_edges), 1,
                         f"duplicate instances draw once (view-layer dedup):\n{out}")
        c1_line = [ln for ln in out.splitlines() if '"two.c1' in ln][0]
        self.assertIn("fan-in 1", c1_line,
                      f"fan-in counts distinct sources, not instances: {c1_line}")

    def test_fan_in_counts_distinct_transitions_per_state(self):
        # A second distinct transition t2 into the same state s2: fan-in 2.
        t2_rows = (
            "two.t2\tTransition\ttransitions.from\ttwo.s1\tState\t\n"
            "two.t2\tTransition\ttransitions.to\ttwo.s2\tState\t\n"
        )
        out = states_view(tsv=TWO_STATE_TSV + t2_rows)
        s2_line = [ln for ln in out.splitlines() if '"two.s2' in ln][0]
        self.assertIn("fan-in 2", s2_line, f"two distinct transitions into s2: {s2_line}")

    def test_groups_transitions_by_owning_file_in_subgraphs(self):
        other = (
            "other.t\tTransition\ttransitions.from\tother.s1\tState\t\n"
            "other.t\tTransition\ttransitions.to\tother.s2\tState\t\n"
        )
        graph = {
            "ok": True,
            "envelope_version": "0.1",
            "data": {"files": 2, "nodes": 9, "intents": ["other", "two"]},
        }
        out = states_view(tsv=TWO_STATE_TSV + other, graph=graph)
        self.assertEqual(
            out.count("subgraph "), 2, f"one subgraph per owning file:\n{out}"
        )
        # Each file's transitions render inside their own subgraph block.
        blocks = out.split("subgraph ")[1:]
        two_block = next(b for b in blocks if "two_t" in b)
        other_block = next(b for b in blocks if "other_t" in b)
        self.assertIn("two_s1", two_block)
        self.assertNotIn("other_t", two_block)
        self.assertIn("other_s2", other_block)
        self.assertNotIn("two_t", other_block)

    def test_renderer_is_deterministic(self):
        self.assertEqual(states_view(), states_view())

    def test_violations_render_as_annotated_elements(self):
        out = states_view(tsv=TWO_STATE_TSV + VIOLATION_ROW)
        self.assertIn("violation:transitions.guard", out)
        self.assertIn("effect Constraint", out, "the reason text rides along")
        self.assertIn(":::violation", out)
        self.assertIn("classDef violation", out)

    def test_every_violation_row_is_annotated_not_omitted(self):
        tsv = TWO_STATE_TSV + VIOLATION_ROW + TRACES_VIOLATION_ROW
        out = states_view(tsv=tsv)
        self.assertIn("violation:transitions.guard", out)
        self.assertIn("violation:constraints.traces_to", out)

    def test_no_transitions_view_is_labeled_not_silently_clean(self):
        tsv = "two.c1\tConstraint\tconstraints.traces_to\ttwo\tIntent\t\n"
        out = states_view(tsv=tsv)
        self.assertIn("no_transitions", out)



if __name__ == "__main__":
    import unittest

    unittest.main()
