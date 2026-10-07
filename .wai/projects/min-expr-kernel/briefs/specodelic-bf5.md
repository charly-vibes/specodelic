# Subagent brief: specodelic-bf5 — external checker claims preserve opaque binding text

You are a pi agent in repo `/var/home/sasha/para/areas/dev/gh/charly/specodelic`
(Rust CLI `specodelic`, alias `spk`). An orchestrator has claimed ticket
`specodelic-bf5` and is hands-off: you own the implementation end-to-end.
The orchestrator will verify with real gates — never claim work the diff
doesn't contain.

## Orientation (do first)

- `export WPROJECT=min-expr-kernel` then `export WAI_PROJECT=min-expr-kernel`
- Read, in order: `openspec/changes/add-min-expr-kernel/design.md` (esp.
  D4 — no registry, opaque binding), `tasks.md` section 5 (tasks 5.1, 5.2
  — yours), `proposal.md` Tier C scoping, then the compile/kernel delta
  specs this ticket extends.
- `wai search "binding"` and `wai search "claim"` — check accumulated
  patterns (the espectacular contract-TOML flags binding path is
  established; see `ah explain` docs and existing contract examples in
  `.espectacular/` for the carrier shape).
- Your ticket maps to a tdd-ro5 run — start it and execute each step's
  prompt literally:
  `wai pipeline start tdd-ro5 --topic="specodelic-bf5: §5.1-5.2 — opaque kernel.binding extraction and claim carrier"`
  Advance ONLY via `wai pipeline next`.
- Baseline: phases 1-4 complete. The kernel grammar, three-valued chain,
  corpus pass (mui), and shared merge (k3h) are landed. `kernel.binding`
  cells do not extract yet.

## What to build

add-min-expr-kernel **§5.1 (RED) + §5.2 (GREEN) — Tier C opaque binding bridge**:

- RED: extraction tests — an invariant-kind Constraint with a
  `kernel.binding` cell extracts the claim as an opaque string; specodelic
  never interprets its contents; a constraint claimed by an external
  checker surfaces in contract-TOML `flags` binding (node id / `-k` /
  `[[tests.shell]]`). Hard gate: at least 3 fixtures preserve ordinary,
  arbitrary, and empty binding strings exactly.
- GREEN: implement `kernel.binding` extraction and the claim-carrier
  surface. No registry — design D4: the format learns nothing of the
  checker's language; binding text is carried verbatim, never parsed,
  validated, or executed.
- Observe RED failing for the intended reason before GREEN; record commands
  and results.

## Hard scope guard

- Allowed files: `src/compile.rs`, `src/commands/corpus.rs`,
  `tests/cli/parse_misc.rs`, new files under `tests/`,
  `openspec/changes/add-min-expr-kernel/tasks.md` (only the §5.1 and §5.2
  checkboxes).
- Never edit: `openspec/specs/`, `.espectacular/` (including config.toml —
  contract TOMLs are archive-commit-only), `pretender.toml`, `specs/`
  (the corpus), `.wai/resources/**`, this template,
  `openspec/changes/add-min-expr-kernel/{proposal,design}.md`.
- Respect shrink-only ratchets: run `pretender check` before committing;
  `src/model_check.rs` is pinned — if your work would touch it, move tests
  to `tests/` instead and stop if the implementation itself must go there
  (report the conflict instead of improvising).
- Agent test runner during iteration: `just test-smart`; `just ci` stays
  the orchestrator's gate.

## Repo facts (boilerplate)

- **Commit hygiene** (every commit): `git status --short` first; stage only
  files YOU authored (`git add <paths>`, never bare `git add`); unstage
  foreign files; attribute messages only to what the diff contains.
- **Output discipline**: every command emits through
  `genesis::guide::Output::emit`; errors carry a remediation hint.
- **beads**: commit the export after `bd close` (`bd export -o
  .beads/issues.jsonl` or the close is invisible to git).
- **Do NOT push** — the orchestrator verifies and pushes.
- **Do NOT run `wai close`** — the orchestrator owns session close.
- **ah/espectacular**: read-only over `openspec/`; contract TOMLs for
  change-phase scenarios are orphans until archive.
- **New source files** need Purpose/Responsibilities/Rationale headers.
- **Non-interactive shells**: use `-f`/`-rf`/`-y` flags.

## Report format (end with this)

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
