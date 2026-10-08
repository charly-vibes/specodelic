# Subagent brief: specodelic-9h3z — Dogfood add-quant-finance-pack: cross-pack, floor, kind-column, and orphan probes

You are a pi subagent in repo `/var/home/sasha/para/areas/dev/gh/charly/specodelic`
(Rust CLI `specodelic`, alias `spk`). An orchestrator has claimed ticket
`specodelic-9h3z` and is hands-off: you own the implementation end-to-end.
The orchestrator will verify with real gates — never claim work the diff
doesn't contain.

## Orientation (do this first)

- `export WAI_PROJECT=domain-specific-extensions`
- Read: `openspec/changes/add-quant-finance-pack/design.md` (D3 intra-file
  resolution, D4 risk floor, D5 Requires semantics, D2 prose-safety) and
  `tasks.md`; the delta
  `openspec/changes/add-quant-finance-pack/specs/quant-finance-pack/spec.md`
  (the Scenarios are the probe contract); the landed pack
  `packs/quant-finance.md` (commit a407ff0); and the bioimage precedent
  probes in `openspec/changes/archive/2026-10-03-add-bioimage-pack/tasks.md`
  §3 (probes 3.1–3.5 are the pattern).
- `wai search "dogfood probe"` — accumulated patterns.
- Start your tdd-ro5 run:
  `wai pipeline start tdd-ro5 --topic="specodelic-9h3z: quant pack dogfood probes"`
  Advance ONLY via `wai pipeline next`.

## What to build

Run the SIX probes and record the evidence in
`openspec/changes/add-quant-finance-pack/design.md` (new "## Probe
outcome" section, mirroring bioimage's "Review outcome" style) and check
off a new §2.3 checklist in `tasks.md` (add the items you actually ran).
Probe fixtures live ONLY in scratch git worktrees under `/var/tmp/qfp-9h3z`
(never /tmp, never inside the repo).

RED first (review finding DEP-001): the pack already landed (a407ff0), so
RED means running each probe in a scratch workspace with the pack REMOVED
or toggled — verify the labeled finding exists — then re-run with the pack
present and verify the clean/activated shape. Each probe = RED evidence +
GREEN evidence.

1. **Cross-pack probe (a)** — `capped_by` resolving against a declared
   `## Limits` row of the DECLARING file: file-local, no join, no
   single_root_reachable carve-out (D3). A dangling `capped_by` fires the
   labeled finding naming the row id and both remediations, exit non-zero
   (delta Scenario "dangling ... labeled").
2. **Kind-column probe (b)** — `kind = quant.risk` (and `quant.pricing`)
   in a base-table kind column accepted with the pack active (the ung
   mechanism); labeled finding with the pack removed from the workspace.
3. **Requires probe (c)** — a `## Requires` dep row naming a pack absent
   from the workspace yields the labeled advisory (not an error).
4. **Floor probe (d)** — a `quant.risk` law/property owing `**horizon:**`
   and `**confidence:**` case labels exercises the risk floor; base `law`
   floor findings BYTE-IDENTICAL with the pack discovered vs not
   (additivity); labels from consumed packs (numeric.predicates,
   data.lineage) ride along.
5. **Corpus-safety probe (e)** — `cargo run -- lint openspec` AND
   `just lint-specs` with all FIVE packs discovered: issues+warnings
   diffed byte-identical with vs without the quant pack on files using
   none of its vocabulary (the `no_pack_no_change` claim, now with all
   five); honest-empty closure: no `## Limits` rows exist in this repo, so
   `quant.limit_closed` reports an empty checked-set — capture the
   advisory wording.
6. **Orphan probe (f)** — a scratch spec file using `quant.risk`
   vocabulary (or `capped_by`) with the pack absent produces the labeled
   `linter.orphan_vocabulary` finding naming the candidate pack
   `quant.finance` and both remediations, non-zero exit; a declared
   `uses: [[quant.finance]]` edge with the pack absent behaves per the
   mechanism (advisory-first / labeled).

Also verify the known dual-format exception: the change's delta file
(`openspec/changes/add-quant-finance-pack/specs/quant-finance-pack/spec.md`)
vocabulary-activates the pack for the `spec` delta file — warnings channel
only, exit 0 (bioimage D7 precedent). Capture the warning text.

## Hard scope guard

- Allowed files: `openspec/changes/add-quant-finance-pack/design.md`
  (append "## Probe outcome"), `openspec/changes/add-quant-finance-pack/tasks.md`
  (§2.3 checklist), `.beads/issues.jsonl` (via `bd export` only).
- Probe fixtures stay OUTSIDE the repo tree (`/var/tmp/qfp-9h3z`).
- Never edit: `openspec/specs/`, `.espectacular/`, `pretender.toml`,
  `specs/`, `src/`, `tests/`, `packs/*.md`, `.wai/resources/**`, this
  template.
- If a probe FAILS (finding missing, wrong label, corpus diff not
  byte-identical): STOP the probe suite, record the failure verbatim in
  the probe outcome section as an open finding, and report it — do NOT
  patch `src/` (out of scope) and do NOT weaken the pack to make a probe
  pass. The orchestrator decides fix-forward.

## Repo facts (boilerplate — applies to every ticket)

- **Commit hygiene** (standing, every commit): before ANY `git commit`, run
  `git status --short` and stage only files YOU authored
  (`git add <paths>`, never bare `git add`); unstage foreign files.
  Attribute the message only to what the diff contains.
- **Output discipline**: every command emits through
  `genesis::guide::Output::emit`; errors carry a remediation hint.
- **beads**: issues live in `.beads/issues.jsonl`; commit the export after
  `bd close`. NOTE: do NOT close specodelic-9h3z yourself — the
  orchestrator closes it after verification. Run
  `bd export -o .beads/issues.jsonl` only if you change issue state.
- **Do NOT push** — the orchestrator verifies and pushes.
- **Do NOT run `wai close`** — the orchestrator owns session close.
- **ah/espectacular**: read-only over `openspec/`; no `.espectacular/`
  changes in this ticket.
- **Non-interactive shells**: use `-f`/`-rf`/`-y` flags.
- **Test runs**: set `TMPDIR=/var/tmp/qfp-9h3z-tmp` before `just test`
  (/tmp quota exhaustion masquerades as mass test failures).

## Repo gates (before you commit)

- `openspec validate --all --strict`
- `cargo run -- lint openspec` → 0 issues
- `just lint-specs` → 0 issues (probe-evidence prose must not activate
  anything)
- `ah check` → no NEW findings vs baseline (0 expected)
- `just test` (should be untouched-green; the pack corpus is additive)

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
