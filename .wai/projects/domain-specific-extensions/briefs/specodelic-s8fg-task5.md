# Subagent brief: specodelic-s8fg — task 5: corpus migration (3 dual deltas → plain + sidecar) + 1.14.2 shape fixture

You are a pi subagent in repo `/var/home/sasha/para/areas/dev/gh/charly/specodelic`
(Rust CLI `specodelic`, alias `spk`). An orchestrator has claimed ticket
`specodelic-s8fg` (already in_progress) and is hands-off: you own the
implementation end-to-end for THIS run's slice — **tasks.md section 5 only
(5.1–5.3)**. The orchestrator will verify with real gates — never claim work
the diff doesn't contain.

## Orientation (do this first)

- `export WAI_PROJECT=domain-specific-extensions`
- Read, in order:
  - `openspec/changes/add-delta-layer-sidecar/tasks.md` section 5 (your
    scope; sections 1–4 are landed — do NOT redo them; 6–7 are NOT yours).
  - `openspec/changes/add-delta-layer-sidecar/design.md` — Decisions 1, 3
    and 5 govern your slice (sidecar = real specodelic file with
    `id: <parent-dir>.layer`; mirror retirement by-construction; archived
    sidecar identity precedent).
  - `openspec/changes/add-graph-views/`,
    `add-py-fragment-emission/`, `add-ts-fragment-emission/` — the 3
    remaining dual-format deltas you are converting.
  - The migrate wrap (landed task 3): `src/migrate.rs` — the sidecar shape,
    naming, and refusal classes are set there. `spk migrate <file>` can do
    much of the mechanical work; verify its output matches "layer content =
    existing delta frontmatter/tables verbatim" before trusting it.
- `wai search "delta sidecar"` and `wai search "corpus migration"` —
  accumulated patterns.
- The 1.14.2 shape fixture source matrix is STILL PRESENT at `/tmp/os1x/`
  (deployed dual spec `openspec/specs/cap-a/spec.md`, plain delta + Purpose
  `openspec/changes/t-delta/specs/cap-b/spec.md`, sidecar
  `openspec/changes/t-delta/specs/cap-a/layer.md`, plus
  `openspec/changes/t-delta/specs/cap-a/spec.md` — snapshot it verbatim
  BEFORE starting, /tmp is volatile).

## What to build (tasks.md 5.1–5.3)

### 5.1 — Convert the 3 dual deltas to plain deltas + `layer.md` sidecars

- For each of `add-graph-views`, `add-py-fragment-emission`,
  `add-ts-fragment-emission`: the delta at
  `openspec/changes/<change>/specs/<cap>/spec.md` becomes a PLAIN delta
  (no frontmatter, no `## Requirements` mirror — the mirror is retired,
  `linter.requirement_drift`/`dual_format_valid` no longer exist) and a
  sibling `layer.md` sidecar appears next to it.
- Layer content = the existing delta's frontmatter + Constraints/Model/
  Properties tables **verbatim**; sidecar id is the parent capability dir's
  stem-derived id + `.layer` suffix (naming-law layer.md arm, Revision 19 —
  `id_matches_file` derives the expected id from the PARENT DIRECTORY for
  `layer.md` files).
- **Requirement text byte-preserved from the current mirror** — do not
  reflow, rewrap, or reword any requirement text while moving it into the
  plain delta body. The mirror `## Requirements` section is DELETED (its
  content already lives in the delta sections; the byte-preservation
  guarantee is about the requirement text surviving in the delta).
- **Scenario headers (`#### Scenario:`) stay verbatim** — ah contract TOMLs
  bind scenario slugs; changing them breaks the bindings.
- The change dirs stay PENDING (no archiving — task 7 does that).
- Lint discipline after each conversion: `spk lint openspec` stays 0
  findings (sidecars are fully linted corpus-wide; watch
  `total_refs`/`single_root_reachable` — a sidecar's refs resolve
  corpus-wide per Revision 18).

### 5.2 — Check in the 1.14.2 shape fixture

- Snapshot the `/tmp/os1x` matrix into a CHECKED-IN fixture directory
  (suggested: `tests/fixtures/os1x-shape/` preserving its relative layout)
  asserting that all three shapes are 1.x-strict-valid: the deployed dual
  spec, the plain delta + Purpose, and the sidecar layer.
- Wire the assertion into the existing validate-gate plumbing (NOT a new
  gate unless necessary): the natural seam is `scripts/spec_validate.py` /
  `tests/test_spec_validate.py` (validate gate + its python tests) or
  `tests/ci_wiring.rs` if wiring-level assertions fit better there. The
  fixture test must invoke the 1.14.x validator the same way the pinned
  gate does (see `scripts/spec_validate.py` for the version-guard + npx
  pattern — task 6 retires that later; do NOT change the pin yourself).
- Follow the file-headers convention for any new script.

### 5.3 — Meters

- `openspec validate --all --strict` (1.14.x): ALL items pass, 0 failures,
  0 warnings (note: tasks.md says 51/51 but the corpus has grown to 52
  items — assert "all items, 0 failed" and record the actual count).
- `spk lint openspec`: 0 findings.
- `just ci`: green (compiled-corpus artifacts may be regenerated — land
  them as a separate `chore(artifacts)` commit).

## Known gotchas (from session recall — read before coding)

- **edit tool atomicity**: multi-edit[] calls roll back ALL edits when one
  oldText misses — verify each edit landed.
- **TMPDIR quota**: set `TMPDIR=/var/tmp/<your-session-slug>` before long
  test runs.
- **ah overlay baseline**: change-phase scenario contract counts (28
  no-toml + 14 overlay-conflict) must not regress — scenario headers
  verbatim is the protection; if `ah check` reports NEW no-toml or
  conflict entries, a scenario slug changed and that's a bug.
- **pre-commit gates rewrite nothing in openspec/** — espectacular is
  read-only over openspec/, but the lefthook pre-commit runs ah-check +
  spk-lint + pretender; they must stay green on your commits.
- **npx 1.14.x availability**: global openspec is 0.19.0; 1.14.x comes via
  npx (mirror `scripts/spec_validate.py`'s invocation pattern). If the
  network is unavailable, `npx --no-install` with the cached package or the
  version the script resolves is acceptable — document what you used.
- Commit hygiene: stage only files YOU authored; the compiled-artifacts
  chore commit is separate.

## Hard scope guard

- Allowed files:
  `openspec/changes/add-graph-views/**`,
  `openspec/changes/add-py-fragment-emission/**`,
  `openspec/changes/add-ts-fragment-emission/**` (conversions),
  new fixture directory under `tests/` (e.g. `tests/fixtures/os1x-shape/**`),
  `scripts/spec_validate.py` + `tests/test_spec_validate.py` and/or
  `tests/ci_wiring.rs` (ONLY for the 5.2 fixture assertion),
  `specodelic/` regenerated artifacts (separate chore commit).
- Never edit: `openspec/specs/` (READ-ONLY — deployed corpus untouched),
  `openspec/changes/archive/`, `.espectacular/`, `pretender.toml`,
  `specs/`, `justfile`, `openspec/project.md`, root `AGENTS.md`,
  `src/` Rust code (task 5 is corpus + fixture data work — no Rust changes
  expected; if you find a code bug, report it instead of fixing),
  `.wai/**`, docs pages.
- Respect shrink-only ratchets: `pretender check` before committing.

## Repo facts (boilerplate — applies to every ticket)

- **Commit hygiene** (standing, every commit): before ANY `git commit`, run
  `git status --short` and stage only files YOU authored
  (`git add <paths>`, never bare `git add`); unstage foreign files.
  Attribute the message only to what the diff contains.
- **Output discipline**: errors carry a remediation hint.
- **beads**: issues live in `.beads/issues.jsonl`; you do NOT close the
  ticket — leave status to the orchestrator.
- **Do NOT push** — the orchestrator verifies and pushes.
- **Do NOT run `wai close`** — the orchestrator owns session close.
- **ah/espectacular**: read-only over `openspec/`; contract TOMLs for
  change-phase scenarios are orphans until archive — never author them
  mid-change.
- **New source files** need Purpose/Responsibilities/Rationale headers.
- **Non-interactive shells**: use `-f`/`-rf`/`-y` flags.

## Gates to run before reporting

- `just ci` → green (or full `cargo test` + `cargo fmt --check` +
  `cargo clippy --all-targets -- -D warnings`)
- `spk lint openspec` → 0 findings
- `ah check` → green, baseline counts unchanged
- `openspec validate --all --strict` (1.14.x) → all items, 0 failed,
  0 warnings
- `pretender check` → gate mode passes

## Report format (end with this — the orchestrator verifies against it)

## Report

**Commits**
- `<hash>` <message> — <what it does, one line>

**Gates run**
- `just ci` → <result>
- `spk lint openspec` → <result>
- `ah check` → <result>
- `openspec validate --all --strict (1.14.x)` → <result, actual item count>
- `pretender check` → <result>

**Deviations** (or "none")
- <any scope/plan deviation and why>

**Next**
- <exact next action for the orchestrator, or "ticket complete">
