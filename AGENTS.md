<!-- TURU:START -->
# Whisper knowledge workspace (managed by turu — regenerate with `turu sync`)

- Workspace root: `/var/home/sasha/.whisper`
- Repo key: `cv/charly-vibes/specodelic` · Branch slug: `main` · Worktree slot: `.git`
- Deterministic routing (resolve, never guess):
- global → `/var/home/sasha/.whisper/rules.md`
- repo → `/var/home/sasha/para/areas/dev/gh/charly/specodelic/.whisper/env.md`
- branch → `/var/home/sasha/para/areas/dev/gh/charly/specodelic/.whisper/branches/main/notes.md`
- worktree → `/var/home/sasha/.whisper/repos/cv/charly-vibes/specodelic/worktrees/.git/env.md`
  - append/resolve default: repo → `/var/home/sasha/para/areas/dev/gh/charly/specodelic/.whisper/env.md`, branch → `/var/home/sasha/para/areas/dev/gh/charly/specodelic/.whisper/branches/main/notes.md`
  - `--global` escape hatch: repo → `/var/home/sasha/.whisper/repos/cv/charly-vibes/specodelic/env.md`, branch → `/var/home/sasha/.whisper/repos/cv/charly-vibes/specodelic/branches/main/notes.md`
  - `--private` zone (never pushed): repo → `/var/home/sasha/para/areas/dev/gh/charly/specodelic/.whisper/private/env.md`, branch → `/var/home/sasha/para/areas/dev/gh/charly/specodelic/.whisper/private/branches/main/notes.md`
- Commands: `turu resolve <scope>` · `turu append <scope> [--global] --text ... [--topic k] [--supersedes id]` · `turu recall <scope> [--topic k] [--budget bytes]` · `turu distill <scope> --begin|--commit` · `turu bundle pack|unpack` · `turu status` · `turu doctor` · `turu feedback <kind>`

<!-- TURU:END -->


<!-- SPECODELIC:START -->
## Specodelic — spec format rules (managed block)

This repo's `specs/`-style markdown spec files (YAML frontmatter +
fixed-schema tables) are linted by `spk` (crates.io: specodelic).
Write specs so `spk lint` passes; embedded format revision: specodelic.md Revision 19

### Lint rules (every violation names its `rule_id`)

- `linter.frontmatter_valid` — frontmatter `kind` must be `intent` — the only top-level intent kind
- `linter.id_matches_file` — frontmatter `id` must equal the filename stem with `-` mapped to `.` (`_` is literal); a `spec.md` file derives its expected id from its parent directory (Revision 18)
- `linter.unique_id` — every row id in a file must be unique across all of the file's tables
- `linter.guard_required` — every transition must carry a non-null guard (a guard may be prose, or cite an invariant Constraint or a State — target typing is `ref_kind_compatible`'s beat)
- `linter.model_present` — the Model section must contain both a States list and a Transitions table (empty-but-present beats absent)
- `linter.ears_syntax` — the intent statement must contain an imperative `SHALL` and match one of the five EARS patterns
- `linter.no_conjoined_id` — an id must not encode two capabilities joined by `and`/`or`
- `linter.no_universal_in_id` — an id must not contain a universal token (all/every/any/always/never)
- `linter.total_refs` — every structured-field [[link]] must resolve to a definition somewhere in the corpus (Revision 18: all files resolve corpus-wide — the former `id: spec` self-containment retired)
- `linter.coverage` — every constraint must have a deriving property (`∃ property.derives_from == <constraint>`)
- `linter.no_orphan_property` — every property must derive from at least one constraint
- `linter.law_cases` — every law-kind property must enumerate its required cases as **name:** labels in its own predicate — the identity and associativity floor is mandatory, extra named cases are checkable declarations
- `linter.terminal_states_emit` — every failure terminal state must emit exactly one file-owned effect Constraint — a mute failure terminal is a finding (openspec/specs/linter-failure_shape/spec.md; timed_out/exploration_only are the stated v1 non-goal)
- `linter.error_labels_unique` — within one file, no two error Constraints may share a variant head — the label is file-id-namespaced (errors.md error_expr_shape), so collisions are a per-file property
- `linter.guard_negation_total` — every failure transition must cite exactly the union of its success siblings' citation sets, or be on the recorded carve-out list (orchestrate.md's stage-fail transitions) — a zero-citation failure guard off the list is a finding
- `linter.every_state_used` — every declared state must appear as from or to in at least one transition — a state no transition reaches is machinery the model can never enter or leave
- `linter.every_transition_valid` — every transition's from and to must name states declared in the same file's States section
- `linter.no_self_ref` — a row must not reference itself via traces_to or derives_from — a self-tracing row has no owning purpose
- `linter.acyclic` — the directed graph formed by constraint-traces_to ∪ property-derives_from ∪ guard-as-edge must contain no cycle (derives_from edges are property-sourced — the Reference Typing Appears-on column is normative, so a Constraint-row derives_from is typing's beat, never an edge)
- `linter.single_root_reachable` — every constraint/property/state/transition row must reach its file's OWN intent row through own-file primary linkage (traces_to/derives_from chains resolved within the file, plus the model's own from/to/guard/emits edges) — cross-file typed edges (guard citations of foreign constraints, satisfies, observes) are outbound leaves, never reachability paths; tiered: cross-file-only rows warn (advisory, exit 0), rows with no path to ANY intent hard-fail
- `linter.observability` — every effect Constraint must be the target of ≥1 `observes` reference from a different row — advisory: warned on the warnings channel (exit 0), never a failure
- `linter.checklist_well_formed` — a declared checklist manifest (`*.checklist.md`) must be a flat item list with stable ids plus a mapping table with exactly item/status/mapped_ids/rationale columns — a manifest the linter cannot read is a checklist going silently unconsulted
- `linter.every_item_accounted` — every checklist item must have exactly one mapping row with status `covered` or `waived` — an unconsulted item is the failure this checker exists to prevent
- `linter.covered_maps_resolve` — a `covered` mapping row must name a non-empty mapped_ids list whose ids resolve to real constraint or property rows — a claim resting on nothing is not a claim
- `linter.waiver_has_rationale` — a `waived` mapping row must carry non-empty rationale prose — an unexplained waiver is an unconsulted item with extra steps
- `linter.no_duplicate_claim` — no two mapping rows may target the same checklist item — one claim per item, on the record
- `linter.constraint_kind_closed` — every Constraint row's kind must be in {invariant, advisory, effect, extension_point} — an unreadable kind cell is outside the closed set (openspec/specs/linter-schema_shape/spec.md)
- `linter.property_kind_closed` — every Property row's kind must be in {unit, law} — an unreadable kind cell is outside the closed set (openspec/specs/linter-schema_shape/spec.md)
- `linter.pack_shape` — a kind: profile pack file's manifest must carry all six facet tables (Sections/Kinds/References/Checkers/Floors/Requires) with well-formed two-column rows, its Kinds rows must be pack-qualified (never a base closed-set name — the narrowing rejection), and manifest tables may not appear on non-profile files (openspec/specs/packs/spec.md, Revision 14)
- `linter.orphan_vocabulary` — orphan vocabulary is a labeled failure naming the candidate pack and both remediations (enable/declare the pack, or fix the vocabulary) — a declared uses edge targeting an id no discovered kind: profile pack carries, or a pack-qualified token used in a kind/field position with no discovered pack in its namespace (candidate prefix-derived when only the namespace is known) (openspec/specs/packs/spec.md, Revision 14)
- `linter.skew_advisory` — a declared pack's Requires base pin older than the workspace corpus revision is a warnings-channel advisory naming the pack's base pin and the corpus revision — never silent, never failing (openspec/specs/packs/spec.md, Revision 14)
- `linter.schema_matches_typing_table` — when the lint target carries the format doc, its Reference Typing table must equal the Schema value row for row — the document and the code cannot drift; a corpus without the format doc is out of the gate's scope (no-op, never fabricated expected rows) (add-acset-core, linter-schema_shape family)

### Commands

- `spk lint <dir>` — check the invariants (fails with a hint on zero files)
- `spk graph <dir>` — typed reference graph (state edges, typing
  violations, supersedes cycles; blast-radius lands later)
- `spk compile <files>` — emit TOML / proptest / TLA+ artifacts
- `spk model-check <files>` — run the model checker against compiled
  output (stateright; reports land as `*.check.json`)
- `spk explain [topic]` — the embedded format primer (works offline)
- `spk doctor` — diagnose workspace + block currency
- `spk feedback bug --dry-run` — file an issue against upstream

Refresh this block after upgrading: `spk init --force`.
<!-- SPECODELIC:END -->

<!-- BEADS:START -->
<!-- BEGIN BEADS INTEGRATION v:1 profile:minimal hash:ca08a54f -->
## Beads Issue Tracker

This project uses **bd (beads)** for issue tracking. Run `bd prime` to see full workflow context and commands.

### Quick Reference

```bash
bd ready              # Find available work
bd show <id>          # View issue details
bd update <id> --claim  # Claim work
bd close <id>         # Complete work
```

### Rules

- Use `bd` for ALL task tracking — do NOT use TodoWrite, TaskCreate, or markdown TODO lists
- Run `bd prime` for detailed command reference and session close protocol
- Use `bd remember` for persistent knowledge — do NOT use MEMORY.md files

## Session Completion

**When ending a work session**, you MUST complete ALL steps below. Work is NOT complete until `git push` succeeds.

**MANDATORY WORKFLOW:**

1. **File issues for remaining work** - Create issues for anything that needs follow-up
2. **Run quality gates** (if code changed) - Tests, linters, builds
3. **Update issue status** - Close finished work, update in-progress items
4. **PUSH TO REMOTE** - This is MANDATORY.
5. **Clean up** - Clear stashes, prune remote branches
6. **Verify** - All changes committed AND pushed
7. **Hand off** - Provide context for next session
<!-- END BEADS INTEGRATION -->
<!-- BEADS:END -->

<!-- ah:managed:start -->
## espectacular

Run `ah check` to verify spec-test correspondence before committing.

- `ah check` — validate all deployed specs
- `ah check --changes <name>` — validate with a change overlay
- `ah init` — set up or refresh espectacular project files
- `ah doctor` — diagnose setup issues
- `ah explain <topic>` — playbook guidance for finding kinds and suggested actions
- `ah doctor --enable <adapter>` — write adapter config into .espectacular/config.toml
- `ah signals` — emit dont drift signals
<!-- ah:managed:end -->

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

## Available Pipelines

| Pipeline | When to Use | Start |
|----------|-------------|-------|
| epic-orchestrator | Autonomous epic execution: orchestrator delegates each ticket to a pi subagent, verifies gates, ships | `wai pipeline start epic-orchestrator --topic=<topic>` |
| tdd-ro5 | Autonomous feature, bugfix, or refactor work where tests and review must gate progress | `wai pipeline start tdd-ro5 --topic=<topic>` |

> Pipeline steps may have gates that enforce artifact creation, review coverage, and oracle checks before advancement. Run `wai pipeline gates <name>` for details.
> **Before any edit**: run `wai pipeline current --json`; if a run is active for this project, resume it (`wai pipeline next`) rather than starting ad-hoc work.

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


<!-- provenance: generator=wai version=0.12.3 source=WAI sha=b807e600 -->
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
- The only sanctioned write path for capability specs is the companion
  archive recipe (`just archive-change id=<id>` — Revision 19, Stage 2:
  openspec applies the plain deltas, the companion re-attaches the
  layer from the archived `layer.md` sidecar): the stock archive
  **destroys** the specodelic layer (frontmatter +
  Constraints/Model/Properties tables) — see the spike findings under
  `openspec/changes/archive/2026-09-28-spike-dual-format/`.
- Spec files here are dual-format; espectacular may only read the
  `#### Scenario:` blocks under openspec requirement headers. It MUST
  NOT enforce anything against the `specs/` domain corpus (that is
  specodelic's own format, governed by `spk lint`).
- **Gate status**: ADOPTED-WITH-SEQUENCING — the 2026-10-01 DEFER was
  overturned per `openspec/decisions/2026-10-01-espectacular-adoption.md`
  (specodelic-tlv, owner instruction 2026-10-03): wiring (`ah init`, gate
  recipe) is allowed; full per-scenario contract authoring is sequenced as a
  follow-up. Espectacular must stay read-only over `openspec/` regardless.

### vampiro (seam/composition checks over source)

- **READ-ONLY over `src/` and `tests/`**: vampiro findings may gate CI
  but must never drive code edits directly.
- Rust frontend data-flow edges are **partial** (vampiro README v0.3.1):
  treat Rust-scope findings as advisory (warn), not blocking (deny),
  until frontend parity; any future deny-level gate requires an explicit
  waiver mechanism agreed here first.
- **Blocker**: no integration ticket exists — the dual-format CI gates
  it was waiting on have landed; file a ticket first when wiring work
  starts.

## What this repo is

**specodelic** — a Rust CLI (`specodelic` (alias `spk`)) for the Specodelic specification
format (formerly `spec-format`). The format corpus is the single tree at
`openspec/specs/` (specodelic-jf1): every corpus file is a markdown spec
written in the format it describes, at `openspec/specs/<dir>/spec.md`.
`openspec/specs/specodelic/spec.md` is the core; `openspec/STATUS.md` §1 is the primer.

- Format questions → `openspec/specs/specodelic/spec.md` (single source of truth).
- Checker semantics → the matching `openspec/specs/linter-*/spec.md` file.
- Tool semantics → `openspec/specs/compile/spec.md`, `verify`, `rename`,
  `merge`, `graph`, `refactor`, `orchestrate`, `model_check`.
- Repo workflow (file naming, revision discipline) → `openspec/specs/AGENTS.md`.

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
  `specodelic` on import (2026-09-28). `openspec/USAGE.md`'s title records
  it. Don't reintroduce `spec-format` references.
- **Issue tracking**: `bd` (beads) in no-db mode — `.beads/issues.jsonl`
  is the source of truth, tracked in git. No Dolt database.
- **Commit hygiene** (specodelic-2g1, dz4 remediation-c — standing
  instruction, every commit): before ANY `git commit`, run
  `git status --short` and check every entry against what *this session*
  actually authored; stage only your own files (`git add <paths>`, never
  bare `git add` / `git add .`), and if a foreign file is already staged,
  unstage it (`git restore --staged <path>`) rather than committing it.
  Attribute the commit message only to what the diff actually contains —
  a message claiming "gate-authored rewrite" on a diff that introduces a
  nonexistent test binding is how incident e7bb19e poisoned the corpus.
  Gate drills must go through `scripts/guards/gate-drill.sh` (worktree or
  advisory lock — specodelic-do8); see
  `openspec/decisions/2026-10-05-precommit-sweep-policy.md`.
- **Quality gates**: `just ci` (fmt-check, clippy `-D warnings`, tests,
  release build) must pass before pushing.
- **Test runner**: during iterative work, agents run `just test-smart`
  (testaruda select --safe) instead of `cargo test` directly — it selects
  the affected test set from the working diff, runs only those tests, and
  ingests results for future selection. Falls back to plain `cargo test`
  when the store/config is missing or confidence is low. `just test-blast`
  previews affected tests without running anything. The full `cargo test`
  via `just ci` remains the gate before push — testaruda selection is for
  fast feedback, never a substitute for the gate.
- **Dogfooding**: `just lint-specs` lints the corpus with the tool
  itself. Known coverage gaps are tracked as beads issues, not ignored.

## File naming (from the format — non-negotiable)

A spec file's frontmatter `id` equals its filename stem with `-` ⇔ `.`:
`linter-graph_shape.md` ⇔ `id: linter.graph_shape`. For the single tree,
a `spec.md` file derives the expected id from its PARENT directory:
`openspec/specs/linter-graph_shape/spec.md` ⇔ `id: linter.graph_shape`
(`_` is literal in both). Non-spec files (no frontmatter) are exempt:
`AGENTS.md`, `STATUS.md`, `USAGE.md`, `CHANGELOG.md`, `theory.md` (in
`openspec/`), and this file. A `*.checklist.md` file is also exempt — it
is the external-completeness manifest
(openspec/specs/linter-external_completeness/spec.md,
mp1 row 10 decision): an artifact outside the four-layer shape, not a
sixth spec kind.
<!-- WAI:REFLECT:REF:START -->
## Accumulated Project Patterns

Project-specific conventions, gotchas, and architecture notes live in
`.wai/resources/reflections/`. Run `wai search "<topic>"` to retrieve relevant
context before starting research or creating tickets.

> **Before research or ticket creation**: always run `wai search "<topic>"` to
> check for known patterns. Do not rediscover what is already documented.


<!-- provenance: generator=wai version=0.12.3 source=WAI:REFLECT:REF sha=73879972 -->
<!-- WAI:REFLECT:REF:END -->
<!-- DDL:START -->

## dulce-de-leche (ddl v0.8.0)

Orchestrator for the charly-vibes tool ecosystem. Run `ddl init` to
install and configure all tools. Run `ddl status` for a health overview.

Commands:
- init     Bootstrap the toolset
- install  Install a single tool
- status   Show health overview
- doctor   Run diagnostics
- version  Show versions
- upgrade  Update tools
- migrate  Migrate configs
- scope    Show active .ddl/

<!-- DDL:END -->
