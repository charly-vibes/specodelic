#!/usr/bin/env python3
"""Re-run the docs' captured quickstart commands and diff against the docs.

Purpose: every fenced ```text block in docs/src/installation.md and
docs/src/examples/worked-example.md claims to be verbatim captured output.
Captured output rots as the tool evolves (the 7-vs-9 explain-topics drift
is the existence proof); the sibling check_doc_examples.py gate covers
fenced SPEC examples, this gate is the complement: it re-executes each
documented command with the fresh binary and diffs real output against
the doc's expected block, failing on drift (doc file + command named).

Responsibilities: extract (command, expected-output) fence pairs adjacent
in each doc; replay each quickstart_manifest scenario in a throwaway dir
(fixtures under docs/fixtures/doc-outputs supply the worked-example's
intermediate spec files); force deterministic human format (--human, no
TTY detection, merged stdout+stderr, offline); diff verbatim
(whitespace-normalized, cargo chatter for the verify scratch ignored —
see _CHATTER); emit a non-failing version advisory (docs X vs binary Y) —
the hard failure is reserved for semantic drift.

Rationale: expected outputs live only in the docs (parsed live); scenario
inputs are checked in because the worked example's intermediate states
cannot be reconstructed from fragments. Never hand-edit an expected block
to go green: re-capture from the real binary instead.

Exit codes: 0 = every documented output matches; 1 = drift, an
unresolvable scenario (fence gone from the doc), or a replay error.
"""

import argparse, difflib, os, re, shlex, shutil, subprocess, sys, tempfile
from pathlib import Path

from quickstart_manifest import (
    FINAL_SPEC,
    INSTALLATION,
    MUTATIONS,
    WORKED,
    scenario_table,
    spk_lines,
)

ROOT = Path(__file__).resolve().parent.parent
FENCE_RE = re.compile(r"^```(\w*)\s*$")
VERSION_CLAIM_RE = re.compile(r"`specodelic (\d+\.\d+\.\d+)`")


def extract_pairs(text):
    """Return [(sh_fence_body, expected_text)] for adjacent sh→text fences.

    A pair is a ```sh fence whose next fence (blank lines allowed) is a
    ```text fence. Fences with other languages or without a text partner
    are skipped. Multi-command sh fences keep their full body; the
    scenario replays only the `spk` lines.
    """
    lines = text.splitlines()
    pairs = []
    i = 0
    while i < len(lines):
        m = FENCE_RE.match(lines[i])
        if not m or m.group(1) != "sh":
            i += 1
            continue
        body, i = read_fence_body(lines, i + 1)
        j = skip_blanks(lines, i + 1)
        m2 = FENCE_RE.match(lines[j]) if j < len(lines) else None
        if not m2 or m2.group(1) != "text":
            continue
        expected, j = read_fence_body(lines, j + 1)
        pairs.append(("\n".join(body).strip(), "\n".join(expected).strip()))
        i = j + 1
    return pairs


def extract_text_blocks(text):
    """Return every fenced ```text block's content, in document order."""
    lines = text.splitlines()
    blocks = []
    i = 0
    while i < len(lines):
        m = FENCE_RE.match(lines[i])
        if not m or m.group(1) != "text":
            i += 1
            continue
        body, i = read_fence_body(lines, i + 1)
        blocks.append("\n".join(body))
    return blocks


def read_fence_body(lines, start):
    """Return (body_lines, index_of_closing_fence) reading from a fence interior."""
    body = []
    i = start
    while i < len(lines) and not FENCE_RE.match(lines[i]):
        body.append(lines[i])
        i += 1
    return body, i


def skip_blanks(lines, start):
    i = start
    while i < len(lines) and lines[i].strip() == "":
        i += 1
    return i


def pair_index(pairs):
    """Map fence body → list of expected outputs in document order."""
    idx = {}
    for sh, expected in pairs:
        idx.setdefault(sh, []).append(expected)
    return idx


def doc_text_blocks(root):
    """All ```text blocks per doc path, resolved eagerly (two docs only)."""
    return {
        root / rel: extract_text_blocks((root / rel).read_text(encoding="utf-8"))
        for rel in (INSTALLATION, WORKED)
    }


def resolve_expected(root, table):
    """Attach each scenario's expected output from its doc; return the first
    unresolvable scenario or None."""
    pair_idxs, text_blocks = {}, doc_text_blocks(root)
    for sc in table:
        doc_path = root / sc["doc"]
        if "fence" in sc:
            if doc_path not in pair_idxs:
                pairs = extract_pairs(doc_path.read_text(encoding="utf-8"))
                pair_idxs[doc_path] = pair_index(pairs)
            idx = pair_idxs[doc_path]
            if sc["fence"] not in idx or sc["occ"] >= len(idx[sc["fence"]]):
                return sc
            sc["expected"] = idx[sc["fence"]][sc["occ"]]
        else:
            matches = [b for b in text_blocks[doc_path]
                       if normalize(b) and normalize(b)[0] == sc["first_line"]]
            if len(matches) != 1:
                return sc
            sc["expected"] = matches[0]
    return None


def run_command(spk, cmd, cwd, env):
    """Run one documented command with the real binary; return (merged_output, code).

    The docs capture terminal output — stdout and stderr interleaved (the
    `→ Run:` hints print on stderr), so the replay merges both streams.
    """
    tokens = shlex.split(cmd)
    if tokens[0] not in ("spk", "specodelic"):
        raise ValueError(f"not a spk command: {cmd!r}")
    argv = [spk] + tokens[1:]
    if "--human" not in argv and "-j" not in argv and "--json" not in argv:
        argv.append("--human")
    proc = subprocess.run(argv, cwd=cwd, env=env, text=True,
                          stdout=subprocess.PIPE, stderr=subprocess.STDOUT)
    return proc.stdout, proc.returncode


def replay(spk, sc, env):
    """Replay one scenario in a scratch dir; return its actual merged output."""
    with tempfile.TemporaryDirectory(prefix="doc-examples-") as tmp:
        if sc["fixture"]:
            source = root_relative(sc["fixture"])
            shutil.copy(source, Path(tmp) / source.name)
        for cmd in sc["setup"]:
            _, code = run_command(spk, cmd, tmp, env)
            if code != 0:
                raise RuntimeError(f"setup failed ({cmd!r})")
        if sc["mutate"]:
            target = Path(tmp) / sc["name"]
            mutated = MUTATIONS[sc["mutate"]](target.read_text(encoding="utf-8"))
            target.write_text(mutated, encoding="utf-8")
        outputs = []
        for cmd in sc.get("cmds") or spk_lines(sc["fence"]):
            out, code = run_command(spk, cmd, tmp, env)
            if not out.strip() and code != 0:
                raise RuntimeError(f"replay failed ({cmd!r}): no output, exit {code}")
            outputs.append(out.rstrip("\n"))
        return "\n".join(outputs)


def root_relative(rel):
    path = ROOT / rel
    if not path.is_file():
        raise FileNotFoundError(f"missing replay input: {rel}")
    return path


# Cargo/rustc build chatter for the verify scratch crate (warm machines print
# none, cold CI all of it; cargo's lines, never spk's — ignored on BOTH diff
# sides so no cache state is baked into the doc; classes: progress headers,
# rustc diagnostics/locations/snippets/gutters/carets, warning summaries).
_CHATTER = re.compile(
    r"^\s*(?:Compiling|Finished|Running|Downloaded|Downloading|Updating|Locking|Dry-run"
    r"|warning:|help:|note:|error: (?:could not compile|aborting)|--> |\d+\s*\|"
    r"|\||\^|=|generated \d+ warnings?)"
)


def normalize(text):
    """Whitespace-normalize captured text: rstrip lines, drop trailing blanks."""
    lines = [line.rstrip() for line in text.splitlines() if not _CHATTER.match(line)]
    while lines and lines[-1] == "":
        lines.pop()
    return lines


def diff_report(expected, actual):
    return "\n".join(difflib.unified_diff(
        expected, actual, fromfile="docs (expected)", tofile="binary (actual)", lineterm=""))


def binary_version(spk, env):
    proc = subprocess.run([spk, "--version", "--human"], env=env,
                          capture_output=True, text=True)
    tokens = proc.stdout.split()
    return tokens[-1] if tokens else proc.stdout.strip()


def version_advisories(root, spk, env):
    """Docs claim 'captured from specodelic X.Y.Z'; warn when binary version drifts."""
    version = binary_version(spk, env)
    for rel in (INSTALLATION, WORKED):
        claims = set(VERSION_CLAIM_RE.findall((root / rel).read_text(encoding="utf-8")))
        if claims and version not in claims:
            yield (f"advisory: {rel} claims output captured from specodelic "
                   f"{', '.join(sorted(claims))} but the binary reports {version} "
                   f"— if command semantics changed, re-capture the fenced blocks; "
                   f"if only the version moved, update the stamp")


def scenario_label(sc):
    return sc.get("fence") or sc.get("first_line")


def main(argv=None):
    parser = argparse.ArgumentParser(description="Docs-accuracy gate: re-run the "
                                     "quickstart commands captured in the docs and "
                                     "diff their output against the doc's fenced blocks")
    parser.add_argument("--spk", default=str(ROOT / "target/debug/spk"),
                        help="path to the freshly built specodelic binary")
    args = parser.parse_args(argv)

    spk = str(Path(args.spk).resolve())
    if not Path(spk).is_file():
        print(f"doc-examples: binary not found at {spk} — run `cargo build` first",
              file=sys.stderr)
        return 1
    env = dict(os.environ, NO_COLOR="1", TERM="dumb")

    scenarios = scenario_table()
    unresolved = resolve_expected(ROOT, scenarios)
    if unresolved is not None:
        print(f"doc-examples: scenario no longer resolvable in {unresolved['doc']}: "
              f"{scenario_label(unresolved)!r} not found — update "
              f"quickstart_manifest.py", file=sys.stderr)
        return 1

    for note in version_advisories(ROOT, spk, env):
        print(note)

    failures = 0
    for sc in scenarios:
        try:
            actual = normalize(replay(spk, sc, env))
            # A cold scratch-crate compile can eat verify's own 600s wall
            # clock on CI (properties_timed_out) — machine speed, not
            # semantics; the shared target/ makes one warm retry honest.
            if any("properties_timed_out" in line for line in actual):
                actual = normalize(replay(spk, sc, env))
            expected = normalize(sc["expected"])
        except (OSError, RuntimeError, ValueError) as exc:
            print(f"doc-examples: DRIFT could not replay {sc['doc']} "
                  f"[{scenario_label(sc)!r}]: {exc}", file=sys.stderr)
            failures += 1
            continue
        if actual != expected:
            label = spk_lines(sc["fence"])[-1] if "fence" in sc else "(unpaired block)"
            print(f"doc-examples: DRIFT in {sc['doc']} — command {label} "
                  f"no longer prints what the doc shows:")
            print(diff_report(expected, actual))
            print("fix: re-capture from the real binary and update the fenced block — "
                  "never hand-edit either side to go green")
            failures += 1

    if failures:
        print(f"doc-examples: {failures} drifted captured output(s) in the docs",
              file=sys.stderr)
        return 1
    print("doc-examples: all documented captured outputs match the binary")
    return 0


if __name__ == "__main__":
    sys.exit(main())
