#!/usr/bin/env python3
"""Replay manifest for the docs-accuracy gate (check_quickstart_outputs.py).

Purpose: one static table describing every command whose output is
captured verbatim in the docs — the gate re-executes it and diffs against
the doc's fenced block.

Responsibilities: fence-body constants (exact text as it appears in the
docs), the per-scenario replay manifest (doc file, fence/first_line
selector, occurrence index, input fixture, optional mutation), and the
c1-link mutation the quickstart's finding stage narrates. Kept separate
from the gate driver so each file stays under the script role's size
ratchet.

Rationale: the manifest is the only place a scenario is declared; the
driver resolves expectations from the docs live, never duplicating them.
"""

from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent

INSTALLATION = "docs/src/installation.md"
WORKED = "docs/src/examples/worked-example.md"
FINAL_SPEC = "docs/src/examples/reservation-order.md"
FIXTURES = "docs/fixtures/doc-outputs"

# Exact sh-fence bodies as they appear in the docs (comments included —
# they are part of the fence the docs render).
F_NEW = "# Scaffold a spec — the template teaches its own tables\n" \
    "spk new order.cancel --file order.cancel.md"
F_LINT_CLEAN = "# Lint it — findings carry their own rule id + semantics\n" \
    "spk lint order.cancel.md"
F_LINT_FINDING = "spk lint order.cancel.md"
F_GRAPH = "# Derive the reference graph (fan-in/out, dangling links)\n" \
    "spk graph order.cancel.md"
F_COMPILE_ORDER = "# Compile to artifacts: TOML + proptest scaffolding + TLA+ module\n" \
    "spk compile order.cancel.md --out-dir specodelic/"
F_MODEL_CHECK_ORDER = "# Model-check the compiled module (embedded stateright backend)\n" \
    "spk model-check order.cancel.md"

NEW_ORDER = "spk new order.cancel --file order.cancel.md"
COMPILE_ORDER = "spk compile order.cancel.md --out-dir specodelic/"
COMPILE_RESERVATION = "spk compile reservation-order.md --out-dir specodelic/"
F_RESERVATION_CP_LINT = "cp reservation-order.md /tmp/reservation/\n" \
    "cd /tmp/reservation\nspk lint reservation-order.md"
F_RESERVATION_COMPILE_CHECK = f"{COMPILE_RESERVATION}\nspk model-check reservation-order.md"


def spk_lines(fence_body):
    """The executable lines of a documented command fence (comments excluded)."""
    return [line for line in fence_body.splitlines() if line.startswith("spk ")]


def break_c1_link(text):
    """Quickstart finding scenario: replace c1's file-qualified traces_to
    wiki-link with bare text (the exact first edit the tutorial narrates)."""
    return text.replace("| [[order.cancel]] |", "| the file intent |")


MUTATIONS = {"break-c1-link": break_c1_link}


def scenario_table():
    """Every documented captured output as a replay scenario.

    fence = exact sh-fence body in the doc (replayed commands are its
    `spk` lines); occ = occurrence index of that fence body (the same
    command can be shown at several narrative stages); cmds overrides the
    derived commands (used once, for the typo-fix stage whose output
    block has no command fence — first_line = the unique first line of
    that unpaired ```text block). fixture = input spec file copied into
    the scratch dir; quickstart scenarios derive their input from
    `spk new` itself.
    """
    fx = FIXTURES
    lint = "spk lint reservation-order.md"
    return [
        # -- installation.md quickstart (order.cancel, from spk new) --
        dict(doc=INSTALLATION, fence=F_NEW, occ=0, fixture=None, mutate=None, setup=[]),
        dict(doc=INSTALLATION, fence=F_LINT_CLEAN, occ=0, fixture=None,
             setup=[NEW_ORDER], mutate=None),
        dict(doc=INSTALLATION, fence=F_LINT_FINDING, occ=0, fixture=None,
             name="order.cancel.md", setup=[NEW_ORDER], mutate="break-c1-link"),
        dict(doc=INSTALLATION, fence=F_GRAPH, occ=0, fixture=None,
             setup=[NEW_ORDER], mutate=None),
        dict(doc=INSTALLATION, fence=F_COMPILE_ORDER, occ=0, fixture=None,
             setup=[NEW_ORDER], mutate=None),
        dict(doc=INSTALLATION, fence=F_MODEL_CHECK_ORDER, occ=0, fixture=None,
             setup=[NEW_ORDER, COMPILE_ORDER], mutate=None),
        # -- worked-example.md: the lint stages of the tutorial --
        dict(doc=WORKED, fence=lint, occ=0,
             fixture=f"{fx}/01-intent-only/reservation-order.md", setup=[], mutate=None),
        dict(doc=WORKED, fence=lint, occ=1,
             fixture=f"{fx}/02-constraints-typo/reservation-order.md", setup=[], mutate=None),
        dict(doc=WORKED, fence=lint, occ=2,
             fixture=f"{fx}/04-model-draft/reservation-order.md", setup=[], mutate=None),
        dict(doc=WORKED, fence=lint, occ=3,
             fixture=f"{fx}/05-model-fixed/reservation-order.md", setup=[], mutate=None),
        # (the typo-fix stage's output block has no command fence in the doc)
        dict(doc=WORKED, first_line="lint: 1 file(s) linted, 7 finding(s), "
             "0 advisory warning(s)",
             fixture=f"{fx}/03-constraints-fixed/reservation-order.md",
             setup=[], mutate=None, cmds=[lint]),
        # -- worked-example.md: final-state pipeline on the finished file --
        dict(doc=WORKED, fence=lint, occ=4, fixture=FINAL_SPEC, setup=[], mutate=None),
        dict(doc=WORKED, fence="spk graph reservation-order.md", occ=0,
             fixture=FINAL_SPEC, setup=[], mutate=None),
        dict(doc=WORKED, fence=COMPILE_RESERVATION, occ=0,
             fixture=FINAL_SPEC, setup=[], mutate=None),
        dict(doc=WORKED, fence="spk model-check reservation-order.md", occ=0,
             fixture=FINAL_SPEC, setup=[COMPILE_RESERVATION], mutate=None),
        # -- worked-example.md: the counterexample stage (over-claimed copy) --
        dict(doc=WORKED, fence=F_RESERVATION_CP_LINT, occ=0,
             fixture=f"{fx}/06-over-claim/reservation-order.md", setup=[], mutate=None),
        dict(doc=WORKED, fence=F_RESERVATION_COMPILE_CHECK, occ=0,
             fixture=f"{fx}/06-over-claim/reservation-order.md", setup=[], mutate=None),
        dict(doc=WORKED, fence="spk verify reservation-order.md", occ=0,
             fixture=FINAL_SPEC,
             setup=[COMPILE_RESERVATION, "spk model-check reservation-order.md"],
             mutate=None),
    ]
