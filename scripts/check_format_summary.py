#!/usr/bin/env python3
"""Gate the hand-maintained copies of the format summary (specodelic-oeuz).

The canonical shape of specodelic's format is four layers, in order:
Intent, Constraints, Model, Properties. That summary is hand-copied into
the homepage layer table, the FAQ one-liner, llms.txt (and its generated
llm.txt mirror), and the mdBook description. The copies drift
independently: a table once said "four tables" while listing three, and
"frontmatter" repeatedly stood in for the Intent layer — naming where the
layer lives rather than the layer. This gate asserts every copy names the
four layers in order and never labels Intent by its location.

Link bullets ("Architecture — the four layers and the lint pipeline") name
a book page, not an enumeration of the layers, so they are references
rather than summary copies and are exempt from the twin check.

Stdlib-only and fail closed: a missing/unreadable copy, a missing expected
region, or a missing layer is a problem, never silence.

Usage: check_format_summary.py [REPO_ROOT]   (default: .)
Exit 0 = every copy agrees; exit 1 = drift (printed as problems).
"""

import re
import sys
from pathlib import Path

LAYERS = ("Intent", "Constraints", "Model", "Properties")

INDEX = Path("docs/src/index.md")
FAQ = Path("docs/src/faq.md")
LLM_TWINS = (Path("llms.txt"), Path("llm.txt"))
BOOK = Path("book.toml")

INDEX_HEADING = "## The four layers"

DESCRIPTION = re.compile(r'^\s*description\s*=\s*"(.*)"\s*$', re.MULTILINE)
SENTENCE_BREAK = re.compile(r"(?<=[.!?])\s+")
LINK_ENTRY = re.compile(r"^\s*[-*]\s*\[.*\]\([^)]*\)")


def _read(root: Path, rel: Path, problems: list[str]) -> str | None:
    """Read a summary copy, recording a fail-closed problem on failure."""
    try:
        return (root / rel).read_text(encoding="utf-8")
    except OSError as exc:
        problems.append(f"{rel}: missing or unreadable ({exc.__class__.__name__}: {exc})")
        return None


def _check_mentions(
    rel: Path,
    where: str,
    text: str,
    problems: list[str],
    forbid_frontmatter: bool,
) -> None:
    """Require all four layers, in order, in one summary snippet."""
    positions = {}
    for name in LAYERS:
        match = re.search(rf"\b{re.escape(name)}\b", text, re.IGNORECASE)
        if match is None:
            problems.append(f"{rel}: {where} is missing the {name} layer")
        else:
            positions[name] = match.start()

    if len(positions) == len(LAYERS):
        found_order = sorted(positions, key=positions.__getitem__)
        if found_order != list(LAYERS):
            problems.append(
                f"{rel}: {where} names the layers out of order "
                f"({', '.join(found_order)}); expected {', '.join(LAYERS)}"
            )

    if forbid_frontmatter and "frontmatter" in text.lower():
        problems.append(
            f"{rel}: {where} labels the Intent layer by its location "
            "('frontmatter'); name the layer instead"
        )


def _check_index(text: str, problems: list[str]) -> None:
    """The homepage table must list the four layers, in order, by name."""
    rel = str(INDEX)
    section = _index_section(text)
    if section is None:
        problems.append(
            f"{rel}: no '{INDEX_HEADING}' section (expected the four-layer table)"
        )
        return
    labels = _table_layer_labels(section)
    if not labels:
        problems.append(f"{rel}: '{INDEX_HEADING}' section has no layer table rows")
        return
    _check_layer_labels(rel, labels, problems)


def _index_section(text: str) -> list[str] | None:
    """The lines under the homepage's four-layer heading, or None."""
    lines = text.splitlines()
    start = None
    for i, line in enumerate(lines):
        if line.strip() == INDEX_HEADING:
            start = i + 1
            break
    if start is None:
        return None
    end = len(lines)
    for j in range(start, len(lines)):
        if lines[j].startswith("## "):
            end = j
            break
    return lines[start:end]


def _table_layer_labels(section: list[str]) -> list[str]:
    """First-column labels of the section's table rows, header/separator skipped."""
    labels = []
    for line in section:
        stripped = line.strip()
        if not stripped.startswith("|"):
            continue
        cells = [cell.strip() for cell in stripped.strip("|").split("|")]
        if not cells:
            continue
        first = cells[0]
        if not first or set(first) <= set("-: "):
            continue  # markdown separator row
        if first.lower() == "layer":
            continue  # header row
        labels.append(first)
    return labels


def _check_layer_labels(rel: str, labels: list[str], problems: list[str]) -> None:
    """Compare the table's labels to the canonical layer sequence."""
    for i, expected in enumerate(LAYERS):
        if i >= len(labels):
            problems.append(f"{rel}: layer table is missing the {expected} row")
            continue
        found = labels[i]
        if found == expected:
            continue
        if "frontmatter" in found.lower():
            problems.append(
                f"{rel}: layer row {i + 1} is labelled {found!r}, which names the "
                f"file location rather than the {expected} layer — drop 'frontmatter'"
            )
        else:
            problems.append(
                f"{rel}: layer row {i + 1} is {found!r}, expected {expected!r}"
            )
    for extra in labels[len(LAYERS):]:
        problems.append(f"{rel}: unexpected extra layer row {extra!r}")


def _check_faq(text: str, problems: list[str]) -> None:
    """The FAQ one-sentence summary must name all four layers."""
    rel = str(FAQ)
    sentences = SENTENCE_BREAK.split(" ".join(text.split()))
    hits = [sentence for sentence in sentences if "four-layer" in sentence.lower()]
    if not hits:
        problems.append(f"{rel}: no sentence mentions the 'four-layer' schema")
        return
    for sentence in hits:
        _check_mentions(
            FAQ, "the four-layer sentence", sentence, problems, forbid_frontmatter=False
        )


def _check_llm_twin(rel: Path, text: str, problems: list[str]) -> None:
    """Every 'four layers' summary block must name the four layers."""
    blocks: list[tuple[int, str]] = []
    for lineno, line in enumerate(text.splitlines(), 1):
        if blocks and line.startswith((" ", "\t")):
            blocks[-1] = (blocks[-1][0], f"{blocks[-1][1]} {line.strip()}")
        else:
            blocks.append((lineno, line))
    hits = 0
    for lineno, block in blocks:
        if "four layers" not in block.lower():
            continue
        if (
            LINK_ENTRY.match(block)
            and "frontmatter" not in block.lower()
            and not any(
                re.search(rf"\b{re.escape(name)}\b", block, re.IGNORECASE)
                for name in LAYERS
            )
        ):
            continue  # a page link title, not an enumeration of the layers
        hits += 1
        _check_mentions(rel, f"line {lineno}", block, problems, forbid_frontmatter=True)
    if hits == 0:
        problems.append(f"{rel}: no line mentions the 'four layers'")


def _check_book(text: str, problems: list[str]) -> None:
    """The mdBook description must name all four layers, not frontmatter."""
    rel = str(BOOK)
    match = DESCRIPTION.search(text)
    if match is None:
        problems.append(f"{rel}: no 'description = \"...\"' in [book]")
        return
    _check_mentions(BOOK, "the description", match.group(1), problems, forbid_frontmatter=True)


def check(root: Path) -> list[str]:
    """Return human-readable problems for drifted format-summary copies."""
    root = Path(root)
    problems: list[str] = []

    text = _read(root, INDEX, problems)
    if text is not None:
        _check_index(text, problems)

    text = _read(root, FAQ, problems)
    if text is not None:
        _check_faq(text, problems)

    for rel in LLM_TWINS:
        text = _read(root, rel, problems)
        if text is not None:
            _check_llm_twin(rel, text, problems)

    text = _read(root, BOOK, problems)
    if text is not None:
        _check_book(text, problems)

    return problems


def main(argv: list[str] | None = None) -> int:
    args = sys.argv[1:] if argv is None else argv
    root = Path(args[0]) if args else Path(".")
    problems = check(root)
    for problem in problems:
        print(problem)
    return 1 if problems else 0


if __name__ == "__main__":
    sys.exit(main())