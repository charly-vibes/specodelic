# Subagent brief: specodelic-q7uo — unescape `\|` before fragment emission

You are a subagent in repo `/var/home/sasha/para/areas/dev/gh/charly/specodelic`
(Rust CLI `specodelic`, alias `spk`). An orchestrator has claimed ticket
`specodelic-q7uo` and is hands-off: you own the implementation end-to-end.
The orchestrator will verify with real gates — never claim work the diff
doesn't contain.

## Orientation (do this first)

- `export WAI_PROJECT=specodelic`
- Read: `bd show specodelic-q7uo`; `src/compile.rs` — `FRAGMENT_MARKER`
  (`**rust:**`, ~line 294), the fragment extraction/validation helpers
  (~lines 440–690: span-opening detection, `fragment.text`,
  `hygiene_violation`, the fragment struct), `properties_to_proptest`
  (~852–1050, fragment emitted verbatim ~983), `constraints_doc`/toml
  emission (~40–62), `model_to_tla` manifest comment (~1157–1164);
  `src/model_check.rs` — `scratch_run_source` (~671–760, fragments
  verbatim in the scratch crate's `src/main.rs`); `src/guide.md` lines
  43–44 (parser splits cells on unescaped pipes; the escape is kept).
- `wai search "fragment"` and `wai search "escape"` — accumulated
  patterns first (catitas study A1; Revision 15/16 fragment machinery).
- Start and execute the executor pipeline literally:
  `wai pipeline start tdd-ro5 --topic="specodelic-q7uo: compile pipe unescape"`
  Advance ONLY via `wai pipeline next` — never hand-edit the run YAML under
  `.wai/pipeline-runs/`.
- Set `export TMPDIR=/var/tmp/specodelic-xch1` (mkdir -p it) — MANDATORY
  for all cargo invocations in this repo.
- Grep once to enumerate the fragment consumers before editing:
  `grep -n "fragment" src/compile.rs src/model_check.rs | head -60`.

## What to build

The parser splits table cells on unescaped pipes and KEEPS the `\|`
escape in the cell text. Fragment extraction slices that text after the
`**rust:**` marker and emits it verbatim into `_props.rs`, the
constraints `.toml`, the `.tla` manifest comment, and the model-check
scratch crate — so a Rust `||`, `|=`, or closure `|x|` is impossible in
a fragment (it arrives as `\|\|` and does not compile).

1. **RED first** (test file(s): `tests/` — follow existing compile/verify
   test layout; do NOT add unit tests inside `src/compile.rs` or
   `src/model_check.rs` if pretender ratchets pin them): a fixture spec
   whose executable fragment cell contains an escaped pipe (e.g. an
   invariant/property fragment using `\|\|` in the cell) compiles to a
   `_props.rs` whose emitted Rust contains the literal `||`; assert the
   compiled artifact compiles as Rust and that `spk verify` can pass on
   it (or the closest honest verify-path assertion the existing test
   harness supports — a compiled-artifact compilation check is the
   minimum). Also pin byte-stability: for a fragment WITHOUT `\|`, the
   emitted artifacts are byte-identical to the pre-change output
   (fixture-level, or rely on an existing byte-stable snapshot suite and
   state which one proves it).
2. **GREEN**: unescape `\|` → `|` at the extraction/emission boundary —
   ONE helper applied where fragment text is produced (so all four
   consumers get the same unescaped string), not four scattered
   replacements. Keep it surgical: no parser change, no lint/graph
   change, no rename/refactor change.
3. **HARD CONSTRAINT**: `src/packs.rs` lines ~154–161 (its own internal
   `\|` protection in the pack-table cell splitter) MUST NOT be touched —
   that is a different path with different semantics.
4. **REFACTOR**: separate commit ONLY if a real tidy emerges; otherwise
   record "none needed" — do not manufacture churn.
5. **Iteration testing**: TARGETED tests only during red-green cycles
   (`TMPDIR=... cargo test --test <name> <filter>`). Do NOT run full
   `just ci`, `ah check`, `just lint-specs`, or `openspec validate` —
   the orchestrator owns full gates after your report.
6. **Before your commit**: run `cargo fmt` (the repo's fmt-check gate is
   red on unformatted commits — last run shipped a rustfmt nit that the
   orchestrator had to amend). `git add` only your files, then commit.

## Hard scope guard

- Allowed files: `src/compile.rs` (fragment extraction/emission helper +
  call sites ONLY), `src/model_check.rs` (scratch-crate emission call
  site ONLY), `tests/**` (new fixture/regression tests).
- Never edit: `src/packs.rs` (any change = contract violation),
  `src/spec.rs` parser, `openspec/**`, `specs/`, `.espectacular/**`,
  `pretender.toml`, `.wai/resources/**`, `.beads/hooks/**`, run YAMLs,
  `src/lint/**`, `justfile`.
- Respect shrink-only ratchets: run `pretender check` before
  committing; if a pinned source file breaches, move new tests to
  `tests/` rather than raising entries.

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
- **New source files** need Purpose/Responsibilities/Rationale headers
  (file-headers skill convention).
- **Non-interactive shells**: use `-f`/`-rf`/`-y` flags; commands may
  be aliased with `-i` and will hang on prompts.

## Report format (end with this — the orchestrator verifies against it)

## Report

**Commits**
- `<hash>` <message> — <what it does, one line>

**Tests run**
- targeted RED/GREEN cycles → <results>

**Deviations** (or "none")
- <any scope/plan deviation and why>

**Next**
- <exact next action for the orchestrator, or "ticket complete">