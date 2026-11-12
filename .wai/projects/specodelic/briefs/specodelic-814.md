# Subagent brief: specodelic-814 — spec-doc sync: guard row must name the `**rust:**` escape hatch

You are a subagent in repo `/var/home/sasha/para/areas/dev/gh/charly/specodelic`
(Rust CLI `specodelic`, alias `spk`). An orchestrator has claimed ticket
`specodelic-814` and is hands-off: you own the edit end-to-end. The
orchestrator will verify with real gates — never claim work the diff
doesn't contain.

## Orientation (do this first)

- `export WAI_PROJECT=specodelic`
- Read: `bd show specodelic-814` (acceptance + meter);
  `openspec/specs/specodelic/spec.md` — the Reference Typing table
  (`guard` row, line ~76), the Revision headings at the end (last is
  `## Revision 19 — 2026-10-24`), and the executable-fragment definition
  rows (`predicate_fragment_opt_in`, `invariant_fragment_opt_in`) plus
  `openspec/specs/model_check/spec.md` (Revision 15,
  `executable_invariants_execute`) for the exact hatch semantics to cite;
  `src/guide.rs` `FORMAT_REVISION` and `src/guide.md` (revision pinning +
  drift guard conventions); `openspec/AGENTS.md` (corpus edit discipline);
  `justfile` docs recipes.
- `wai search "machine-uninterpreted"` and `wai search "revision"` —
  accumulated patterns.
- Start and execute the executor pipeline literally:
  `wai pipeline start tdd-ro5 --topic="specodelic-814: guard-typing rust escape hatch"`
  Advance ONLY via `wai pipeline next` — never hand-edit run YAMLs. This is
  a docs/spec edit: map RED→GREEN honestly (agreement-asserting check first
  — e.g. a grep/doc assertion pinning the row's both-regime wording — then
  the edit), and record the mapping; do not fabricate code test cycles.
- `export TMPDIR=/var/tmp/specodelic-xch1` (mkdir -p it) — MANDATORY for
  cargo invocations.

## What to change

The `guard` row's "Must resolve to" cell currently ends:
"…a prose-only guard is machine-uninterpreted (the model-check backend
reports `invariants_checked: []`)". It must state BOTH regimes:

1. prose-only guard → exploration-only; `invariants_checked: []` honestly —
   never counted as verified;
2. a guard/invariant fragment opted in with the `**rust:**` marker
   (Revision 15) → executed natively by model_check's executable backend;
   `no_counterexample` can be honestly earned.

Keep the honest-reporting framing (the old sentence is incomplete, not
wrong). Grep `machine-uninterpreted` across the whole repo first: the ONLY
canonical site is `openspec/specs/specodelic/spec.md`; `docs/src/specs/**`
and `docs/book/**` are generated/gitignored — do NOT hand-edit them.

Discipline (ticket): typing-table wording is revision-gated — append a
`## Revision 20 — 2026-11-12` heading to the corpus file (append-only; do
not rewrite Revision 19) recording the guard-row sync in one or two
sentences. Then:
- bump `FORMAT_REVISION` in `src/guide.rs` 19 → 20 if the drift-guard
  test pins it (read the test first; if the pin is numeric-newer-only,
  bump and make the guide's revision note accurate);
- refresh the AGENTS.md managed block via `spk init --force` and verify
  the diff is catalog/block-only;
- run the docs pipeline once (`just docs-build` or the narrowest docs
  recipe that regenerates docs/src/specs) and confirm it either
  regenerates the mirror or is clean.

Acceptance meter (verbatim): `grep -c 'machine-uninterpreted'` sites show
only occurrences followed by the `**rust:**` escape-hatch mention (target:
0 bare hits); `spk lint` on the corpus at 0 findings; if you find the edit
genuinely needs an openspec change instead (it should not), stop and say so
in the report rather than half-doing both paths.

## Hard scope guard

- Allowed files: `openspec/specs/specodelic/spec.md` (guard row + new
  Revision 20 heading ONLY), `src/guide.rs` (FORMAT_REVISION + its pinning
  test if needed), `AGENTS.md` ONLY via `spk init --force`, docs generated
  artifacts only via the docs recipe.
- Never edit: other corpus files, `src/**` beyond guide.rs, `tests/**`
  beyond a revision pin if one exists, `.espectacular/**`, `specs/`,
  `pretender.toml`, `.wai/resources/**`, run YAMLs.
- Respect shrink-only ratchets: run `pretender check` before committing.

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
- **ah/espectacular**: read-only over `openspec/`.
- **Non-interactive shells**: use `-f`/`-rf`/`-y` flags; commands may
  be aliased with `-i` and will hang on prompts.
- **Run `cargo fmt` before your commit** if you touch Rust.

## Report format (end with this — the orchestrator verifies against it)

## Report

**Commits**
- `<hash>` <message> — <what it does, one line>

**Acceptance meter**
- grep hits before/after (bare vs both-regime), spk lint result, revision heading added

**Deviations** (or "none")

**Next**
- <exact next action for the orchestrator, or "ticket complete">