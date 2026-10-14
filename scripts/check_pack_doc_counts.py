#!/usr/bin/env python3
"""Anti-drift guard for doc pages claiming pack-inventory counts.

The pack inventory is hardcoded prose in six user-facing surfaces and
has drifted twice on the same day: README.md said "four standard packs"
while docs/src/commands.md said "the three in-repo standard packs" and
docs/src/index.md said "Five packs ship in-repo — the four standard
packs" (fd2fbe0 fixed all six by hand; specodelic-typ0). This guard
parses the pack set from packs/*.md frontmatter as source of truth and
fails CI naming:

  1. any count phrase about packs on an audited surface whose number
     matches no derived count — "N standard packs" must equal the
     standard-pack count, "N packs ship in-repo" / "N ship in-repo"
     must equal the total pack count;
  2. any audited surface that fails to name every pack id it is
     required to enumerate — backticked id (`data.lineage`) or
     packs/<stem>.md file link both count. Policy is per-page:
     "enumerate" surfaces must name every pack; "standard-only"
     surfaces may omit the pilot ids (bioimage.data is the D6 pilot —
     a historical artifact, NOT structurally derivable: quant.finance
     also carries cross-pack Requires deps but is a standard pack);
  3. fail-closed: a missing packs dir, zero kind:profile files, or a
     pack file whose frontmatter carries no id is a finding — never a
     silent pass;
  4. llm.txt freshness (EDGE-002): the committed llm.txt must equal
     stamp_llms.narrative_llm(llms.txt, docs/src/SUMMARY.md). The
     generated narrative drifting from its source is the same failure
     class as a hand-edited count; the remediation is `just
     stamp-artifacts` (or `python3 scripts/stamp_llms.py root`), never
     a hand-edit.

Usage: check_pack_doc_counts.py [ROOT]   (default: repo root)
Exit 0 = all surfaces agree with the pack set; exit 1 = drift found.
"""

import re
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))

PACKS_DIR = "packs"
LLMS_TXT = "llms.txt"
LLM_TXT = "llm.txt"
SUMMARY_MD = "docs/src/SUMMARY.md"

# Per-page policy: pages whose framing carries a pack enumeration must
# name every pack id ("enumerate"); pages scoped to the standard packs
# ("standard-only") must name every non-pilot id and may omit pilots.
PAGES = (
    ("README.md", "enumerate"),
    ("docs/src/index.md", "enumerate"),
    ("docs/src/faq.md", "enumerate"),
    ("docs/src/commands.md", "standard-only"),
    ("llms.txt", "enumerate"),
)
POLICIES = ("enumerate", "standard-only")

# The bioimage D6 pilot is the only pack excluded from "standard pack"
# counts. Hardcoded deliberately: there is no structural discriminator
# (quant.finance carries cross-pack Requires deps yet is standard), so
# derivation would fabricate precision. Update this set when a new
# pilot-class pack lands or the pilot graduates.
PILOT_PACK_IDS = frozenset({"bioimage.data"})

WORD_NUMBERS = {
    "one": 1, "two": 2, "three": 3, "four": 4, "five": 5,
    "six": 6, "seven": 7, "eight": 8, "nine": 9, "ten": 10,
    "eleven": 11, "twelve": 12,
}
_NUMBER = r"(one|two|three|four|five|six|seven|eight|nine|ten|eleven|twelve|\d+)"

# "the five standard packs", "five in-repo standard packs"
STANDARD_COUNT_RE = re.compile(
    rf"\b{_NUMBER}\s+(?:[\w-]+\s+)?standard packs", re.IGNORECASE
)
# "Six packs ship in-repo", "Six ship in-repo"
TOTAL_COUNT_RE = re.compile(
    rf"\b{_NUMBER}\s+(?:packs?\s+)?ship in-repo", re.IGNORECASE
)


def _parse_number(token: str) -> int:
    return int(token) if token.isdigit() else WORD_NUMBERS[token.lower()]


def pack_ids(root: Path) -> list[tuple[str, Path]]:
    """(id, file) for every kind: profile pack under packs/, sorted."""
    packs_dir = root / PACKS_DIR
    if not packs_dir.is_dir():
        raise FileNotFoundError(f"no {PACKS_DIR}/ directory under {root}")
    found: list[tuple[str, Path]] = []
    for path in sorted(packs_dir.glob("*.md")):
        text = path.read_text()
        fm = re.match(r"^---\n(.*?)\n---", text, re.DOTALL)
        if not fm:
            raise ValueError(f"{PACKS_DIR}/{path.name}: no frontmatter block")
        id_m = re.search(r"^id:\s*(\S+)", fm.group(1), re.MULTILINE)
        if not id_m:
            raise ValueError(f"{PACKS_DIR}/{path.name}: frontmatter has no id")
        kind_m = re.search(r"^kind:\s*(\S+)", fm.group(1), re.MULTILINE)
        if not kind_m or kind_m.group(1) != "profile":
            continue  # non-pack spec files under packs/ are not inventory
        found.append((id_m.group(1), path))
    if not found:
        raise ValueError(f"no kind: profile pack files found under {PACKS_DIR}/")
    return found


def _id_mentioned(text: str, pack_id: str) -> bool:
    """True when the page names the pack: backticked id or file link."""
    stem = pack_id.replace(".", "-")
    return (
        f"`{pack_id}`" in text
        or f"{PACKS_DIR}/{stem}.md" in text
    )


def _llm_freshness_finding(root: Path) -> str | None:
    """EDGE-002: committed llm.txt must equal its derivation inputs."""
    llms = root / LLMS_TXT
    summary = root / SUMMARY_MD
    llm = root / LLM_TXT
    if not llms.is_file() or not summary.is_file():
        return None  # not this repo's layout; counts check still applies
    import stamp_llms

    expected = stamp_llms.narrative_llm(llms.read_text(), summary.read_text())
    if not llm.is_file():
        return (
            f"{LLM_TXT}: missing — the repo-root narrative summary is "
            f"generated from {LLMS_TXT} + {SUMMARY_MD}\n"
            f"  remediation: run `python3 scripts/stamp_llms.py root` "
            f"(or `just stamp-artifacts`), never hand-edit"
        )
    actual = llm.read_text()
    if actual == expected:
        return None
    return (
        f"{LLM_TXT}: stale — differs from narrative_llm({LLMS_TXT}, "
        f"{SUMMARY_MD}); a hand-edited or un-regenerated copy is the "
        f"same drift class as a stale count\n"
        f"  remediation: run `python3 scripts/stamp_llms.py root` "
        f"(or `just stamp-artifacts`), never hand-edit"
    )


def _count_findings(text: str, rel: str, standard_n: int, total_n: int) -> list[str]:
    """Count-phrase findings: every claimed count must match a derived one."""
    out: list[str] = []
    for match in STANDARD_COUNT_RE.finditer(text):
        n = _parse_number(match.group(1))
        if n == standard_n:
            continue
        out.append(
            f"{rel}: '{match.group(0)}' says {n} standard packs "
            f"but packs/ frontmatter derives {standard_n} "
            f"(total {total_n} minus pilots {sorted(PILOT_PACK_IDS)})\n"
            f"  remediation: update the count to {standard_n} "
            f"and name every standard pack id"
        )
    for match in TOTAL_COUNT_RE.finditer(text):
        n = _parse_number(match.group(1))
        if n == total_n:
            continue
        out.append(
            f"{rel}: '{match.group(0)}' says {n} packs ship "
            f"in-repo but packs/ frontmatter derives {total_n}\n"
            f"  remediation: update the count to {total_n} "
            f"and name every pack id"
        )
    return out


def _missing_required_id(text: str, pack_id: str, rel: str, policy: str) -> str | None:
    """One enumeration finding: a required pack id the page never names."""
    if _id_mentioned(text, pack_id):
        return None
    scope = "pack" if policy == "enumerate" else "standard pack"
    return (
        f"{rel}: pack `{pack_id}` is not named (policy "
        f"'{policy}' requires every {scope} id — backticked id or "
        f"packs/<stem>.md link)\n"
        f"  remediation: add `{pack_id}` to this surface's pack inventory"
    )


def _surface_findings(
    path: Path, rel: str, policy: str,
    all_ids: list[str], standard_ids: list[str],
    standard_n: int, total_n: int,
) -> list[str]:
    """Count + enumeration findings for one audited surface."""
    if not path.is_file():
        return [
            f"{rel}: audited pack-inventory surface is missing\n"
            f"  remediation: create it or drop it from PAGES in "
            f"scripts/check_pack_doc_counts.py"
        ]
    text = path.read_text()
    findings = _count_findings(text, rel, standard_n, total_n)
    required = all_ids if policy == "enumerate" else standard_ids
    for pack_id in required:
        finding = _missing_required_id(text, pack_id, rel, policy)
        if finding:
            findings.append(finding)
    return findings


def check(root: Path) -> list[str]:
    """All findings for the tree at root; empty list means clean."""
    try:
        packs = pack_ids(root)
    except (FileNotFoundError, ValueError) as exc:
        return [f"packs inventory unreadable (fail-closed): {exc}"]

    all_ids = [pid for pid, _ in packs]
    standard_ids = [pid for pid in all_ids if pid not in PILOT_PACK_IDS]
    standard_n, total_n = len(standard_ids), len(all_ids)

    findings: list[str] = []
    for rel, policy in PAGES:
        findings.extend(
            _surface_findings(
                root / rel, rel, policy,
                all_ids, standard_ids, standard_n, total_n,
            )
        )

    llm_finding = _llm_freshness_finding(root)
    if llm_finding:
        findings.append(llm_finding)

    return findings


def main(argv: list[str]) -> int:
    root = Path(argv[1]).resolve() if len(argv) > 1 else Path.cwd()
    findings = check(root)
    if not findings:
        print(
            f"check_pack_doc_counts: OK — pack inventory agrees with "
            f"{PACKS_DIR}/ across all audited surfaces"
        )
        return 0
    for finding in findings:
        print(finding, file=sys.stderr)
    print(
        f"check_pack_doc_counts: {len(findings)} finding(s) — pack-count "
        f"prose drifted from packs/ frontmatter",
        file=sys.stderr,
    )
    return 1


if __name__ == "__main__":
    sys.exit(main(sys.argv))
