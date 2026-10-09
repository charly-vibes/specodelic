# Subagent brief: specodelic-lf4b.10 — Add docs-accuracy gate: re-run quickstart commands and diff captured outputs

You are a pi subagent in repo `/var/home/sasha/para/areas/dev/gh/charly/specodelic`
(Rust CLI `specodelic`, alias `spk`). An orchestrator has claimed ticket
`specodelic-lf4b.10` and is hands-off: you own the implementation end-to-end.
The orchestrator will verify with real gates — never claim work the diff
doesn't contain.

## Orientation (do this first)

- `export WAI_PROJECT=domain-specific-extensions`
- Read: `bd show specodelic-lf4b.10` (acceptance criteria + anti-goals),
  the epic `bd show specodelic-lf4b`, `docs/src/installation.md` (lf4b.2
  added expected outputs — "All output below was captured verbatim from
  `specodelic 0.7.0`"), `docs/src/examples/worked-example.md` (lf4b.5 —
  12 captured CLI outputs, same drift class), and the justfile (repo
  convention: prefer just targets).
- **Existing precedent (do not duplicate):** `scripts/check_doc_examples.py`
  + `just lint-doc-examples` extract fenced SPEC examples from the corpus
  and lint them — that covers format artifacts, NOT captured command
  output. Your gate is the complement: re-run documented COMMANDS and diff
  their output against what the docs claim they print (the 7-vs-9
  explain-topics drift is the existence proof this class of rot happens).
- `wai search "doc-examples"` / `wai search "drift"` — accumulated
  patterns. lf4b.5's subagent suggested wiring its re-capture approach
  into this ticket.

## What to build

**A `just doc-examples` target** (repo justfile) that:

1. Re-runs the documented quickstart commands (start with
   `docs/src/installation.md`'s captured block — the command sequence it
   claims to show, at the version the docs state)
2. Captures real output at HEAD (build spk first if the target needs to —
   see how other just targets handle it; the target must be deterministic
   and offline — anti-goal: no network-dependent steps)
3. Diffs captured output against the expected blocks checked into docs
   (either parse the fenced blocks from the .md directly — preferred, no
   duplicate fixtures — or maintain fixtures under `docs/fixtures/` if
   parsing is impractical; justify the choice)
4. **Fails on drift** — nonzero exit naming the drifted command and file

**Meter (all must pass):**
- corrupting one expected block in docs → `just doc-examples` fails
  (demonstrate, revert, show pristine tree passes)
- pristine tree passes
- wire into `just ci` ONLY if the target runs in under ~30s (the
  existing `check_doc_examples` pipeline slot took a name — if wiring in,
  place it adjacent; if too slow, wire into docs-build or leave standalone
  and say why in the report)
- fixtures/blocks regenerate only via the target itself — never hand-edit
  expected output to make the gate green (anti-goal)

**Design constraints:**
- Version skew is expected and real: the docs say "captured verbatim from
  specodelic 0.7.0" — a gate that fails on every version bump is a gate
  that gets removed. Design the drift signal to be actionable: compare
  structured content (e.g. JSON envelopes) where possible, and/or emit a
  clear "docs claim version X, binary is Y" advisory vs hard-fail split
  (hard-fail: command semantics changed; advisory: version stamp only).
  Keep it simple — this is complexity:m, not a framework.
- Reuse `genesis` envelope output (`--json` where the documented command
  supports it) for stable comparison.

## Hard scope guard

- Allowed files: `justfile`, new script under `scripts/` (if the target
  needs one — python3 like the sibling checks), `docs/src/installation.md`
  (only if a captured block needs a mechanical fix the gate exposed),
  `docs/src/examples/worked-example.md` (only if its captured blocks need
  regeneration via the new target), `.github/workflows/**` only if you
  wire the target into an existing workflow's step list (do not create new
  workflows).
- Never edit: `openspec/**`, `.espectacular/`, `pretender.toml`, `specs/`
  (the corpus — its fenced examples are covered by lint-doc-examples),
  `book.toml`, `src/**`, `tests/**`, `.wai/resources/**`, `docs/src/specs/**`,
  `docs/src/openspec/**` (generated), this template.
- New script files need a Purpose/Responsibilities/Rationale docstring
  header (follow `scripts/check_doc_examples.py` as the style precedent).

## Repo facts (boilerplate — applies to every ticket)

- **Commit hygiene** (standing, every commit): before ANY `git commit`, run
  `git status --short` and stage only files YOU authored
  (`git add <paths>`, never bare `git add`); unstage foreign files.
- **beads**: `bd close specodelic-lf4b.10` FIRST, then `bd export -o
  .beads/issues.jsonl`, then commit the export.
- **Do NOT push** — the orchestrator verifies and pushes.
- **Do NOT run `wai close`** — the orchestrator owns session close.
- **ah/espectacular**: read-only over `openspec/`.
- **Non-interactive shells**: use `-f`/`-rf`/`-y` flags.
- **Scratch work outside /tmp**: use `/var/tmp/<slug>`.
- **Quality gates before finish**: `just test` must be green (the
  pretender known-red was fixed in a59448e — full green is the bar now);
  run `pretender check` if you touch anything the ratchet scans (scripts/
  has its own role thresholds — `[thresholds.script]` cyc 15, cognitive 31,
  function_lines 59 — keep the script small or split functions).

## Mechanical finish checklist (execute in order, do not skip)

1. `just doc-examples` on pristine tree → exit 0
2. Corrupt one expected block → target fails naming the drift → revert →
   passes again (paste evidence)
3. `just test` → green; `pretender check` → exit 0
4. `just docs-build` → green
5. `git status --short` → only your files; stage explicitly
6. `git commit` with honest attribution
7. `bd close specodelic-lf4b.10` → `bd export -o .beads/issues.jsonl` →
   commit the export
8. Do NOT push; do NOT run `wai close`
9. End with the Report block below

## Report format (end with this — the orchestrator verifies against it)

## Report

**Commits**
- `<hash>` <message> — <what it does, one line>

**Gates run**
- `just test` → <result>
- `just docs-build` → <result>
- `ah check` → <result>
- `openspec validate --all --strict` → <result>
- `just doc-examples` → <result, incl. corrupt-fixture drill evidence>

**Wiring decision**
- <where the target is wired (ci/docs-build/standalone) and runtime>

**Deviations** (or "none")

**Next**
- <exact next action for the orchestrator, or "ticket complete">