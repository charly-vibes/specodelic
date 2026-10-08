# Subagent brief: specodelic-91kt — Implement add-quant-finance-pack phase 2: packs/quant-finance.md artifact

You are a pi subagent in repo `/var/home/sasha/para/areas/dev/gh/charly/specodelic`
(Rust CLI `specodelic`, alias `spk`). An orchestrator has claimed ticket
`specodelic-91kt` and is hands-off: you own the implementation end-to-end.
The orchestrator will verify with real gates — never claim work the diff
doesn't contain.

## Orientation (do this first)

- `export WAI_PROJECT=domain-specific-extensions`
- Read, in order: `openspec/changes/add-quant-finance-pack/design.md` (esp.
  D1 thin vocabulary, D2 hygiene audit, D3 intra-file resolution, D4 risk
  floor, D5 Requires pins, D7 dual-format shape), `tasks.md` §2 (this
  ticket), `proposal.md` (slice scoping), then the delta
  `openspec/changes/add-quant-finance-pack/specs/quant-finance-pack/spec.md`
  — it is the authoritative contract for the artifact you are creating.
- Model the artifact on the existing pilot: `packs/bioimage-data.md` is the
  closest structural precedent (four-layer spec shape, six manifest tables,
  full three-state Model). Also skim `packs/numeric-predicates.md`.
- `wai search "quant"` and `wai search "pack artifact"` — check accumulated
  patterns before designing.
- Start your tdd-ro5 run:
  `wai pipeline start tdd-ro5 --topic="specodelic-91kt: packs/quant-finance.md pack artifact"`
  Advance ONLY via `wai pipeline next` — never hand-edit the run YAML under
  `.wai/pipeline-runs/`.

## What to build

Create `packs/quant-finance.md` (id `quant.finance`, `kind: profile`,
`checked_against_core: clear`, state `published` via the full three-state
Model — draft→published→deprecated transitions with guards citing the
delta's constraints, mirroring the bioimage pattern).

Six manifest tables per design.md D1 — declare EXACTLY this vocabulary, no
more:

- `## Sections` — `Limits` with row shape `| name | kind | unit | bound |`
- `## Kinds` — `quant.risk`, `quant.pricing` (per-pack closed set)
- `## References` — `capped_by` (outbound leaf resolving to a `## Limits`
  row of the declaring file; joins no reachability path and no acyclic
  edge set)
- `## Checkers` — `quant.limit_closed`, `quant.risk_labels` (declarations
  only — honest-empty; the mechanism interprets them, no code)
- `## Floors` — `quant.risk` requires `horizon` + `confidence` case labels
- `## Requires` — `base` pinned at `specodelic.md Revision 18` (the current
  FORMAT_REVISION; earlier packs' Revision 14 pins stay valid — design D5),
  `numeric.predicates`, `data.lineage`

Artifact must be a four-layer spec file (frontmatter statement, Constraints,
Model, Properties) so it lints clean as a spec in its own right and
self-hosts. Vocabulary hygiene (design D2, pre-paid): NO bare-English
vocabulary tokens — `limit`, `confidence`, `Fixtures` are corpus-
contaminated and excluded from vocabulary-carrying facets; `horizon`/
`confidence` appear only as floor case labels.

TDD shape (probe before you land):
1. RED: in a scratch git worktree (e.g. `/var/tmp/qfp-91kt` — do NOT use
   /tmp, quota), create scratch spec files that use `quant.risk` vocabulary
   / `capped_by` / `## Limits` and verify the findings EXIST with the pack
   absent (orphan_vocabulary etc.), then add a draft variant of the pack
   and verify activation/finding shapes change as the delta's Scenarios
   declare.
2. GREEN: land the pack as `published`; verify in the scratch worktree:
   - `pack_shape` zero findings over the pack's own declared vocabulary
   - self-exemption holds (manifest rows trigger no checkers, no orphan
     findings)
   - lint 0 issues in draft, published, and deprecated variants (edit state
     in the scratch copy; restore `published` before committing)
   - `spk graph` 0 dangling/violations over the pack file; compile emits
     artifacts
   - vocabulary surface over the pack file is exactly [quant.risk,
     quant.pricing, Limits, capped_by] (checker rule names never join
     vocabulary())
3. Then repo gates in the real tree.

## Hard scope guard

- Allowed files: `packs/quant-finance.md` (new),
  `openspec/changes/add-quant-finance-pack/tasks.md` (check off §2.1/§2.2
  boxes only), `.beads/issues.jsonl` (via `bd export` only).
- Scratch/probe fixtures stay OUTSIDE the repo tree (use `/var/tmp/qfp-91kt`
  — not /tmp).
- Never edit: `openspec/specs/`, `.espectacular/`, `pretender.toml`,
  `specs/` (the corpus), `src/`, `tests/`, other `packs/*.md`,
  `.wai/resources/**`, this template.
- Respect shrink-only ratchets: run `pretender check` before committing; if a
  pinned source file breaches, move new tests to `tests/` rather than raising
  entries. (This ticket adds no src/ code, so this should be trivially green.)
- Compiled-corpus artifacts under `specodelic/` regenerate via `just ci` —
  if they change, leave them UNCOMMITTED and note it in your report; the
  orchestrator lands them as a separate chore(artifacts) commit.

## Repo facts (boilerplate — applies to every ticket)

- **Commit hygiene** (standing, every commit): before ANY `git commit`, run
  `git status --short` and stage only files YOU authored
  (`git add <paths>`, never bare `git add`); unstage foreign files.
  Attribute the message only to what the diff contains.
- **Output discipline**: every command emits through
  `genesis::guide::Output::emit`; errors carry a remediation hint.
- **beads**: issues live in `.beads/issues.jsonl`; commit the export after
  `bd close` (`bd export` gotcha: run it or the close is invisible to git).
  NOTE: do NOT close specodelic-91kt yourself — the orchestrator closes it
  after verification. Run `bd export -o .beads/issues.jsonl` only if you
  change issue state.
- **Do NOT push** — the orchestrator verifies and pushes.
- **Do NOT run `wai close`** — the orchestrator owns session close.
- **ah/espectacular**: read-only over `openspec/`; contract TOMLs for
  change-phase scenarios are orphans until archive — author them in the
  archive commit, never mid-change. Do not touch `.espectacular/` at all
  in this ticket.
- **New source files** need Purpose/Responsibilities/Rationale headers
  (file-headers skill convention) — for a pack spec file, the frontmatter
  `statement` serves this role; make it complete.
- **Non-interactive shells**: use `-f`/`-rf`/`-y` flags; commands may be
  aliased with `-i` and will hang on prompts.
- **Test runs**: set `TMPDIR=/var/tmp/qfp-91kt` before `just test`/`just ci`
  (/tmp quota exhaustion masquerades as mass test failures).

## Repo gates (all must be green before you commit)

- `openspec validate --all --strict`
- `cargo run -- lint openspec` → 0 issues
- `just sync-sections`
- `just lint-specs`
- `just ci` (or `just test` + fmt/clippy if ci is too slow — say which)
- `ah check` → no NEW findings vs current baseline (0 structural, 0
  execution expected)

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
