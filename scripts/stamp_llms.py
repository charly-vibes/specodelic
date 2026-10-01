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

Two build-time phases (shared by docs.yml and `just docs-build` so the
CI path and the local path cannot drift):

  stamp_llms.py page   # before `mdbook build`: docs/src/release.md
  stamp_llms.py llms   # after  `mdbook build`: book/llms.txt (stamped copy)

Version comes from Cargo.toml (never hand-typed text — §6, DDL-j0u); the
ref/commit come from the environment (GITHUB_REF_NAME / GITHUB_SHA) with
git fallbacks for local builds.

stdlib only, no deps.
"""

from __future__ import annotations

import re
import subprocess
import sys
from pathlib import Path

RELEASE_URL = "https://charly-vibes.github.io/specodelic/release.html"


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
    )
    return source.rstrip("\n") + "\n" + stamp


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
    if len(argv) != 1 or argv[0] not in ("page", "llms"):
        raise SystemExit("usage: stamp_llms.py page|llms")
    phase = argv[0]
    version = crate_version(root)
    tag, sha = build_ref(root, tag, sha)
    if phase == "page":
        out = root / "docs" / "src" / "release.md"
        out.parent.mkdir(parents=True, exist_ok=True)
        out.write_text(release_page(version, tag, sha))
    else:
        out = root / "book" / "llms.txt"
        out.parent.mkdir(parents=True, exist_ok=True)
        out.write_text(
            stamped_llms((root / "llms.txt").read_text(), version, tag, sha)
        )


if __name__ == "__main__":
    main(sys.argv[1:])