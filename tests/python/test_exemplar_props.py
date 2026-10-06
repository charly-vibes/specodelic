"""Hypothesis property test binding parse's Export full IR scenario.

The pytest exemplar for the language-neutral property-binding seam
(add-language-neutral-property-binding tasks 1.2-1.4): the scenario
contract is asserted over HYPOTHESIS-GENERATED inputs, not a fixture —
the binding-level (espectacular) counterpart to specodelic's compiled
`**rust:**` scaffolds. The generator vocabulary is deliberately
hand-written here (design D5: the in-format vocabulary is deferred to
the emitter follow-ups); this file is the honest shape of that decision.

Contract: `.espectacular/parse/export-full-ir.toml` ([[tests.pytest]]).
"""

import json
import os
import subprocess
import tempfile

from hypothesis import given, settings
from hypothesis import strategies as st

ENVELOPE_FIELDS = {
    "ok",
    "envelope_version",
    "cli_version",
    "envelope_kind",
    "data",
    "warnings",
    "hints",
    "meta",
}


def _run_parse(binary: str, text: str) -> dict:
    with tempfile.NamedTemporaryFile("w", suffix=".md", delete=False) as f:
        f.write(text)
        path = f.name
    try:
        proc = subprocess.run(
            [binary, "parse", path],
            capture_output=True,
            text=True,
            check=False,
        )
    finally:
        os.unlink(path)
    assert proc.returncode == 0, f"parse failed on a valid spec: {proc.stderr}"
    return json.loads(proc.stdout)


# A valid minimal spec: frontmatter id (two lowercase word segments) plus
# an arbitrary safe statement — the smallest corpus shape that exercises
# the full IR envelope.
_id_strategy = st.builds(
    lambda a, b: f"{a}.{b}",
    st.from_regex(r"[a-z]{3,8}", fullmatch=True),
    st.from_regex(r"[a-z]{3,8}", fullmatch=True),
)
_statement_strategy = st.text(
    alphabet=st.characters(
        min_codepoint=32, max_codepoint=126, exclude_characters='"\\|`'
    ),
    min_size=8,
    max_size=80,
)


@settings(max_examples=10, deadline=None)
@given(
    spec_id=_id_strategy,
    statement=_statement_strategy,
)
def test_parse_envelope_wellformed_over_generated_specs(spk_binary, spec_id, statement):
    """Export full IR (deployed parse scenario): over generated valid
    specs, parse emits the full Spec IR as a json envelope that carries
    the whole envelope discipline and the generated intent verbatim."""
    text = (
        "---\n"
        f"id: {spec_id}\n"
        "kind: intent\n"
        f'statement: "{statement}"\n'
        "---\n"
    )
    envelope = _run_parse(spk_binary, text)
    assert envelope["ok"] is True
    assert envelope["envelope_version"] == "0.1"
    assert ENVELOPE_FIELDS <= set(envelope), f"envelope fields: {set(envelope)}"
    intent = envelope["data"]["intent"]
    assert intent["id"] == spec_id
    assert intent["statement"] == statement
    # parse's hints suggest spk lint on the parsed file (C-parse-envelope).
    assert any(
        "lint" in str(h).lower() for h in envelope["hints"]
    ), f"hints must suggest spk lint: {envelope['hints']}"
