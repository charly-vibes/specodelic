# Subagent brief: specodelic-6aj0 — archive add-conform dual-format + contract TOMLs + RO5U M2 render tidy

You are a pi subagent in repo `/var/home/sasha/para/areas/dev/gh/charly/specodelic`
(Rust CLI `specodelic`, alias `spk`). An orchestrator has claimed ticket
`specodelic-6aj0` and is hands-off: you own the implementation end-to-end.
The orchestrator will verify with real gates — never claim work the diff
doesn't contain.

## Orientation (do this first)

- `export WAI_PROJECT=min-expr-kernel`
- `bd show specodelic-6aj0` — three sub-tasks: (a) dual-format archive of
  add-conform, (b) the 17 sequenced per-scenario contract TOMLs under
  `.espectacular/conform/` at archive time (`ah check --changes
  add-conform` shows the no-toml inventory), (c) RO5U M2 from eczv.4:
  consolidate exit-1/exit-0 report rendering through ONE path (main.rs vs
  conform.rs dual render). "Regenerate compiled artifacts chore commit
  after archive if corpus compile outputs change."
- Read, in order: `openspec/changes/add-conform/` (design.md, proposal.md,
  tasks.md — all tasks [x], the change is Complete), the delta
  `openspec/changes/add-conform/specs/conform/spec.md` (34 `#### Scenario:`
  blocks), and the PRECEDENT archive commit `f2c08b6` (add-skill-pack:
  the full recipe step by step — delta gained frontmatter + Purpose +
  Constraints + Model + Properties + ADDED Requirements + mirrored
  `## Requirements` sibling BEFORE archive; then `just archive-change`
  runs `openspec archive --skip-specs` + verbatim deploy; archived delta
  stripped back to frontmatter-less; contract TOMLs bound to the existing
  suite in the same commit).
- `git show f2c08b6` and `git show 648b1d9` (svc-contracts pack draft
  landed) — the two most recent dual-format archives to imitate.
- `ah explain workflow` and `ah explain contract-stale` — playbook
  guidance for contract authoring.
- `openspec/decisions/2026-10-01-espectacular-adoption.md` — the sanctioned
  recipe (espectacular read-only over openspec/; contracts in
  `.espectacular/`).
- `wai search "archive dual-format"` — accumulated patterns.

## What to build

### (a) Dual-format archive of add-conform — THE sanctioned recipe only

- The delta must gain the full dual-format layer BEFORE archive, exactly
  like `f2c08b6`: frontmatter (`id: conform` — the naming-law stem of the
  `conform/` dir), Purpose prose with a SHALL, Constraints (one invariant
  per requirement — expect full constraint→property coverage with 2-3
  deriving rows per the ticket), Model (States + Transitions — conform's
  lifecycle: the report/verdict pipeline states), Properties rows (kind
  unit/law, each deriving from ≥1 constraint, every law's named cases in
  its own predicate), the `## ADDED Requirements` section PLUS its
  mirrored `## Requirements` sibling (identical requirement text —
  `linter.requirement_drift` compares per requirement).
- Lint discipline: `spk lint` the delta until 0 findings BEFORE archive
  (guard_required: every transition carries a guard; single-root-reachable;
  every_state_used; observability warnings channel).
- Then `just archive-change` (or the explicit recipe if the just recipe
  differs): `openspec archive add-conform --skip-specs` + verbatim deploy
  to `openspec/specs/conform/spec.md` — byte-identical, layer_preserved.
  NEVER plain `openspec archive` (it destroys the specodelic layer).
- Post-archive: strip the ARCHIVED delta back to frontmatter-less (else
  dangling refs + id collision) — the deployed spec keeps frontmatter.
- If the archive changes corpus compile outputs, regenerate artifacts in
  a chore commit (the ticket's last line).
- Anchors: prefer authoring the layer with `[[conform.<row>]]` anchors
  directly (the f2c08b6 move — no retarget step needed).

### (b) The 17 per-scenario contract TOMLs

- `ah check --changes add-conform` gives the authoritative no-toml
  inventory (the ticket says 17 — reconcile against the live count; the
  delta carries 34 `#### Scenario:` blocks, some already have bindings
  from the change-phase overlay).
- Author `*.toml` contracts under `.espectacular/conform/` binding each
  scenario to EXISTING executable tests (the conform suite:
  `tests/cli/conform.rs`, `tests/conform_gate.rs`, `tests/conform_
  report.rs`, `tests/conform_lifecycle.rs`, `tests/conform_
  classification.rs`) — prefer scenario-scoped test bindings where the
  suite already pins the behavior; a contract that invents behavior the
  suite does not test is a lie.
- The archive commit carries the TOMLs (change-phase scenarios are
  orphans until archive — author them IN the archive commit, never
  mid-change).
- Gate: `ah check --run-tests` → 0 findings, 0 no-toml for conform.

### (c) RO5U M2 — single rendering path (SEPARATE commit)

- eczv.4's M2 tidy: conform's exit-1/exit-0 report rendering is
  duplicated between `src/main.rs` and `src/conform.rs` — consolidate
  through ONE path. Read the two render sites first; the tidy must not
  change any emitted bytes (rendering parity: run `spk conform` on a
  fixture before/after and diff the output byte-for-byte).
- TDD where applicable: a parity test pinning the unified path's output
  if one doesn't exist; otherwise pure refactor with the byte-parity
  check as the meter.
- This is a REFACTOR ticket component — land it in its own commit after
  the archive commit (tidy-after-feature discipline).

### Sequencing

1. Commit 1: dual-format layer on the delta (lint 0 findings).
2. Commit 2: `just archive-change` verbatim deploy + archived-delta
   strip + the 17 contract TOMLs + sibling-guard/espectacular config
   updates if the machinery needs them (f2c08b6 shape) + artifact regen
   chore if needed.
3. Commit 3: RO5U M2 single-rendering-path tidy (byte-parity).
4. Run `bd export -o .beads/issues.jsonl` at the very end if you close
   nothing (the orchestrator closes the ticket).

## Hard scope guard

- Allowed files: `openspec/changes/add-conform/**` (the delta pre-archive),
  `openspec/specs/conform/spec.md` (new, via verbatim deploy),
  `openspec/changes/archive/<dated-add-conform>/**` (post-archive strip),
  `.espectacular/conform/*.toml` (new), `.espectacular/config.toml` (only
  if the machinery requires it), `src/conform.rs` + `src/main.rs` (ONLY
  for sub-task c, byte-parity refactor), `tests/cli/conform.rs` or the
  fitting test file (parity test), compiled artifacts under `specodelic/`
  (only the regen chore if outputs change)
- NEVER plain `openspec archive` — the recipe is `--skip-specs` + verbatim
  deploy, always.
- Never edit: `openspec/specs/**` OTHER than the new `conform/spec.md`
  deploy, `.wai/resources/**`, this template.
- Respect shrink-only ratchets: run `pretender check` before committing.

## Repo facts (boilerplate — applies to every ticket)

- **Commit hygiene** (standing, every commit): before ANY `git commit`, run
  `git status --short` and stage only files YOU authored
  (`git add <paths>`, never bare `git add`); unstage foreign files.
  Attribute the message only to what the diff contains.
- **Output discipline**: every command emits through
  `genesis::guide::Output::emit`; errors carry a remediation hint.
- **beads**: issues live in `.beads/issues.jsonl`; commit the export after
  `bd close` (`bd export -o .beads/issues.jsonl` — stdout is the default,
  you must pass the repo path).
- **Do NOT push** — the orchestrator verifies and pushes.
- **Do NOT run `wai close`** — the orchestrator owns session close.
- **ah/espectacular**: read-only over `openspec/`; contract TOMLs for
  change-phase scenarios are orphans until archive — author them in the
  archive commit, never mid-change.
- **New source files** need Purpose/Responsibilities/Rationale headers
  (file-headers skill convention).
- **Non-interactive shells**: use `-f`/`-rf`/`-y` flags; commands may be
  aliased with `-i` and will hang on prompts.

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
