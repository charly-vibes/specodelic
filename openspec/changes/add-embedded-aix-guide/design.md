# Design: Embedded format guide

## Context

`spk` is published as a standalone binary. Consumer repos adopt the
Specodelic format by writing markdown files and running `spk` — they do
not clone this repo, so `specs/specodelic.md`, `specs/kinds.md`, and the
linter spec files are unreachable at runtime. The only knowledge an
agent can currently obtain is clap help text and a skeletal scaffold
template. This design covers how format knowledge gets into the binary
and how it stays in sync with the corpus it summarizes.

Constraints from the repo:

- Domain logic in the library (`src/lib.rs`), `main.rs` thin
- Everything emits via `genesis::guide::Output::emit` (envelope)
- The corpus is self-hosting; `spk lint specs` must stay clean
- Errors carry remediation hints (genesis Invariant 3.2.5)

## Goals / Non-Goals

- Goals: agent can learn the format offline from the binary; lint
  findings are self-describing; drift between binary and corpus is
  mechanically detected; consumer workspaces get valid doctor output
- Non-Goals: replacing the corpus as the format's source of truth;
  embedding the full human-oriented corpus; making the guide parseable
  by the linter (it is output, not a spec file); implementing pipeline
  commands

## Decisions

### Decision 1: Curated primer, not the raw corpus

Embed a distilled `src/guide.md` (~machine-facing tables and closed
sets, no revision archaeology, no CHANGELOG references), rendered into
six topics.

- Alternative considered: `include_str!` the corpus files wholesale.
  Rejected — the corpus is written for humans doing revision
  archaeology; it is large, self-referential ("this repo", filenames),
  and would force agents to filter noise. A primer is the API doc; the
  corpus is the annotated source.
- Alternative considered: vendoring the corpus into a `spk docs --raw`
  dump. Rejected for v1 — same noise problem, plus a second sync burden.
  Can be added later without breaking the topic API.

### Decision 2: Closed sets live in Rust, the guide renders them

The closed value sets (intent kinds, constraint kinds, property kinds,
reference typing pairs) become `pub const` slices in `src/guide.rs`.
The parser/linter consume the constants (replacing inline literal
matching); `explain kinds` / `explain references` render the same
constants into markdown rows at runtime. The embedded `src/guide.md`
prose therefore cannot disagree with the enforced sets — the sets are
not duplicated in prose at all.

- Alternative considered: two sources + a sync test comparing them.
  Rejected — a sync test catches drift after the fact; one source
  makes drift unrepresentable, which is the format's own design bias.
- Drift of *prose* (EARS patterns, lifecycle order, rule semantics) is
  still possible → a unit test asserts each topic renders non-empty and
  that `explain lint-rules` enumerates exactly the rule ids the linter
  can emit (the catalog is generated from the same table the linter
  uses, so this is near-tautological by construction).

### Decision 3: Topic API is stable and envelope-native

`spk explain` → `data.topics: [{id, title}]`; `spk explain <topic>` →
`data: {topic, format_revision, body}` where `body` is the rendered
markdown. `--human` prints `body` directly. Unknown topic → envelope
failure, exit 1, hint lists valid topics. Adding topics later is
append-only (same OCP bias as the format's tables).

### Decision 4: Format revision is a crate constant

`pub const FORMAT_REVISION: &str = "specodelic.md Revision 7"` (name it
after the corpus revision it mirrors, updated by hand when the corpus
revision bumps; a corpus-lint style test greps
`specs/specodelic.md`'s latest `Revision N` marker and fails if the
constant is stale — the repo's own CI catches the drift, not the
consumer). Surfaced in `--version --json` as `format_revision` and in
every `explain` payload.

### Decision 5: Doctor mode detection by marker file

Self-hosting mode ⇔ `specs/specodelic.md` exists; consumer mode
otherwise (including no `specs/` at all). Consumer mode checks: specs
dir present/absent (informational), beads config (informational, as
today), embedded guide revision (always ok), and — when a local corpus
exists — parses its latest `Revision N` marker and *warns* (not fails)
if it is newer than `FORMAT_REVISION`. Self-hosting mode preserves
today's checks. No marker file format is invented; the existing
`Revision N` heading in `specs/specodelic.md` is the marker.

### Decision 6: Scaffold guidance as HTML comments

`spk new` template guidance is markdown/HTML comments inside the
generated file (e.g. `<!-- kind: one of invariant | advisory | effect | extension_point -->`)
so they teach in-place and are trivially deleted, and the linter — which
never parses prose — is unaffected by them.

## Risks / Trade-offs

- Primer rot vs corpus → mitigated by Decision 2 (sets are one source)
  and the revision-marker test (Decision 4); prose rot is cosmetic, the
  enforced semantics can't drift.
- Guide content duplicated with corpus prose → accepted; the corpus
  remains normative (`specodelic.md` Revision discipline unchanged),
  the guide is a rendering with its own tiny maintenance surface.
- Envelope schema additions → additive only; documented as such in the
  proposal.

## Migration Plan

1. Add `guide.rs` + constants; refactor parser/linter literal matches to
   use them (pure refactor, tests unchanged).
2. Add `explain` command + revision constant + tests.
3. Extend lint findings; update `tests/cli.rs` assertions.
4. Upgrade `spk new` template; update scaffold tests.
5. Rework `doctor`; add consumer-mode fixtures in tempdirs.
6. `just ci` gates every step; corpus stays lint-clean throughout
   (guide is not a spec file, so it is exempt by the no-frontmatter skip
   rule — but it lives in `src/`, not `specs/`, so it is never scanned).

Rollback: revert the commit; no data or corpus changes.

## Open Questions

- Should `explain topics` include one topic per implemented linter rule
  eventually (e.g. `explain linter.coverage`) vs the single
  `lint-rules` catalog? Deferred until the remaining checkers
  (`specodelic-b15`) land; the catalog already carries per-rule
  semantics, so the per-rule topics are sugar.
