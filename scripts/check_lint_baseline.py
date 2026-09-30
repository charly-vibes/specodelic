#!/usr/bin/env python3
"""Honest dogfood gate: spk lint findings vs a shrink-only baseline.

`just lint-specs` was a trivial pass while the corpus violated its own
typing table — 0 findings read as "clean" even when the checker was
blind (specodelic-oet evidence). This gate pins the other direction:
corpus findings may only ever SHRINK. A baseline entry (one
`rule_id:filename` signature per line) is a known, accepted finding;

- a finding not in the baseline fails (new debt is never silent), and
- a baseline entry with no live finding fails too (accepted debt must be
  retired — the baseline may shrink, never stay flat forever).

With no baseline file (today's state: the corpus lints fully clean) the
gate is strict zero-findings. Never auto-write the baseline from the
gate; update it deliberately with --update after reviewing the findings.
"""

import json
import pathlib
import subprocess
import sys

BASELINE = pathlib.Path("specs/.lint-baseline")


def findings(target):
    """Set of `rule_id:filename` signatures from spk lint's envelope."""
    r = subprocess.run(
        ["cargo", "run", "-q", "--", "lint", target],
        capture_output=True, text=True,
    )
    try:
        env = json.loads(r.stdout)
    except json.JSONDecodeError:
        print(f"error: spk lint did not emit a JSON envelope (exit {r.returncode})")
        print(r.stdout)
        print(r.stderr, file=sys.stderr)
        sys.exit(1)
    issues = (env.get("data") or {}).get("issues") or []
    return {
        f"{i['rule_id']}:{pathlib.Path(i['file']).name}" for i in issues
    }


def read_baseline():
    if not BASELINE.exists():
        return set()
    return {
        ln.strip()
        for ln in BASELINE.read_text().splitlines()
        if ln.strip() and not ln.startswith("#")
    }


def main(argv):
    target = argv[1] if len(argv) > 1 else "specs"
    live = findings(target)
    base = read_baseline()

    new = sorted(live - base)
    stale = sorted(base - live)

    if new and "--update" in argv:
        BASELINE.write_text(
            "# specodelic dogfood baseline — one rule_id:file per line;\n"
            "# entries may only be REMOVED (findings must shrink, never grow)\n"
            + "\n".join(sorted(live)) + "\n"
        )
        print(f"baseline rewritten with {len(live)} entr(y/ies) — review before committing")
        return 0

    failed = False
    if new:
        failed = True
        print("FAIL: new corpus findings not in the baseline (fix the corpus):")
        for s in new:
            print(f"  + {s}")
    if stale:
        failed = True
        print("FAIL: baseline entries with no live finding (the baseline must shrink):")
        for s in stale:
            print(f"  - {s}")

    if failed:
        print(f"\nbaseline: {BASELINE} (update deliberately with --update, after review)")
        return 1

    print(f"dogfood gate green: {len(live)} finding(s), baseline honest "
          f"({len(base)} accepted, shrink-only)")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
