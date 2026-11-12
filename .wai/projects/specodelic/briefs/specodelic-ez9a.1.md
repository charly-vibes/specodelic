# Subagent brief: specodelic-ez9a.1 — implement add-shared-fragment-helpers

You are a subagent in repo `/var/home/sasha/para/areas/dev/gh/charly/specodelic`
(Rust CLI `specodelic`, alias `spk`). An orchestrator has claimed ticket
`specodelic-ez9a.1` and is hands-off: you own the implementation end-to-end.
The orchestrator will verify with real gates — never claim work the diff
doesn't contain.

## Orientation (do this first)

- `export WAI_PROJECT=specodelic`
- Read: `bd show specodelic-ez9a.1` (bd comment carries the E2 scope note);
  `openspec/changes/add-shared-fragment-helpers/proposal.md`, `design.md`,
  `specs/compile/spec.md` (the ADDED requirement + seven scenarios),
  `tasks.md` (the exact task list you tick); `src/compile.rs` (fragment
  extraction `fragment_of`/`fragment_of_strict`, `FRAGMENT_MARKER`,
  `properties_to_proptest`, `model_to_tla`, `constraints_doc`,
  `compile_spec`, `validate_fragments`); `src/spec.rs` (section parsing —
  `## Fragments` must be captured as a new opaque section without touching
  typed-field parsing); `src/lint/` (warnings channel for the dead-helper
  warning); `openspec/changes/archive/2026-10-19-add-delta-layer-sidecar/`
  and `openspec/specs/compile/spec.md` (layer sidecar shape + current
  deployed layer).
- `wai search "fragment"`, `wai search "archive companion"` — accumulated
  patterns.
- Start and execute the executor pipeline literally:
  `wai pipeline start tdd-ro5 --topic="specodelic-ez9a.1: add-shared-fragment-helpers"`
  Advance ONLY via `wai pipeline next` — never hand-edit run YAMLs.
- `export TMPDIR=/var/tmp/specodelic-xch1` (mkdir -p it) — MANDATORY for
  cargo invocations.

## What to build (change of record: openspec/changes/add-shared-fragment-helpers/)

New optional per-file `## Fragments` section: `def <name>:` lines followed
by fenced rust bodies. Fragment cells in the same file may reference a
definition by name; compile inlines each referenced body at every use site.
Linter treats the section as opaque prose.

Work the tasks.md list EXACTLY, RED first — tick each box in
`openspec/changes/add-shared-fragment-helpers/tasks.md` as you land it:

**1. Red (failing tests first, tasks 1.1–1.6)**
- 1.1 Fixture: one helper defined, referenced from two property fragments →
  emission test asserts the helper body inlined at both use sites (fails
  today).
- 1.2 Collision test: helper name colliding with a generated symbol fails
  labeled naming both.
- 1.3 Byte-preservation: identical file WITHOUT the section compiles
  byte-identically before/after (pin pre-change bytes).
- 1.4 Hygiene: helper body with a banned token fails labeled with the same
  rule as fragments (shared FRAGMENT_BANNED mechanism).
- 1.5 Cycle: mutual references fail labeled naming the cycle; a transitive
  acyclic chain inlines fully.
- 1.6 Linter interaction: fixture with `## Fragments` lints clean —
  section is opaque prose, no gating rule disturbed.

**2. Green (tasks 2.1–2.3)**
- 2.1 Extraction: parse `## Fragments` (def-lines + fenced bodies), resolve
  references by name, labeled failure on unknown reference.
- 2.2 Emission: inline bodies at use sites (inline, NOT a shared module —
  design.md decision); dead-helper warning on the warnings channel, success
  preserved.
- 2.3 Docs: format summary section (embedded guide payload — check
  `src/guide.md`/`src/guide.rs` conventions; if the canon lives in the
  corpus `openspec/specs/compile/spec.md`, it arrives via the archive
  below) + the case-study size note if the Wave 1 docs page exists
  (grep for it; if absent, record that the note is covered by the archived
  requirement and skip — do not invent a page).

**3. Refactor (task 3.1–3.2, separate commit)**
- 3.1 Unify fragment-body extraction for inline and helper bodies.
- 3.2 `just ci` green (you may run it once here as the refactor gate).

**4. Layer + archive (task 4.1, fail-closed)**
- Extend the `compile.layer` sidecar as the FULL post-change layer:
  `openspec/changes/add-shared-fragment-helpers/specs/compile/layer.md`
  (convention: `archive_companion.rs` expects `specs/<capability>/layer.md`
  beside the delta; model on `openspec/changes/archive/2026-10-19-add-delta-layer-sidecar/specs/*/layer.md`
  and the current deployed `openspec/specs/compile/spec.md` layer —
  frontmatter + Constraints/Model/Properties tables, extended with the new
  requirement's rows; every property must derive_from a constraint and every
  constraint be covered, per `spk lint`).
- After ALL tasks 1.x–3.x are ticked: `just archive-change add-shared-fragment-helpers`
  (the ONLY sanctioned write into `openspec/specs/`; never hand-edit it).
  Then `spk init --force` to refresh the AGENTS.md catalog and verify the
  diff is catalog-only.

## Hard scope guard

- Allowed files: `src/compile.rs` (extraction + emission + helper
  machinery), `src/spec.rs` (only to capture the opaque `## Fragments`
  section — no typed-field changes), `src/lint/**` ONLY for the dead-helper
  warning on the existing warnings channel, `src/guide.md`/`src/guide.rs`
  ONLY for the format-summary text if the embedded guide carries format
  text, `tests/**` (new fixture/regression tests), `openspec/changes/add-shared-fragment-helpers/**`
  (task ticks + layer.md), `openspec/specs/**` ONLY via `just archive-change`
  output, `AGENTS.md` ONLY via `spk init --force`.
- Never edit: `src/packs.rs`, `src/rename.rs`/`src/refactor.rs` (not in
  scope), `specs/` (domain corpus), `.espectacular/**` except new contract
  TOMLs for YOUR change's scenarios in the archive commit, `pretender.toml`,
  `.wai/resources/**`, `.beads/hooks/**`, run YAMLs.
- Respect shrink-only ratchets: run `pretender check` before committing; if
  a pinned source file breaches, move new tests to `tests/`.
- Do NOT rescope to data-helpers: the E2 verdict is "proposal stands as
  written". Kernel findings F1/F2 are separate tickets (specodelic-8mta /
  specodelic-9o1h) — do not fold them in.

## Repo facts (boilerplate — applies to every ticket)

- **Commit hygiene** (standing, every commit): before ANY `git commit`,
  run `git status --short` and stage only files YOU authored
  (`git add <paths>`, never bare `git add`); unstage foreign files.
  Attribute the message only to what the diff contains.
- **Output discipline**: every command emits through
  `genesis::guide::Output::emit`; errors carry a remediation hint.
- **beads**: issues live in `.beads/issues.jsonl`. Do NOT claim/close/
  update the ticket — the orchestrator owns bd state for this run.
- **Do NOT push** — the orchestrator verifies and pushes.
- **Do NOT run `wai close`** — the orchestrator owns session close.
- **ah/espectacular**: read-only over `openspec/` except `just archive-change`.
- **New source files** need Purpose/Responsibilities/Rationale headers
  (file-headers skill convention).
- **Non-interactive shells**: use `-f`/`-rf`/`-y` flags; commands may
  be aliased with `-i` and will hang on prompts.
- **Run `cargo fmt` before your commits** (fmt-check is a repo gate).

## Report format (end with this — the orchestrator verifies against it)

## Report

**Commits**
- `<hash>` <message> — <what it does, one line>

**Tasks ticked**
- 1.1–1.6 / 2.1–2.3 / 3.1–3.2 / 4.1 → <each with one-line evidence>

**Gates run**
- targeted RED/GREEN cycles → <results>; `just ci` (refactor gate) → <result>;
  `just archive-change` → <result>

**Deviations** (or "none")
- <any scope/plan deviation and why>

**Next**
- <exact next action for the orchestrator, or "ticket complete">