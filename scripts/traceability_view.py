"""File-level traceability renderer — scripts/traceability_view.py
(add-graph-views task 2.5, gre.7).

Purpose: render the corpus-level traceability view — one node per intent,
file dependencies as edges, fan-in annotated per intent — from the
six-column `spk graph --format edges` TSV plus the `spk graph --json`
envelope (optionally the `spk lint --json` envelope for the scope gate).

Responsibilities: run the shared scope gate first (same
`out_of_scope_refused` legs as the states view — intentless or lint-dirty
corpora, design D7), collapse every non-violation edge's endpoints to
their owning intents (the graph envelope's intent set is authoritative;
unattributable endpoints are `artifact_invalid`, never silently dropped),
dedupe to distinct cross-file (source, target) pairs — the file
dependencies (intra-file edges collapse to self and are not file
dependencies) — and annotate each intent with fan-in counted over
DISTINCT source intents (design D2's multiplicity rule: the raw TSV keeps
duplicate instances; duplicate rows add 0 beyond a source's first
contribution). Violation rows stay annotated as red-dashed elements
dashed into their collapsed intent target (D3: never silently clean; they
are not recorded edges and contribute nothing to fan-in). A corpus with
no cross-file dependencies renders the labeled `no_dependencies` note.
Deterministic: sorted everywhere.

Rationale: the view derives from the artifacts alone (design D1) — it
never re-walks corpus markdown and embeds no reference-typing table.
Split out of graph_views.py to honor the script-role structural ratchet;
graph_views.py re-exports the entry point so the CLI surface stays one
import home.

Usage: python3 scripts/graph_views.py trace <edges.tsv> --graph <graph.json> [--lint <lint.json>]
"""

from state_view import (
    _mermaid_escape,
    _mermaid_names,
    _owning_file,
    _scope_gate,
    parse_edges_tsv,
)
from view_common import ArtifactInvalid

NO_DEPENDENCIES_NOTE = (
    "no cross-file dependency edges in the artifact — file dependencies "
    "are derived from the edge list alone"
)


def _dependency_pairs(deps, intents):
    """Collapse non-violation rows to distinct cross-file (source, target)
    intent pairs. An endpoint collapsing outside the intent set means the
    TSV and the graph envelope disagree — refused as artifact_invalid
    (D3: never silently dropped)."""
    pairs = set()
    for r in deps:
        src = _owning_file(r["from"], intents)
        dst = _owning_file(r["to"], intents)
        stray = src if src not in intents else dst if dst not in intents else None
        if stray is not None:
            raise ArtifactInvalid(
                f"edge endpoint {stray} collapses outside the intent set — "
                "the edges TSV and the graph envelope disagree"
            )
        if src != dst:
            pairs.add((src, dst))
    return pairs


def _fan_in(pairs):
    """fan-in per target over DISTINCT source intents — D2: the raw TSV
    keeps duplicate instances; duplicate rows add 0 beyond a source's
    first contribution."""
    fan = {}
    for src, dst in pairs:
        fan.setdefault(dst, set()).add(src)
    return fan


def _violation_lines(violations, intents, names):
    """Violations as annotated red-dashed elements dashed into their
    collapsed intent target — annotated, never omitted (D3), and not
    recorded edges (they contribute nothing to fan-in)."""
    lines = []
    for i, r in enumerate(sorted(violations, key=lambda r: (r["to"], r["annotation"])), 1):
        target = _owning_file(r["to"], intents)
        if target not in intents:
            raise ArtifactInvalid(
                f"violation target {target} collapses outside the intent set — "
                "the edges TSV and the graph envelope disagree"
            )
        label = f"{r['field']}: {r['annotation']}".strip(": ")
        lines.append(
            f'  violation_{i}(["{_mermaid_escape(label)}"]):::violation'
            f' -.->|"{r["field"]}"| {names[target]}'
        )
    return lines


def render_traceability(edges_tsv_text, graph_payload, lint_payload=None):
    """Render the file-level traceability view (task 2.5): scope gate
    first (D7), then one deterministic Mermaid flowchart — one node per
    intent, deduped cross-file dependency edges, distinct-source fan-in
    labels, annotated violations. Raises OutOfScopeRefused or
    ArtifactInvalid before any rendering."""
    data = _scope_gate(graph_payload, lint_payload)
    rows = parse_edges_tsv(edges_tsv_text)
    intents = set(data["intents"])
    intents |= {r["from"] for r in rows if r["from_kind"] == "Intent"}
    intents |= {r["to"] for r in rows if r["to_kind"] == "Intent"}
    deps = [r for r in rows if not r["field"].startswith("violation:")]
    violations = [r for r in rows if r["field"].startswith("violation:")]
    pairs = _dependency_pairs(deps, intents)
    fan_in = _fan_in(pairs)
    reserved = ["no_dependencies"] + [
        f"violation_{i}" for i in range(1, len(violations) + 1)
    ]
    names = _mermaid_names(intents, reserved)
    lines = [
        "%% traceability view — derived from spk graph --format edges"
        " (edges collapsed to owning intents; fan-in over distinct source"
        " intents per D2)",
        "flowchart LR",
    ]
    for intent in sorted(intents):
        sources = fan_in.get(intent)
        label = f"{intent} (fan-in {len(sources)})" if sources else intent
        lines.append(f'  {names[intent]}["{_mermaid_escape(label)}"]')
    for src, dst in sorted(pairs):
        lines.append(f"  {names[src]} --> {names[dst]}")
    if not pairs:
        lines.append(
            f'  no_dependencies["no_dependencies: {NO_DEPENDENCIES_NOTE}"]:::violation'
        )
    lines.extend(_violation_lines(violations, intents, names))
    if violations or not pairs:
        lines.append("  classDef violation stroke:red,stroke-dasharray:5 5")
    return "\n".join(lines) + "\n"
