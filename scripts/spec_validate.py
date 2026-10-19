#!/usr/bin/env python3
"""Plain openspec validation gate (specodelic-s8fg task 6.1).

The version-guarded 0.19.0 pin is retired (decision
2026-10-19-validate-pin-retired.md, superseding 2026-10-19-openspec-
validate-pin.md): the pin existed only because 1.x --strict rejected the
dual-format delta shape 0.19 accepted. Revision 19 replaced that shape
with plain deltas + layer.md sidecars, and the corpus validates 52/52
green under 1.14.2 --strict (task 5's fixture gate). No version
sniffing, no fallback invocation: this gate runs the bare
`openspec` on PATH exactly once.

Expected validator: @fission-ai/openspec@1.14.2. The contract is that
`openspec` on PATH IS that pinned version — CI installs it globally
(.github/workflows/ci.yml) and dev machines must too:
`npm install -g @fission-ai/openspec@1.14.2`.

Exit code is the validator's exit code — CI safe.

Run: python3 scripts/spec_validate.py
"""

import subprocess
import sys

GATE_COMMAND = [
    "openspec", "validate", "--all", "--strict", "--no-interactive",
]

REMEDIATION = (
    "remediation hint: the gate runs the pinned 1.x validator "
    "(decision 2026-10-19-validate-pin-retired.md / GH-20); if `openspec` "
    "on PATH is missing or not @fission-ai/openspec@1.14.2, run "
    "`npm install -g @fission-ai/openspec@1.14.2`"
)


def run_gate(echo=print) -> int:
    proc = subprocess.run(
        GATE_COMMAND, check=False, capture_output=True, text=True, timeout=600,
    )
    if proc.stdout:
        echo(proc.stdout.rstrip())
    if proc.stderr:
        echo(proc.stderr.rstrip())
    if proc.returncode != 0:
        echo(REMEDIATION)
    return proc.returncode


def main() -> int:
    return run_gate()


if __name__ == "__main__":
    sys.exit(main())
