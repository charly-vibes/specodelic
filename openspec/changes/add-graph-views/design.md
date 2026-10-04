# Design: graph views

## Context

`spk graph` already derives the typed reference graph from any
specodelic-format corpus (verified against this repo's corpus: 411
nodes/~500 edges; against `../bajan/specs`: 159 nodes/188 edges, zero
corpus-specific code). The Reference Typing table
(`specs/specodelic.md`) is part of the *format*, so every compliant
corpus shares the same morphism signatures — extraction is corpus-agnostic
by construction. What is missing is a parseable projection between the
JSON envelope and any rendering, and the views themselves.

A Rule-of-5 review of the originating discussion converged with binding
corrections, folded in as D2, D3, D4, D5, and D7 below. A grounding Ro5
review (2026-09-29) verified the referenced surfaces against the live
corpus and folded in: the empty-corpus vs intentless-corpus distinction
(tool-level projection stays well-formed per `specs/graph.md`'s
parsed-not-linted note; the transform's scope gate refuses intentless
corpora), the AGENTS.md output-discipline exception note in D3, and the
tab-safety pin on reason text (task 1.2). Two review
findings are *evidence-backed*: the raw graph output contains display
labels (`refactor (intent)`) alongside ids, and this repo's own corpus
currently produced 38 typing violations when drafted (verified via
`spk graph -j` at the time; the acset-core landing typed them away —
0 as of 2026-10-04) that
a rendered view must not silently hide. A second review of this proposal
itself verified the referenced CLI surfaces, catching that `spk guide
--json` did not exist (D4).

## Goals / Non-Goals

- Goals:
  - A deterministic, pipeline-friendly edge list (`--format edges`) that
    `awk`/`jq`/scripts consume without a JSON parser.
  - Views a reader can think with: per-file state machines, file-level
    traceability, the schema's own shape — all derived, none authored.
  - Views are never cleaner than the artifact: violations ride along.
  - Works on any specodelic-compliant corpus (see D7 for the operational
    definition of "compliant").
- Non-Goals:
  - The free-monoidal-category / string-diagram IR (D5 — deferred).
  - Interactive rendering of the full entity graph (L3).
  - An adapter for openspec-layout corpora (`specs/<domain>/spec.md`;
    `spec.md` filenames cannot satisfy `id_matches_file` without a
    parse-boundary adapter — separate future change).
  - Any format Revision; any change to `specs/*.md`.
  - Committing rendered artifacts.

## Decisions

### D1 — The transform lives in `scripts/` as a prototype

`scripts/graph_views.py` (Python 3 stdlib only, no new dependency) consumes
the edge TSV plus the JSON envelope and emits Mermaid. Promotion into a
crate or an `spk` subcommand is deliberately deferred until the v1 views
demonstrate value — avoids building a beautiful IR nothing renders.

*Alternatives considered:* `spk graph --format mermaid` directly (rejected:
couples extraction to one rendering and grows main.rs); a new crate
(rejected: premature until value is proven).

### D2 — Canonical node ids only in projections

The raw graph output mixes frontmatter ids with display labels
(`refactor (intent)`, frontmatter-derived pseudo-nodes). The edge
projection emits frontmatter `id` values exclusively; display labels never
appear in the TSV. Where extraction currently attaches label-qualified
nodes, normalization happens at projection (or extraction, if cleaner —
implementation detail, but the TSV contract is ids-only).

*Evidence:* specodelic-corpus graph JSON contains edges with
`from: "refactor (intent)"` and fan-in keys `merge (intent)`.

Multiplicity: normalization can collapse distinct pseudo-edge instances
onto identical `(from, kind, to)` triples. The projection preserves
multiplicity (one row per reference instance — dedup would violate
total_extraction); view-layer fan-in counts *distinct* targets.

### D3 — Violations ride along; silent-clean is forbidden

`graph.md` mandates forbidden edges are "reported, never recorded" — so an
edge list alone would render the 38 typing violations this corpus
knows when they exist as a clean graph, a false model. The projection therefore includes one
annotation row per violation, mapped into the same six columns (empty
source id/kind, `violation:<edge_kind>` in the field column, target
id/kind, annotation column carrying the finding), and the transform
renders them as dashed/annotated elements. A view with zero violations is
byte-honest only because violations were absent from the artifact, not
dropped by a view. Open for task 1.2: whether the annotation column
carries the full reason text or a class code (reason then only via JSON).

*Evidence:* `spk graph` over `specs/` reported 38 `violations[]`
entries when drafted (e.g. `linter.frontmatter.has_id` →
`specodelic.frontmatter_valid`, `transitions.guard` → intent targets);
0 as of 2026-10-04 after the acset-core landing — violation-rendering
is exercised by fixture corpora instead.

Flag precedence: `--format edges` emits raw TSV directly to stdout and
overrides envelope formatting (`--json`/`--human`); documented in the
flag's help. This is a deliberate, documented exception to AGENTS.md's
output-discipline convention ("every command emits through
`genesis::guide::Output::emit`") — the raw-projection use case (awk/jq
pipelines) is the point, and the exception is scoped to this one flag,
not the command.

### D4 — The schema view is derived from the acset `Schema` value, revision-labeled

*Revised 2026-10-04 (specodelic-hya, after add-acset-core landed).* The
Reference Typing table now exists as data — `acset::schema::canonical()`
— and is **lint-gated** against `specs/specodelic.md` row-for-row by
`schema_matches_typing_table` (lint fires on divergence in either
direction). That gate closes the drift concern this decision originally
answered: the view's source is verified current by the same discipline
that polices the format corpus itself.

The schema view therefore derives from the `Schema` value (objects +
morphisms) and nothing else. `guide`'s `REFERENCE_TYPING` const stays
out of the derivation path entirely — it is ungated code and would
recreate the two-sources-for-one-table problem. It remains the embedded
guide's rendering surface only (`{{reference_typing}}` prose), a
value-set consumer.

The revision label comes from `guide::FORMAT_REVISION` (the binary's
implemented-revision marker) — a version label, not a duplicate of the
typing table. `spk guide --json` still ships (kinds, row shapes,
format_revision) for value-set-only consumers, but the schema view no
longer consumes it.

*Alternatives considered:* keep both sources plus a cross-check failing
the build on divergence (rejected: redundant machinery — the Schema is
already doc-gated at lint time; a second gate re-verifies a verified
value and doubles the sync surface).

*History:* the original D4 (2026-09-29) pinned the view to `guide`'s
closed value sets served via `spk guide --json`, because no data source
for the table existed outside the ungated `guide` module.
Never-hardcoded-in-the-renderer is retained unchanged.

### D5 — Categorical IR deferred, with operational definitions recorded

The monoidal/string-diagram layer is a non-goal for v1. To keep the
option open without re-derivation, the intended operational semantics are
recorded here: objects = kinds; generators = typed reference fields;
`⊗` (parallel) = disjoint union of edge sets; `;` (sequential) =
composition along shared nodes. A series-parallel expression may be
derived from a file's state machine **only if** that machine is
series-parallel (acyclic apart from a single failure sink) — arbitrary
Moore machines are not, and the design must not assume otherwise
(orchestrate's own machine has a failure sink reachable from every stage).

### D6 — Rendered artifacts are build-time, never committed

`just docs-graphs` generates Mermaid includes into `docs/src/views/`
(gitignored — chosen because `docs-build`'s `rm -rf docs/src/specs
docs/src/openspec` does not touch it) at build time. Nothing is
committed, so `graph_is_derived_not_authored` is satisfied by
construction (no hand-edit path exists) and no staleness check is
needed — an artifact that is never stored cannot go stale.

*Alternatives considered:* committing generated `.mermaid` files with a CI
drift check (rejected: drift machinery for zero benefit — regeneration is
cheap and deterministic).

### D7 — Corpus scope is operational, not aspirational

A corpus is in scope iff (a) `spk lint` over it reports no
`invariant`-rule findings and (b) ≥1 intent file parses. This admits
bajan-style corpora and formally
excludes openspec-layout repositories (`espectacular`, `poco`), whose
deployed layer deliberately ignores the structured specodelic layer and
whose `spec.md` filenames violate `id_matches_file`. Scope widening is a
parse-boundary adapter change, not a graph-views change.

### D8 — Native text projections; rendering is always external

(Added 2026-10-01, specodelic-5qj decision,
`openspec/research/2026-10-01-wiring-view-decision/`.) `spk graph` emits
DOT and mermaid natively (`--format dot|mermaid`) as plain-text string
templates — zero external crates, byte-stable re-runs. The "no viz
dependencies" constraint rejects rendering *libraries*, not output *text
projections*: conflating them forced a JSON→jq→renderer ceremony that no
consumer could reproduce (the jq bridge was repo-local, absent from the
installed binary). Rendering stays 100 % external and user-chosen
(`dot`, `graph-easy`, `mmdc`, viz-js); the tool never shells out to a
renderer (no `--render` in v1 — it would break byte-stable artifacts via
graphviz version drift). Guidance ships embedded via `spk explain
graph-views`. An in-terminal ASCII renderer (ascii-dag) was evaluated and
dropped: it requires Rust 1.92 vs repo MSRV 1.88, and the terminal niche
is served by documenting `graph-easy`.

## Risks / Trade-offs

- Script rot (`graph_views.py` drifting from the TSV contract)
  → the TSV contract is tested in `tests/cli.rs` (Rust side) and the
  transform is tested against checked-in fixture corpora (empty,
  single-intent, violation-bearing) in the script's own tests; both fail
  loudly on drift.
- Mermaid ceiling (readability collapses past ~a few hundred nodes)
  → accepted for v1; file-level and per-file views stay small by
  construction; the full-entity view is a non-goal.
- Bajan-style corpora with prose-only wiring render sparse composition
  views → accepted; the gap is real corpus feedback (follow-up beads
  issue against bajan), not view-layer magic.

## Migration Plan

Additive: new flag, new script, new just recipe, one docs page. No
existing output changes; JSON envelope untouched. Rollback = revert.

## Open Questions

- Should the TSV annotation rows carry the full violation reason (wide
  rows) or a violation class code only (narrow rows, reason via JSON)?
  Decided during task 1.2 against real output.
- Does the file-level traceability view want fan-out too, or fan-in only?
  Dogfood and decide.
