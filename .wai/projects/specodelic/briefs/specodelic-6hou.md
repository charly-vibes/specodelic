# Subagent brief: specodelic-6hou — audit slice 4 refactor: just ci green gate pass (task 4.3)

You are a pi subagent in repo `/var/home/sasha/para/areas/dev/gh/charly/specodelic`
(Rust CLI `specodelic`, alias `spk`). An orchestrator has claimed ticket
`specodelic-6hou` and is hands-off: you own the implementation end-to-end.
The orchestrator will verify with real gates — never claim work the diff
doesn't contain.

## Orientation (do this first)

- `export WAI_PROJECT=specodelic`
- This ticket is the CLOSURE GATE for slice 4 (tasks.md 4.3): full
  `just ci` green after the rung aggregate landed — tracked separately so
  the feature ticket (37hl, shipped) closed on red-green, not gate luck.
  The full suite stands at 997 tests, `just ci` exit 0 at HEAD as of the
  orchestrator's last verification — your job is to PROVE it from a clean
  state and land the slice-4 refactor pass, not to fix breakage.
- Read: `openspec/changes/add-evidence-audit/tasks.md` §Slice 4 (your task
  is 4.3 ONLY), design.md's slice discipline (each slice ships alone),
  and `src/audit.rs` at HEAD — the rung aggregate (RungAggregate /
  EvidenceMix / rung_from_evidence / current_check_evidence, 37hl's
  7fc3301) is the newest code; `just ci` covers fmt-check, clippy
  `-D warnings`, the full test suite, release build, sibling blockers,
  gate-drill lock, and `ah check --run-tests`.
- **Ratchet fact you must respect**: `src/audit.rs` sits at **2294/2300**
  on pretender's `file_lines_max` (6 lines of headroom). Any edit that
  grows the file will breach the ratchet. Relocation into a submodule
  (`src/audit/` directory — the acset module shows the pattern) is the
  sanctioned shape if you need room; NEVER raise `pretender.toml`
  thresholds (forbidden file) and NEVER game the metric.
- `wai search "rung"`, `wai search "slice 4"` — the 37hl run records.
- Start your tdd-ro5 pipeline and execute each step's prompt literally:
  `wai pipeline start tdd-ro5 --topic="specodelic-6hou: slice 4 gate pass"`
  Advance ONLY via `wai pipeline next` — never hand-edit the run YAML under
  `.wai/pipeline-runs/`.

## What to build

tasks.md 4.3 (slice 4 refactor): `just ci` green, full gate, honestly run.

- PRIMARY deliverable: run the complete `just ci` from a clean tree and
  report the real result (test totals, each gate step). If anything is
  red, fix it WITHIN the scope guard below — do not expand scope.
- REFACTOR pass (the ticket's substance, since the gate is already green):
  ONE zero-behavior-change tidy in `src/audit.rs` that leaves the file
  NET-LINE-NEGATIVE (relieving the 2294/2300 headroom), chosen from what
  the 37hl code actually left behind. Candidates, best first:
  (a) relocate a cohesive unit (e.g. the rung aggregate block — types +
    pure core + collector — or the emit plumbing) into `src/audit/rung.rs`
    or similar submodule, updating nothing else (Rust module split,
    `pub(crate)`/`pub(super)` visibility preserved semantically);
  (b) dedupe remaining test-module duplication in-module.
  Byte-identical envelope proof required for whatever you land (before/
  after `spk audit` invocations compared with `cmp` on fixed fixtures —
  your own dir under `/var/tmp/6hou-proof/`), ALL tests green throughout,
  pretender check clean or strictly-no-worse.
- Do NOT start 5.x (x0wy: docs page + layer sidecar) or reopen 4.1/4.2.
- tasks.md checkbox 4.3 only.

## Hard scope guard

- Allowed files: `src/audit.rs`, new `src/audit/**` submodule files you
  create for the relocation (file-headers convention applies),
  `src/lib.rs` ONLY for the `mod audit;` declaration line if the submodule
  split requires it (nothing else — one line, no other lib.rs edits),
  `openspec/changes/add-evidence-audit/tasks.md` (checkbox 4.3 only).
- Never edit: `openspec/specs/`, `.espectacular/`, `pretender.toml`,
  `specs/` (the corpus), `.wai/resources/**`, this template,
  `src/conform.rs`, `src/main.rs`, `src/lint/**`, `src/kernel.rs`,
  `src/verify.rs`, `src/parse.rs`, `justfile` (the stale ticket metadata
  lists it — ignore that; the gate runs, it is not edited).
- If the gate proves red on something OUTSIDE the allowed files, STOP and
  report it as a blocker — do not fix foreign files.

## Repo facts (boilerplate — applies to every ticket)

- **Commit hygiene** (standing, every commit): before ANY `git commit`, run
  `git status --short` and stage only files YOU authored
  (`git add <paths>`, never bare `git add`); unstage foreign files.
  Attribute the message only to what the diff contains.
- **Output discipline**: every command emits through
  `genesis::guide::Output::emit`; errors carry a remediation hint.
- **beads**: issues live in `.beads/issues.jsonl`; the orchestrator closes
  the ticket — you do NOT run `bd close`.
- **Do NOT push** — the orchestrator verifies and pushes.
- **Do NOT run `wai close`** — the orchestrator owns session close.
- **ah/espectacular**: read-only over `openspec/`; contract TOMLs for
  change-phase scenarios are orphans until archive — author them in the
  archive commit, never mid-change.
- **New source files** need Purpose/Responsibilities/Rationale headers
  (file-headers skill convention).
- **Non-interactive shells**: use `-f`/`-rf`/`-y` flags; commands may be
  aliased with `-i` and will hang on prompts.
- Test runner during iteration: `just test-smart` (falls back to
  `cargo test`); full `just ci` gate before you report done.
  **`export TMPDIR=/var/tmp/specodelic-xch1` (create it; it exists) is
  MANDATORY for every cargo run** — /tmp tmpfs OOM-killed cargo twice in
  the 8ebr run; don't wait for it to bite you.

## Report format (end with this — the orchestrator verifies against it)

## Report

**Commits**
- `<hash>` <message> — <what it does, one line>

**Gates run**
- `just test` → <result>
- `ah check` → <result>
- `openspec validate --all --strict` → <result>

**Deviations** (or "none")
- <any scope/plan deviation and why>

**Next**
- <exact next action for the orchestrator, or "ticket complete">
