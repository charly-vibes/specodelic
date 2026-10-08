"""Schema-view consumer tests (add-graph-views 2.3 schema portion, gre.5).

Feeds serialized schema-export envelopes — the exact shape
`spk guide --schema --json` emits (design D4) — to the renderer and pins:
the revision label, the one-edge perturbation contract (two valid
constructed schemas differing in ONE morphism render a diagram differing
by exactly that edge), refinement labels, and the schema_export_invalid
refusals which must fail BEFORE any output is written. Re-imported by
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

REVISION = "specodelic.md Revision 16"

BASE_OBJECTS = ["Constraint", "Intent"]

TRACES_TO = {
    "name": "traces_to",
    "column": "traces_to",
    "source": "Constraint",
    "target": "Intent",
    "refinements": [],
    "source_rule": "unchecked",
    "endo_acyclic": None,
}

# The single morphism the perturbation fixture adds.
USES = {
    "name": "uses",
    "column": "uses",
    "source": "Constraint",
    "target": "Intent",
    "refinements": [],
    "source_rule": "unchecked",
    "endo_acyclic": None,
}

EMITS_LIKE = {
    "name": "emits",
    "column": "emits",
    "source": "State",
    "target": "Constraint",
    "refinements": [{"side": "target", "kind": "effect"}],
    "source_rule": "unchecked",
    "endo_acyclic": None,
}


def envelope(data, ok=True):
    return {"ok": ok, "envelope_version": "0.1", "data": data}


def schema_data(objects, morphisms, revision=REVISION, version=1):
    return {
        "schema_version": version,
        "format_revision": revision,
        "objects": objects,
        "morphisms": morphisms,
    }


def base_morphisms():
    return [dict(TRACES_TO)]


def perturbed_morphisms():
    # Sorted by (source, name): traces_to < uses, both sourced on Constraint.
    return [dict(TRACES_TO), dict(USES)]


def run_cli(payload):
    """Run `python3 graph_views.py schema <tmpfile>`; return (proc, exit)."""
    with tempfile.NamedTemporaryFile(
        "w", suffix=".json", prefix="schema_export_", delete=False
    ) as f:
        json.dump(payload, f)
        export_path = f.name
    try:
        proc = subprocess.run(
            [sys.executable, str(Path(__file__).parent / "graph_views.py"),
             "schema", export_path],
            capture_output=True, text=True,
        )
        return proc
    finally:
        Path(export_path).unlink(missing_ok=True)



class TestSchemaViewRendering(unittest.TestCase):
    def test_renders_objects_edges_and_revision_label(self):
        out = graph_views.render_schema(
            envelope(schema_data(BASE_OBJECTS, base_morphisms()))
        )
        self.assertIn("Constraint", out)
        self.assertIn("Intent", out)
        self.assertIn("traces_to", out)
        self.assertIn(REVISION, out)

    def test_perturbation_one_morphism_changes_exactly_one_edge(self):
        base = graph_views.render_schema(
            envelope(schema_data(BASE_OBJECTS, base_morphisms()))
        )
        perturbed = graph_views.render_schema(
            envelope(schema_data(BASE_OBJECTS, perturbed_morphisms()))
        )
        base_lines = [ln for ln in base.splitlines() if ln.strip()]
        pert_lines = [ln for ln in perturbed.splitlines() if ln.strip()]
        removed = [ln for ln in base_lines if ln not in pert_lines]
        added = [ln for ln in pert_lines if ln not in base_lines]
        self.assertEqual(removed, [], "perturbation must not remove lines")
        self.assertEqual(len(added), 1, "one morphism in = one edge out")
        self.assertIn("uses", added[0])

    def test_refinement_labels_render(self):
        out = graph_views.render_schema(
            envelope(schema_data(
                ["Constraint", "State"], [dict(EMITS_LIKE)]
            ))
        )
        self.assertIn("effect", out)
        self.assertIn("emits", out)

    def test_renderer_is_deterministic(self):
        env = envelope(schema_data(BASE_OBJECTS, base_morphisms()))
        self.assertEqual(graph_views.render_schema(env),
                         graph_views.render_schema(env))


class TestSchemaExportInvalid(unittest.TestCase):
    """Malformed exports fail BEFORE output is written (schema_export_invalid)."""

    def assert_invalid(self, payload):
        with self.assertRaises(graph_views.SchemaExportInvalid):
            graph_views.render_schema(payload)

    def test_unsuccessful_envelope_refused(self):
        self.assert_invalid(
            envelope(schema_data(BASE_OBJECTS, base_morphisms()), ok=False)
        )

    def test_missing_morphism_field_refused(self):
        broken = dict(TRACES_TO)
        del broken["column"]
        self.assert_invalid(
            envelope(schema_data(BASE_OBJECTS, [broken]))
        )

    def test_unknown_schema_version_refused(self):
        self.assert_invalid(
            envelope(schema_data(BASE_OBJECTS, base_morphisms(), version=2))
        )

    def test_duplicate_morphism_identity_refused(self):
        self.assert_invalid(
            envelope(schema_data(BASE_OBJECTS,
                                 [dict(TRACES_TO), dict(TRACES_TO)]))
        )

    def test_dangling_endpoint_refused(self):
        broken = dict(TRACES_TO)
        broken["target"] = "State"  # not in objects
        self.assert_invalid(
            envelope(schema_data(BASE_OBJECTS, [broken]))
        )

    def test_missing_objects_key_refused(self):
        data = schema_data(BASE_OBJECTS, base_morphisms())
        del data["objects"]
        self.assert_invalid(envelope(data))

    def test_missing_format_revision_refused(self):
        data = schema_data(BASE_OBJECTS, base_morphisms())
        del data["format_revision"]
        self.assert_invalid(envelope(data))

    def test_missing_data_key_refused(self):
        self.assert_invalid({"ok": True, "envelope_version": "0.1"})


# ---- states view fixtures (gre.6) --------------------------------------
#
# TWO_STATE_TSV mirrors the byte-pinned `spk graph --format edges` output
# over a two-state one-transition corpus (tests/cli/parse_misc.rs
# `edges_projection_pins_exact_tsv_bytes` — same shape, same sort order);
# the guard rows are duplicated to pin the D2 multiplicity rule: the raw
# TSV keeps both instances, the view draws one guard edge and counts


class TestSchemaCliRefusals(unittest.TestCase):
    """The CLI writes no diagram and exits non-zero with the named failure."""

    def test_invalid_export_cli_fails_before_output(self):
        proc = run_cli(envelope(schema_data(BASE_OBJECTS, base_morphisms()),
                                ok=False))
        self.assertNotEqual(proc.returncode, 0)
        self.assertIn("schema_export_invalid", proc.stderr)
        self.assertEqual(proc.stdout, "")

    def test_valid_export_cli_writes_diagram(self):
        proc = run_cli(envelope(schema_data(BASE_OBJECTS, base_morphisms())))
        self.assertEqual(proc.returncode, 0, proc.stderr)
        self.assertIn(REVISION, proc.stdout)
        self.assertIn("traces_to", proc.stdout)

    def test_unknown_version_cli_fails_before_output(self):
        proc = run_cli(envelope(schema_data(BASE_OBJECTS, base_morphisms(),
                                            version=2)))
        self.assertNotEqual(proc.returncode, 0)
        self.assertIn("schema_export_invalid", proc.stderr)
        self.assertEqual(proc.stdout, "")




if __name__ == "__main__":
    import unittest

    unittest.main()
