#!/usr/bin/env python3
"""Stamp llms.txt and generate the Release Status page at docs build time.

specodelic-2m7: the deployed site rebuilds from main, but llms.txt used to
carry no version identifier — an agent writing specs from the deployed
grammar could target a format newer than its installed spk crate, and lint
failures were then undiagnosable. The book build therefore stamps the
deployed llms.txt copy with the crate version + build ref and links the
generated Release Status page. The repo-source llms.txt is NEVER touched
(it stays the static, versionless input); the stamp lands only in the
build output (book/llms.txt).

Three build-time phases (shared by docs.yml and `just docs-build` so the
CI path and the local path cannot drift):

  stamp_llms.py page   # before `mdbook build`: docs/src/release.md
  stamp_llms.py llms   # after  `mdbook build`: book/llms.txt (stamped copy)
  stamp_llms.py root   # repo-root llm.txt, from llms.txt + SUMMARY.md

specodelic-lf4b.11 — why `root` exists: llm.txt (repo-root narrative
summary, required by specodelic-mvl) used to be hand-maintained and
silently fell behind llms.txt (missing graph-views, dual-format, hooks,
archive-companion). Two similarly-named summaries with divergent content
is a retrieval/RAG hazard. `root` derives llm.txt AT STAMP TIME: the body
is the repo-source llms.txt verbatim (minus its title), plus a header
naming both files (an agent grabbing either can tell them apart), plus a
book-page map parsed from docs/src/SUMMARY.md so prose pages (including
graph-views) surface too. llm.txt is therefore generated — never
hand-edit; run `just stamp-artifacts`.

specodelic-lf4b.11 — release.md mechanism (commit-stamp, chosen):
docs/src/release.md is tracked in git, so repo browsers see the checked-in
copy, but it was only refreshed by a docs build — between builds it could
lag the crate version. Choice: commit-stamp at release time. release.yml
(tag push = release authorization) runs `page` + `root` and commits both
to main after the release ships; `just stamp-artifacts` is the local
equivalent for drift between releases. The placeholder alternative was
rejected: the checked-in file IS the mdbook source for the page phase, so
a placeholder is overwritten by every docs build and keeps the tree dirty
— it fights the existing flow. Version always comes from Cargo.toml
(never hand-typed text — §6, DDL-j0u); the ref/commit come from the
environment (GITHUB_REF_NAME / GITHUB_SHA) with git fallbacks for local
builds.

stdlib only, no deps.
"""

from __future__ import annotations

import os
import re
import subprocess
import sys
from pathlib import Path

RELEASE_URL = "https://charly-vibes.github.io/specodelic/release.html"
LLMS_URL = "https://charly-vibes.github.io/specodelic/llms.txt"
BOOK_URL = "https://charly-vibes.github.io/specodelic/"


def crate_version(root: Path) -> str:
    """First `version = "..."` in Cargo.toml — the single source of truth."""
    text = (root / "Cargo.toml").read_text()
    m = re.search(r'^version\s*=\s*"([^"]+)"', text, re.MULTILINE)
    if not m:
        raise SystemExit("error: no `version = \"...\"` found in Cargo.toml")
    return m.group(1)


def _git(*args: str) -> str | None:
    try:
        r = subprocess.run(
            ["git", *args], capture_output=True, text=True, check=True
        )
        return r.stdout.strip()
    except (subprocess.CalledProcessError, FileNotFoundError):
        return None


def build_ref(root: Path, tag: str | None, sha: str | None) -> tuple[str, str]:
    """(short-ref, short-sha) for the stamp; env first, git fallback second."""
    tag = tag or _git("rev-parse", "--abbrev-ref", "HEAD") or "main"
    full_sha = sha or _git("rev-parse", "HEAD") or "unknown"
    return tag, full_sha[:7]


def stamped_llms(source: str, version: str, tag: str, sha: str) -> str:
    """Static llms.txt body + appended Release Status stamp section."""
    stamp = (
        f"\n## Release Status\n\n"
        f"Deployed site rebuilds from main; installed crates lag or lead this\n"
        f"grammar — check the stamp before writing specs against it.\n\n"
        f"- **Version:** v{version}\n"
        f"- **Built from:** {tag} @ {sha}\n"
        f"- [Release Status]({RELEASE_URL})\n"
        f"- A similarly named repo-root `llm.txt` summary is regenerated\n"
        f"  from this file — check the filename before citing.\n"
    )
    return source.rstrip("\n") + "\n" + stamp


def book_pages(summary: str) -> list[tuple[str, str]]:
    """Human-facing (title, source-path) pages from docs/src/SUMMARY.md.

    specs/ and openspec/ entries are excluded — llms.txt already links the
    spec pages; this maps the book's prose pages, which go stale
    invisibly. release.md is excluded: it is the generated stamp target
    itself.
    """
    pages: list[tuple[str, str]] = []
    for m in re.finditer(r"\[([^\]]+)\]\(([^)]+\.md)\)", summary):
        title, path = m.group(1), m.group(2)
        if path.startswith(("specs/", "openspec/")) or path.endswith(
            "release.md"
        ):
            continue
        pages.append((title, path))
    return pages


def narrative_llm(llms_source: str, summary: str) -> str:
    """Repo-root llm.txt derived from llms.txt + the book SUMMARY.

    Divergence-proofing (specodelic-lf4b.11): the body is the repo-source
    llms.txt verbatim (minus its H1), so llm.txt can never silently drift
    from llms.txt; the header names both files and links the deployed
    stamped copy; the book-page map (SUMMARY.md) adds the prose pages —
    including graph-views — that llms.txt's spec links do not cover.
    """
    body = llms_source.lstrip("\n")
    if body.startswith("# specodelic\n"):
        body = body[len("# specodelic\n") :].lstrip("\n")
    header = (
        "# specodelic\n\n"
        "> **`llm.txt` — repo-root machine summary, regenerated by\n"
        "> `scripts/stamp_llms.py`; do not hand-edit — run `just\n"
        "> stamp-artifacts`.** Content is derived from the repo-source\n"
        f"> `llms.txt` (identical body). The deployed, version-stamped copy\n"
        f"> lives at {LLMS_URL} — cite that one when the version matters.\n\n"
    )
    out = header + body
    pages = book_pages(summary)
    if pages:
        lines = ["## Docs book — prose pages", ""]
        for title, path in pages:
            html = path[:-3] + ".html"
            lines.append(f"- [{title}]({BOOK_URL}{html})")
        out = out.rstrip("\n") + "\n\n" + "\n".join(lines) + "\n"
    return out


def release_page(version: str, tag: str, sha: str) -> str:
    """The generated Release Status book page (docs/src/release.md)."""
    return (
        "# Release Status\n\n"
        "> **Generated artifact** (from Cargo.toml + git ref).\n"
        "> Do not hand-edit — the repo source files remain authoritative.\n\n"
        f"- **Version:** v{version}\n"
        f"- **Built from:** {tag} @ {sha}\n"
    )


def main(argv: list[str], root: Path | None = None, tag: str | None = None,
         sha: str | None = None) -> None:
    root = (root or Path(__file__).parent.parent).resolve()
    if len(argv) != 1 or argv[0] not in ("page", "llms", "root"):
        raise SystemExit("usage: stamp_llms.py page|llms|root")
    phase = argv[0]
    version = crate_version(root)
    # Environment first (CI names the real ref/tag), explicit arg second,
    # git fallback last — the module docstring's contract.
    tag = tag or os.environ.get("GITHUB_REF_NAME")
    sha = sha or os.environ.get("GITHUB_SHA")
    tag, sha = build_ref(root, tag, sha)
    if phase == "page":
        out = root / "docs" / "src" / "release.md"
        out.parent.mkdir(parents=True, exist_ok=True)
        out.write_text(release_page(version, tag, sha))
    elif phase == "root":
        summary = root / "docs" / "src" / "SUMMARY.md"
        out = root / "llm.txt"
        out.write_text(
            narrative_llm((root / "llms.txt").read_text(), summary.read_text())
        )
    else:
        out = root / "book" / "llms.txt"
        out.parent.mkdir(parents=True, exist_ok=True)
        out.write_text(
            stamped_llms((root / "llms.txt").read_text(), version, tag, sha)
        )


if __name__ == "__main__":
    main(sys.argv[1:])