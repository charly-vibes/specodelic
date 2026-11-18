#!/usr/bin/env python3
"""Tests for scripts/check_evidence_axes.py (stdlib unittest, no deps).

Fixture trees mirror the real layout (src/conform.rs, src/audit.rs,
docs/src/graded-evidence.md) in miniature and cover every drift class the
guard must catch, in both directions, plus fail-closed behavior on
unparseable input (add-evidence-record-axes tasks 1.1-1.6 and the delta
spec scenarios, including reserved-value-in-code and duplicate rows).

Run: python3 -m unittest discover -s scripts -p 'test_*.py'
"""

import sys
import tempfile
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))
import check_evidence_axes as cea  # noqa: E402

CONFORM = """\
pub enum Verdict {
    /// permitted doc.
    Permitted,
    /// forbidden doc.
    Forbidden,
    /// underspecified doc.
    Underspecified,
    /// unknown doc.
    Unknown,
}

impl Verdict {
    pub fn as_str(self) -> &'static str {
        match self {
            Verdict::Permitted => "permitted",
            Verdict::Forbidden => "forbidden",
            Verdict::Underspecified => "underspecified",
            Verdict::Unknown => "unknown",
        }
    }
}
"""

AUDIT = """\
pub enum AuditClass {
    /// current doc.
    Current,
    /// stale doc.
    Stale,
    /// unchecked doc.
    Unchecked,
}

impl AuditClass {
    fn of(verdict: Verdict) -> AuditClass {
        match verdict {
            Verdict::Permitted => AuditClass::Current,
            Verdict::Forbidden => AuditClass::Stale,
            Verdict::Underspecified | Verdict::Unknown => AuditClass::Unchecked,
        }
    }

    fn rank(self) -> u8 {
        match self {
            AuditClass::Current | AuditClass::Stale => 0,
            AuditClass::Unchecked => 1,
        }
    }
}
"""


def table(rows: list[str]) -> str:
    """Wrap data rows (already full markdown table lines) in the normative section."""
    head = (
        "## The evidence-record axes\n\n"
        "| Value | Axis | Code source | Status | of() mapping | Trigger |\n"
        "|-------|------|-------------|--------|--------------|---------|\n"
    )
    return head + "\n".join(rows) + "\n"


def synced_docs() -> str:
    """A docs table exactly matching the CONFORM/AUDIT fixtures."""
    return table([
        "| `permitted` | run status | `src/conform.rs` `Verdict` | implemented | `→ current` | — |",
        "| `forbidden` | run status | `src/conform.rs` `Verdict` | implemented | `→ stale` | — |",
        "| `underspecified` | spec defect | `src/conform.rs` `Verdict` | implemented | `→ unchecked` | — |",
        "| `unknown` | knowledge absence | `src/conform.rs` `Verdict` | implemented | `→ unchecked` | — |",
        "| `current` | currency | `src/audit.rs` `AuditClass` | implemented | — | — |",
        "| `stale` | currency | `src/audit.rs` `AuditClass` | implemented | — | — |",
        "| `unchecked` | knowledge absence | `src/audit.rs` `AuditClass` | implemented | — | — |",
        "| `inconclusive` | run status | `ITU-T X.290 3.3.124` | reserved | — | D-10 check fusion |",
        "| `origin` | annotation field | design only | reserved | — | annotation-syntax change |",
    ])


class EvidenceAxesTest(unittest.TestCase):
    def write(self, tree: dict[str, str]) -> Path:
        tmp = tempfile.mkdtemp()
        root = Path(tmp)
        for rel, text in tree.items():
            p = root / rel
            p.parent.mkdir(parents=True, exist_ok=True)
            p.write_text(text, encoding="utf-8")
        return root

    def synced(self) -> Path:
        return self.write({
            "src/conform.rs": CONFORM,
            "src/audit.rs": AUDIT,
            "docs/src/graded-evidence.md": synced_docs(),
        })

    # 1.1 — the synchronized-tree pass, with the counts it compared.
    def test_synchronized_tree_passes_with_counts(self):
        problems, ok = cea.check(self.synced())
        self.assertEqual(problems, [])
        self.assertIn("4 verdicts", ok)
        self.assertIn("3 audit classes", ok)
        self.assertIn("9 table rows", ok)
        return None

    # 1.2 — a code variant absent from the table is named.
    def test_code_value_missing_from_table_is_named(self):
        root = self.write({
            "src/conform.rs": CONFORM,
            "src/audit.rs": AUDIT,
            "docs/src/graded-evidence.md": synced_docs().replace(
                "| `unknown` | knowledge absence", "| `zzz` | knowledge absence"),
        })
        problems, _ = cea.check(root)
        self.assertEqual(len(problems), 1, problems)
        self.assertIn("'unknown'", problems[0])
        self.assertIn("src/conform.rs", problems[0])
        return None

    # 1.3 — an implemented row with no code counterpart is named.
    def test_implemented_row_without_code_is_named(self):
        root = self.write({
            "src/conform.rs": CONFORM,
            "src/audit.rs": AUDIT,
            "docs/src/graded-evidence.md": synced_docs().replace(
                "| `permitted` | run status", "| `ghost` | run status"),
        })
        problems, _ = cea.check(root)
        self.assertEqual(len(problems), 1, problems)
        self.assertIn("'ghost'", problems[0])
        self.assertIn("implemented", problems[0])
        return None

    # 1.4 — a documented mapping that disagrees with the of() arm is named
    # with both sides.
    def test_mapping_disagreement_names_both_sides(self):
        root = self.write({
            "src/conform.rs": CONFORM,
            "src/audit.rs": AUDIT,
            "docs/src/graded-evidence.md": synced_docs().replace(
                "`permitted` | run status | `src/conform.rs` `Verdict` | implemented | `→ current`",
                "`permitted` | run status | `src/conform.rs` `Verdict` | implemented | `→ unchecked`"),
        })
        problems, _ = cea.check(root)
        self.assertEqual(len(problems), 1, problems)
        self.assertIn("'permitted'", problems[0])
        self.assertIn("unchecked", problems[0])
        self.assertIn("current", problems[0])
        return None

    # 1.5 — a reserved row without a trigger is named.
    def test_reserved_row_without_trigger_is_named(self):
        root = self.write({
            "src/conform.rs": CONFORM,
            "src/audit.rs": AUDIT,
            "docs/src/graded-evidence.md": synced_docs().replace(
                "| `inconclusive` | run status | `ITU-T X.290 3.3.124` | reserved | — | D-10 check fusion |",
                "| `inconclusive` | run status | `ITU-T X.290 3.3.124` | reserved | — | — |"),
        })
        problems, _ = cea.check(root)
        self.assertEqual(len(problems), 1, problems)
        self.assertIn("'inconclusive'", problems[0])
        self.assertIn("trigger", problems[0].lower())
        return None

    # 1.6 — a malformed enum fails closed with a remediation hint.
    def test_malformed_enum_fails_closed(self):
        root = self.write({
            "src/conform.rs": "pub struct NotAnEnum;\n",
            "src/audit.rs": AUDIT,
            "docs/src/graded-evidence.md": synced_docs(),
        })
        problems, ok = cea.check(root)
        self.assertEqual(ok, "", "fail-closed: no success line on unparseable input")
        self.assertTrue(any("conform.rs" in p and "pub enum Verdict" in p for p in problems),
                        problems)
        return None

    # Delta scenario: a reserved row naming a code-present value fails.
    def test_reserved_value_present_in_code_fails(self):
        root = self.write({
            "src/conform.rs": CONFORM,
            "src/audit.rs": AUDIT,
            "docs/src/graded-evidence.md": synced_docs().replace(
                "| `inconclusive` | run status", "| `permitted` | run status"),
        })
        problems, _ = cea.check(root)
        self.assertTrue(any("'permitted'" in p and "reserved" in p for p in problems),
                        problems)
        return None

    # Delta scenario: every value appears exactly once — duplicates fail.
    def test_duplicate_row_fails(self):
        dup = ("| `current` | currency | `src/audit.rs` `AuditClass` "
               "| implemented | — | — |")
        root = self.write({
            "src/conform.rs": CONFORM,
            "src/audit.rs": AUDIT,
            "docs/src/graded-evidence.md": synced_docs() + dup,
        })
        problems, _ = cea.check(root)
        self.assertTrue(any("'current'" in p and "twice" in p for p in problems),
                        problems)
        return None

    # Delta scenario: unparseable docs table fails closed with a hint.
    def test_missing_table_fails_closed(self):
        root = self.write({
            "src/conform.rs": CONFORM,
            "src/audit.rs": AUDIT,
            "docs/src/graded-evidence.md": "# No axes here\n",
        })
        problems, ok = cea.check(root)
        self.assertEqual(ok, "")
        self.assertTrue(any("evidence-record axes" in p for p in problems), problems)
        return None

    # The grouped of() arm (A | B => Class) maps every listed verdict.
    def test_grouped_of_arm_extracts_every_verdict(self):
        mapping = cea.parse_of_mapping(AUDIT)
        self.assertEqual(mapping["underspecified"], "unchecked")
        self.assertEqual(mapping["unknown"], "unchecked")
        self.assertEqual(mapping["permitted"], "current")
        self.assertEqual(mapping["forbidden"], "stale")
        return None

    # rank()/as_str() arms must not leak into the of() mapping — the
    # mapping is keyed by verdicts only, never by class names.
    def test_of_mapping_ignores_other_fns(self):
        mapping = cea.parse_of_mapping(AUDIT)
        self.assertEqual(sorted(mapping), ["forbidden", "permitted",
                                           "underspecified", "unknown"])
        return None


if __name__ == "__main__":
    unittest.main()
