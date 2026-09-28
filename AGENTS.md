<!-- WAI:START --># Workflow Tools

This project uses **wai** to track the *why* behind decisions — research,
reasoning, and design choices that shaped the code. Run `wai status` first
to orient yourself.

Detected workflow tools:
- **wai** — research, reasoning, and design decisions
- **beads** — issue tracking (tasks, bugs, dependencies). CLI command: **`bd`** (not `beads`)
- **openspec** — specifications and change proposals (see `openspec/AGENTS.md`)

> **CRITICAL**: Apply TDD and Tidy First throughout — not just when writing code:
> - **Planning/task creation**: each ticket should map to a red→green→refactor cycle; refactoring tasks must be separate tickets from feature tasks.
> - **Design**: define the test shape (inputs/outputs) before designing the implementation.
> - **Implementation**: write the failing test first, then make it pass, then tidy in a separate commit.

> **When beginning research or creating a ticket**: run `wai search "<topic>"` to check for existing patterns before writing new content.

## Quick Start

1. `wai sync` — ensure agent tools are projected
2. `wai status` — see active projects, phase, and suggestions
3. `bd ready` — find available work items

When context reaches ~40%: stop and tell the user — responses degrade past
this point. Recommend `wai close` then `/clear` to resume cleanly.
Do NOT skip `wai close` — it enables resume detection.

## Autonomous Work Policy

Proceed without routine confirmation when the next step is clear.
Do not ask to continue, fix, or commit — just do it.

**Stop and ask** only when:
- Conflicting requirements or ambiguous intent
- Destructive actions (data loss, force-push, drop table)
- Credentials, secrets, or external services not yet authorized
- Unresolved test failures after two attempts
- Push, deploy, or release — always get explicit authorization
- Context approaching 40% — recommend `wai close` then `/clear`

## Detailed Instructions

Full workflow reference — session lifecycle, capturing work, command cheat
sheets, cross-tool sync, and PARA structure — lives in **`.wai/AGENTS.md`**.
Read it at the start of your first session or when you need detailed guidance.

Keep this managed block so `wai init` can refresh the instructions.

<!-- WAI:END -->

<!-- OPENSPEC:START -->
# OpenSpec Instructions

These instructions are for AI assistants working in this project.

Always open `@/openspec/AGENTS.md` when the request:
- Mentions planning or proposals (words like proposal, spec, change, plan)
- Introduces new capabilities, breaking changes, architecture shifts, or big performance/security work
- Sounds ambiguous and you need the authoritative spec before coding

Use `@/openspec/AGENTS.md` to learn:
- How to create and apply change proposals
- Spec format and conventions
- Project structure and guidelines

Keep this managed block so 'openspec update' can refresh the instructions.

<!-- OPENSPEC:END -->

# AGENTS.md

Standing instructions for any agent working in this repo.

## Sibling-tool constraints (hard blockers)

Non-negotiable boundaries for charly-family tools that touch this repo.
Violations are bugs, not preferences. Established 2026-09-28.

### pretender (structural quality; native git-hook shims)

- **MUST NOT claim `core.hooksPath`** — beads owns it (`.beads/hooks`,
  verified in this repo's git config). Any hook pretender installs here
  must be marker-guarded and **chain to** the existing `.beads/hooks`
  pre-commit, never replace it.
- No unguarded writes to `.git/hooks/*`; refuse to overwrite or uninstall
  hooks it does not own (pattern: pretender `main.rs:1196-1259`).
- **Blocker**: `spk hooks install` (specodelic-cxr) stays blocked until
  genesis-vibes ships a `genesis::hooks` module implementing the
  claim-or-chain strategy. Do not hand-roll a repo-local workaround.

### espectacular (scenario-conformance over openspec specs)

- **READ-ONLY over `openspec/specs/` and `openspec/changes/`**: never
  edit, regenerate, or run `openspec archive` from espectacular.
- The only sanctioned write path for capability specs is the dual-format
  archive recipe (`openspec archive <id> --skip-specs` + verbatim cp):
  default archive **destroys** the specodelic layer (frontmatter +
  Constraints/Model/Properties tables) — see the spike findings under
  `openspec/changes/archive/2026-09-28-spike-dual-format/`.
- Spec files here are dual-format; espectacular may only read the
  `#### Scenario:` blocks under openspec requirement headers. It MUST
  NOT enforce anything against the `specs/` domain corpus (that is
  specodelic's own format, governed by `spk lint`).
- **Blockers**: adoption decision pending in specodelic-4ae (decision
  note only — no wiring before it lands); specodelic-6pi (non-recursive
  `spk lint <dir>`) must be fixed before any CI gate chains lint +
  scenario-conformance checks.

### vampiro (seam/composition checks over source)

- **READ-ONLY over `src/` and `tests/`**: vampiro findings may gate CI
  but must never drive code edits directly.
- Rust frontend data-flow edges are **partial** (vampiro README v0.3.1):
  treat Rust-scope findings as advisory (warn), not blocking (deny),
  until frontend parity; any future deny-level gate requires an explicit
  waiver mechanism agreed here first.
- **Blocker**: no integration ticket exists — do not wire vampiro into
  this repo's CI before the dual-format CI gates (blocked by
  specodelic-6pi) land; file a ticket first when that work starts.

## What this repo is

**specodelic** — a Rust CLI (`specodelic` (alias `spk`)) for the Specodelic specification
format (formerly `spec-format`). The format corpus lives in `specs/`:
every file there is a markdown spec written in the format it describes.
`specs/specodelic.md` is the core; `specs/STATUS.md` §1 is the primer.

- Format questions → `specs/specodelic.md` (single source of truth).
- Checker semantics → the matching `specs/linter-*.md` file.
- Tool semantics → `specs/compile.md`, `verify.md`, `rename.md`,
  `merge.md`, `graph.md`, `refactor.md`, `orchestrate.md`,
  `model_check.md`.
- Repo workflow (file naming, revision discipline) → `specs/AGENTS.md`.

## Conventions

- **Genesis baseline**: shared CLI/AIX infrastructure comes from the
  `genesis-vibes` crate — envelope output (`Output`/`Envelope`), CLI
  helpers (completions, `--version --json`), verbosity/format flags.
  Domain logic stays here; only cross-cutting pieces go upstream.
- **Binary name is `specodelic` (alias `spk`)**, crate name is `specodelic`.
- **Output discipline**: every command emits through
  `genesis::guide::Output::emit` — JSON envelope by default for pipes,
  human-readable for TTYs. Errors must carry a remediation hint.
- **Rename provenance**: the corpus was renamed from `spec-format` to
  `specodelic` on import (2026-09-28). `specs/USAGE.md`'s title records
  it. Don't reintroduce `spec-format` references.
- **Issue tracking**: `bd` (beads) in no-db mode — `.beads/issues.jsonl`
  is the source of truth, tracked in git. No Dolt database.
- **Quality gates**: `just ci` (fmt-check, clippy `-D warnings`, tests,
  release build) must pass before pushing.
- **Dogfooding**: `just lint-specs` lints the corpus with the tool
  itself. Known coverage gaps are tracked as beads issues, not ignored.

## File naming (from the format — non-negotiable)

A spec file's frontmatter `id` equals its filename stem with `-` ⇔ `.`:
`linter-graph_shape.md` ⇔ `id: linter.graph_shape`. `_` is literal in
both. Non-spec files (no frontmatter) are exempt: `AGENTS.md`,
`STATUS.md`, `USAGE.md`, `CHANGELOG.md`, `theory.md` (in `specs/`), and
this file.
<!-- WAI:REFLECT:REF:START -->
## Accumulated Project Patterns

Project-specific conventions, gotchas, and architecture notes live in
`.wai/resources/reflections/`. Run `wai search "<topic>"` to retrieve relevant
context before starting research or creating tickets.

> **Before research or ticket creation**: always run `wai search "<topic>"` to
> check for known patterns. Do not rediscover what is already documented.
<!-- WAI:REFLECT:REF:END -->
