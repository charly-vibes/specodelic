#!/usr/bin/env python3
"""SUMMARY.md completeness check (specodelic-b3p, per add-mdbook-docs 5.1).

docs/src/SUMMARY.md is hand-written: a new spec file can land without a
SUMMARY entry and mdbook silently won't render it. This gate requires:

- every frontmatter-bearing corpus file (specs/*.md — the naming law's
  id-equals-stem files; non-spec md like AGENTS/STATUS/USAGE/theory and
  *.checklist.md carry no frontmatter and are exempt) and
- every capability spec (openspec/specs/<cap>/spec.md)

to be linked from docs/src/SUMMARY.md. Archived change deltas under
openspec/changes/ are not part of the rendered book and are not required.

Follows the dual-format section-sync pattern (stdlib-only, fails closed).

Usage: check_summary_completeness.py [REPO_ROOT]   (default: .)
Exit 0 = every spec is listed; exit 1 = missing entries (or a missing /
unreadable SUMMARY — fails closed, never silent).
"""

import re
import sys
from pathlib import Path

# mdbook link target inside a SUMMARY bullet, e.g. (specs/a.md)
LINK = re.compile(r"\]\(([^)\s]+)\)")


def linked_paths(summary_text: str) -> set[str]:
    return set(LINK.findall(summary_text))


def corpus_spec_files(root: Path) -> list[Path]:
    """specs/*.md whose first line is frontmatter ('---')."""
    specs = root / "specs"
    if not specs.is_dir():
        return []
    out = []
    for p in sorted(specs.glob("*.md")):
        try:
            first = p.read_text(encoding="utf-8").splitlines()[0].strip()
        except (OSError, IndexError):
            continue
        if first == "---":
            out.append(p)
    return out


def capability_spec_files(root: Path) -> list[Path]:
    """openspec/specs/<cap>/spec.md — archived change deltas excluded."""
    base = root / "openspec" / "specs"
    if not base.is_dir():
        return []
    return sorted(base.glob("*/spec.md"))


def check(root: Path) -> list[str]:
    summary = root / "docs" / "src" / "SUMMARY.md"
    if not summary.is_file():
        return [
            "missing docs/src/SUMMARY.md — cannot verify spec completeness "
            "(fails closed)"
        ]
    try:
        text = summary.read_text(encoding="utf-8")
    except OSError as e:
        return [f"unreadable docs/src/SUMMARY.md: {e}"]
    links = linked_paths(text)

    problems: list[str] = []
    for spec in corpus_spec_files(root) + capability_spec_files(root):
        rel = spec.relative_to(root).as_posix()
        # mdbook src mirrors specs/ -> docs/src/specs and
        # openspec/specs/ -> docs/src/openspec, so a capability spec's
        # SUMMARY link target is openspec/<cap>/spec.md
        link = rel.replace("openspec/specs/", "openspec/", 1)
        if link not in links:
            problems.append(
                f"{rel} is not linked from docs/src/SUMMARY.md "
                f"(expected link target {link}; mdbook will not render "
                "it) — add a SUMMARY entry (docs/src/SUMMARY.md)"
            )
    return problems


def main() -> int:
    root = Path(sys.argv[1]) if len(sys.argv) > 1 else Path(".")
    problems = check(root)
    if problems:
        for p in problems:
            print(f"summary-completeness: {p}", file=sys.stderr)
        print(
            f"{len(problems)} spec file(s) missing from "
            "docs/src/SUMMARY.md",
            file=sys.stderr,
        )
        return 1
    print("summary-completeness: all spec files are linked (0 missing)")
    return 0


if __name__ == "__main__":
    sys.exit(main())