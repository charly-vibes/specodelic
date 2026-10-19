#!/usr/bin/env python3
"""1.14.x shape fixture gate (specodelic-s8fg task 5.2).

Purpose: assert the checked-in shape fixture (tests/fixtures/os1x-shape/)
is valid under the openspec 1.14.x strict validator — the validator whose
dual-shape complaints forced the 0.19.0 pin (specodelic-uv3v). The fixture
preserves the three post-Revision-19 shapes: the deployed dual spec, the
plain delta + Purpose, and the layer.md sidecar.

Responsibilities: invoke the 1.14.x validator the same way the pinned gate
(scripts/spec_validate.py) does — `npx -y @fission-ai/openspec@<ver>
validate --all --strict --no-interactive` from the fixture root — and fail
with a labeled remediation hint on any non-zero verdict. The fixture
version is deliberately separate from the gate pin (0.19.0): task 6
retires the pin; this fixture keeps proving the 1.x target shape until
then and afterwards.

Rationale: the /tmp matrix this fixture was snapshotted from is volatile;
without a checked-in copy the pin's retirement (task 6) has no regression
guard for the shapes it was pinned against.

Run: python3 -m unittest discover -s scripts -p 'test_*.py'
"""

import subprocess
import unittest
from pathlib import Path

FIXTURE_VALIDATOR = "1.14.2"

FIXTURE_ROOT = Path(__file__).parent.parent / "tests" / "fixtures" / "os1x-shape"

COMMAND = [
    "npx",
    "-y",
    f"@fission-ai/openspec@{FIXTURE_VALIDATOR}",
    "validate",
    "--all",
    "--strict",
    "--no-interactive",
]

REMEDIATION = (
    f"remediation hint: install or warm the npx cache for "
    f"@fission-ai/openspec@{FIXTURE_VALIDATOR} "
    f"(`npm install -g @fission-ai/openspec@{FIXTURE_VALIDATOR}`), then rerun"
)


class Os1xShapeFixtureTest(unittest.TestCase):
    def test_fixture_is_1x_strict_valid(self):
        self.assertTrue(
            (FIXTURE_ROOT / "openspec").is_dir(),
            f"shape fixture missing at {FIXTURE_ROOT} — restore it from the "
            "matrix description in add-delta-layer-sidecar tasks.md 5.2",
        )
        try:
            proc = subprocess.run(
                COMMAND,
                cwd=FIXTURE_ROOT,
                check=False,
                capture_output=True,
                text=True,
                timeout=600,
            )
        except (OSError, subprocess.TimeoutExpired) as exc:
            self.skipTest(f"1.14.x validator unavailable via npx: {exc}")
        self.assertEqual(
            proc.returncode,
            0,
            f"shape fixture failed openspec {FIXTURE_VALIDATOR} --strict\n"
            f"{proc.stdout}\n{proc.stderr}\n{REMEDIATION}",
        )
        # Zero-warning assertion lives in the verdict itself: --strict
        # fails on any warning, so returncode 0 already means 0 warnings.
        self.assertIn("0 failed", proc.stdout, proc.stdout)


if __name__ == "__main__":
    unittest.main()
