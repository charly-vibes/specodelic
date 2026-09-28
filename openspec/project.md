# Project Context

## Purpose

specodelic implements the **Specodelic specification format** (formerly
`spec-format`) and ships the `ddl` CLI that operates on it. One markdown
file gives you four things at once: human-readable intent (EARS
statements), machine-lintable constraints, a simulatable state model, and
PBT-style properties — all parsed from frontmatter and fixed-schema
tables only; prose is never inspected.

The format corpus in `specs/` is self-hosting: it is written in
Specodelic itself, and `ddl lint specs` dogfoods the tool against the
format's own spec files. `specs/specodelic.md` is the single source of
truth for the format; `specs/STATUS.md` §1 is the primer.

Relation to OpenSpec: openspec manages the *engineering change workflow*
of this repo (proposals → tasks → archive). Specodelic is a *domain spec
format* the repo also implements tooling for. Keep both; don't conflate
them (see "Domain Context").

## Tech Stack

- Rust (2024 edition), stable toolchain; rustfmt + clippy clean (`-D warnings`)
- CLI: clap 4 (derive), genesis-vibes 0.7 for envelope output,
  verbosity/format flags, completions, `--version --json`
- Serialization: serde / serde_json / serde_yaml / toml
- Issue tracking: bd (beads) in no-db mode — `.beads/issues.jsonl` is
  the source of truth, tracked in git
- Build orchestration: just (`just ci` = fmt-check + clippy + tests +
  release build)

## Project Conventions

### Code Style

- Every source file carries a Purpose / Responsibilities / Rationale
  header doc comment
- Domain logic lives in the library (`src/lib.rs` modules: `spec`,
  `ears`, `lint`, `graph`); `main.rs` stays thin (clap wiring + emit)
- Errors always carry a remediation hint (genesis envelope Invariant 3.2.5)

### Architecture Patterns

- Every command emits through `genesis::guide::Output::emit` — JSON
  envelope by default for pipes/agents, `--human` for TTYs
- The format's naming law is non-negotiable: a spec file's frontmatter
  `id` equals its filename stem with `-` ⇔ `.` mapping; `_` is literal
- Reference typing is structural: `advisory`/`effect` constraints can
  never gate a transition — enforced by the format's typing table, not
  by per-checker invariants

### Testing Strategy

- Unit tests per module; integration tests (`tests/cli.rs`) run the
  binary with `assert_cmd` against the repo's own `specs/` corpus
- The corpus is the primary fixture; synthetic specs in tempdirs cover
  failure shapes
- Known corpus gaps are tracked as beads issues, never silenced in tests

### Git Workflow

- Trunk-based on `main`; work is not complete until `git push` succeeds
- Beads workflow: `bd ready` → `bd update <id> --claim` → work →
  `bd close <id>`; keep `.beads/issues.jsonl` committed

## Domain Context

- The five schema objects (Intent, Constraint, State, Transition,
  Property) and the reference typing table live in
  `specs/specodelic.md`; sub-kind shapes in `specs/kinds.md`
- The lifecycle is `draft → parsed → linted → compiled → model_checked
  → verified`; the orchestrator (`specs/orchestrate.md`) drives it and
  never gates more or less than the specified guards
- Checker semantics are one file per check (`specs/linter-*.md`); when
  implementing a checker, read its spec file first — the implementation
  must match the spec's constraints, not a reinterpretation
- The corpus was renamed from `spec-format` to `specodelic` on import
  (2026-09-28); don't reintroduce `spec-format` references

## Important Constraints

- `ddl lint specs` currently reports 23 coverage gaps (beads
  `specodelic-qc8`) — fixing the corpus and implementing `compile` are
  the two work streams; do not weaken lint rules to make them pass
- Pipeline semantics (`compile`, `model_check`, `verify`, `rename`,
  `merge`, `refactor`, `orchestrate`) are fully specced in `specs/*.md`
  but not implemented; stubs must keep exiting non-zero with hints

## External Dependencies

- genesis-vibes (crates.io) — shared CLI/AIX infrastructure; boundary
  rule: only cross-cutting pieces go upstream, domain logic stays here
- openspec CLI — change-proposal workflow under `openspec/changes/`
