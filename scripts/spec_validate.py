#!/usr/bin/env python3
"""Version-guarded openspec validation gate (specodelic-uv3v).

The org-wide openspec sweep installs 1.x on dev machines; 1.x --strict
flags the dual-format layer (the byte-identical `## Requirements` mirror
and the Specodelic Constraints/Model/Properties tables) that the
ratified 0.19.0 pin accepts — see GH-20 / beads specodelic-uv3v. A bare
`openspec validate --all` therefore fails for FORMAT reasons on swept
machines, not content reasons.

This gate pins the validator: when the installed `openspec` is 0.19.x
(ratified channel, caret semantics) it runs directly; on any other
version it falls back to the pinned version via `npx -y` (labeled), so
the gate's verdict tracks corpus content, never the ambient npm state.

Exit code is the validator's exit code — CI safe.

Run: python3 scripts/spec_validate.py
"""

import subprocess
import sys

PINNED_VERSION = "0.19.0"

DIRECT_COMMAND = [
    "openspec", "validate", "--all", "--strict", "--no-interactive",
]

FALLBACK_COMMAND = [
    "npx", "-y", f"@fission-ai/openspec@{PINNED_VERSION}",
    "validate", "--all", "--strict", "--no-interactive",
]

REMEDIATION = (
    "remediation hint: the gate pins openspec "
    f"{PINNED_VERSION} (1.x strict flags the dual-format layer — "
    "beads specodelic-uv3v / GH-20); if the npx fallback fails, run "
    f"`npm install -g @fission-ai/openspec@{PINNED_VERSION}`"
)


class Plan:
    def __init__(self, mode: str, command: list[str]):
        self.mode = mode  # "direct" | "pinned-fallback"
        self.command = command


def parse_openspec_version(output: str) -> str | None:
    """Extract the first semver token from `openspec --version` output.

    npm wrappers may interleave notice lines; a token of the form
    N.N.N wins. Returns None when no version is detectable.
    """
    for line in output.splitlines():
        for token in line.split():
            parts = token.split(".")
            if (
                len(parts) == 3
                and all(p.isdigit() for p in parts)
            ):
                return token
    return None


def version_matches(installed: str, pinned: str) -> bool:
    """Caret match: same major.minor, any patch (0.19.x ≡ pinned 0.19.0)."""
    a, b = installed.split("."), pinned.split(".")
    return a[0] == b[0] and a[1] == b[1]


def build_plan(installed_version: str | None, pinned: str = PINNED_VERSION) -> Plan:
    """Decide how to run the gate from the detected installed version."""
    if installed_version is not None and version_matches(installed_version, pinned):
        return Plan("direct", list(DIRECT_COMMAND))
    return Plan("pinned-fallback", list(FALLBACK_COMMAND))


def detect_installed_version() -> str | None:
    try:
        proc = subprocess.run(
            ["openspec", "--version"],
            check=False, capture_output=True, text=True, timeout=60,
        )
    except (OSError, subprocess.TimeoutExpired):
        return None
    if proc.returncode != 0:
        return None
    return parse_openspec_version(proc.stdout)


def run_plan(plan: Plan, echo=print) -> int:
    if plan.mode == "pinned-fallback":
        echo(f"note: installed openspec is not {PINNED_VERSION} — "
             f"running pinned validator via npx ({PINNED_VERSION})")
    proc = subprocess.run(
        plan.command, check=False, capture_output=True, text=True, timeout=600,
    )
    if proc.stdout:
        echo(proc.stdout.rstrip())
    if proc.stderr:
        echo(proc.stderr.rstrip())
    if proc.returncode != 0:
        echo(REMEDIATION)
    return proc.returncode


def main() -> int:
    version = detect_installed_version()
    plan = build_plan(version)
    return run_plan(plan)


if __name__ == "__main__":
    sys.exit(main())
