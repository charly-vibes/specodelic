# Subagent brief: specodelic-68m.7 — Strict verification policy ships with current contracts and guidance

You are a pi subagent in repo `/var/home/sasha/para/areas/dev/gh/charly/specodelic`
(Rust CLI `specodelic`, alias `spk`). An orchestrator has claimed ticket
`specodelic-68m.7` and is hands-off: you own the implementation end-to-end.
The orchestrator will verify with real gates — never claim work the diff
doesn't contain.

## Orientation (do this first)

- `export WAI_PROJECT=min-expr-kernel`
- Read, in order: the openspec change's `design.md`, `tasks.md` §4 (tasks
  4.1–4.3), `proposal.md`, `specs/AGENTS.md` (revision discipline), the
  dual-format archive recipe in
  `openspec/decisions/2026-10-01-espectacular-adoption.md` + spike findings
  under `openspec/changes/archive/2026-09-28-spike-dual-format/`, and the
  bxk precedent commits (`4d8181b`, `d50cc40`, `91ae76b`) which executed the
  identical recipe for add-min-expr-kernel.
- `wai search "<topic>"` — check accumulated patterns before designing.
- **User authorization**: the epic owner instructed the orchestrator to run
  the 68m epic to completion, including the sanctioned archive. The
  proposal's "after implementation approval" is satisfied by that
  instruction + all gates green.
- A parallel session works graph-view tickets — never stage its files.

## What to build

tasks.md tasks **4.1–4.3** (release discipline for
define-verification-claim-gates):

1. **4.1 — domain specs under Revision rules**: update
   `specs/model_check.md`, `specs/verify.md`, and `specs/specodelic.md`
   with the deployed requirements of the change (claim-gated verification,
   scope-bound evidence, report-view parity). Revision discipline: the
   corpus was just bumped to Revision 17 by bxk (`4d8181b`) — if your
   spec edits add NEW normative Constraints/requirement blocks to
   `specs/specodelic.md` itself, a further Revision 18 bump may be
   required (FORMAT_REVISION const + all `'specodelic.md Revision N'`
   literals: tests/cli.rs ×several, tests/cli/parse_misc.rs, guide-drift
   guard) with deriving Properties for every new Constraint. If your edits
   are confined to the dedicated `specs/model_check.md` / `specs/verify.md`
   capability files (no governed id-sets in specodelic.md touched), no
   bump. Decide by the rules in `specs/AGENTS.md` + guide-drift guard;
   document the decision. Every new Constraint needs deriving Properties.
   Regenerate compiled artifacts via `just ci`.
2. **4.2 — contracts + SUMMARY**: author genuine scenario contract TOMLs
   bound to the real tests for EVERY scenario the change deploys (bxk's
   `d50cc40` is the shape precedent; `.espectacular/` writable only for new
   TOMLs); update `docs/src/SUMMARY.md` for the capability; run `ah check
   --changes define-verification-claim-gates` with `--run-tests` — 0
   orphans, 0 missing bindings.
3. **4.3 — validation + archive**: `openspec validate
   define-verification-claim-gates --strict` green, section sync
   (`openspec validate --all --strict` 25/25), corpus lint 0 findings,
   `just ci` green — then archive ONLY via the dual-format recipe:
   `openspec archive define-verification-claim-gates --skip-specs` +
   verbatim `cp` of the specodelic-format deltas into `openspec/specs/`
   (default archive destroys the frontmatter/Constraints/Model/Properties
   layer). Preserve every deployed requirement and scenario identity
   through the archive (dry set-diff first, as bxk did). Contracts promote
   via `ah archive`. Post-archive: `ah check --run-tests` 0 orphans,
   `openspec validate --all --strict` green.
4. Update ONLY checkboxes 4.1–4.3 in the change's tasks.md (annotate with
   evidence, as bxk did). No push, no release, no beads close.

## Hard scope guard

- Allowed files: `specs/model_check.md`, `specs/verify.md`,
  `specs/specodelic.md`, `specs/CHANGELOG.md`, `specs/STATUS.md`,
  `src/guide.rs` (ONLY if a Revision 18 bump is required),
  `tests/cli.rs` (only revision-literal fixtures if bump), `tests/cli/parse_misc.rs`
  (only revision-literal fixtures if bump), `tests/cli/model_check.rs`,
  `tests/cli/orchestrate_verify.rs`, `.espectacular/` (new contract TOMLs
  only), `docs/src/SUMMARY.md`, `openspec/changes/define-verification-claim-gates/**`,
  `openspec/specs/**` (ONLY via the dual-format archive cp — verbatim,
  no hand edits), regenerated `specodelic/` artifacts.
- Never edit: `.espectacular/config.toml` and managed blocks,
  `.wai/resources/**`, this template, `src/graph.rs`, `src/main.rs`,
  `scripts/graph_views.py`, `scripts/test_graph_views.py`,
  `openspec/changes/add-graph-views/**` (parallel ticket's files).
- Respect shrink-only ratchets: run `pretender check` before committing.
- If the design requires touching files outside the allowed list, STOP and
  report — do not expand scope.

## Repo facts (boilerplate — applies to every ticket)

- **Commit hygiene** (standing, every commit): before ANY `git commit`, run
  `git status --short` and stage only files YOU authored
  (`git add <paths>`, never bare `git add`); unstage foreign files.
- **Output discipline**: every command emits through
  `genesis::guide::Output::emit`; errors carry a remediation hint.
- **beads**: issues live in `.beads/issues.jsonl`. — **you may NOT close
  this ticket**; the orchestrator owns close.
- **Do NOT push** — the orchestrator verifies and pushes.
- **Do NOT run `wai close`** — the orchestrator owns session close.
- **ah/espectacular**: the ONLY sanctioned write to `openspec/specs/` is the
  dual-format archive cp in this ticket.
- **Compiled-corpus artifacts** under `specodelic/` regenerate via just ci.
- **New source files** need Purpose/Responsibilities/Rationale headers.
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