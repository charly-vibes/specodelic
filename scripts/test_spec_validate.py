#!/usr/bin/env python3
"""Tests for scripts/spec_validate.py (stdlib unittest, no deps).

specodelic-s8fg task 6.1: the validate-gate pin is retired (decision
2026-10-19-validate-pin-retired.md, superseding 2026-10-19-openspec-
validate-pin.md). The pin existed only for the dual-format delta shape,
which 0.19.0 accepted and 1.x --strict rejected; Revision 19 replaced
that shape with plain deltas + layer.md sidecars, and the corpus
validates 52/52 green under 1.14.2 --strict (task 5's fixture gate).
The gate is therefore a PLAIN single invocation of `openspec validate
--all --strict --no-interactive` — no version sniffing, no npx
fallback. The contract is that `openspec` on PATH IS the pinned 1.x
validator: CI installs `@fission-ai/openspec@1.14.2` globally, and dev
machines must do the same (decision record documents the install
requirement).

Run: python3 -m unittest discover -s scripts -p 'test_*.py'
"""

import sys
import unittest
from pathlib import Path
from unittest import mock

sys.path.insert(0, str(Path(__file__).parent))
import spec_validate as sv  # noqa: E402

SCRIPT_SOURCE = (Path(__file__).parent / "spec_validate.py").read_text()


class PlainInvocationTest(unittest.TestCase):
    """The gate is one plain 1.x invocation — the retirement contract."""

    def test_command_is_plain_openspec_validate(self):
        self.assertEqual(
            sv.GATE_COMMAND,
            ["openspec", "validate", "--all", "--strict", "--no-interactive"],
        )

    def test_no_version_guard_machinery_exists(self):
        # the version-sniffing behavior no longer exists
        for name in (
            "PINNED_VERSION",
            "parse_openspec_version",
            "version_matches",
            "build_plan",
            "detect_installed_version",
            "Plan",
        ):
            self.assertFalse(
                hasattr(sv, name),
                f"spec_validate.{name} is version-guard machinery — "
                "the pin is retired; delete it",
            )

    def test_no_fallback_invocation_in_the_script(self):
        # the fallback mechanism itself is gone; the pinned package name
        # appears only as documentation prose (docstring + remediation hint)
        code_only = SCRIPT_SOURCE.split('"]', 1)[-1].split('REMEDIATION', 1)[0]
        self.assertNotIn("npx", code_only)
        self.assertNotIn("subprocess.run([", code_only)

    def test_expected_validator_version_is_documented(self):
        # docstring note (not a runtime constant) names the 1.x pin so a
        # drifted dev machine has a remediable lead
        self.assertIn("1.14.2", SCRIPT_SOURCE)


class RunnerTest(unittest.TestCase):
    def test_run_invokes_the_gate_command_exactly_once(self):
        with mock.patch("subprocess.run") as run:
            run.return_value = mock.Mock(returncode=0, stdout="", stderr="")
            rc = sv.run_gate(echo=lambda *_: None)
        self.assertEqual(rc, 0)
        run.assert_called_once_with(
            sv.GATE_COMMAND, check=False, capture_output=True, text=True,
            timeout=mock.ANY,
        )

    def test_run_streams_validator_output(self):
        lines = []
        with mock.patch("subprocess.run") as run:
            run.return_value = mock.Mock(
                returncode=0, stdout="52/52 items passed", stderr=""
            )
            rc = sv.run_gate(echo=lines.append)
        self.assertEqual(rc, 0)
        self.assertTrue(any("52/52" in ln for ln in lines), lines)

    def test_run_failure_reports_remediation_hint(self):
        lines = []
        with mock.patch("subprocess.run") as run:
            run.return_value = mock.Mock(
                returncode=1, stdout="boom", stderr="err"
            )
            rc = sv.run_gate(echo=lines.append)
        self.assertEqual(rc, 1)
        self.assertTrue(
            any("remediation" in ln.lower() or "hint" in ln.lower() for ln in lines),
            lines,
        )
        # the hint names the pinned validator so a drifted machine can fix it
        self.assertTrue(any("1.14.2" in ln for ln in lines), lines)

    def test_exit_code_is_the_validators_exit_code(self):
        with mock.patch("subprocess.run") as run:
            run.return_value = mock.Mock(returncode=7, stdout="", stderr="")
            rc = sv.run_gate(echo=lambda *_: None)
        self.assertEqual(rc, 7)


class JustfileWiringTest(unittest.TestCase):
    def test_justfile_openspec_validate_uses_the_script(self):
        just = (Path(__file__).parent.parent / "justfile").read_text()
        self.assertIn("spec_validate.py", just)
        # the retirement is visible at the seam too: no fallback language
        self.assertNotIn("npx", just)


if __name__ == "__main__":
    unittest.main()
