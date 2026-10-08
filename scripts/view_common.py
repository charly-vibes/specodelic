"""Shared rendering helpers for the graph view renderers
(scripts/graph_views.py's views — add-graph-views D1/D2/D3/D4/D7/D8).

Purpose: one home for the pieces the artifact-driven views genuinely
share — the labeled failure classes, the six-column TSV contract, the
corpus scope gate (D7), canonical-id owning-file collapse (D2), Mermaid
name/label escaping, distinct-key fan-in (D2's multiplicity rule),
violation annotation rendering (D3) and the labeled no-output note — so
the state and traceability views differ only in their actual semantics.

Responsibilities: define `OutOfScopeRefused` and `ArtifactInvalid` with
their stderr failure codes; parse and validate the artifacts' shapes;
provide the deterministic building blocks the per-view modules compose.
Nothing here knows any individual view's diagram grammar (subgraphs,
per-file grouping, dependency collapsing stay in their view modules).

Rationale: the views fail loudly with named classes BEFORE any output is
written (D3/D7) and derive from the artifacts alone (D1); a shared
module keeps those contracts from drifting while each view module stays
under the script-role file-size ratchet. Extracted from state_view.py /
traceability_view.py (specodelic-gre.8, task 2.7 tidy) without changing
any view's rendered bytes.
"""

# --- labeled failure classes (D3/D7: refuse before any output) ---

OUT_OF_SCOPE_REFUSED = "out_of_scope_refused"
ARTIFACT_INVALID = "artifact_invalid"


class OutOfScopeRefused(Exception):
    """The corpus is out of scope for the views (design D7): intentless
    (zero spec files) or lint-dirty (invariant lint findings)."""


class ArtifactInvalid(Exception):
    """A graph artifact does not match its contract (six-column TSV,
    successful envelope) — refused before any output, like the schema
    view refuses malformed exports."""


# --- artifact contract (task 1.2's six-column TSV; D7's scope gate) ---

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


def _failure_message(payload):
    """Best-effort message from a refusal envelope — genesis rides the
    reason on warnings/hints, not an error key."""
    for item in payload.get("warnings") or []:
        if isinstance(item, dict) and item.get("message"):
            return item["message"]
    return "unsuccessful envelope"


def envelope_data(payload):
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


def scope_gate(graph_payload, lint_payload):
    """Corpus scope before any output (design D7, task 2.3): ≥1 intent
    file (the graph envelope's files count — an intentless corpus is
    refused) and no invariant lint findings when the lint envelope is
    supplied. Returns the envelope data, whose sorted `intents` list is
    the owning-file collapse key."""
    data = envelope_data(graph_payload)
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


def owning_file(id_, intents):
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


def declared_intents(data, rows):
    """The declared intent set (graph JSON) plus any Intent-kind
    endpoints the TSV shows — the JSON is authoritative for owning-file
    grouping (an intent without typed-reference endpoints never appears
    in the TSV; dotted intent ids make prefix splitting ambiguous without
    it)."""
    intents = set(data["intents"])
    intents |= {r["from"] for r in rows if r["from_kind"] == "Intent"}
    intents |= {r["to"] for r in rows if r["to_kind"] == "Intent"}
    return intents


def partition_violations(rows):
    """Split rows into (ordinary rows, violation rows) — violation rows
    carry the `violation:` field prefix (task 1.2's annotation mapping)."""
    violations = [r for r in rows if r["field"].startswith("violation:")]
    ordinary = [r for r in rows if not r["field"].startswith("violation:")]
    return ordinary, violations


# --- Mermaid rendering primitives (D8: deterministic text output) ---

def mermaid_escape(text):
    """Label escaping — mirrors src/graph.rs's mermaid_escape: labels
    embed arbitrary spec-cell text, so everything that can break out of
    the quoted-label context rides as an HTML entity — the double quote
    (string terminator), & (entity smuggling), < and > (HTML tags and
    the -->/-.->/==> arrow grammar), # (mermaid's #nn; entity codes) and
    % (the %% comment syntax) — and newlines flatten to spaces so label
    text can never start a diagram line (a line starting `end` closes
    the enclosing subgraph, `%%` opens a comment). Already-safe text
    stays byte-identical (D8: deterministic text output)."""
    escaped = (
        text.replace("&", "&amp;")
        .replace('"', "&quot;")
        .replace("<", "&lt;")
        .replace(">", "&gt;")
        .replace("#", "&num;")
        .replace("%", "&percnt;")
    )
    return escaped.replace("\r", " ").replace("\n", " ")


def mermaid_names(ids, reserved=()):
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


def fan_in(rows, target, source):
    """fan-in per target over DISTINCT source keys — D2: the raw TSV
    keeps duplicate instances; the view-layer count does not. `target`
    and `source` map a row to its target node and its counted key."""
    fan = {}
    for r in rows:
        fan.setdefault(target(r), set()).add(source(r))
    return fan


VIOLATION_CLASSDEF = "  classDef violation stroke:red,stroke-dasharray:5 5"


def violation_lines(violations, target_of):
    """Violations as annotated red-dashed elements dashed into their
    target node — annotated, never omitted (D3). `target_of` maps a
    violation row to its target node name (raw id for the state view,
    owning-intent name for the traceability view, which additionally
    refuses unattributable targets)."""
    lines = []
    for i, r in enumerate(sorted(violations, key=lambda r: (r["to"], r["annotation"])), 1):
        label = f"{r['field']}: {r['annotation']}".strip(": ")
        lines.append(
            f'  violation_{i}(["{mermaid_escape(label)}"]):::violation '
            f'-.->|"{r["field"]}"| {target_of(r)}'
        )
    return lines


def no_output_note(note_name, note_text):
    """The labeled no-output view line (D3: never silently clean — a
    corpus with nothing to render shows why, as a violation-classed
    node)."""
    return f'  {note_name}["{note_name}: {note_text}"]:::violation'