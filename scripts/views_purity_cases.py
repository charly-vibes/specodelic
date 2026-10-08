"""Prose-independence characterization tests (add-graph-views 2.7,
specodelic-gre.8) — `views_from_artifact_only`.

Perturbs the PROSE blocks of a checked-in fixture corpus (frontmatter
statement, prose under headings, trailing prose after the tables —
anything outside structured tables and ids), regenerates the graph
artifacts with the real `specodelic` binary, and asserts the states and
traceability view outputs are BYTE-IDENTICAL to the unperturbed renders.
The TSV bytes and the envelope data objects are asserted equal too, so a
failure names the guilty layer: a producer that lets prose leak into the
artifact, or (impossible by construction — the renderers only see the
artifacts) a view reading something else.

This pins existing correct behavior (characterization): the views derive
from the artifacts alone (design D1/D2) and prose must never reach them.
If this test FAILS, that is a real bug in the chain — stop and retag
DESIGN rather than adjusting a renderer to match prose.

Re-imported by test_graph_views.py. Uses `target/{debug,release}/
specodelic` relative to the repo root (the same binary `just ci`
builds); if neither exists the test fails loudly with a build hint —
never a silent skip.
"""

import json
import shutil
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))

import graph_views  # noqa: E402

REPO_ROOT = Path(__file__).resolve().parent.parent
FIXTURE = REPO_ROOT / "tests" / "fixtures" / "typing_violations"

# Prose perturbations — (file, old, new). Plain sentences only: no
# wikilinks, no bullets, no pipes, no tables — nothing the parser could
# read as structure. They touch the frontmatter statement, prose under a
# heading, and trailing prose after the last table.
PERTURBATIONS = (
    (
        "c1.md",
        "A corpus that triggers EVERY Reference Typing violation class "
        "named in specs/specodelic.md's Reference Typing table, plus a "
        "supersedes cycle.",
        "PERTURBED statement text that must never reach any view output.",
    ),
    (
        "c2.md",
        "The second intent file whose rows c1's forbidden edges point at.",
        "Second file, statement rewritten for the prose-independence check.",
    ),
)
APPENDED_PROSE = (
    "Trailing prose paragraph appended after all tables, deliberately "
    "plain: words, the number 42, and punctuation — none of it structural.",
)


def run_graph(corpus_dir, *args, expect=(0,)):
    """Run `spk graph <dir> <args>` over a corpus directory."""
    proc = subprocess.run(
        [str(spk_binary()), "graph", str(corpus_dir), *args],
        capture_output=True,
        text=True,
    )
    assert proc.returncode in expect, (
        f"spk graph {args} exited {proc.returncode}: {proc.stderr}"
    )
    return proc.stdout


def spk_binary():
    """The built specodelic binary (target/debug first, then release)."""
    for profile in ("debug", "release"):
        candidate = REPO_ROOT / "target" / profile / "specodelic"
        if candidate.exists():
            return candidate
    raise AssertionError(
        "no built specodelic binary under target/{debug,release} — run "
        "`just build` before running this suite"
    )


def artifacts_for(corpus_dir):
    """The artifact pair the views consume: TSV text + graph envelope.
    `--format edges` exits 0; `--json` over a violation-bearing corpus
    exits 1 (diagnostic: graphed-with-findings) while still emitting an
    ok:true envelope — only the envelope contract is asserted here."""
    tsv = run_graph(corpus_dir, "--format", "edges", expect=(0,))
    graph = json.loads(run_graph(corpus_dir, "--json", expect=(0, 1)))
    assert graph.get("ok") is True, "graph envelope must be ok:true"
    return tsv, graph


def render_all(tsv, graph):
    """Every artifact-driven view over one artifact pair."""
    return {
        "states": graph_views.render_states(tsv, graph),
        "trace": graph_views.render_traceability(tsv, graph),
    }


class ViewsFromArtifactOnly(unittest.TestCase):
    """Task 2.7: prose perturbation leaves every view byte-identical."""

    def test_views_from_artifact_only(self):
        with tempfile.TemporaryDirectory(prefix="views_purity_") as tmp:
            corpus = Path(tmp) / "corpus"
            corpus.mkdir()
            for f in sorted(FIXTURE.glob("*.md")):
                shutil.copy(f, corpus / f.name)
            originals = {p.name: p.read_text() for p in sorted(corpus.glob("*.md"))}

            tsv, graph = artifacts_for(corpus)
            before = render_all(tsv, graph)

            # Perturb the prose of the copied corpus — never the tables.
            for name, old, new in PERTURBATIONS:
                text = (corpus / name).read_text()
                assert old in text, f"perturbation anchor missing in {name}"
                (corpus / name).write_text(text.replace(old, new))
            for name in ("c1.md", "c2.md"):
                with open(corpus / name, "a") as f:
                    f.write("\n" + APPENDED_PROSE[0] + "\n")

            # The perturbation must actually change the files (no-op guard).
            perturbed = {p.name: p.read_text() for p in sorted(corpus.glob("*.md"))}
            self.assertNotEqual(originals, perturbed, "perturbation was a no-op")

            tsv2, graph2 = artifacts_for(corpus)
            after = render_all(tsv2, graph2)

            # Producer half: the artifacts themselves are prose-independent.
            self.assertEqual(tsv, tsv2, "edges TSV changed under prose perturbation")
            self.assertEqual(
                graph.get("data"),
                graph2.get("data"),
                "graph envelope data changed under prose perturbation",
            )
            # View half: byte-identical renders.
            self.assertEqual(before, after, "a view moved when only prose moved")

            # Not vacuously passing: the fixture exercises real structure —
            # transitions, fan-in and violation annotations all present.
            self.assertIn("subgraph", before["states"])
            self.assertIn("violation_1", before["states"])
            self.assertIn("(fan-in", before["trace"])


if __name__ == "__main__":
    import unittest

    unittest.main()
