"""Shared failure classes and codes for the graph view renderers
(scripts/graph_views.py's views — add-graph-views D1/D4/D8).

Purpose: one home for the labeled failure classes every view module
raises, so refusals keep a single vocabulary across the schema and state
views.

Responsibilities: define `OutOfScopeRefused` (the corpus-scope refusal,
design D7 — intentless or lint-dirty corpora) and `ArtifactInvalid`
(a graph artifact breaching its contract), plus their stderr failure
codes.

Rationale: the views fail loudly with named classes BEFORE any output is
written (D3/D7); a shared module keeps that contract from drifting while
the per-view modules stay under the script-role file-size ratchet.
"""

OUT_OF_SCOPE_REFUSED = "out_of_scope_refused"
ARTIFACT_INVALID = "artifact_invalid"


class OutOfScopeRefused(Exception):
    """The corpus is out of scope for the views (design D7): intentless
    (zero spec files) or lint-dirty (invariant lint findings)."""


class ArtifactInvalid(Exception):
    """A graph artifact does not match its contract (six-column TSV,
    successful envelope) — refused before any output, like the schema
    view refuses malformed exports."""