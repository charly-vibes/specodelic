# Subagent brief: specodelic-lf3 (scoped-first) — executable property cells + verify gate

You are a subagent in repo `/var/home/sasha/para/areas/dev/gh/charly/specodelic`
(Rust CLI `specodelic`, alias `spk`). An orchestrator has claimed ticket
`specodelic-lf3` and is hands-off: you own the analysis + scoped
implementation end-to-end. The orchestrator verifies with real gates —
never claim work the diff doesn't contain.

## Owner decision (countersigned)

Scoped-first migration: migrate the core format files' property cells to
executable `**rust:**` fragments, wire `spk verify specs` into `just ci`
over an explicit scoped file list, extend per run. The ticket's original
"all cells" meter is stale (18/18 → 319 rows/23 files); the owned scope is
the four core files named below plus, only if none of them qualifies, the
smallest fully-expressible corpus file.

## Measured constraint you must design around (verified at HEAD)

- The props runner's scratch crate is `proptest`-only
  (`src/verify.rs` `SCRATCH_MANIFEST`) and generator cells compile to
  `Just(name)` constants — there is NO dependency on the specodelic lib.
- Therefore checker-behavior predicates (`check(file) == failed` style,
  the bulk of these files) are NOT honestly expressible today; pure
  text/format predicates (string/token logic over generated values) may be.
- Anti-goals: never weaken the partial-pass-is-failure rule
  (`verify.md` both_gates_required); never write `true`/trivial
  placeholders or `Just(name)` scaffolds as acceptance evidence.

## Your job

1. **Feasibility classification (deliverable A, committed)**: for
   `openspec/specs/specodelic/spec.md` (30 property rows),
   `openspec/specs/kinds/spec.md` (17), `openspec/specs/packs/spec.md`
   (16), `openspec/specs/compile/spec.md` (35) — classify every row:
   *expressible now* (predicate + generator honorably realizable as pure
   Rust over generated values, using `## Fragments` helpers where a body
   recurs) vs *needs runner harness* (requires the tool as a dependency, a
   fixture API, or real generator vocabulary). Write the table into the
   ticket's bd notes area via the report (bd is orchestrator-owned) and a
   committed research artifact under
   `.wai/projects/specodelic/research/2026-11-13-lf3-feasibility-<file>.md`.
2. **Scoped implementation (deliverable B, only if honest)**: if a core
   file is FULLY expressible, migrate its property cells to
   `**rust:**` predicates (generators too — real strategies, not
   Just(literal)) with `## Fragments` shared helpers where repeated, then
   add the `verify-specs` recipe to `justfile` scoped to exactly the
   migrated file list, failing on `properties_failed`, and confirm
   `spk verify` reports properties_failed 0 for that scope
   (`TMPDIR=/var/tmp/specodelic-xch1`). If no core file is fully
   expressible, do the same for the smallest fully-expressible corpus
   file, and name the scope explicitly in the recipe.
3. **If NOTHING qualifies fully**: do NOT wire the gate and do NOT fake
   cells. Deliverable becomes the classification + a concrete
   runner-readiness prerequisite report (what the harness must provide:
   tool-dependency seam in SCRATCH_MANIFEST or equivalent, fixture API,
   generator vocabulary) and file ONE bd ticket for it via `bd create`
   (orchestrator allows this single ticket because the run's outcome is a
   blocker; everything else in bd stays untouched). Then report
   `ticket NOT complete — prerequisite filed` with the evidence.
4. Corpus edits follow the dual-format discipline: property-cell
   migrations are corpus edits in `openspec/specs/**` — do them directly
   ONLY if they are layer-table rows (typing-table wording is not
   involved); any requirement/scenario change must ride
   `just archive-change`. `cargo fmt` before commits; `pretender check`
   green; targeted tests only (orchestrator owns full gates).

## Scope guard

- Allowed files: the four core corpus files above (+ the chosen fallback
  file), `justfile` (verify-specs recipe only), `tests/**`,
  `.wai/projects/specodelic/research/**` (your classification).
- Never edit: `src/**` (if the runner seam needs a code change, that is
  the prerequisite ticket, not this run), other corpus files,
  `.espectacular/**`, `pretender.toml`, run YAMLs, `.beads/**` except the
  ONE prerequisite ticket in case 3.

## Repo facts (boilerplate — applies to every ticket)

- **Commit hygiene**: `git status --short` before every commit; stage only
  your files (`git add <paths>`); blame only what the diff contains.
- **beads**: do NOT claim/close/update the ticket — the orchestrator owns
  bd state (single `bd create` exception above).
- **Do NOT push**; **do NOT run `wai close`** (a prior subagent's `wai
  close` corrupts the orchestrator's run bookkeeping).
- **Non-interactive shells**: `-f`/`-rf`/`-y`. **TMPDIR** as above.
- **Run `cargo fmt` before commits.**

## Report format (end with this — the orchestrator verifies against it)

## Report

**Commits**
- `<hash>` <message> — <what it does, one line>

**Classification**
- per-file: rows total / expressible / needs-harness (one line each)

**Gate wiring**
- recipe line + `spk verify` scope result, or "not wired: prerequisite <id>"

**Deviations** (or "none")

**Next**
- <exact next action for the orchestrator, or "ticket complete" / "ticket NOT complete — prerequisite filed">