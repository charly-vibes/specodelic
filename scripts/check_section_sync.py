#!/usr/bin/env python3
"""Section-sync check for dual-format spec files (add-dual-format-deltas task 2.4).

A dual-format file carries both an `## ADDED Requirements` section
(openspec delta grammar) and a `## Requirements` section (capability
grammar) that must hold identical requirement text. This script fails
CI naming any file whose sections drift, plus the divergent requirement.

Usage: check_section_sync.py [DIR...]   (default: openspec)
Exit 0 = all in sync; exit 1 = drift found (or a section is missing one
side while the other has requirements).
"""

import re
import sys
from pathlib import Path

REQ = re.compile(r"^### Requirement: (.+)$", re.MULTILINE)


def sections(text: str) -> dict[str, str]:
    """Map `## Header` -> body text, for the two sections we compare."""
    out: dict[str, list[str]] = {}
    current = None
    for line in text.splitlines():
        if line.startswith("## "):
            current = line[3:].strip()
            out.setdefault(current, [])
        elif current is not None:
            out[current].append(line)
    return {k: "\n".join(v) for k, v in out.items()}


def norm(body: str) -> str:
    """Normalize a section body: strip per-line trailing space and blank lines."""
    return "\n".join(line.rstrip() for line in body.splitlines() if line.strip())


def is_dual_format(text: str) -> bool:
    lines = text.splitlines()
    return bool(lines) and lines[0].strip() == "---" \
        and "## ADDED Requirements" in text and "## Requirements" in text


def is_capability_spec(path: Path) -> bool:
    """True for openspec capability specs: .../openspec/specs/<cap>/spec.md.

    Archived deltas (openspec/changes/archive/<id>/specs/<cap>/spec.md)
    are pre-protocol evidence and stay exempt — the trailing-`openspec`
    component distinguishes them.
    """
    parts = path.parts
    return (
        len(parts) >= 4
        and parts[-1] == "spec.md"
        and parts[-3] == "specs"
        and parts[-4] == "openspec"
    )


def has_specodelic_tables(text: str) -> bool:
    """True when the specodelic layer (frontmatter + the three tables) is present."""
    lines = text.splitlines()
    return bool(lines) and lines[0].strip() == "---" \
        and "## Constraints" in text and "## Model" in text and "## Properties" in text


def check(path: Path) -> list[str]:
    text = path.read_text(encoding="utf-8")
    # Capability-format check (Rule-of-5 EDGE-001): capability specs live
    # under openspec/specs/<cap>/spec.md and MUST be dual-format files.
    # Files without frontmatter never reach spk lint (parse_batch skips
    # them), so a plain openspec regeneration output here would otherwise
    # sail through CI lint-clean forever.
    if is_capability_spec(path) and not has_specodelic_tables(text):
        if not (text.lstrip().startswith("---")):
            return [
                f"{path}: capability spec has no frontmatter — not a dual-format "
                "file (plain openspec regeneration output); see the migration "
                "recipe in openspec/project.md: add id:spec frontmatter + "
                "## Constraints/## Model/## Properties, then the ## Requirements "
                "sibling (deltas: mirror the ADDED text; capability specs: keep "
                "## Requirements)"
            ]
        return [
            f"{path}: capability spec carries frontmatter but is missing the "
            "specodelic tables (## Constraints / ## Model / ## Properties) — "
            "see the migration recipe in openspec/project.md"
        ]
    if not is_dual_format(text):
        return []
    secs = sections(text)
    added, reqs = norm(secs["ADDED Requirements"]), norm(secs["Requirements"])
    problems: list[str] = []
    if added == reqs:
        return problems
    added_reqs = REQ.findall(added)
    req_reqs = REQ.findall(reqs)
    if set(added_reqs) != set(req_reqs):
        only_added = sorted(set(added_reqs) - set(req_reqs))
        only_reqs = sorted(set(req_reqs) - set(added_reqs))
        if only_added:
            problems.append(f"{path}: requirement only in ADDED section: {', '.join(only_added)}")
        if only_reqs:
            problems.append(f"{path}: requirement only in Requirements section: {', '.join(only_reqs)}")
    else:
        for name in added_reqs:
            block = re.compile(
                r"^### Requirement: " + re.escape(name) + r"$.*?(?=^### Requirement:|\Z)",
                re.MULTILINE | re.DOTALL,
            )
            a, r = block.search(added), block.search(reqs)
            if a and r and norm(a.group(0)) != norm(r.group(0)):
                problems.append(f"{path}: divergent requirement text: {name}")
    if not problems:
        problems.append(f"{path}: ADDED Requirements and Requirements sections differ")
    return problems


def main() -> int:
    roots = [Path(a) for a in sys.argv[1:]] or [Path("openspec")]
    problems: list[str] = []
    for root in roots:
        if root.is_file():
            problems += check(root)
            continue
        for path in sorted(root.rglob("*.md")):
            problems += check(path)
    for p in problems:
        print(f"section-sync: {p}", file=sys.stderr)
    if problems:
        print(
            "section-sync: run `just sync-sections` guidance — edit the ADDED "
            "Requirements section, then mirror it verbatim into Requirements",
            file=sys.stderr,
        )
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
