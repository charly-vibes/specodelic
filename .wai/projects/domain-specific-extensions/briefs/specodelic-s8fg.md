# Subagent brief: specodelic-s8fg — task 4: archive-companion apply + layer re-attach (Rev 19)

You are a pi subagent in repo `/var/home/sasha/para/areas/dev/gh/charly/specodelic`
(Rust CLI `specodelic`, alias `spk`). An orchestrator has claimed ticket
`specodelic-s8fg` (already in_progress, commit 5aa84df) and is hands-off: you
own the implementation end-to-end for THIS run's slice — **tasks.md section 4
only (4.1–4.3)**. The orchestrator will verify with real gates — never claim
work the diff doesn't contain.

## Orientation (do this first)

- `export WAI_PROJECT=domain-specific-extensions`
- Read, in order:
  - `openspec/changes/add-delta-layer-sidecar/design.md` — **Decision 2** is
    your implementation contract (companion: openspec applies, companion
    re-attaches; 4 numbered steps + guard list); also Decisions 4 (deltas may
    be partial) and 5 (sidecar identity across archive history).
  - `openspec/changes/add-delta-layer-sidecar/tasks.md` section 4 (your scope;
    sections 1–3 are landed — do not redo them, 5–7 are NOT yours).
  - `src/archive_companion.rs` (744 lines, Stage-1 shape) — you are REWORKING
    this file: spec-application skipped + stripped-copy deploy + the
    dual-era guards (`delta_self_contained`, existing-target coverage,
    post-strip shape assertion) are the subjects of this task.
  - `src/migrate.rs` wrap path + its tests (landed task 3) — the sidecar
    shape, naming (`id: <parent-dir>.layer`), and refusal classes are set
    there; the companion consumes the same shape.
- `wai search "archive companion"` and `wai search "delta sidecar"` —
  accumulated patterns and gotchas.
- The three remaining dual deltas (`add-graph-views`,
  `add-py-fragment-emission`, `add-ts-fragment-emission`) are NOT yet
  migrated (task 5, later run). Your tests must construct their own plain
  delta + sidecar fixture in tempdirs, not rely on those changes.

## What to build (tasks.md 4.1–4.3, TDD: RED first)

### 4.1 — RED then GREEN: apply + re-attach assembly

- RED: `tests/archive_companion.rs` (new file, or extend the existing suite
  wherever the Stage-1 companion tests live — check `tests/` first) — a
  plain delta + sidecar change under a tempdir openspec workspace deploys a
  dual-format spec: the deployed `openspec/specs/<cap>/spec.md` carries
  frontmatter + the three specodelic tables + `## Requirements` produced by
  openspec's own apply; the archived delta (moved to
  `changes/archive/<date>-<id>/`) stays byte-plain; the sidecar rides along
  in the archive.
- GREEN: rework `src/archive_companion.rs` per Decision 2's steps:
  1. Refuse (BEFORE invoking openspec — the archive is destructive) any
     affected capability with no `layer.md` sidecar in the change, or a
     sidecar that fails `spk lint`/parse. Sidecar-less refusal must fire
     pre-invocation (nothing moved).
  2. Invoke `openspec archive <id>` with DEFAULT spec application (no
     `--skip-specs`); the change dir moves to `changes/archive/`, sidecar
     rides along.
  3. Merge the archived sidecar's frontmatter + Constraints/Model/Properties
     tables + the deployed target's Purpose with openspec's applied
     `## Requirements` into `openspec/specs/<cap>/spec.md`.
  4. Post-attach shape assertion: the written deployed file must carry NO
     delta headers (no `## ADDED/MODIFIED/REMOVED Requirements`), must
     carry frontmatter + all three tables, and must lint clean — else
     REVERT the write (restore pre-archive state as far as possible) and
     fail with a remediation hint.
- The dual-era guards retire with their subject: `delta_self_contained`
  (self-containedness) and existing-target coverage guards are superseded by
  openspec's native merge (Decision 4). Remove them together with their
  tests; keep every guard that protects against NEW failure shapes.

### 4.2 — fail-closed guards (covered inside 4.1's steps 1 and 4)

- Sidecar-less / unlintable sidecar refuses BEFORE the openspec invocation.
- Post-attach shape assertion reverts and fails on violation.
- Tests for both refusal classes: assert the change dir was NOT moved on
  pre-invocation refusal, and that a violated post-attach assertion leaves a
  clear error + remediation hint (envelope output discipline:
  `genesis::guide::Output::emit`, exit ≠ 0).

### 4.3 — idempotent rerun + newest-archive-wins + `--dry-run`

- Rerunning the companion after archive: re-resolves from the archive dir,
  re-attaches, and converges to IDENTICAL deployed bytes (byte-equality test).
- If multiple archives of the same capability exist under
  `changes/archive/`, the NEWEST archive wins.
- `--dry-run` reports the planned assembly (what would be applied, which
  sidecars consumed, what would be written) without invoking openspec or
  touching the tree.

## Known gotchas (from session recall — read before coding)

- **edit tool atomicity**: multi-edit[] calls roll back ALL edits when one
  oldText misses — verify each edit landed (a silent rollback cost a lost
  naming-law arm last session; the CLI test caught it).
- **cli test binary rebuild lag**: `spk()`-style cli tests can run against a
  stale binary mid-iteration — rebuild before concluding a RED/GREEN result.
- **TMPDIR quota**: `/tmp` can exhaust under gates — set
  `TMPDIR=/var/tmp/<your-session-slug>` before long test runs.
- **`just archive-change`** (justfile:211) currently invokes
  `cargo run -q -- archive-companion "<id>"` — it should keep working with
  no justfile edit if the CLI surface is unchanged; if you must change the
  CLI flags, keep backward-compatible defaults and note it in your report.
- `openspec/project.md` still describes the Stage-1 strip recipe — its
  archive-recipe bullet update is NOT in your slice (later run); do not
  touch it.

## Hard scope guard

- Allowed files: `src/archive_companion.rs`, new/extended tests under
  `tests/` (e.g. `tests/archive_companion.rs`, `tests/cli/*` if the suite
  lives there), `specodelic/` regenerated artifacts ONLY if `just ci`
  regenerates them (separate `chore(artifacts)` commit).
- Never edit: `openspec/` (READ-ONLY for you — no corpus, no change dirs
  except your tempdir fixtures), `.espectacular/`, `pretender.toml`,
  `specs/`, `justfile`, `openspec/project.md`, root `AGENTS.md`,
  `.wai/**`, docs pages (mirror-wording sweep is task 7.2, not yours).
- Respect shrink-only ratchets: `pretender check` before committing;
  `src/model_check.rs` is pinned. If `src/archive_companion.rs` is under a
  cognitive-complexity pin and your rework breaches it, refactor inside the
  file or move logic to tests — never raise ratchet entries.

## Repo facts (boilerplate — applies to every ticket)

- **Commit hygiene** (standing, every commit): before ANY `git commit`, run
  `git status --short` and stage only files YOU authored
  (`git add <paths>`, never bare `git add`); unstage foreign files.
  Attribute the message only to what the diff contains.
- **Output discipline**: every command emits through
  `genesis::guide::Output::emit`; errors carry a remediation hint.
- **beads**: issues live in `.beads/issues.jsonl`; commit the export after
  `bd close` (`bd export` gotcha: run it or the close is invisible to git).
  You do NOT close the ticket — leave status to the orchestrator.
- **Do NOT push** — the orchestrator verifies and pushes.
- **Do NOT run `wai close`** — the orchestrator owns session close.
- **ah/espectacular**: read-only over `openspec/`; contract TOMLs for
  change-phase scenarios are orphans until archive — author them in the
  archive commit, never mid-change.
- **New source files** need Purpose/Responsibilities/Rationale headers.
- **Non-interactive shells**: use `-f`/`-rf`/`-y` flags; commands may be
  aliased with `-i` and will hang on prompts.

## Gates to run before reporting

- `just test` (or `just test-smart` during iteration; full `cargo test`
  before the final commit) → green
- `spk lint openspec` → 0 findings
- `ah check` → green (0 findings; expected orphan pattern for change-phase
  scenarios is fine if the counts don't regress: 28 no-toml + 14
  overlay-conflict baseline)
- `pretender check` → gate mode passes

## Report format (end with this — the orchestrator verifies against it)

## Report

**Commits**
- `<hash>` <message> — <what it does, one line>

**Gates run**
- `cargo test` → <result>
- `spk lint openspec` → <result>
- `ah check` → <result>
- `pretender check` → <result>

**Deviations** (or "none")
- <any scope/plan deviation and why>

**Next**
- <exact next action for the orchestrator, or "ticket complete">
