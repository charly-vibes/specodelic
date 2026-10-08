"""Traceability-view rendering consumer tests (add-graph-views 2.5, gre.7).

Feeds the six-column `spk graph --format edges` TSV plus the `spk graph
--json` envelope — the exact shapes the spk commands emit — to the
traceability renderer and pins: the node set equals the intent set
(exactly one node per intent, including a single-intent corpus), file
dependencies as edges between intents only (row-level ids collapse),
fan-in counting DISTINCT source intents (D2 multiplicity — duplicate TSV
instances add 0 beyond a source's first contribution), violations staying
annotated red-dashed elements (D3, gre.6 precedent), and the labeled
no_dependencies view. Scope/refusal legs are covered here too (same
out_of_scope_refused contract as the states view). Re-imported by
test_graph_views.py.
"""

import json
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))

import graph_views  # noqa: E402
from state_view_cases import (  # noqa: E402
    GRAPH_INTENTLESS,
    GRAPH_ZERO_FILES,
    LINT_CLEAN,
    LINT_DIRTY,
    LINT_FAILED_ENVELOPE,
)

# alpha depends on beta (duplicate raw instances — the TSV's multiplicity
# contract), gamma's transition collapses into beta, alpha's own
# transitions are intra-file (collapse to self, not file dependencies).
DEP_TSV = (
    "alpha.c1\tConstraint\tconstraints.traces_to\tbeta\tIntent\t\n"
    "alpha.c1\tConstraint\tconstraints.traces_to\tbeta\tIntent\t\n"
    "gamma.t\tTransition\ttransitions.to\tbeta.s1\tState\t\n"
    "alpha.t\tTransition\ttransitions.from\talpha.s1\tState\t\n"
    "alpha.t\tTransition\ttransitions.to\talpha.s2\tState\t\n"
)
GRAPH_DEPS = {
    "ok": True,
    "envelope_version": "0.1",
    "data": {"files": 3, "nodes": 8, "intents": ["alpha", "beta", "gamma"]},
}
# A fourth intent with no edge endpoints at all still owns a node — the
# node set equals the intent set (spec: Collapse to intents).
GRAPH_DEPS_PLUS_DELTA = {
    "ok": True,
    "envelope_version": "0.1",
    "data": {"files": 4, "nodes": 9, "intents": ["alpha", "beta", "delta", "gamma"]},
}
VIOLATION_ROW = (
    "\t\tviolation:constraints.traces_to\tbeta.c9\tConstraint\t"
    "a constraints.traces_to edge must resolve to an Intent — `beta.c9` is "
    "a Constraint\n"
)


def traceability_view(tsv=DEP_TSV, graph=GRAPH_DEPS, lint=None):
    return graph_views.render_traceability(tsv, graph, lint)


def write_artifacts(tsv=DEP_TSV, graph=GRAPH_DEPS, lint=None):
    """Serialize the artifacts to temp files; return the CLI argv tail."""
    paths = {}
    for name, content in (
        ("tsv", tsv),
        ("graph", json.dumps(graph)),
        ("lint", json.dumps(lint) if lint is not None else None),
    ):
        if content is None:
            continue
        f = tempfile.NamedTemporaryFile(
            "w", suffix=f"_{name}", prefix="trace_view_", delete=False
        )
        f.write(content)
        f.close()
        paths[name] = f.name
    return paths


def run_trace_cli(tsv=DEP_TSV, graph=GRAPH_DEPS, lint=None):
    paths = write_artifacts(tsv, graph, lint)
    try:
        argv = [
            sys.executable,
            str(Path(__file__).parent / "graph_views.py"),
            "trace",
            paths["tsv"],
            "--graph",
            paths["graph"],
        ]
        if lint is not None:
            argv += ["--lint", paths["lint"]]
        return subprocess.run(argv, capture_output=True, text=True)
    finally:
        for p in paths.values():
            Path(p).unlink(missing_ok=True)


class TestTraceabilityRendering(unittest.TestCase):
    """File-level traceability: nodes = intents, edges = file deps (2.5)."""

    def test_node_set_equals_intent_set_exactly_one_node_each(self):
        out = traceability_view(graph=GRAPH_DEPS_PLUS_DELTA)
        for intent in ("alpha", "beta", "delta", "gamma"):
            decls = [ln for ln in out.splitlines() if f'{intent}["' in ln]
            self.assertEqual(
                len(decls), 1, f"exactly one node per intent {intent}:\n{out}"
            )

    def test_row_level_ids_never_appear_as_endpoints(self):
        out = traceability_view()
        for row_id in ("alpha.s1", "alpha.s2", "alpha.c1", "beta.s1", "gamma.t"):
            self.assertNotIn(row_id, out, f"row ids collapse to intents: {row_id}")

    def test_file_dependencies_draw_once_between_intents(self):
        out = traceability_view()
        self.assertIn("alpha --> beta", out, out)
        self.assertIn("gamma --> beta", out, out)
        # Duplicate raw instances draw ONE dependency edge.
        self.assertEqual(out.count("alpha --> beta"), 1, out)
        # Intra-file edges are not file dependencies: no self-loop.
        self.assertNotIn("alpha --> alpha", out, out)

    def test_fan_in_counts_distinct_source_intents(self):
        # beta is targeted by alpha (twice, raw) and gamma: fan-in 2, not 3.
        out = traceability_view()
        self.assertIn('beta["beta (fan-in 2)"]', out, out)
        # Sources with no incoming cross-file edges carry no annotation.
        self.assertIn('alpha["alpha"]', out, out)
        self.assertIn('gamma["gamma"]', out, out)

    def test_duplicate_rows_add_zero_beyond_first_contribution(self):
        # D2 multiplicity: alpha's duplicate instances contribute once —
        # one instance per source gives the same view as duplicates.
        single = (
            "alpha.c1\tConstraint\tconstraints.traces_to\tbeta\tIntent\t\n"
            "gamma.t\tTransition\ttransitions.to\tbeta.s1\tState\t\n"
        )
        self.assertEqual(traceability_view(), traceability_view(tsv=single))
        self.assertIn(
            'beta["beta (fan-in 2)"]',
            traceability_view(tsv=single),
            "one instance per source gives the same fan-in as duplicates",
        )

    def test_intentless_endpoint_beyond_intent_set_refused(self):
        # A TSV endpoint that collapses outside the intent set means the
        # artifacts disagree — artifact_invalid, never silently dropped.
        tsv = "ghost.x\tConstraint\tconstraints.traces_to\tbeta\tIntent\t\n"
        with self.assertRaises(graph_views.ArtifactInvalid) as ctx:
            traceability_view(tsv=tsv)
        self.assertIn("ghost", str(ctx.exception))

    def test_renderer_is_deterministic(self):
        self.assertEqual(traceability_view(), traceability_view())

    def test_violations_stay_annotated_not_omitted(self):
        out = traceability_view(tsv=DEP_TSV + VIOLATION_ROW)
        self.assertIn("violation:constraints.traces_to", out)
        self.assertIn("must resolve to an Intent", out, "the reason rides along")
        self.assertIn(":::violation", out)
        self.assertIn("classDef violation", out)

    def test_violations_do_not_contribute_to_fan_in(self):
        # Violations are not recorded edges (D3): beta stays fan-in 2.
        out = traceability_view(tsv=DEP_TSV + VIOLATION_ROW)
        self.assertIn('beta["beta (fan-in 2)"]', out, out)

    def test_empty_tsv_renders_nodes_and_labeled_note(self):
        # The zero-row artifact (states precedent pins the empty TSV too):
        # nodes still cover the intent set, the note is labeled, nothing
        # is silently clean.
        out = traceability_view(tsv="")
        self.assertEqual(
            len([ln for ln in out.splitlines() if 'alpha["' in ln]), 1, out
        )
        self.assertIn("no_dependencies", out)
        self.assertIn("classDef violation", out)

    def test_single_intent_corpus_renders_exactly_one_intent_node(self):
        # tests/fixtures/single_intent shape: one intent, intra-file rows
        # only — 1 intent node plus the labeled no_dependencies note
        # (never silently clean), exit 0.
        tsv = (
            "two.c1\tConstraint\tconstraints.traces_to\ttwo\tIntent\t\n"
            "two.t\tTransition\ttransitions.from\ttwo.s1\tState\t\n"
        )
        graph = {
            "ok": True,
            "envelope_version": "0.1",
            "data": {"files": 1, "nodes": 5, "intents": ["two"]},
        }
        out = traceability_view(tsv=tsv, graph=graph)
        self.assertEqual(
            len([ln for ln in out.splitlines() if 'two["' in ln]), 1, out
        )
        self.assertIn("no_dependencies", out)


class TestTraceabilityScope(unittest.TestCase):
    """The scope gate refuses out-of-scope corpora BEFORE output (2.5:
    same out_of_scope_refused legs as the states view)."""

    def test_intentless_refused(self):
        with self.assertRaises(graph_views.OutOfScopeRefused) as ctx:
            traceability_view(graph=GRAPH_INTENTLESS)
        self.assertIn("intentless", str(ctx.exception))

    def test_unsuccessful_graph_envelope_refused(self):
        with self.assertRaises(graph_views.OutOfScopeRefused):
            traceability_view(graph=GRAPH_ZERO_FILES)

    def test_lint_dirty_refused_naming_the_failed_gate(self):
        with self.assertRaises(graph_views.OutOfScopeRefused) as ctx:
            traceability_view(lint=LINT_DIRTY)
        self.assertIn("guard_required", str(ctx.exception))

    def test_unsuccessful_lint_envelope_refused(self):
        with self.assertRaises(graph_views.OutOfScopeRefused):
            traceability_view(lint=LINT_FAILED_ENVELOPE)

    def test_clean_lint_envelope_renders(self):
        self.assertIn("alpha --> beta", traceability_view(lint=LINT_CLEAN))


class TestTraceabilityCli(unittest.TestCase):
    """The trace CLI writes Mermaid on success and refuses with the named
    failure before any output."""

    def test_valid_cli_writes_mermaid(self):
        proc = run_trace_cli()
        self.assertEqual(proc.returncode, 0, proc.stderr)
        self.assertIn("flowchart", proc.stdout)
        self.assertIn("alpha --> beta", proc.stdout)

    def test_intentless_cli_fails_before_output(self):
        proc = run_trace_cli(graph=GRAPH_ZERO_FILES)
        self.assertNotEqual(proc.returncode, 0)
        self.assertIn("out_of_scope_refused", proc.stderr)
        self.assertEqual(proc.stdout, "")

    def test_lint_dirty_cli_fails_before_output(self):
        proc = run_trace_cli(lint=LINT_DIRTY)
        self.assertNotEqual(proc.returncode, 0)
        self.assertIn("out_of_scope_refused", proc.stderr)
        self.assertEqual(proc.stdout, "")

    def test_single_intent_cli_exits_zero_with_labeled_view(self):
        tsv = "two.c1\tConstraint\tconstraints.traces_to\ttwo\tIntent\t\n"
        graph = {
            "ok": True,
            "envelope_version": "0.1",
            "data": {"files": 1, "nodes": 5, "intents": ["two"]},
        }
        proc = run_trace_cli(tsv=tsv, graph=graph)
        self.assertEqual(proc.returncode, 0, proc.stderr)
        self.assertIn("no_dependencies", proc.stdout)
        self.assertIn('two["', proc.stdout)


if __name__ == "__main__":
    import unittest

    unittest.main()