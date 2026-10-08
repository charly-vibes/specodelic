"""Graph view renderers — scripts/graph_views.py (add-graph-views D1/D4/D8).

Purpose: the single CLI surface for the derived-view prototype: the
revision-labeled schema diagram (schema_view), the per-file state-machine
diagrams (state_view), and the file-level traceability view (gre.7, to
come) — all rendered from graph artifacts alone.

Responsibilities: own the subcommand CLI (artifact reading, usage
handling, labeled refusals with remediation hints) and re-export the
pure renderers and failure classes so consumers (and the test suite)
keep one import surface. Every view validates its inputs and refuses
out-of-scope corpora or malformed artifacts BEFORE any output is written
(D3/D7).

Rationale: the views derive from the artifacts alone (design D1/D4) —
they never parse Rust source, never re-walk corpus markdown, and embed no
second reference-typing table. Prototype lives in scripts/ per D1;
promotion out of scripts/ is deliberately deferred. The pure renderers
are split out to honor the script-role structural ratchet; this module
keeps the documented CLI stable.

Usage: python3 scripts/graph_views.py schema <export.json>  (Mermaid to stdout)
       python3 scripts/graph_views.py states <edges.tsv> --graph <graph.json> [--lint <lint.json>]
"""

import json
import sys
from pathlib import Path

from schema_view import (  # noqa: F401 — re-exported render surface
    SCHEMA_EXPORT_INVALID,
    SchemaExportInvalid,
    render_schema,
)
from state_view import (  # noqa: F401 — re-exported render surface
    ARTIFACT_INVALID,
    OUT_OF_SCOPE_REFUSED,
    ArtifactInvalid,
    OutOfScopeRefused,
    render_states,
)
ARTIFACT_HINT = (
    "regenerate the artifact with a matching tool version: "
    "specodelic graph <corpus> --format edges"
)
INTENTLESS_HINT = (
    "the corpus has zero spec files — graph views require at least one "
    "intent file; regenerate the artifacts with: specodelic graph <corpus> --format edges"
)
LINT_HINT = (
    "fix the reported invariants (each rule: specodelic explain lint-rules), "
    "then regenerate the artifacts with: specodelic graph <corpus> --format edges"
)
USAGE = (
    "usage: graph_views.py schema <export.json> | "
    "states <edges.tsv> --graph <graph.json> [--lint <lint.json>]"
)


def _read_json(path):
    return json.loads(Path(path).read_text())


def schema_cli(rest):
    """`schema <export.json>` — read, validate, render to stdout; refusals
    print `schema_export_invalid` to stderr, exit 1; usage errors exit 2.
    Output is written only after validation (D4)."""
    if len(rest) != 1:
        print(f"usage: graph_views.py schema <export.json>", file=sys.stderr)
        return 2
    try:
        diagram = render_schema(_read_json(rest[0]))
    except (
        SchemaExportInvalid,
        json.JSONDecodeError,
        UnicodeDecodeError,
        OSError,
    ) as error:
        print(f"{SCHEMA_EXPORT_INVALID}: {error}", file=sys.stderr)
        return 1
    sys.stdout.write(diagram)
    return 0


def states_read_args(rest):
    """Parse `states <edges.tsv> --graph <graph.json> [--lint <lint.json>]`
    — returns (tsv_path, graph_path, lint_path) or None on usage error."""
    tsv_path = graph_path = lint_path = None
    i = 0
    while i < len(rest):
        if rest[i] == "--graph" and i + 1 < len(rest):
            graph_path = rest[i + 1]
            i += 2
        elif rest[i] == "--lint" and i + 1 < len(rest):
            lint_path = rest[i + 1]
            i += 2
        elif tsv_path is None and not rest[i].startswith("--"):
            tsv_path = rest[i]
            i += 1
        else:
            return None
    if tsv_path is None or graph_path is None:
        return None
    return tsv_path, graph_path, lint_path


def states_cli(rest):
    """`states <edges.tsv> --graph <graph.json> [--lint <lint.json>]` —
    read the artifacts, run the scope gate, render to stdout. Refusals
    print the labeled failure with a remediation hint to stderr and exit
    1; usage errors exit 2. Output is written only after validation
    (D7/D3: fail before output writes)."""
    parsed = states_read_args(rest)
    if parsed is None:
        print(USAGE, file=sys.stderr)
        return 2
    tsv_path, graph_path, lint_path = parsed
    try:
        tsv = Path(tsv_path).read_text()
        graph_payload = _read_json(graph_path)
        lint_payload = _read_json(lint_path) if lint_path else None
        diagram = render_states(tsv, graph_payload, lint_payload)
    except (
        OutOfScopeRefused,
        ArtifactInvalid,
        json.JSONDecodeError,
        UnicodeDecodeError,
        OSError,
    ) as error:
        refused = isinstance(error, OutOfScopeRefused)
        failure = OUT_OF_SCOPE_REFUSED if refused else ARTIFACT_INVALID
        hint = LINT_HINT if "lint" in str(error) else (
            INTENTLESS_HINT if refused else ARTIFACT_HINT
        )
        print(f"{failure}: {error} — {hint}", file=sys.stderr)
        return 1
    sys.stdout.write(diagram)
    return 0


def main(argv):
    if argv and argv[0] == "states":
        return states_cli(argv[1:])
    if len(argv) == 2 and argv[0] == "schema":
        return schema_cli(argv[1:])
    print(USAGE, file=sys.stderr)
    return 2


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))