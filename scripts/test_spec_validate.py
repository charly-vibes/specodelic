#!/usr/bin/env python3
"""Tests for scripts/spec_validate.py (stdlib unittest, no deps).

specodelic-uv3v: the openspec-validate gate must be independent of the
globally installed openspec version. The org-wide sweep installs 1.x,
whose --strict flags the dual-format layer (mirror requirements +
Specodelic tables) the ratified 0.19.0 pin accepts — a bare
`openspec validate --all` on a swept machine fails the gate for format
reasons, not content reasons. The script pins the validator: 0.19.x on
PATH runs directly; any other version falls back to the pinned npx
invocation (labeled, still strict).

Run: python3 -m unittest discover -s scripts -p 'test_*.py'
"""

import sys
import unittest
from pathlib import Path
from unittest import mock

sys.path.insert(0, str(Path(__file__).parent))
import spec_validate as sv  # noqa: E402

PIN = sv.PINNED_VERSION


class VersionGuardTest(unittest.TestCase):
    def parse(self, out: str):
        return sv.parse_openspec_version(out)

    def test_parse_plain_version(self):
        self.assertEqual(self.parse("0.19.0\n"), "0.19.0")

    def test_parse_version_with_prefix_noise(self):
        # npm wrappers may add lines; the version token is what matters
        self.assertEqual(self.parse("npm notice x\n1.14.2\n"), "1.14.2")

    def test_parse_garbage_is_none(self):
        self.assertIsNone(self.parse("command not found\n"))

    def test_parse_empty_is_none(self):
        self.assertIsNone(self.parse(""))

    def test_pinned_matches_itself(self):
        self.assertTrue(sv.version_matches(PIN, PIN))

    def test_patch_drift_within_pin_is_accepted(self):
        # caret semantics: 0.19.x is the ratified channel
        self.assertTrue(sv.version_matches("0.19.9", PIN))

    def test_minor_drift_is_rejected(self):
        self.assertFalse(sv.version_matches("0.20.0", PIN))
        self.assertFalse(sv.version_matches("1.14.2", PIN))
        self.assertFalse(sv.version_matches("0.18.0", PIN))


class PlanTest(unittest.TestCase):
    def plan(self, version):
        return sv.build_plan(installed_version=version, pinned=PIN)

    def test_plan_direct_when_installed_is_pinned(self):
        plan = self.plan("0.19.0")
        self.assertEqual(plan.mode, "direct")
        self.assertEqual(
            plan.command,
            ["openspec", "validate", "--all", "--strict", "--no-interactive"],
        )

    def test_plan_direct_accepts_patch_drift(self):
        self.assertEqual(self.plan("0.19.3").mode, "direct")

    def test_plan_falls_back_to_pinned_npx_on_drift(self):
        plan = self.plan("1.14.2")
        self.assertEqual(plan.mode, "pinned-fallback")
        self.assertIn("-y", plan.command)
        self.assertIn(f"@fission-ai/openspec@{PIN}", plan.command)
        self.assertIn("--strict", plan.command)
        self.assertIn("--no-interactive", plan.command)

    def test_plan_fallback_when_version_undetectable(self):
        # a missing/broken binary must not silently run the wrong validator
        plan = self.plan(None)
        self.assertEqual(plan.mode, "pinned-fallback")


class RunnerTest(unittest.TestCase):
    def _fake_popen(self, plan):
        class P:
            def __init__(self, cmd):
                self.cmd = cmd

        return P

    def test_run_direct_success(self):
        plan = sv.build_plan("0.19.0", PIN)
        with mock.patch("subprocess.run") as run:
            run.return_value = mock.Mock(returncode=0, stdout="", stderr="")
            rc = sv.run_plan(plan, echo=lambda *_: None)
        self.assertEqual(rc, 0)
        run.assert_called_once_with(
            plan.command, check=False, capture_output=True, text=True,
            timeout=mock.ANY,
        )

    def test_run_pinned_fallback_notes_the_drift(self):
        plan = sv.build_plan("1.14.2", PIN)
        lines = []
        with mock.patch("subprocess.run") as run:
            run.return_value = mock.Mock(returncode=0, stdout="", stderr="")
            rc = sv.run_plan(plan, echo=lines.append)
        self.assertEqual(rc, 0)
        self.assertTrue(any(PIN in ln for ln in lines), lines)

    def test_run_failure_reports_remediation_hint(self):
        plan = sv.build_plan("0.19.0", PIN)
        lines = []
        with mock.patch("subprocess.run") as run:
            run.return_value = mock.Mock(
                returncode=1, stdout="boom", stderr="err"
            )
            rc = sv.run_plan(plan, echo=lines.append)
        self.assertEqual(rc, 1)
        self.assertTrue(
            any("remediation" in ln.lower() or "hint" in ln.lower() for ln in lines),
            lines,
        )


class JustfileWiringTest(unittest.TestCase):
    def test_justfile_openspec_validate_uses_the_script(self):
        just = (Path(__file__).parent.parent / "justfile").read_text()
        self.assertIn("spec_validate.py", just)


if __name__ == "__main__":
    unittest.main()
