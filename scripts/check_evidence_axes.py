#!/usr/bin/env python3
"""Evidence-axis drift guard (add-evidence-record-axes steps 2.x).

Purpose:
    Keep the normative five-axis evidence-record table in
    docs/src/graded-evidence.md in lockstep with the code closed sets it
    documents — conform's `Verdict`, audit's `AuditClass`, and the
    `AuditClass::of` mapping — failing CI when they drift.

Responsibilities:
    - Parse the two enums from src/conform.rs and src/audit.rs and every
      `Verdict::X =>` arm of `AuditClass::of`.
    - Parse the axis table under `## The evidence-record axes`.
    - Fail naming the divergent value when: a code value has no row; an
      `implemented` row has no code value; a documented mapping differs
      from the `of()` arm; a `reserved` row lacks its trigger or names a
      value the code already declares; a value appears twice; or any
      input is unparseable (fail-closed, never a silent pass).
    - Emit the counts compared on success.

Rationale:
    The vocabulary page is the source of truth the next change quotes;
    hand-maintained docs drifted before (the verdict list once omitted
    `agree`). Parse-and-compare follows the existing guard convention
    (check_pack_doc_counts.py, check_section_sync.py): a small stdlib
    script, fixtures in scripts/test_check_evidence_axes.py, wired into
    `just ci`.

Usage: check_evidence_axes.py [ROOT]   (default: the repo root)
Exit 0 = in sync; exit 1 = drift (each problem names its value and file).
"""

import re
import sys
from pathlib import Path

TABLE_HEADING = "## The evidence-record axes"
COLUMNS = ["Value", "Axis", "Code source", "Status", "of() mapping", "Trigger"]
CONFORM_REL = "src/conform.rs"
AUDIT_REL = "src/audit.rs"
DOCS_REL = "docs/src/graded-evidence.md"
NO_CELL = {"", "—", "-"}
REMEDIATION = (
    "fail-closed, never a silent pass — fix the file shape or update the "
    "guard if the format legitimately changed"
)


class Unparseable(Exception):
    """An input file could not be parsed into its closed set."""


def camel_to_snake(name: str) -> str:
    return re.sub(r"(?<!^)(?=[A-Z])", "_", name).lower()


def _enum_block(source: str, enum_name: str, label: str) -> str:
    """The `pub enum <Name> { ... }` block (brace line to closing `}`)."""
    lines = source.splitlines()
    start = next((i for i, l in enumerate(lines)
                  if re.match(rf"pub enum {enum_name}\b.*\{{", l.strip())), None)
    if start is None:
        raise Unparseable(
            f"{label}: cannot extract 'pub enum {enum_name}' — expected a "
            f"'pub enum {enum_name} {{' block with plain `Variant,` lines "
            f"(doc comments allowed); {REMEDIATION}")
    body: list[str] = []
    for line in lines[start + 1:]:
        if line.rstrip() == "}":
            break
        body.append(line)
    return "\n".join(body)


def parse_enum(source: str, enum_name: str, label: str) -> list[str]:
    """Variant names of `pub enum <Name>`, in declaration order."""
    names: list[str] = []
    for line in _enum_block(source, enum_name, label).splitlines():
        text = line.strip()
        if text and not text.startswith(("///", "//", "#[")):
            m = re.match(r"([A-Z][A-Za-z0-9_]*)[,;]?$", text)
            if m:
                names.append(m.group(1))
    if not names:
        raise Unparseable(
            f"{label}: 'pub enum {enum_name}' block parsed to zero variants "
            f"— expected `Variant,` lines; {REMEDIATION}")
    return names


def parse_of_mapping(source: str) -> dict[str, str]:
    """Verdict-to-class arms of `AuditClass::of`, keyed by snake_case verdict.

    Arms are matched by splitting the fn body on `=>`: the verdicts of
    arm i are the `Verdict::X` tokens before the i-th `=>`, its class is
    the first `AuditClass::Y` token after it. Grouped arms
    (`Verdict::A | Verdict::B => AuditClass::C`) map every listed
    verdict; `rank()`/`as_str()` arms outside `fn of` are never read.
    """
    lines = source.splitlines()
    start = next((i for i, l in enumerate(lines)
                  if "fn of(verdict: Verdict) -> AuditClass" in l), None)
    if start is None:
        raise Unparseable(
            "audit source: cannot extract 'AuditClass::of' — expected a "
            "'fn of(verdict: Verdict) -> AuditClass' match; " + REMEDIATION)
    body: list[str] = []
    for line in lines[start + 1:]:
        if line.rstrip() == "    }":  # fn-of closing brace (impl indent)
            break
        body.append(line)
    joined = " ".join(body)
    if "Verdict::" not in joined:
        raise Unparseable(
            "audit source: 'AuditClass::of' body has no 'Verdict::X =>' "
            f"arms — the mapping cannot be empty; {REMEDIATION}")
    segments = joined.split("=>")
    mapping: dict[str, str] = {}
    for i in range(len(segments) - 1):
        verdicts = re.findall(r"Verdict::([A-Za-z0-9_]+)", segments[i])
        classes = re.findall(r"AuditClass::([A-Za-z0-9_]+)", segments[i + 1])
        if verdicts and not classes:
            raise Unparseable(
                "audit source: of() arm for "
                f"{', '.join(verdicts)} has no 'AuditClass::Y' target "
                f"— every arm needs a class; {REMEDIATION}")
        for v in verdicts:
            mapping[camel_to_snake(v)] = camel_to_snake(classes[0])
    return mapping


def _cells(line: str) -> list[str]:
    return [c.strip() for c in line.split("|")[1:-1]]


def parse_table(docs: str) -> list[dict[str, str]]:
    """Rows of the normative axis table, as dicts over COLUMNS."""
    lines = docs.splitlines()
    start = next((i for i, l in enumerate(lines)
                  if l.strip() == TABLE_HEADING), None)
    if start is None:
        raise Unparseable(
            f"{DOCS_REL}: cannot find the '{TABLE_HEADING}' section — add "
            f"the normative axis table (columns: {', '.join(COLUMNS)}) under "
            f"that heading; {REMEDIATION}")
    rest = lines[start + 1:]
    while rest and not rest[0].lstrip().startswith("|"):
        rest = rest[1:]
    if not rest:
        raise Unparseable(
            f"{DOCS_REL}: no table follows '{TABLE_HEADING}' — expected a "
            f"markdown table with columns {', '.join(COLUMNS)}; {REMEDIATION}")
    header = [c.lower() for c in _cells(rest[0])]
    if header != [c.lower() for c in COLUMNS]:
        raise Unparseable(
            f"{DOCS_REL}: axis-table header is {header} — expected exactly "
            f"{[c.lower() for c in COLUMNS]}; {REMEDIATION}")
    rows: list[dict[str, str]] = []
    for line in rest[2:]:  # skip the `|---|` separator
        if not line.lstrip().startswith("|"):
            break
        cells = _cells(line)
        if len(cells) != len(COLUMNS):
            raise Unparseable(
                f"{DOCS_REL}: axis-table row has {len(cells)} cells, expected "
                f"{len(COLUMNS)}: {line.strip()}; {REMEDIATION}")
        row = dict(zip([c.lower() for c in COLUMNS], cells))
        row["value"] = row["value"].strip("`")
        row["code source"] = row["code source"].strip("`")
        row["of() mapping"] = row["of() mapping"].strip("`").strip()
        rows.append(row)
    return rows


def _mapping_cell(cls: str) -> str:
    return f"→ {cls}"


def _duplicate_problems(rows: list[dict[str, str]]) -> list[str]:
    counts: dict[str, int] = {}
    for row in rows:
        counts[row["value"]] = counts.get(row["value"], 0) + 1
    return [
        f"{DOCS_REL}: value '{value}' appears {n} times — every value gets "
        "exactly one row"
        for value, n in counts.items() if n > 1
    ]


def _implemented_problems(row: dict[str, str], root: Path,
                          of_map: dict[str, str]) -> list[str]:
    """Drift checks for one `implemented` row."""
    value, source = row["value"], row["code source"]
    if source not in (CONFORM_REL, AUDIT_REL):
        if not (root / source).exists():
            return [
                f"{DOCS_REL}: implemented row '{value}' names code source "
                f"'{source}' which does not exist in the tree — point the "
                "row at the file that carries the value"
            ]
        return []
    expected = of_map.get(value)
    cell = row["of() mapping"]
    if expected is None:
        return []
    if cell in NO_CELL:
        return [
            f"{DOCS_REL}: no documented mapping for '{value}' — the code "
            f"of() arm maps it to '{expected}'; document the mapping"
        ]
    if cell != _mapping_cell(expected):
        return [
            f"{DOCS_REL}: mapping drift for '{value}': table says "
            f"'{cell}', code of() says '{_mapping_cell(expected)}'"
        ]
    return []


def _reserved_problems(row: dict[str, str],
                       code_values: set[str]) -> list[str]:
    """Drift checks for one `reserved` row."""
    value = row["value"]
    problems: list[str] = []
    if row["trigger"].strip() in NO_CELL:
        problems.append(
            f"{DOCS_REL}: reserved row '{value}' has no trigger — name the "
            "change or ticket that will give the value a producer")
    if value in code_values:
        problems.append(
            f"{DOCS_REL}: reserved row '{value}' names a value already "
            "present in the code closed set — either the row's status or "
            "the code is wrong")
    return problems


def check(root: Path) -> tuple[list[str], str]:
    """Return (problems, ok_line); ok_line is '' whenever problems exist."""
    try:
        verdicts = [camel_to_snake(v) for v in
                    parse_enum((root / CONFORM_REL).read_text(encoding="utf-8"),
                               "Verdict", CONFORM_REL)]
        audit_text = (root / AUDIT_REL).read_text(encoding="utf-8")
        classes = [camel_to_snake(c) for c in
                   parse_enum(audit_text, "AuditClass", AUDIT_REL)]
        of_map = parse_of_mapping(audit_text)
        rows = parse_table((root / DOCS_REL).read_text(encoding="utf-8"))
    except Unparseable as exc:
        return [str(exc)], ""

    problems = _duplicate_problems(rows)
    code_sets = {CONFORM_REL: set(verdicts), AUDIT_REL: set(classes)}
    documented: dict[str, set[str]] = {CONFORM_REL: set(), AUDIT_REL: set()}
    for row in rows:
        status = row["status"]
        if status.startswith("implemented"):
            problems.extend(_implemented_problems(row, root, of_map))
            if row["code source"] in code_sets:
                documented[row["code source"]].add(row["value"])
                if row["value"] not in code_sets[row["code source"]]:
                    problems.append(
                        f"{DOCS_REL}: table row '{row['value']}' is marked "
                        f"implemented but no code enum in "
                        f"{row['code source']} declares it — fix the row or "
                        "the code")
        elif status.startswith("reserved"):
            problems.extend(_reserved_problems(row,
                                               set(verdicts) | set(classes)))
        else:
            problems.append(
                f"{DOCS_REL}: row '{row['value']}' status is '{status}' — "
                "status must be 'implemented' or 'reserved'")
    for rel, code_set in code_sets.items():
        for missing in sorted(code_set - documented[rel]):
            problems.append(
                f"{DOCS_REL}: code value '{missing}' declared in {rel} has "
                "no axis-table row — document it or fix the code")
    if problems:
        return problems, ""
    impl = sum(1 for r in rows if r["status"].startswith("implemented"))
    reserved = sum(1 for r in rows if r["status"].startswith("reserved"))
    ok = (f"OK: {len(verdicts)} verdicts, {len(classes)} audit classes, "
          f"{len(of_map)} of() arms vs {len(rows)} table rows "
          f"({impl} implemented, {reserved} reserved) — evidence axes in sync")
    return [], ok


def main(argv: list[str]) -> int:
    root = Path(argv[1]) if len(argv) > 1 else \
        Path(__file__).resolve().parent.parent
    problems, ok = check(root)
    for problem in problems:
        print(problem, file=sys.stderr)
    if problems:
        return 1
    print(ok)
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
