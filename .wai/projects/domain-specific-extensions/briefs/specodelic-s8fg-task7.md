# Subagent brief: specodelic-s8fg — task 7 (FINAL): archive pending plain-shaped changes + close the epic

You are a pi subagent in repo `/var/home/sasha/para/areas/dev/gh/charly/specodelic`
(Rust CLI `specodelic`, alias `spk`). An orchestrator has claimed ticket
`specodelic-s8fg` (already in_progress) and is hands-off: you own the
implementation end-to-end for THIS run's slice — **tasks.md section 7 only
(7.0–7.3)**. The orchestrator will verify with real gates — never claim work
the diff doesn't contain. This is the FINAL slice of the change; work is
sequential (7.0 → 7.1 → 7.2 → 7.3) — do not parallelize.

## Orientation (do this first)

- `export WAI_PROJECT=domain-specific-extensions`
- Read, in order:
  - `openspec/changes/add-delta-layer-sidecar/tasks.md` section 7 (your
    scope; sections 1–6 are landed and ticked — do NOT redo them).
  - `openspec/changes/add-delta-layer-sidecar/design.md` — Decision 2
    (companion: openspec applies, companion re-attaches), Decision 5
    (sidecar identity across archive history; this change's OWN capability
    layer wording updates via its own sidecar at its own archive).
  - `src/archive_companion.rs` + `tests/cli/archive_companion.rs` — the
    Stage-2 companion you will USE (do not modify): refusal guards,
    apply + re-attach, post-attach assertion.
  - `.espectacular/spec-integration/modified-delta-is-dual-format.toml` —
    its provenance comment (specodelic-2pcd, commit d6e69ef) says the
    contract re-authors against the Rev 19 wording at THIS archive. Same
    for the sibling spec-integration contracts
    (`drifted-modified-mirror-fails-ci.toml`, `drifted-sections-fail-ci.toml`,
    `coexisting-identical-ids.toml`, `both-parsers-accept.toml`) — check
    each one's scenario still exists post-archive.
  - `openspec/decisions/2026-10-19-validate-pin-retired.md` and
    `2026-10-16-archive-companion-single-tree.md` — the recipe context.
  - `wai search "archive dual-format"`, `wai search "archive companion"` —
    accumulated patterns, incl. the quant-pack archive precedent
    (frontmatter must be REMOVED from the archived delta after archive —
    dangling refs + id collision; deployed spec keeps frontmatter and gets
    anchors retargeted; expect 2-3 new deriving property rows because
    change-phase deltas do not enforce coverage — the companion's
    post-attach lint assertion will tell you).
- Pending changes (verify with `ls openspec/changes/`): the SIX to land are
  `update-id-derivation-spec-md`, `add-migrate-rekey` (named by 7.1) plus
  `add-delta-layer-sidecar` itself and the three task-5 conversions
  `add-graph-views`, `add-py-fragment-emission`,
  `add-ts-fragment-emission` (section 7's title is "the pending
  plain-shaped changes" — plural; Decision 5 explicitly plans this
  change's own archive).

## What to build (tasks.md 7.0–7.3, in order)

### 7.0 — Reconcile the cross-change MODIFIED conflict FIRST

- `openspec/changes/update-id-derivation-spec-md/specs/spec-integration/
  spec.md` restates "Dual-format delta" with the RETIRED mirror-pairing
  wording ("pairing its carried delta sections with a sibling
  `## Requirements` mirror"). Rewrite that delta block to the Revision 19
  wording (plain delta + layer.md sidecar; naming-law layer.md arm) so
  openspec apply at archive deploys the NEW semantics, not the retired
  ones. Keep `#### Scenario:` headers verbatim if scenarios survive; if a
  scenario's subject is retired (e.g. "Modified delta is dual-format"
  asserted `dual_format_valid` firing), rewrite the scenario to the Rev 19
  subject (the rule no longer exists; the retirement test
  `lint_no_longer_flags_half_format_dual_file_via_cli` is the code truth —
  see d6e69ef's provenance comment).
- Commit 7.0 alone (small, reviewable) before any archive.

### 7.1 — Archive through the NEW companion, one change per commit

- For each of the six changes (order: `update-id-derivation-spec-md`,
  `add-migrate-rekey`, then the four Rev-19-shaped ones ending with
  `add-delta-layer-sidecar` LAST):
  1. Pre-flight: the change's deltas are PLAIN (no frontmatter) with
     sibling `layer.md` sidecars. If `add-delta-layer-sidecar`'s own
     deltas (specs/archive-companion, specs/migrate, specs/spec-integration)
     still carry frontmatter/tables, convert them first — same pattern as
     task 5.1 (sidecar = frontmatter + tables verbatim, id =
     parent-dir stem + `.layer`; mirrors deleted; requirement text +
     scenario headers byte-preserved).
  2. `just archive-change id=<change-id>` (runs the Stage-2 companion:
     refuses sidecar-less/unlintable BEFORE the destructive invocation,
     applies via openspec, re-attaches the layer, post-attach asserts).
  3. If the companion's deployed-corpus lint assertion fails on coverage
     (change-phase deltas don't enforce it), add the missing deriving
     property rows to the DEPLOYED spec (2-3 expected per the precedent)
     and retry.
  4. Post-archive per change: `openspec validate --all --strict` (plain
     1.x gate now — no fallback) + `spk lint openspec` stay green;
     `ah check --changes <next-remaining-id>` for the next one.
  5. Contract authoring (`.espectacular/<cap>/`, your sanctioned write
     zone for the gate's OWN config dir): before archiving, run
     `ah check --changes <id>` to see the delta-phase orphan inventory;
     after archiving each capability, reconcile that capability's
     contracts — re-author `modified-delta-is-dual-format` against the
     Rev 19 wording (bind real tests that exist; name them from the
     suites), delete contracts whose scenarios no longer exist, keep
     `ah check` (and `ah check --run-tests`) green. Loose binding to
     shared mechanism tests is the accepted pattern (data-lineage
     precedent).
  6. If `specodelic/` compiled artifacts change, land them as a separate
     `chore(artifacts)` commit.
- The 7.0-rebound contract's provenance promise is FULFILLED here.

### 7.2 — Mirror-wording residue sweep (after archives, before close)

- Reword to the plain-delta + sidecar shape (Rev 19):
  `openspec/specs/hooks/spec.md` ("deltas not yet migrated to dual"
  advisory), `openspec/specs/verification-claims/spec.md` deploy
  transition guard ("dual-format archive preserves every requirement" —
  only if its archive didn't already rewrite it via openspec apply), the
  root `AGENTS.md` espectacular sibling-constraint bullet (still teaches
  the dual-format archive recipe), `docs/src/tool-relationships.md`
  ("Why not plain openspec archive?"), `docs/src/commands.md` archive
  section. Also `openspec/project.md` archive-recipe bullet if it still
  describes the Stage-1 strip recipe.
- Prose-only: `ah check` stays green (contract TOMLs bind scenario slugs,
  not constraint prose) — verify.
- If `specs/CHANGELOG.md` exists, add the change entry; if the repo has a
  `just stamp-artifacts` / llm.txt derived-file seam
  (`python3 scripts/stamp_llms.py root`), refresh stamped artifacts.

### 7.3 — Close the epic

- Meter trail: confirm final meters (1.x strict validate all-pass,
  `spk lint openspec` 0 findings, `just ci` green, `ah check` green) and
  record them in the bd close resolutions.
- `bd close specodelic-s8fg`, `bd close specodelic-dlf`,
  `bd close specodelic-3le9` (both folded into this change) — with
  resolutions noting the fold-in; then `bd export -o .beads/issues.jsonl`
  and commit the export (source-of-truth invariant).
- Refresh the root `AGENTS.md` managed SPECODELIC block:
  `spk init --force` (embedded format revision 19).
- Tick tasks.md 7.0–7.3 BEFORE the change's own archive commits if
  possible, or note any untickable residual honestly.

## Known gotchas (from session recall — read before coding)

- **edit tool atomicity**: multi-edit[] calls roll back ALL edits when one
  oldText misses — verify each edit landed.
- **TMPDIR quota**: set `TMPDIR=/var/tmp/<your-session-slug>` before long
  test runs.
- **`spk` binary staleness**: installed spk can lag the repo;
  `cargo install --path .` before trusting pre-commit lint output.
- **Archived delta frontmatter**: after archive, the ARCHIVED delta must
  be stripped back to frontmatter-less (dangling refs + id collision
  otherwise) — the Stage-2 companion archives the PLAIN delta as-is, so
  this should not arise for the six; but VERIFY the archived deltas carry
  no frontmatter (the 1.x validator on archived shape is the check).
- **fetch.pruneTags gotcha**: never interpose a `git pull` between creating
  and pushing a tag (no tags here, but don't create any).
- **Contract counts**: the ah overlay baseline counts (28 no-toml +
  14 overlay-conflict during change phase) collapse to ~0 as changes
  archive — that is EXPECTED to drop; what must NOT appear is any NEW
  structural finding on deployed specs (every deployed scenario needs a
  contract).

## Hard scope guard

- Allowed files: `openspec/changes/<the six pending changes>/**`
  (incl. their archive moves), `openspec/specs/**` — **sanctioned write
  paths ONLY**: (a) companion-archive deploys, (b) the 7.2 prose rewording
  files, (c) deriving-property additions when the post-attach lint fails.
  This is an explicit, tasks.md-mandated exception to the standing
  espectacular read-only rule for this final slice; document every
  deployed-spec edit in your report.
- Allowed: `.espectacular/**` (contract authoring/retirement — the gate's
  own config dir), `.beads/issues.jsonl` (via bd + export), root
  `AGENTS.md` (espectacular bullet + managed block refresh),
  `openspec/project.md` (archive-recipe bullet), `docs/src/tool-relationships.md`,
  `docs/src/commands.md`, `specs/CHANGELOG.md` (tracked source of truth —
  then `cp` to `docs/src/specs/` if that copy is tracked), stamped
  artifacts via the repo's own stamping commands, `specodelic/` artifacts
  (chore commit).
- Never edit: `src/` Rust code, `tests/` rust suites, `justfile`
  (archive-change already runs the companion), `pretender.toml`,
  `.github/**`, `openspec/decisions/**` (no new decisions scoped),
  `.wai/**`.
- Respect shrink-only ratchets: `pretender check` before committing.

## Repo facts (boilerplate — applies to every ticket)

- **Commit hygiene** (standing, every commit): before ANY `git commit`, run
  `git status --short` and stage only files YOU authored
  (`git add <paths>`, never bare `git add`); unstage foreign files.
  Attribute the message only to what the diff contains.
- **beads**: close tickets via bd, then `bd export -o .beads/issues.jsonl`
  and commit the export — the close is invisible to git otherwise.
- **Do NOT push** — the orchestrator verifies and pushes.
- **Do NOT run `wai close`** — the orchestrator owns session close.
- **Non-interactive shells**: use `-f`/`-rf`/`-y` flags.

## Gates to run before reporting (final meters)

- `just ci` → green (compiled artifacts regenerated → separate chore commit)
- `spk lint openspec` → 0 findings
- `ah check` + `ah check --run-tests` → green, every deployed scenario
  contracted
- `python3 scripts/spec_validate.py` → all pass (plain 1.x gate)
- `openspec validate --all --strict` → all items, 0 failed
- `pretender check` → gate mode passes
- `ls openspec/changes/` → only `archive/` remains (all six landed)

## Report format (end with this — the orchestrator verifies against it)

## Report

**Commits**
- `<hash>` <message> — <what it does, one line> (one per archived change)

**Gates run**
- <each meter → result>

**Deviations** (or "none")

**Next**
- <exact next action for the orchestrator, or "ticket complete">
