# Subagent brief: specodelic-75m — revision-upgrade tooling (spec first, then verb)

You are a subagent in repo `/var/home/sasha/para/areas/dev/gh/charly/specodelic`
(Rust CLI `specodelic`, alias `spk`). An orchestrator has claimed ticket
`specodelic-75m` and is hands-off: you own the design record + implementation
end-to-end. The orchestrator verifies with real gates — never claim work the
diff doesn't contain.

## Orientation (do this first)

- `export WAI_PROJECT=specodelic`
- Read: `bd show specodelic-75m`; `openspec/specs/migrate/spec.md` (current
  migrate contract — in-place dual-format wrap, rekey, dry-run);
  `openspec/specs/rename/spec.md` and `openspec/specs/refactor/spec.md`
  (placement + atomicity conventions for a bulk rewriter);
  `openspec/specs/specodelic/spec.md` — the append-only Revision headings
  (2..20) and what each revision actually changed (most are additive; find
  the genuinely breaking/renaming ones a real step exists for), plus
  `FORMAT_REVISION` in `src/guide.rs` and `spk --version --json`'s
  `format_revision`; `src/migrate.rs` (542 lines: `migrate`, `rekey`,
  `scaffold_layer`); `src/main.rs` (migrate verb wiring + flag conventions);
  `openspec/AGENTS.md` (change conventions); `just archive-change` recipe.
- `wai search "migrate"`, `wai search "revision"` — accumulated patterns.
- Start and execute the executor pipeline literally:
  `wai pipeline start tdd-ro5 --topic="specodelic-75m: revision-upgrade tooling"`
  Advance ONLY via `wai pipeline next` — never hand-edit run YAMLs.
- `export TMPDIR=/var/tmp/specodelic-xch1` (mkdir -p it) — MANDATORY for
  cargo invocations.

## What to build

**Spec first** (ticket instruction): scaffold `openspec/changes/add-revision-upgrade/`
(proposal.md, design.md, tasks.md, delta on `migrate` — ADDED requirement(s)
+ `#### Scenario:` blocks per delta convention, and the layer sidecar
`specs/migrate/layer.md` = full post-change layer for the migrate
capability, convention per `src/archive_companion.rs` and
`openspec/changes/archive/2026-10-19-add-delta-layer-sidecar/specs/*/layer.md`).

The delta must define (decide and record in design.md):
1. **A thin core-owned revision registry** — a const manifest in core code:
   known format revisions (id/date/one-line change note), each with an
   OPTIONAL migration step where a real transform exists (most revisions
   are additive → no-op steps).
2. **The upgrade contract** — `spk migrate --to-revision <N>` (or the
   subcommand shape you justify; dry-run default per the existing verb's
   convention, `--write`/`--force` to apply — mirror what migrate already
   does). Applies registered steps from the file's source revision to the
   target; unknown source/target revision → labeled failure naming the
   known range + remediation; files needing no step stay BYTE-IDENTICAL
   (CRLF/terminators preserved); never touches files outside the given
   paths.
3. **Placement**: the rewriter lives in `src/migrate.rs` beside the existing
   wrap/rekey (rename/refactor convention: atomic or unchanged).

**Then implement** (RED first): tests in `tests/**` —
- fixture corpus authored at an older revision (use a REAL older form from
  the revision history): one file that a registered step transforms, one
  file needing nothing;
- upgrade to the current revision → `spk lint` on the fixture reports 0
  findings;
- unchanged files byte-identical (pin bytes before/after);
- dry-run writes nothing; unknown `--to-revision` fails labeled;
- `--to-revision` below the file's current revision fails labeled.
GREEN: registry + upgrade path in `src/migrate.rs`, wiring in
`src/main.rs` (keep it minimal, follow existing flag/output patterns via
`genesis::guide::Output::emit`).

**Dual-format finish**: after tasks are ticked, deploy via
`just archive-change add-revision-upgrade` (never hand-edit
`openspec/specs/`), then `spk init --force` and verify the AGENTS.md diff is
catalog-only. Revert any `.genesis/tools.toml` no-op reorder churn rather
than committing it.

## Hard scope guard

- Allowed files: `src/migrate.rs`, `src/main.rs` (migrate verb wiring only),
  `tests/**`, `openspec/changes/add-revision-upgrade/**` (then archived via
  the recipe), `openspec/specs/**` ONLY via `just archive-change`,
  `.espectacular/` ONLY new contract TOMLs for YOUR change's scenarios in
  the archive commit, `AGENTS.md` ONLY via `spk init --force`.
- Never edit: `src/packs.rs`, other verbs' modules (`rename.rs`,
  `refactor.rs`, `compile.rs`, `verify.rs`), `specs/` (domain corpus),
  `pretender.toml`, `.wai/resources/**`, `.beads/hooks/**`, run YAMLs.
- Respect shrink-only ratchets: `pretender check` before committing; if a
  pinned source file breaches, move new tests to `tests/`.
- Do NOT try to make every historical revision transformative — the
  registry is thin by design; no-op steps are correct where revisions were
  additive. Do not weaken the existing wrap/rekey contract.

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
- **New source files** need Purpose/Responsibilities/Rationale headers.
- **Non-interactive shells**: use `-f`/`-rf`/`-y` flags.
- **Run `cargo fmt` before your commits.**

## Report format (end with this — the orchestrator verifies against it)

## Report

**Commits**
- `<hash>` <message> — <what it does, one line>

**Tasks ticked**
- <each with one-line evidence>

**Gates run**
- targeted RED/GREEN cycles → <results>; `just archive-change` → <result>

**Deviations** (or "none")

**Next**
- <exact next action for the orchestrator, or "ticket complete">