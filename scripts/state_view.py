"""State view renderer — scripts/state_view.py (add-graph-views 1.7/2.3/2.4).

Purpose: render per-file state-machine diagrams from the six-column
`spk graph --format edges` TSV plus the `spk graph --json` envelope
(optionally the `spk lint --json` envelope for the scope gate's lint
leg) — the graph-artifact-driven view of the transform prototype.

Responsibilities: enforce the corpus scope gate first (`out_of_scope_refused`:
an intentless corpus — zero spec files — or a lint-dirty corpus with
invariant findings is refused with a remediation hint, design D7) and
refuse malformed artifacts (`artifact_invalid`), then emit a deterministic
Mermaid flowchart per owning file: states, transitions and guard targets
stay distinct (task 1.7 — only file-level views collapse), guard edges
render dashed (D8), every violation row renders as an annotated
red-dashed element (D3: never silently clean), fan-in counts DISTINCT
sources (D2's multiplicity rule), and a corpus with no transition edges
renders the labeled `no_transitions` view.

Rationale: the view derives from the artifacts alone (design D1) — it
never re-walks corpus markdown and embeds no reference-typing table.
Split out of graph_views.py to honor the script-role structural ratchet;
graph_views.py re-exports the entry points so the CLI surface is
unchanged.

Usage: python3 scripts/graph_views.py states <edges.tsv> --graph <graph.json> [--lint <lint.json>]
"""

from view_common import (
    ARTIFACT_INVALID,
    OUT_OF_SCOPE_REFUSED,
    ArtifactInvalid,
    OutOfScopeRefused,
)

# The state view's edge scope (add-graph-views spec: derived from
# `transitions.from`, `transitions.to` and `transitions.guard` edges alone).
TRANSITION_FIELDS = ("transitions.from", "transitions.to", "transitions.guard")
NO_TRANSITIONS_NOTE = (
    "no file in the artifact carries transitions.from/to/guard edges — "
    "state machines are derived from the edge list alone"
)


def _mermaid_escape(text):
    """Label escaping — mirrors src/graph.rs's mermaid_escape."""
    return text.replace('"', "&quot;")


def _mermaid_names(ids, reserved=()):
    """Injective [A-Za-z0-9_] names for ids carrying dots — mirrors
    src/graph.rs's mermaid_node_ids (`_2`, `_3` suffixes on collision)."""
    names = {}
    used = set(reserved)
    for id_ in sorted(ids):
        base = "".join(c if c.isascii() and (c.isalnum() or c == "_") else "_" for c in id_)
        if not base:
            base = "n"
        name, n = base, 2
        while name in used:
            name = f"{base}_{n}"
            n += 1
        used.add(name)
        names[id_] = name
    return names


def _failure_message(payload):
    """Best-effort message from a refusal envelope — genesis rides the
    reason on warnings/hints, not an error key."""
    for item in payload.get("warnings") or []:
        if isinstance(item, dict) and item.get("message"):
            return item["message"]
    return "unsuccessful envelope"


def _envelope_data(payload):
    """A successful genesis envelope carrying a data object, or refusal:
    an unsuccessful envelope means the producing command refused the
    corpus (the intentless leg rides here — `spk graph --json` over a
    zero-file directory is ok:false); an envelope without data is an
    artifact-contract breach."""
    if not isinstance(payload, dict):
        raise ArtifactInvalid("graph envelope is not a JSON object")
    if payload.get("ok") is not True:
        raise OutOfScopeRefused(
            f"the graph envelope refuses the corpus: {_failure_message(payload)}"
        )
    data = payload.get("data")
    if not isinstance(data, dict):
        raise ArtifactInvalid("graph envelope carries no data object")
    return data


def _lint_leg_refusal(lint_payload):
    """The scope gate's lint leg (design D7): an unverifiable or
    lint-dirty corpus is refused, naming the failed gate by rule id."""
    if not isinstance(lint_payload, dict) or lint_payload.get("ok") is not True:
        message = (
            _failure_message(lint_payload)
            if isinstance(lint_payload, dict)
            else "not a JSON object"
        )
        raise OutOfScopeRefused(
            f"the lint envelope cannot verify the scope gate: {message}"
        )
    lint_data = lint_payload.get("data")
    if not isinstance(lint_data, dict) or not isinstance(lint_data.get("issues"), list):
        raise ArtifactInvalid("lint envelope carries no issues list")
    if lint_data["issues"]:
        rules = sorted(
            i.get("rule_id", "?") for i in lint_data["issues"] if isinstance(i, dict)
        )
        raise OutOfScopeRefused(
            "lint-dirty corpus — invariant findings: " + ", ".join(rules)
        )


def _scope_gate(graph_payload, lint_payload):
    """Corpus scope before any output (design D7, task 2.3): ≥1 intent
    file (the graph envelope's files count — an intentless corpus is
    refused) and no invariant lint findings when the lint envelope is
    supplied. Returns the envelope data, whose sorted `intents` list is
    the owning-file collapse key."""
    data = _envelope_data(graph_payload)
    files = data.get("files")
    if not isinstance(files, int) or isinstance(files, bool):
        raise ArtifactInvalid("graph envelope data carries no files count")
    if files < 1:
        raise OutOfScopeRefused(
            f"intentless corpus — {files} spec files parse, nothing to view"
        )
    intents = data.get("intents")
    if not isinstance(intents, list) or not all(
        isinstance(i, str) and i for i in intents
    ):
        raise ArtifactInvalid("graph envelope data carries no intents list")
    if lint_payload is not None:
        _lint_leg_refusal(lint_payload)
    return data


def parse_edges_tsv(text):
    """The six-column TSV contract (task 1.2): source_id, source_kind,
    field, target_id, target_kind, annotation. Every row carries the
    trailing empty annotation column."""
    rows = []
    for lineno, line in enumerate(text.splitlines(), start=1):
        if not line:
            continue
        cols = line.split("\t")
        if len(cols) != 6:
            raise ArtifactInvalid(
                f"edges TSV line {lineno} has {len(cols)} columns, expected 6"
            )
        rows.append(
            {
                "from": cols[0],
                "from_kind": cols[1],
                "field": cols[2],
                "to": cols[3],
                "to_kind": cols[4],
                "annotation": cols[5],
            }
        )
    return rows


def _owning_file(id_, intents):
    """Owning file of a canonical node id — mirrors src/graph.rs's
    owning_file: an intent id is its own file; a qualified row id's file
    prefix is everything before the last dot when that prefix is a known
    intent. An unattributable id falls back to itself (never silently
    dropped, D3)."""
    if id_ in intents:
        return id_
    file, _, _row = id_.rpartition(".")
    if file and file in intents:
        return file
    return id_


def _fan_in(transitions):
    """fan-in per target over DISTINCT (source, field) pairs — D2: the
    raw TSV keeps duplicate instances; the view-layer count does not."""
    fan = {}
    for r in transitions:
        fan.setdefault(r["to"], set()).add((r["from"], r["field"]))
    return fan


def _machine_lines(file, rows, ctx):
    """One owning file's subgraph: nodes declared (states as stadium
    nodes with fan-in), then the file's deduped edges — guards dashed,
    from/to solid (D8). A cross-file guard target stays outside the
    subgraph; Mermaid draws the edge to it either way."""
    names, fan_in = ctx["names"], ctx["fan_in"]
    file_nodes = set()
    for r in rows:
        file_nodes.update((r["from"], r["to"]))
    lines = [f'  subgraph {ctx["subs"][f"sg_{file}"]}["{file}"]', "    direction LR"]
    for id_ in sorted(file_nodes):
        kind = next(
            (r["to_kind"] for r in rows if r["to"] == id_),
            next((r["from_kind"] for r in rows if r["from"] == id_), ""),
        )
        n = fan_in.get(id_, set())
        label = f"{id_} (fan-in {len(n)})" if n else id_
        if kind == "State":
            lines.append(f'    {names[id_]}(["{_mermaid_escape(label)}"])')
        else:
            lines.append(f'    {names[id_]}["{_mermaid_escape(label)}"]')
    # Duplicate TSV instances draw once (the raw projection's multiplicity
    # contract is the TSV's, not the drawing's).
    drawn = {(r["from"], r["field"], r["to"]) for r in rows}
    for (src, field, dst) in sorted(drawn):
        arrow = "-.->" if field == "transitions.guard" else "-->"
        lines.append(f'    {names[src]} {arrow}|"{field}"| {names[dst]}')
    lines.append("  end")
    return lines


def _violation_lines(violations, names):
    """Violations as annotated red-dashed elements: the TSV drops the
    forbidden edge's source (empty source columns, task 1.2), so each
    violation renders as an annotated node dashed into its target —
    annotated, never omitted (D3)."""
    lines = []
    for i, r in enumerate(sorted(violations, key=lambda r: (r["to"], r["annotation"])), 1):
        label = f"{r['field']}: {r['annotation']}".strip(": ")
        field, target = r["field"], names[r["to"]]
        lines.append(
            f'  violation_{i}(["{_mermaid_escape(label)}"]):::violation '
            f'-.->|"{field}"| {target}'
        )
    return lines


def render_states(edges_tsv_text, graph_payload, lint_payload=None):
    """Render the per-file state-machine view (tasks 1.7/2.4): scope gate
    first (D7), then one Mermaid flowchart grouping transition edges by
    owning file. Raises OutOfScopeRefused or ArtifactInvalid before any
    rendering."""
    _scope_gate(graph_payload, lint_payload)
    rows = parse_edges_tsv(edges_tsv_text)
    # The declared intent set (graph JSON) plus any Intent-kind endpoints
    # the TSV shows — the JSON is authoritative for owning-file grouping
    # (an intent without typed-reference endpoints never appears in the
    # TSV; dotted intent ids make prefix splitting ambiguous without it).
    intents = set(graph_payload.get("data", {}).get("intents", []))
    intents |= {r["from"] for r in rows if r["from_kind"] == "Intent"}
    intents |= {r["to"] for r in rows if r["to_kind"] == "Intent"}
    transitions = [r for r in rows if r["field"] in TRANSITION_FIELDS]
    violations = [r for r in rows if r["field"].startswith("violation:")]
    if not transitions and not violations:
        return (
            "%% state view — derived from spk graph --format edges (no transitions)\n"
            "flowchart LR\n"
            f'  no_transitions["no_transitions: {NO_TRANSITIONS_NOTE}"]:::violation\n'
            "  classDef violation stroke:red,stroke-dasharray:5 5\n"
        )
    groups = {}
    for r in transitions:
        groups.setdefault(_owning_file(r["from"], intents), []).append(r)
    node_ids = set()
    for r in transitions:
        node_ids.update((r["from"], r["to"]))
    node_ids.update(r["to"] for r in violations)
    reserved = ["no_transitions"] + [f"violation_{i}" for i in range(1, len(violations) + 1)]
    ctx = {
        "names": _mermaid_names(node_ids, reserved),
        "fan_in": _fan_in(transitions),
        "subs": _mermaid_names({f"sg_{file}" for file in groups}, reserved=reserved),
    }
    lines = [
        "%% state view — derived from spk graph --format edges"
        " (transitions grouped by owning file; guards dashed per D8)",
        "flowchart LR",
    ]
    for file in sorted(groups):
        lines.extend(_machine_lines(file, groups[file], ctx))
    lines.extend(_violation_lines(violations, ctx["names"]))
    if violations:
        lines.append("  classDef violation stroke:red,stroke-dasharray:5 5")
    return "\n".join(lines) + "\n"
