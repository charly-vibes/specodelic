# Subagent brief: specodelic-x0wy — audit docs + layer: graded-evidence page + specs/audit sidecar (tasks 5.1-5.2)

You are a pi subagent in repo `/var/home/sasha/para/areas/dev/gh/charly/specodelic`
(Rust CLI `specodelic`, alias `spk`). An orchestrator has claimed ticket
`specodelic-x0wy` and is hands-off: you own the implementation end-to-end.
The orchestrator will verify with real gates — never claim work the diff
doesn't contain. **This is the LAST ticket of the add-evidence-audit change:
your archive step closes the whole change.**

## Orientation (do this first)

- `export WAI_PROJECT=specodelic`
- Read, in order: `openspec/changes/add-evidence-audit/tasks.md` §Docs +
  layer (your tasks are 5.1 + 5.2), `proposal.md` (the Wave-1 scatter
  list), `design.md`, the delta spec `specs/audit/spec.md`, and the
  EXISTING layer sidecar `openspec/changes/add-evidence-audit/specs/audit/
  layer.md` (66 lines, already drafted — your job is to validate it
  against the IMPLEMENTED capability, not write it from scratch).
- All four slices are LANDED at HEAD: audit oracle rows + pack rows
  (xch1/8ebr), unified emission (qqgn), binds_nothing vacuity (edrv),
  shared declared-ids index (l6md), rung aggregate + relocation
  (37hl/6hou — `src/audit/rung.rs`). `just ci` exit 0 at HEAD.
- **docs-home DECISION (recorded)**: the graded-evidence page is a NEW
  page `docs/src/graded-evidence.md` — per-verb page convention
  (`conform.md` exists), NOT a fold into `verification-boundaries.md`
  (that page explicitly scopes itself as the conceptual ladder +
  limitations page). Cross-link both ways. `wai search "docs-home"` for
  the full decision record.
- `wai search "rung"`, `wai search "verification boundaries"`,
  `wai search "glossary"` — the ladder docs landed via lf4b.6; glossary
  discipline (lf4b.7): every jargon term used must be defined in
  `docs/src/glossary.md` or added there.
- Start your tdd-ro5 pipeline and execute each step's prompt literally:
  `wai pipeline start tdd-ro5 --topic="specodelic-x0wy: docs + layer + archive"`
  Advance ONLY via `wai pipeline next` — never hand-edit the run YAML under
  `.wai/pipeline-runs/`.

## What to build

### 5.1 — the graded-evidence docs page

- New `docs/src/graded-evidence.md`: the `spk audit` verb page. Cover: what
  audit grades (evidence the toolchain touches but never re-validates), the
  closed class vocabulary {current, stale, unchecked, contradicted,
  binds_nothing, new} and the both-layer record (conform verdict + audit
  class), honesty channels (unchecked/binds_nothing: countable, never a
  pass, never exit-nonzero), the per-file ladder-rung aggregate with the
  delta scenarios, and a runnable example end to end.
- **Limitations discipline** (the verification-boundaries.md pattern: every
  limitation backed by a command the reader can run): carry the 37hl
  findings — (1) rung 4 is unreachable through `spk audit` in v1 (verify
  persists no report — "verified is not cached"; the IO collector never
  sets verify_pass); (2) the rung-1 floor is CLI-unreachable (the artifact
  gate's lint-clean implies model presence; pinned at the pure core).
- Do NOT duplicate verification-boundaries.md's rung-by-rung exposition —
  link it as the conceptual ladder home.
- `docs/src/SUMMARY.md`: one entry, adjacent to the
  verification-boundaries entry. `just docs-build` must be green (it
  regenerates llm.txt via the stamp root phase); glossary discipline
  applies.
- Tick checkbox 5.1.

### 5.2 — layer sidecar validation + the archive (closes the change)

- Validate every `layer.md` row against the IMPLEMENTED behavior:
  `closed_classes`, `honesty_channels` (unchecked AND binds_nothing),
  `binds_respect_sparse`, `rung_derived`; properties `p_honesty`
  (`arbitrary_row_mix`), `p_sparse` (`sparse_phase_model`), `p_rung`
  (`arbitrary_evidence_mix`) must match the landed test shapes
  (`binds_nothing_tests` in src/audit.rs, `tests/audit_rung.rs`,
  `tests/audit_binds_nothing.rs`). Fix rows ONLY where the draft drifted
  from what actually shipped — the layer documents what IS, not wishes.
- Tick checkbox 5.2 AFTER the archive passes (the checkbox claims it).
- Run `spk lint openspec/changes/add-evidence-audit` green (the change
  corpus must lint before archiving).
- **`just archive-change id=add-evidence-audit`** — the ONLY sanctioned
  write path to `openspec/specs/` (Revision 19 companion recipe:
  `openspec archive --skip-specs` + verbatim delta deploy, fail-closed on
  any delta lacking the dual-format layer). NEVER run stock
  `openspec archive` (it destroys the specodelic layer). NEVER hand-edit
  deployed specs under `openspec/specs/`.
- Post-archive gates, in order: `openspec validate --all --strict`;
  `ah check` — author the espectacular contract TOMLs for the deployed
  audit spec's scenarios IN the archive commit (they are orphans until
  archive — this is the sanctioned moment); `spk lint` over the deployed
  corpus green; if `just ci` demands a `specodelic/audit.check.json`
  model-check artifact, produce it with an honest `spk model-check` run
  (NEVER hand-write/fabricate one — the 12e incident precedent); full
  `just ci` green as the final gate.
- Commit shape (precedent: bhp/edrv archives): the archive + contract
  TOMLs may be one commit; keep docs page separate if cleaner.

## Hard scope guard

- Allowed files: `docs/src/graded-evidence.md` (new),
  `docs/src/SUMMARY.md` (one entry line),
  `docs/src/glossary.md` (only if a new term is unavoidable),
  `openspec/changes/add-evidence-audit/**` (layer.md row fixes, tasks.md
  ticks 5.1/5.2),
  `openspec/specs/**` ONLY as produced by `just archive-change`,
  `.espectacular/**` contract TOMLs for the audit spec's scenarios,
  `specodelic/audit.check.json` ONLY if an honest model-check run
  produces it and a gate demands it.
- **NO source-code changes**: `src/**` is off-limits — this is a docs +
  spec ticket. If a code change seems required, STOP and report it as a
  blocker with the exact evidence.
- Never edit: `pretender.toml`, `.wai/resources/**`, this template,
  `README.md`, `docs/src/verification-boundaries.md` (the DECISION keeps
  it scoped as-is; cross-link, don't rewrite).

## Repo facts (boilerplate — applies to every ticket)

- **Commit hygiene** (standing, every commit): before ANY `git commit`, run
  `git status --short` and stage only files YOU authored
  (`git add <paths>`, never bare `git add`); unstage foreign files.
  Attribute the message only to what the diff contains.
- **Output discipline**: every command emits through
  `genesis::guide::Output::emit`; errors carry a remediation hint.
- **beads**: issues live in `.beads/issues.jsonl`; the orchestrator closes
  the ticket — you do NOT run `bd close`.
- **Do NOT push** — the orchestrator verifies and pushes.
- **Do NOT run `wai close`** — the orchestrator owns session close.
- **ah/espectacular**: read-only over `openspec/` EXCEPT the sanctioned
  archive recipe path described above (the recipe is the write; you never
  hand-edit specs).
- **New source files** need Purpose/Responsibilities/Rationale headers
  (file-headers skill convention).
- **Non-interactive shells**: use `-f`/`-rf`/`-y` flags; commands may be
  aliased with `-i` and will hang on prompts.
- Test runner during iteration: `just test-smart` (falls back to
  `cargo test`); full `just ci` gate before you report done.
  **`export TMPDIR=/var/tmp/specodelic-xch1` (create it; it exists) is
  MANDATORY for every cargo run** — /tmp tmpfs OOM-killed cargo twice in
  the 8ebr run; don't wait for it to bite you.

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
