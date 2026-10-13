#!/usr/bin/env python3
"""One-time migration: move the specs/ corpus into the openspec/specs single
tree (specodelic-jf1, mcy rollout steps 3+5).

Transform (per corpus file specs/<name>.md -> openspec/specs/<name>/spec.md),
under specodelic format Revision 18 (naming law):

  1. the frontmatter id is kept verbatim — a spec.md file derives its
     expected id from its PARENT DIRECTORY (`-` <-> `.` mapping), and the
     corpus naming law already matches (openspec/specs/linter-graph_shape/spec.md declares
     id: linter.graph_shape; deployed dir linter-graph_shape/spec.md expects
     the same). A mismatch is a loud failure, never a silent rewrite.
  2. no link rewriting or cross-file downgrades — Revision 18 retired the
     `id: spec` self-containment; wiki-refs resolve corpus-wide, so
     [[compile.c1]] and [[linter.graph_shape]] stay literal.
  3. a ## Requirements mirror is generated from the Properties table:
     one Scenario per property, each carrying a
     - **VERIFIES** [[<own-id>.<property-id>]] bullet (ah sync coverage link)

Divergence policy (specodelic-jf1): the corpus is the source of truth —
deployed capability specs backed by a corpus file (compile, packs) are
OVERWRITTEN by the migrated corpus version. Deployed capability specs
without a corpus counterpart are untouched.

Idempotent: running twice produces the same output (the generated
## Purpose and ## Requirements sections are detected and not duplicated).

Non-spec prose companions (AGENTS.md, STATUS.md, USAGE.md, CHANGELOG.md,
theory.md, *.checklist.md) are NOT transformed here — they are relocated
by hand alongside this script's run (no frontmatter, exempt from the
naming law).
"""

import re
import sys
import pathlib

LINK = re.compile(r"\[\[([a-zA-Z0-9_.\-]+)\]\]")

CORPUS = pathlib.Path("specs")
OUT_ROOT = pathlib.Path("openspec/specs")


def parse_row(line):
    return [c.strip() for c in line.strip().strip("|").split("|")]


def is_sep(cells):
    return all(set(c) <= set(":- ") and c for c in cells)


def properties_rows(lines):
    """Parse the ## Properties table into a list of cell dicts."""
    rows = []
    cols = None
    in_props = False
    for line in lines:
        stripped = line.strip()
        in_props = in_props or stripped == "## Properties"
        if not in_props:
            continue
        if stripped.startswith("## ") and stripped != "## Properties":
            break
        if not stripped.startswith("|"):
            continue
        cells = parse_row(stripped)
        if is_sep(cells):
            continue
        if cols is None:
            cols = [c.lower() for c in cells]
            continue
        rows.append(dict(zip(cols, cells)))
    return rows


def requirement_block(prop, own):
    pid = prop["id"]
    gen = prop.get("generator") or "the documented inputs"
    pred = prop.get("predicate") or ""
    return [
        f"#### Scenario: {pid}",
        "",
        f"- **WHEN** {gen}",
        f"- **THEN** {pred}",
        f"- **VERIFIES** [[{own}.{pid}]]",
        "",
    ]


def purpose_section(text):
    """Insert a ## Purpose section from the prose block after the H1 title.

    openspec strict validation requires "## Purpose" before "## Requirements".
    The corpus keeps a prose paragraph directly under the H1; promote it.
    Idempotent: skipped when a ## Purpose heading already exists.
    """
    if re.search(r"^## Purpose\s*$", text, flags=re.M):
        return text
    lines = text.splitlines()
    try:
        h1 = next(i for i, l in enumerate(lines) if l.startswith("# "))
    except StopIteration:
        return text
    first_h2 = next((i for i, l in enumerate(lines[h1:], h1) if l.startswith("## ")), len(lines))
    prose = [l for l in lines[h1 + 1:first_h2] if l.strip()]
    if not prose:
        prose = ["TBD — see the frontmatter statement."]
    block = ["## Purpose", ""] + prose + [""]
    return "\n".join(lines[:first_h2] + block + lines[first_h2:]) + "\n"


def requirements_mirror(text, own):
    if re.search(r"^## Requirements\s*$", text, flags=re.M):
        return text
    mirror = [
        "## Requirements",
        "",
        "### Requirement: Property coverage mirror",
        "",
        "Every property row SHALL be verified by exactly one dedicated scenario;",
        "`ah sync` SHALL derive one contract per VERIFIES link.",
        "",
    ]
    for prop in properties_rows(text.splitlines()):
        mirror += requirement_block(prop, own)
    return text.rstrip("\n") + "\n\n" + "\n".join(mirror) + "\n"


def expected_id(stem):
    """Naming law: filename stem with `-` mapped to `.` (`_` is literal)."""
    return stem.replace("-", ".")


def has_frontmatter(path):
    return path.read_text().startswith("---\n")


def migrate_one(path, out_root, dry_run):
    text = path.read_text()
    own = re.search(r"^id:\s*(\S+)", text, re.M).group(1)
    expected = expected_id(path.stem)
    if own != expected:
        sys.exit(f"migrate: {path}: frontmatter id {own!r} != naming-law id {expected!r} — fix the corpus, do not rewrite ids on migration")
    dest = out_root / path.stem / "spec.md"
    if dry_run:
        action = "overwrite" if dest.exists() else "create"
        print(f"would {action}: {dest} (own id {own})")
        return
    dest.parent.mkdir(parents=True, exist_ok=True)
    dest.write_text(requirements_mirror(purpose_section(text), own))
    action = "overwrote" if dest.exists() else "created"
    print(f"migrated {path.name} -> {dest} (own id {own})")


def main():
    dry_run = "--dry-run" in sys.argv
    corpus_files = sorted(p for p in CORPUS.glob("*.md") if has_frontmatter(p))
    if not corpus_files:
        sys.exit("migrate: no frontmatter-bearing corpus files found under specs/")
    for path in corpus_files:
        migrate_one(path, OUT_ROOT, dry_run)
    print(f"{len(corpus_files)} corpus file(s) {'(dry run)' if dry_run else 'migrated'}")


if __name__ == "__main__":
    sys.exit(main())
