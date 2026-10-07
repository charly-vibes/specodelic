# Subagent brief: specodelic-68m.3 — Verification rejects evidence from another input scope

You are a pi subagent in repo `/var/home/sasha/para/areas/dev/gh/charly/specodelic`
(Rust CLI `specodelic`, alias `spk`). An orchestrator has claimed ticket
`specodelic-68m.3` and is hands-off: you own the implementation end-to-end.
The orchestrator will verify with real gates — never claim work the diff
doesn't contain.

## Orientation (do this first)

- `export WAI_PROJECT=min-expr-kernel`
- Read, in order: the openspec change's `design.md` (this ticket is governed
  by **D3 — Report version and freshness** and **D2 — Aggregate rules**),
  `tasks.md` §2, `proposal.md` (slice scoping), then the delta specs this
  ticket extends.
- `wai search "<topic>"` — check accumulated patterns before designing.
- This ticket **maps to a tdd-ro5 run**. Start it and execute each step's
  prompt literally:
  `wai pipeline start tdd-ro5 --topic="specodelic-68m.3: verification rejects evidence from another input scope"`
  Advance ONLY via `wai pipeline next` — never hand-edit the run YAML under
  `.wai/pipeline-runs/`. Set `export WAI_PROJECT=min-expr-kernel` first so
  artifacts land in the right project.
- In the tdd-ro5 run, the `refactor` step must be a **no-op**: task 2.3 TIDY
  is a separate ticket (specodelic-68m.4). Do not bundle cleanup.

## What to build

tasks.md tasks **2.1 (RED)** and **2.2 (GREEN)**:

**2.1 RED — fixtures first.** Add command fixtures (CLI level, in
`tests/cli/`) for: old/unrecognized report schema version, missing claim,
duplicate claim, reversed file order, cross-file source edit, missing input
file, artifact mismatch, and two dual-format (`id: spec`) files with
identical local claim IDs but opposite outcomes. The last must fail a
combined invocation with `isolated_scope_required` and a hint to run each
file separately using separate artifact/report directories; separate runs
preserve outcomes, differ in digest, and reject swapped reports. Pin the
same preflight in CLI/model-check and orchestrate; multi-file dual-format
**lint** must remain valid. Observe each RED fixture failing for the
intended reason at HEAD before GREEN; record commands and results. A
zero-test filtered run is not evidence.

**2.2 GREEN — smallest implementation.** Versioned report
(`claim_schema_version=1`), canonical qualified claim records (id, evaluator
kind, status, reason/evidence where not verified), `expected_claim_ids`,
`unchecked_claim_ids`, and `scope_sha256` — SHA-256 of a deterministic
canonical serialization of all parsed structured input content plus sorted
compiled artifact hashes consumed for the invocation (sort files by
canonical intent ID, maps by key, preserve meaningful row/state order;
exclude prose, absolute paths, byte spans, timestamps, and CLI file order;
include pack/generator registry inputs when consumed; version the encoding
with the claim schema). Verify recomputes scope and the live required set
rather than trusting a stored `expected_claim_ids`. Old/unrecognized schema
versions, absent scope fingerprints, missing claims, and mismatched input
sets require a model-check rerun — emit rerun hints; **no hashes synthesized
for old reports** and no report rewriting to manufacture evidence. Prose-only
edits and input order changes preserve the digest. Ordinary corpus intent
IDs must be unique (`duplicate_corpus_identity` otherwise); a dual-format
`id: spec` file keeps `spec.<row>` claim IDs and resolves citations only
within that file, accepted only as the sole parsed input for evaluation
commands.

Do not change the D2 aggregate rules themselves — those landed in 68m.1.
This ticket is scope/freshness binding, not verdict policy.

## Hard scope guard

- Allowed files: `src/model_check.rs`, `src/commands/model_check.rs`,
  `src/orchestrate.rs`, `src/verify.rs`, `src/compile.rs`,
  `tests/cli/model_check.rs`, `tests/cli/orchestrate_verify.rs`,
  `openspec/changes/define-verification-claim-gates/tasks.md` (the 2.1 and
  2.2 checkboxes only).
- Never edit: `openspec/specs/`, `.espectacular/`, `pretender.toml`,
  `specs/` (the corpus), `.wai/resources/**`, this template.
- Respect shrink-only ratchets: run `pretender check` before committing; if
  a pinned source file breaches, move new tests to `tests/` rather than
  raising entries. `src/model_check.rs` is pinned (see `pretender.toml`) —
  new model-check tests go in `tests/` integration files.
- If the design requires touching files outside the allowed list, STOP and
  report — do not expand scope.

## Repo facts (boilerplate — applies to every ticket)

- **Commit hygiene** (standing, every commit): before ANY `git commit`, run
  `git status --short` and stage only files YOU authored
  (`git add <paths>`, never bare `git add`); unstage foreign files.
  Attribute the message only to what the diff contains.
- **Output discipline**: every command emits through
  `genesis::guide::Output::emit`; errors carry a remediation hint.
- **beads**: issues live in `.beads/issues.jsonl`; commit the export after
  `bd close` (`bd export` gotcha: run it or the close is invisible to git).
  — **you may NOT close this ticket**; the orchestrator owns close.
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
