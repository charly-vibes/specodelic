#!/usr/bin/env python3
"""Lint the fenced spec examples embedded in the corpus docs.

Every ```markdown block whose first non-empty line is `---` (frontmatter
start) is a runnable format artifact — the USAGE.md quick-start, worked
examples in the spec files. These drifted once already (specodelic-vpx:
the quick-start failed the tool's own lint) because nothing executed
them. This script extracts each example, names it per the format's
naming law (`.` in id ⇔ `-` in filename stem, `_` literal), writes it to
a temp directory, and lints that directory with the freshly built spk.

Table fragments (no frontmatter) are prose-adjacent snippets and are
deliberately skipped — they are not standalone specs.

Exit codes: 0 = all examples lint clean; 1 = a finding, an example with
no parsable id, or no examples found at all (a silent no-op check is a
false green).
"""

import pathlib
import re
import subprocess
import sys
import tempfile

FENCE_RE = re.compile(r"^```([\w-]*)\s*$")
ID_RE = re.compile(r"^id:\s*['\"]?([A-Za-z0-9_.-]+)['\"]?\s*$")
MARKDOWN_LANGS = {"markdown", "md", ""}


def extract_spec_examples(text):
    """Return the bodies of fenced markdown blocks that start with frontmatter."""
    examples = []
    in_fence = False
    is_markdown = False
    body = []
    for line in text.splitlines():
        m = FENCE_RE.match(line)
        if m:
            if not in_fence:
                in_fence, is_markdown, body = True, m.group(1) in MARKDOWN_LANGS, []
            else:
                in_fence = False
                non_blank = [ln for ln in body if ln.strip()]
                if is_markdown and non_blank and non_blank[0].strip() == "---":
                    examples.append("\n".join(body))
            continue
        if in_fence:
            body.append(line)
    return examples


def frontmatter_id(body):
    """The `id:` scalar from a spec's frontmatter, or None."""
    lines = body.splitlines()
    if not lines or lines[0].strip() != "---":
        return None
    for line in lines[1:]:
        if line.strip() == "---":
            break
        m = ID_RE.match(line.strip())
        if m:
            return m.group(1)
    return None


def stem_for_id(spec_id):
    """The format's naming law: `-` in filename ⇔ `.` in id; `_` is literal."""
    return spec_id.replace(".", "-")


def main(argv):
    root = pathlib.Path(argv[1]) if len(argv) > 1 else pathlib.Path("specs")
    found = []
    for f in sorted(root.glob("*.md")):
        for i, body in enumerate(extract_spec_examples(f.read_text())):
            found.append((f, i, body))

    if not found:
        print(f"error: no fenced spec examples found in {root} — "
              "a no-op check is a false green; fix the directory or the extractor")
        return 1

    failures = []
    with tempfile.TemporaryDirectory() as td:
        td = pathlib.Path(td)
        for f, i, body in found:
            fid = frontmatter_id(body)
            if not fid:
                failures.append(f"{f}: example #{i + 1} has no parsable frontmatter id")
                continue
            (td / (stem_for_id(fid) + ".md")).write_text(body.rstrip() + "\n")

        if not failures:
            r = subprocess.run(
                ["cargo", "run", "-q", "--", "lint", str(td)],
                capture_output=True, text=True,
            )
            if r.returncode != 0:
                print(r.stdout)
                print(r.stderr, file=sys.stderr)
                failures.append(
                    "spk lint rejected embedded doc example(s) — "
                    "fix the examples in the docs, never the checker"
                )

    if failures:
        for msg in failures:
            print(f"FAIL: {msg}")
        return 1

    print(f"doc examples lint clean: {len(found)} example(s) from {root}/*.md")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
