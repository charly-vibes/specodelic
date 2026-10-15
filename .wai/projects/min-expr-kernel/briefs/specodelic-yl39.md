# Subagent brief: specodelic-yl39 — orchestrate stem collision, namespaced per-file artifacts (gh-13)

You are a pi subagent in repo `/var/home/sasha/para/areas/dev/gh/charly/specodelic`
(Rust CLI `specodelic`, alias `spk`). An orchestrator has claimed ticket
`specodelic-yl39` and is hands-off: you own the implementation end-to-end.
The orchestrator will verify with real gates — never claim work the diff
doesn't contain.

## Orientation (do this first)

- `export WAI_PROJECT=min-expr-kernel`
- `bd show specodelic-yl39` — ticket carries a grounded Meter note
  (2026-10-15 @ a1bda72): stem-collision warning verified at
  `src/orchestrate.rs:313` — every file with `id: spec` compiles to
  `spec.toml/spec_props.rs/spec.tla` in one out-dir, later file wins.
  Green: `spk orchestrate` over a corpus with duplicate stems reports zero
  stem collisions and model_check reports zero `stale_artifact`. Test
  home: `tests/cli/orchestrate_verify.rs`.
- `gh issue view 13 --repo charly-vibes/specodelic` — upstream report:
  corpus-wide orchestrate with uniform `id: spec` → (1) all files compile
  to the same three artifact names in one out-dir, 29 stem-collision
  warnings, later file wins; (2) model_check then resolves every file's
  `.tla` to that single surviving module → `stale_artifact` for 29/30
  files, overall failed, verify skipped; (3) per-file orchestrate works.
- `wai search "orchestrate"` — accumulated patterns.

## Grounded code facts (verified live by the orchestrator)

- `compile::artifact_stem` (`src/compile.rs:1321`): a `spec.md` file
  derives its stem from the frontmatter id in dash form (Revision 18
  naming law); any other file uses its file stem. A corpus where every
  file is `spec.md` with `id: spec` yields stem `spec` for ALL files →
  collision.
- Compile stage `run_compile_stage` (`src/orchestrate.rs:~286`): writes
  all artifacts into ONE `out_dir`, tracks `seen_stems` and pushes the
  "stem collision … later file wins" warning at `:313` (collision is a
  warning; the stage still passes).
- Model-check stage: `citation_corpus::run_backend_pass`
  (`src/citation_corpus.rs:43`) resolves each file's module as
  `Path::new(out_dir).join(format!("{stem}.tla"))` — independent stem
  derivation, NO shared routing with the compile stage. With a colliding
  stem every file reads the SAME (last-written) `.tla` → the freshness
  digest matches only for the file that wrote it last → `stale_artifact`
  for the rest. This out-of-sync re-derivation is the root cause of
  symptom (2); the flat out-dir is the root cause of symptom (1).

## What to build

Make corpus-wide orchestrate never self-inflict stem collisions or
`stale_artifact` verdicts:

- Introduce ONE shared per-file artifact routing decision used by BOTH
  the orchestrate compile stage and the model-check backend pass
  (`run_backend_pass`), so the two stages can never disagree about where
  a file's artifacts live. Do not fix one stage and leave the other
  re-deriving with stale logic.
- Routing shape (decide, then document): either
  - **always-namespaced**: every orchestrate compile/model-check writes
    under `out_dir/<stem>/` (or another deterministic per-file dir), or
  - **collide-only**: unique stems keep today's flat layout; duplicate
    stems get a deterministic namespaced dir (must be derived from
    something guaranteed unique across the corpus — NOT the id, which is
    exactly what collides; the source file path is the honest key).
  Either is acceptable; the invariants that matter:
  1. two files that resolve to the same stem NEVER share an artifact
     path;
  2. both stages route through the same helper (no second derivation);
  3. a re-run of orchestrate over an unchanged corpus is byte-stable
     (same routing every run — no hash-of-content keys, no iteration
     order dependence);
  4. per-file orchestrate behavior must not regress.
- The compile stage's stem-collision warning becomes unreachable by
  construction — keep or retire it deliberately: if the routing makes
  the flat-dir collision impossible, retire the warning and say so in
  the commit message; if collide-only routing keeps a residual flat
  path, keep the warning as the failsafe.
- The `compiled[].written` paths in the compile stage's Output payload
  must reflect the real written paths (they flow to consumers).
- The persisted `<stem>.check.json` report path follows the same
  routing (it is written next to the `.tla` today).
- TDD: red test first in `tests/cli/orchestrate_verify.rs` — build a
  temp corpus with TWO (or three) `spec.md` files both carrying
  `id: spec` (each in its own dir, each with a distinct Model section so
  a collision is observable), run `spk orchestrate` over the corpus:
  red today (29:1 style — stem-collision warnings + `stale_artifact`
  failures for all but one file), green after: zero stem collisions,
  zero `stale_artifact`, both files' artifacts coexist, `overall` not
  failed by artifact staleness. Add the byte-stable re-run assertion
  (same routing twice).
- Out of scope: upstream item (3) — the `exploration_only`-still-failed
  distinction (CLAR-003) for per-file runs. Do NOT half-land it; if you
  touch adjacent behavior accidentally, report it explicitly.

## Hard scope guard

- Allowed files: `src/orchestrate.rs`, `src/compile.rs`,
  `src/citation_corpus.rs`, `tests/cli/orchestrate_verify.rs`
  (`citation_corpus.rs` added because the model-check stage's stem
  re-derivation lives there; keep the change minimal — the shared
  routing helper should live in `compile.rs` and be called from both)
- Never edit: `openspec/specs/`, `.espectacular/`, `pretender.toml`,
  `specs/` (the corpus), `.wai/resources/**`, this template.
- Respect shrink-only ratchets: run `pretender check` before committing; if a
  pinned source file breaches, move new tests to `tests/` rather than raising
  entries. `src/model_check.rs` is pinned (see `pretender.toml`) — new
  model-check tests go in `tests/` integration files.

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
