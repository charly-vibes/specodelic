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
import home. The genuinely shared pieces (TSV contract, scope gate,
Mermaid primitives, fan-in, violation annotation) live in view_common.py
(specodelic-gre.8, task 2.7 tidy) — only the traceability view's own
semantics (owning-intent collapsing, dependency dedupe, refusal on
unattributable endpoints) remain here.

Usage: python3 scripts/graph_views.py trace <edges.tsv> --graph <graph.json> [--lint <lint.json>]
"""

from view_common import (
    ArtifactInvalid,
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
        src = owning_file(r["from"], intents)
        dst = owning_file(r["to"], intents)
        stray = src if src not in intents else dst if dst not in intents else None
        if stray is not None:
            raise ArtifactInvalid(
                f"edge endpoint {stray} collapses outside the intent set — "
                "the edges TSV and the graph envelope disagree"
            )
        if src != dst:
            pairs.add((src, dst))
    return pairs


def _violation_target_of(violations, intents, names):
    """Target-node mapper for the shared violation renderer: each
    violation collapses to its owning intent's node — refusing
    unattributable targets as artifact_invalid (D3)."""
    for r in sorted(violations, key=lambda r: (r["to"], r["annotation"])):
        target = owning_file(r["to"], intents)
        if target not in intents:
            raise ArtifactInvalid(
                f"violation target {target} collapses outside the intent set — "
                "the edges TSV and the graph envelope disagree"
            )
    return lambda r: names[owning_file(r["to"], intents)]


def render_traceability(edges_tsv_text, graph_payload, lint_payload=None):
    """Render the file-level traceability view (task 2.5): scope gate
    first (D7), then one deterministic Mermaid flowchart — one node per
    intent, deduped cross-file dependency edges, distinct-source fan-in
    labels, annotated violations. Raises OutOfScopeRefused or
    ArtifactInvalid before any rendering."""
    data = scope_gate(graph_payload, lint_payload)
    rows = parse_edges_tsv(edges_tsv_text)
    intents = declared_intents(data, rows)
    deps, violations = partition_violations(rows)
    pairs = _dependency_pairs(deps, intents)
    # fan-in per target over DISTINCT source intents — D2: the raw TSV
    # keeps duplicate instances; duplicate rows add 0 beyond a source's
    # first contribution.
    fan = fan_in(pairs, target=lambda p: p[1], source=lambda p: p[0])
    reserved = ["no_dependencies"] + [
        f"violation_{i}" for i in range(1, len(violations) + 1)
    ]
    names = mermaid_names(intents, reserved)
    lines = [
        "%% traceability view — derived from spk graph --format edges"
        " (edges collapsed to owning intents; fan-in over distinct source"
        " intents per D2)",
        "flowchart LR",
    ]
    for intent in sorted(intents):
        sources = fan.get(intent)
        label = f"{intent} (fan-in {len(sources)})" if sources else intent
        lines.append(f'  {names[intent]}["{mermaid_escape(label)}"]')
    for src, dst in sorted(pairs):
        lines.append(f"  {names[src]} --> {names[dst]}")
    if not pairs:
        lines.append(no_output_note("no_dependencies", NO_DEPENDENCIES_NOTE))
    target_of = _violation_target_of(violations, intents, names)
    lines.extend(violation_lines(violations, target_of))
    if violations or not pairs:
        lines.append(VIOLATION_CLASSDEF)
    return "\n".join(lines) + "\n"