#!/usr/bin/env python3
"""Anti-drift guard for doc pages claiming counts of `spk explain` topics.

`spk explain` serves an append-only `TOPICS` list from src/guide.rs.
When new topics land, the human-facing pages that name the served count
drift: README.md said "six" while index.md said "eight" and
installation.md said "seven" — all against a binary serving nine
(specodelic-lf4b.1). This guard parses the served topic ids from
src/guide.rs as source of truth and fails CI naming:

  1. any page whose stated topic count (digit or word: "six topics",
     "seven topics", "eight topics", "9 topics") != len(TOPICS);
  2. any page that fails to enumerate every served topic id — but only
     pages whose framing carries a topic enumeration (README.md,
     docs/src/index.md). docs/src/installation.md is COUNT-ONLY: it
     must state the served count and is never required to enumerate
     ids (installation.md is a cross-reference page — count-only);
  3. the word/digit count fallback itself: any count phrase that
     matches no TOPICS-derived count is a finding (rule 1 restated —
     there is no TOPICS-derived count other than len(TOPICS));
  4. any inline topic enumeration adjacent to the word "topics" — a
     slash-list immediately before it (README's original drift shape:
     "format/ears/kinds/.../lint-rules topics") or a backticked
     comma/slash-list immediately after it — that names at least one
     served id but omits another (a page can mention a new topic id
     elsewhere and still carry a stale inline list).

The guard is fail-closed: an unparseable or empty TOPICS block fails.

Usage: check_explain_topic_docs.py [ROOT]   (default: repo root)
Exit 0 = all pages agree with the binary; exit 1 = drift found.
"""

import re
import sys
from pathlib import Path

GUIDE_RS = "src/guide.rs"
# Per-page policy map: pages whose framing carries a topic enumeration
# must enumerate every served id ("enumerate"); count-only pages must
# state the served count and are NOT required to enumerate ids
# ("count-only" — installation.md is a cross-reference page).
PAGES = (
    ("README.md", "enumerate"),
    ("docs/src/index.md", "enumerate"),
    ("docs/src/installation.md", "count-only"),
)
POLICIES = ("enumerate", "count-only")

TOPICS_BLOCK_RE = re.compile(
    r"pub const TOPICS: &\[\(&str, &str\)\] = &\[(.*?)\];", re.S
)
# First element of each ("id", "title") tuple — second elements (the
# one-line titles) are skipped because the regex requires the `("id",`
# quote-comma shape.
TOPIC_TUPLE_ID_RE = re.compile(r'\(\s*"([a-z][a-z-]*)"\s*,')
# Prose counts of topics: digit or word counts, singular fallback words
# one..ten (COUNT_FALLBACK). "lint-rules topics" never matches — count
# words must immediately precede "topics".
COUNT_PHRASE_RE = re.compile(
    r"\b(\d+|one|two|three|four|five|six|seven|eight|nine|ten) topics\b",
    re.I,
)
COUNT_FALLBACK = {
    "one": 1, "two": 2, "three": 3, "four": 4, "five": 5,
    "six": 6, "seven": 7, "eight": 8, "nine": 9, "ten": 10,
}
# Inline topic enumerations adjacent to the word "topics". Before: a
# slash-separated id list ending in "topics" ("format/ears/... topics").
# After: "topics" followed by a colon/dash and a backticked comma- or
# slash-separated list ("nine topics (`format`, `ears`, ...)"). Both
# must name EVERY served id when they name any served id at all —
# non-topic word lists ("migrations/notes topics") are ignored.
ENUM_BEFORE_RE = re.compile(
    r"\b([a-z][a-z-]*(?:/[a-z][a-z-]*)+)\s+topics\b", re.I
)
ENUM_AFTER_RE = re.compile(
    r"\btopics\b\s*[:—-]?\s*\(?\s*`?([a-z][a-z-]*`?(?:\s*[,/]\s*`?[a-z][a-z-]*`?)+)`?\s*\)?",
    re.I,
)
ID_TOKEN_RE = re.compile(r"[a-z][a-z-]*", re.I)


def enum_problems(page: Path, text: str, topics: list[str]) -> list[str]:
    """Rule 4: adjacent inline enumerations must list every served id."""
    problems: list[str] = []
    served = set(topics)
    for regex in (ENUM_BEFORE_RE, ENUM_AFTER_RE):
        for m in regex.finditer(text):
            tokens = ID_TOKEN_RE.findall(m.group(1).lower())
            named = [t for t in tokens if t in served]
            if not named:
                continue  # an enumeration that is not about explain topics
            missing = sorted(served - set(named))
            if missing:
                problems.append(
                    f"{page}: inline topic enumeration '{m.group(1)}' names "
                    f"{', '.join(named)} but omits served topic(s) "
                    f"{', '.join(missing)} — enumerate all {len(topics)} "
                    "or drop the list"
                )
    return problems


def parse_topics(guide_src: str) -> list[str]:
    """Parse the served topic ids from src/guide.rs `TOPICS` (fail-closed)."""
    m = TOPICS_BLOCK_RE.search(guide_src)
    if m is None:
        raise ValueError(
            "src/guide.rs: TOPICS block not found — the guard cannot "
            "derive the served count (fail-closed: fix the regex or the "
            "const, never the pages)"
        )
    ids = TOPIC_TUPLE_ID_RE.findall(m.group(1))
    if not ids:
        raise ValueError(
            "src/guide.rs: TOPICS block parsed but no topic ids found "
            "(fail-closed: TOPICS is append-only and non-empty)"
        )
    return ids


def stated_count(count_word: str) -> int:
    return COUNT_FALLBACK[count_word.lower()] if count_word.lower() in COUNT_FALLBACK else int(count_word)


def check_page(page: Path, topics: list[str], policy: str) -> list[str]:
    if policy not in POLICIES:  # fail-closed on unknown policy
        raise ValueError(
            f"unknown page policy '{policy}' — must be one of "
            f"{', '.join(POLICIES)}"
        )
    problems: list[str] = []
    text = page.read_text(encoding="utf-8")
    n = len(topics)
    served = ", ".join(topics)
    for m in COUNT_PHRASE_RE.finditer(text):
        count_word = m.group(1)
        if stated_count(count_word) != n:
            problems.append(
                f"{page}: stated topic count '{count_word} topics' != "
                f"the {n} topics spk explain serves ({served})"
            )
    if policy == "enumerate":
        missing = [t for t in topics if t not in text]
        if missing:
            problems.append(
                f"{page}: missing served explain topics {', '.join(missing)} "
                f"— a page naming the topic count must enumerate all {n}"
            )
    problems.extend(enum_problems(page, text, topics))
    return problems


def check(root: Path) -> list[str]:
    problems: list[str] = []
    try:
        topics = parse_topics((root / GUIDE_RS).read_text(encoding="utf-8"))
    except ValueError as e:
        return [f"{GUIDE_RS}: {e}"]
    for rel, policy in PAGES:
        page = root / rel
        if not page.exists():
            problems.append(f"{rel}: doc page not found (guard checks a "
                            "fixed page set — a renamed page must be "
                            "re-listed in PAGES)")
            continue
        problems.extend(check_page(page, topics, policy))
    return problems


def main() -> int:
    root = Path(sys.argv[1]) if len(sys.argv) > 1 else Path(__file__).parent.parent
    problems = check(root)
    if problems:
        for p in problems:
            print(f"DRIFT: {p}", file=sys.stderr)
        print(
            f"check_explain_topic_docs: {len(problems)} problem(s) — "
            "doc pages must agree with src/guide.rs TOPICS (fix the pages, "
            "never the guard or the const)", file=sys.stderr,
        )
        return 1
    print(f"check_explain_topic_docs: OK — all pages agree with the "
          f"topics served by spk explain")
    return 0


if __name__ == "__main__":
    sys.exit(main())
