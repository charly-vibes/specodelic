# Design — Dual-format deltas

## Context

Two spec systems coexist in this repo:

- **specodelic** (`specs/*.md`): the domain format this crate implements
  and dogfoods. Frontmatter + fixed-schema tables only; prose never
  inspected (`prose_untouched`).
- **openspec** (`openspec/`): the engineering change workflow
  (proposal → tasks → approval → archive). Parses only
  `### Requirement:` / `#### Scenario:` headers under delta/spec section
  headers.

The spike (`openspec/changes/archive/2026-09-28-spike-dual-format/`)
established the empirical facts this design rests on:

| Fact | Evidence |
|---|---|
| One file satisfies both parsers strictly | `spk lint` 0 issues ∧ `openspec validate --strict` valid on the same file |
| The parsers are grammar-disjoint | openspec reads headers; spk reads frontmatter+tables (`prose_untouched` makes the overlap benign) |
| Delta filename must be `spec.md` | renaming the delta → "No deltas found" |
| `unique_id` is per-file | two `id: spec` files lint clean when linted together |
| Default archive destroys the dual-format layer | regenerated `openspec/specs/compile/spec.md` had no frontmatter/tables |
| A file with `## Purpose` + `## ADDED Requirements` + `## Requirements` validates as delta AND capability spec | verified; delta parser counted 1 delta (reads only ADDED) |
| openspec requires `proposal.md` to recognize a change | bare delta-only change was "Unknown item" |
| `spk lint <dir>` is non-recursive and silently ok on 0 files | `collect_specs` single `read_dir` (`src/main.rs:176`); filed `specodelic-6pi` |

## Goals / Non-Goals

- Goals:
  - Requirement content authored once, in specodelic format, for both systems
  - Engineering truth (`openspec/specs/`) lintable and graphable forever
  - Coverage discipline (every SHALL has a deriving Property) applied to engineering requirements
  - Change lifecycle modeled as a state machine per change
- Non-Goals:
  - No changes to openspec upstream or to the specodelic format (no Revision)
  - No new `spk` lint rules in this change (convention first, tooling later)
  - No cross-file wiki-refs between deltas and the domain corpus

## Decisions

- **Decision: dual-format files, not generated deltas.**
  Alternatives: (a) hand mirrors — status quo, drift-prone, rejected;
  (b) `spk` renders openspec grammar from domain rows — lowest
  authoring cost but premature: zero deltas have archived yet, and the
  generator would be new tool surface built on an unproven grammar
  isomorphism. Dual-format is the minimal verified step; a generator
  remains a possible follow-up once the pattern has history.
- **Decision: accept `id: spec` on dual-format files.**
  Alternative: format revision giving openspec-owned paths a derived
  namespace (e.g. `openspec.<capability>`) — cleaner ids but touches
  `id_matches_file`, a core invariant, under `append_only_variants`
  discipline. Deferred until cross-file refs between capability specs
  are actually needed (the stated trigger).
- **Decision: self-contained deltas.** Wiki-refs resolve only within the
  delta; domain semantics referenced by prose path. This sidesteps the
  two-intents-named-`spec` ambiguity that cross-file refs would create.
- **Decision: `--skip-specs` archive + verbatim copy.** The archiver's
  regeneration is lossy by design (it can only keep what it parses);
  bypassing it preserves the specodelic layer. The copy is one line in a
  `just` recipe.
- **Decision: section-sync checked in CI, not merged.** The ADDED and
  Requirements sections must carry identical requirement text; a small
  diff-check fails loudly rather than a merge script silently
  normalizing (which would hide authorial edits to one side).

## Risks / Trade-offs

- **Intra-file duplication (ADDED ≡ Requirements)** → mitigated by the
  CI sync check (tasks 2.4); the wart is documented, not hidden.
- **Both parsers could change grammar independently** (openspec
  upstream updates; specodelic revisions) → the CI gates catch
  divergence at the first commit that breaks either side; the dual
  format is verified per-file, so breakage is localized.
- **Authoring overhead of a full Model section per delta** → accepted
  deliberately: the lifecycle model is the point (orchestration), not
  ceremony. Minimal viable model is 2 states.
- **`openspec validate --strict` is not yet in CI** → this change wires
  it; until then, drift is caught manually.

## Migration Plan

1. Land policy + tooling (tasks §1–2) — no existing artifact changes.
2. Pilot: convert `add-compile-functor`'s delta (tasks §3) — single file,
   both validators must stay green.
3. This change itself is dual-format from commit one (tasks §4) — the
   first archived dual-format file exercises the archive recipe.
Rollback: revert policy wording and remove recipes; dual-format files
remain valid openspec deltas either way (the specodelic layer is inert to
openspec).

## Open Questions

- Should `spk lint` grow a structural rule recognizing dual-format files
  (e.g. fail on frontmatter'd files that lack required openspec
  sections, or vice versa)? → follow-up ticket, not this change.
- Does `spk graph` need cross-root support (`specs/` + `openspec/specs/`
  in one graph) once archived capability specs accumulate? → revisit
  after the first real archive.
- Where exactly the section-sync check lives (shell script vs tiny
  `spk` subcommand) → implementation detail for the apply stage.