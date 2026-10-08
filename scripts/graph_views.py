"""Schema view renderer — scripts/graph_views.py (add-graph-views D1/D4).

Purpose: render the revision-labeled schema diagram from the versioned
`spk guide --schema --json` export — the only structural input this
script accepts for the schema view.

Responsibilities: validate the exported envelope (unsuccessful envelopes,
missing fields, unknown schema versions, duplicate object/morphism
identities and references to absent objects fail as `schema_export_invalid`
with a regeneration hint BEFORE any diagram is written), then emit a
deterministic Mermaid flowchart naming the format revision, one node per
schema object and one typed edge per morphism with refinement labels.

Rationale: the view derives from the lint-gated acset Schema value via the
export alone (design D4) — it never parses Rust source, never reconstructs
schema structure from corpus edges, and embeds no second reference-typing
table. Prototype lives in scripts/ per D1; promotion out of scripts/ is
deliberately deferred.

Usage: python3 scripts/graph_views.py schema <export.json>  (Mermaid to stdout)
"""

import json
import sys
from pathlib import Path

SUPPORTED_SCHEMA_VERSION = 1
SCHEMA_EXPORT_INVALID = "schema_export_invalid"
REGEN_HINT = (
    "regenerate the export with a matching tool version: "
    "specodelic guide --schema --json"
)
MORPHISM_FIELDS = (
    "name",
    "column",
    "source",
    "target",
    "refinements",
    "source_rule",
    "endo_acyclic",
)
IDENTITY_FIELDS = ("name", "column", "source", "target")
SIDES = ("source", "target")


class SchemaExportInvalid(Exception):
    """The schema export is not a valid versioned envelope (D4 refusals)."""


def _fail(reason):
    raise SchemaExportInvalid(f"{reason} — {REGEN_HINT}")


def _validated_envelope(payload):
    """Reject anything but a successful genesis envelope carrying data."""
    if not isinstance(payload, dict):
        _fail("schema export is not a JSON object")
    if payload.get("ok") is not True:
        _fail("schema export envelope is unsuccessful")
    data = payload.get("data")
    if not isinstance(data, dict):
        _fail("schema export envelope carries no data object")
    return data


def _validated_refinements(refinements):
    if not isinstance(refinements, list):
        _fail("morphism refinements must be a list")
    for r in refinements:
        if not isinstance(r, dict) or "side" not in r or "kind" not in r:
            _fail("refinement must carry side and kind")
        if r["side"] not in SIDES:
            _fail(f"refinement side must be one of {SIDES}, got {r['side']!r}")
        if not isinstance(r["kind"], str) or not r["kind"]:
            _fail("refinement kind must be a non-empty string")


def _validated_morphisms(morphisms, objects):
    if not isinstance(morphisms, list):
        _fail("morphisms must be a list")
    seen = set()
    for m in morphisms:
        if not isinstance(m, dict):
            _fail("morphism must be a JSON object")
        for field in MORPHISM_FIELDS:
            if field not in m:
                _fail(f"morphism missing field {field!r}")
        for field in IDENTITY_FIELDS:
            if not isinstance(m[field], str) or not m[field]:
                _fail(f"morphism field {field!r} must be a non-empty string")
        identity = (m["source"], m["name"])
        if identity in seen:
            _fail(f"duplicate morphism identity {identity[0]}.{identity[1]}")
        seen.add(identity)
        for endpoint in ("source", "target"):
            if m[endpoint] not in objects:
                _fail(
                    f"dangling endpoint: morphism {m['name']} references "
                    f"absent object {m[endpoint]!r}"
                )
        _validated_refinements(m["refinements"])
    return morphisms


def _validated_data(data):
    if data.get("schema_version") != SUPPORTED_SCHEMA_VERSION:
        _fail(
            f"unknown schema_version {data.get('schema_version')!r} "
            f"(supported: {SUPPORTED_SCHEMA_VERSION})"
        )
    revision = data.get("format_revision")
    if not isinstance(revision, str) or not revision:
        _fail("format_revision must be a non-empty string")
    objects = data.get("objects")
    if not isinstance(objects, list) or not objects:
        _fail("objects must be a non-empty list of object names")
    if not all(isinstance(o, str) and o for o in objects):
        _fail("objects must be non-empty strings")
    if len(set(objects)) != len(objects):
        _fail("duplicate object identity in objects")
    morphisms = _validated_morphisms(data.get("morphisms"), set(objects))
    return objects, morphisms, revision


def _edge_label(morphism):
    """Edge label: morphism name (column when split), refinement labels."""
    label = morphism["name"]
    if morphism["column"] != morphism["name"]:
        label += f" (column: {morphism['column']})"
    refinements = sorted(
        morphism["refinements"], key=lambda r: (r["side"], r["kind"])
    )
    for r in refinements:
        label += f" [{r['side']}={r['kind']}]"
    return label


def render_schema(payload):
    """Render the validated schema export as a deterministic Mermaid
    flowchart: revision-labeled, objects sorted, edges sorted by
    (source, name). Raises SchemaExportInvalid before any rendering."""
    data = _validated_envelope(payload)
    objects, morphisms, revision = _validated_data(data)
    lines = [
        f"%% schema view — derived from the acset Schema export"
        f" (format revision: {revision})",
        "flowchart LR",
        f'  schema["schema — {revision}"]',
    ]
    for obj in sorted(objects):
        lines.append(f'  {obj}["{obj}"]')
    edges = sorted(morphisms, key=lambda m: (m["source"], m["name"]))
    for m in edges:
        lines.append(
            f'  {m["source"]} -- "{_edge_label(m)}" --> {m["target"]}'
        )
    return "\n".join(lines) + "\n"


def main(argv):
    if len(argv) != 2 or argv[0] != "schema":
        print(
            f"usage: {Path(__file__).name} schema <export.json>",
            file=sys.stderr,
        )
        return 2
    try:
        payload = json.loads(Path(argv[1]).read_text())
        diagram = render_schema(payload)
    except (
        SchemaExportInvalid,
        json.JSONDecodeError,
        UnicodeDecodeError,
        OSError,
    ) as error:
        print(f"{SCHEMA_EXPORT_INVALID}: {error}", file=sys.stderr)
        return 1
    # Written only after validation — an invalid export never produces
    # output (D4: fail before output writes).
    sys.stdout.write(diagram)
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
