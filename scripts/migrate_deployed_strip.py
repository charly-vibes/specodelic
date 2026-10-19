#!/usr/bin/env python3
"""One-time migration: strip delta sections from deployed capability
specs (specodelic-s8fg Stage 1 follow-through).

openspec 1.x --strict rejects exactly two shapes in main specs — a delta
header and a requirement header outside `## Requirements`. Our deployed
specs inherited both from the verbatim delta copy; this script removes
the `## ADDED/MODIFIED/REMOVED/RENAMED Requirements` sections from every
frontmatter-bearing `openspec/specs/<cap>/spec.md`, leaving the
`## Requirements` mirror as the deployed Requirements section. The
mirror already carries every requirement block including scenarios, so
nothing is lost (the same invariant the archive-companion strip relies
on, src/archive_companion.rs).

Idempotent: already-clean files are byte-untouched and unreported.
Non-spec files (no frontmatter) are exempt. Run with --dry-run to
preview. After running, the meters are: just lint-specs, just
sync-sections, ah check, and both openspec validators (0.19 pinned +
1.x strict) — 1.x should report zero deployed-spec ERRORs afterwards.

Run: python3 scripts/migrate_deployed_strip.py [--dry-run] [root]
"""

import sys
from pathlib import Path

DELTA_TITLES = {
    "added requirements",
    "modified requirements",
    "removed requirements",
    "renamed requirements",
}


def strip_delta_sections(text: str) -> str:
    """Drop the four delta-section blocks, fence-aware; everything else
    byte-identical (mirrors the Rust strip_delta_sections)."""
    out: list[str] = []
    in_delta = False
    fence: str | None = None
    for line in text.splitlines(keepends=True):
        trimmed = line.lstrip()
        if fence is None and (trimmed.startswith("```") or trimmed.startswith("~~~")):
            fence = trimmed[:3]
            out.append(line)  # the fence marker itself is content
            continue
        if fence is not None:
            out.append(line)
            if trimmed.startswith(fence):
                fence = None
            continue
        if trimmed.startswith("## "):
            title = trimmed[2:].strip().lower()
            if title in DELTA_TITLES:
                in_delta = True
                continue
            in_delta = False
        if not in_delta:
            out.append(line)
    return "".join(out)


def has_frontmatter(text: str) -> bool:
    first = next((l for l in text.splitlines() if l.strip()), "")
    return first.strip() == "---"


def migrate(root: Path, dry_run: bool = False) -> list[Path]:
    """Strip every dual-format deployed spec under root/openspec/specs/.

    Returns the files changed (or that WOULD change, for a dry run),
    sorted. Idempotent: an already-clean file is never written.
    """
    specs_dir = root / "openspec" / "specs"
    changed: list[Path] = []
    for spec in sorted(specs_dir.glob("*/spec.md")):
        text = spec.read_text()
        if not has_frontmatter(text):
            continue
        stripped = strip_delta_sections(text)
        if stripped == text:
            continue
        changed.append(spec)
        if not dry_run:
            spec.write_text(stripped)
    return changed


def main() -> int:
    argv = [a for a in sys.argv[1:]]
    dry_run = "--dry-run" in argv
    argv = [a for a in argv if a != "--dry-run"]
    root = Path(argv[0]) if argv else Path(".")
    changed = migrate(root, dry_run=dry_run)
    label = "would strip" if dry_run else "stripped"
    for path in changed:
        print(f"{label}: {path.relative_to(root)}")
    print(f"{len(changed)} file(s) {'to strip' if dry_run else 'stripped'}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
