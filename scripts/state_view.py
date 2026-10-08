"""State view renderer — scripts/state_view.py (add-graph-views 1.7/2.3/2.4).

Purpose: render per-file state-machine diagrams from the six-column
`spk graph --format edges` TSV plus the `spk graph --json` envelope
(optionally the `spk lint --json` envelope for the scope gate's lint
leg) — the graph-artifact-driven view of the transform prototype.

Responsibilities: run the shared corpus scope gate first
(`out_of_scope_refused`: an intentless corpus — zero spec files — or a
lint-dirty corpus with invariant findings is refused with a remediation
hint, design D7) and refuse malformed artifacts (`artifact_invalid`),
then emit a deterministic Mermaid flowchart per owning file: states,
transitions and guard targets stay distinct (task 1.7 — only file-level
views collapse), guard edges render dashed (D8), every violation row
renders as an annotated red-dashed element (D3: never silently clean),
fan-in counts DISTINCT sources (D2's multiplicity rule), and a corpus
with no transition edges renders the labeled `no_transitions` view.

Rationale: the view derives from the artifacts alone (design D1) — it
never re-walks corpus markdown and embeds no reference-typing table.
Split out of graph_views.py to honor the script-role structural ratchet;
graph_views.py re-exports the entry points so the CLI surface is
unchanged. The genuinely shared pieces (TSV contract, scope gate,
Mermaid primitives, fan-in, violation annotation) live in view_common.py
(specodelic-gre.8, task 2.7 tidy) — only the state view's own semantics
(subgraph grouping, state-node grammar, transition scoping) remain here.

Usage: python3 scripts/graph_views.py states <edges.tsv> --graph <graph.json> [--lint <lint.json>]
"""

from view_common import (  # noqa: F401 — re-exported render surface
    ARTIFACT_INVALID,
    OUT_OF_SCOPE_REFUSED,
    ArtifactInvalid,
    OutOfScopeRefused,
    VIOLATION_CLASSDEF,
    declared_intents,
    fan_in,
    mermaid_escape,
    mermaid_names,
    no_output_note,
    owning_file,
    parse_edges_tsv,
    partition_violations,
    scope_gate,
    violation_lines,
)

# The state view's edge scope (add-graph-views spec: derived from
# `transitions.from`, `transitions.to` and `transitions.guard` edges alone).
TRANSITION_FIELDS = ("transitions.from", "transitions.to", "transitions.guard")
NO_TRANSITIONS_NOTE = (
    "no file in the artifact carries transitions.from/to/guard edges — "
    "state machines are derived from the edge list alone"
)


def _machine_lines(file, rows, ctx):
    """One owning file's subgraph: nodes declared (states as stadium
    nodes with fan-in), then the file's deduped edges — guards dashed,
    from/to solid (D8). A cross-file guard target stays outside the
    subgraph; Mermaid draws the edge to it either way."""
    names, fan_in_map = ctx["names"], ctx["fan_in"]
    file_nodes = set()
    for r in rows:
        file_nodes.update((r["from"], r["to"]))
    lines = [f'  subgraph {ctx["subs"][f"sg_{file}"]}["{file}"]', "    direction LR"]
    for id_ in sorted(file_nodes):
        kind = next(
            (r["to_kind"] for r in rows if r["to"] == id_),
            next((r["from_kind"] for r in rows if r["from"] == id_), ""),
        )
        n = fan_in_map.get(id_, set())
        label = f"{id_} (fan-in {len(n)})" if n else id_
        if kind == "State":
            lines.append(f'    {names[id_]}(["{mermaid_escape(label)}"])')
        else:
            lines.append(f'    {names[id_]}["{mermaid_escape(label)}"]')
    # Duplicate TSV instances draw once (the raw projection's multiplicity
    # contract is the TSV's, not the drawing's).
    drawn = {(r["from"], r["field"], r["to"]) for r in rows}
    for (src, field, dst) in sorted(drawn):
        arrow = "-.->" if field == "transitions.guard" else "-->"
        lines.append(f'    {names[src]} {arrow}|"{field}"| {names[dst]}')
    lines.append("  end")
    return lines


def render_states(edges_tsv_text, graph_payload, lint_payload=None):
    """Render the per-file state-machine view (tasks 1.7/2.4): scope gate
    first (D7), then one Mermaid flowchart grouping transition edges by
    owning file. Raises OutOfScopeRefused or ArtifactInvalid before any
    rendering."""
    data = scope_gate(graph_payload, lint_payload)
    rows = parse_edges_tsv(edges_tsv_text)
    intents = declared_intents(data, rows)
    transitions, violations = partition_violations(rows)
    transitions = [r for r in transitions if r["field"] in TRANSITION_FIELDS]
    if not transitions and not violations:
        return (
            "%% state view — derived from spk graph --format edges"
            " (no transitions)\n"
            "flowchart LR\n"
            + no_output_note("no_transitions", NO_TRANSITIONS_NOTE)
            + "\n"
            + VIOLATION_CLASSDEF
            + "\n"
        )
    groups = {}
    for r in transitions:
        groups.setdefault(owning_file(r["from"], intents), []).append(r)
    node_ids = set()
    for r in transitions:
        node_ids.update((r["from"], r["to"]))
    node_ids.update(r["to"] for r in violations)
    reserved = ["no_transitions"] + [f"violation_{i}" for i in range(1, len(violations) + 1)]
    ctx = {
        "names": mermaid_names(node_ids, reserved),
        # fan-in per target over DISTINCT (source, field) pairs — D2: the
        # raw TSV keeps duplicate instances; the view-layer count does not.
        "fan_in": fan_in(
            transitions,
            target=lambda r: r["to"],
            source=lambda r: (r["from"], r["field"]),
        ),
        "subs": mermaid_names({f"sg_{file}" for file in groups}, reserved=reserved),
    }
    lines = [
        "%% state view — derived from spk graph --format edges"
        " (transitions grouped by owning file; guards dashed per D8)",
        "flowchart LR",
    ]
    for file in sorted(groups):
        lines.extend(_machine_lines(file, groups[file], ctx))
    lines.extend(violation_lines(violations, target_of=lambda r: ctx["names"][r["to"]]))
    if violations:
        lines.append(VIOLATION_CLASSDEF)
    return "\n".join(lines) + "\n"