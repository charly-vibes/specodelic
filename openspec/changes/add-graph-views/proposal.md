# Change: Add graph views — deterministic edge-list projection and derived corpus diagrams

## Why

The spec corpus is designed to be the implementation-free model of a
system, but the only way to *see* its structure today is the raw JSON
envelope of `spk graph` — machine-readable, not human-readable, and not
pipeline-friendly. Nothing renders the typed reference graph into views a
reader can think with: per-file state machines, file-level traceability,
the schema's own shape. Meanwhile the sibling corpora (e.g. bajan) prove
the extraction layer is already corpus-agnostic (188 edges over 6 files,
zero corpus-specific code), yet no view layer exists for any of them.
Dogfooding note: `spk lint` caught a coverage gap in this very proposal's
first draft (a constraint with no deriving property) — the tool reviewing
its own change is the discipline this change extends to diagrams.

Design was reviewed under a Rule-of-5 pass (converged at Stage 4); its
binding corrections are folded in as decisions D1–D7 in `design.md`,
notably: canonical node ids in projections (raw output contains display
labels like `refactor (intent)`), typing violations are never silently
hidden by a rendered view (the specodelic corpus carried 38 when this
was drafted — the acset-core landing typed them away; 0 as of
2026-10-04, verified via `spk graph -j`),
the schema view is derived from `guide`'s closed value sets and labeled
with the format revision (not hardcoded), and the categorical
string-diagram IR is explicitly deferred.

## What Changes

- **Tool — `spk guide --json`:** new subcommand exposing the format's closed
  value sets (kinds, constraint/property row shapes, reference fields with
  allowed targets) and the format revision as JSON — the derivation source
  the schema view requires, which today exists only as the internal
  `guide` module (nothing serves it: `spk explain` serves prose topics
  only).
- **Tool — `spk graph --format edges`:** new output projection emitting a
  sorted, deterministic TSV edge list (canonical ids, one row per recorded
  edge plus annotation rows for violations). Purely additive to the
  existing `graph` command; JSON envelope remains canonical.
- **Tool — native text projections (`--format dot|mermaid`):** `spk graph`
  also emits Graphviz DOT and Mermaid as plain-text string templates —
  zero external crates, same discipline as the TSV (decision recorded in
  `openspec/research/2026-10-01-wiring-view-decision/`). Rendering stays
  100 % external (`| dot -Tsvg`, `graph-easy`, `mmdc`, viz-js): the tool
  depends on no graphing/viz library, and the happy path is one standard
  pipe instead of a JSON→jq→renderer chain.
- **Tool — `--view wiring`:** file-level producer→consumer projection of
  `constraints.satisfies` edges (the declared-dataflow view explored in
  specodelic-5qj), alongside the states/traceability/schema views.
- **Tool — transform prototype:** `scripts/graph_views.py` consuming the
  edge list plus `spk graph --json` (for violation reasons and fan-in) and
  `spk guide --json` (for closed value sets + format revision), emitting
  Mermaid. Lives in `scripts/` as a prototype — promotion out of `scripts/`
  is out of scope until a renderer proves the views' value.
- **Views (v1):** (a) per-file state-machine diagrams derived from
  `transitions.from/to/guard` edges; (b) file-level traceability map
  (edges collapsed to intents, fan-in annotated); (c) schema view derived
  from the format's Reference Typing value sets, labeled with the format
  revision; (d) the wiring view above. Violations render as annotated
  (dashed) rows — a view is never
  silently cleaner than the graph artifact (the corpus carried 38 typing
  violations when drafted; acset-core landed a lint-gated `Schema` value
  and typed them away — 0 as of 2026-10-04 — so future violations come
  from drift, caught by fixtures and the lint-time gate). Empty views are labeled
  (`no_transitions`, `no_wiring`), never silently clean.
- **Agent guidance:** a `spk explain graph-views` primer topic teaching
  the one-pipe render recipes (dot/graph-easy/mermaid consumers) — the
  embedded-principle pattern, so consumers can act without repo access.
- **Build wiring:** `just docs-graphs` regenerates all rendered views into
  the mdbook build at build time; rendered artifacts are never committed
  and have no hand-edit path (`graph_is_derived_not_authored`, trivially
  satisfied by construction).
- **Corpus feedback (separate follow-up, not this change):** bajan's
  extension-point wiring (e.g. `extraction.claims` → `ex_output_schema` →
  `eval.claims`) lives in prose, not typed cells — file a beads issue
  against bajan to type it. Until then, composition-structure views show
  only what structured cells express.

Not in scope: the free-monoidal-category / string-diagram IR (deferred
until the v1 views prove value; operational definitions sketched in
design.md D5); openspec-layout corpora (`specs/<domain>/spec.md` — needs a
parse-boundary adapter, see D7); interactive full-entity rendering (411
nodes is the last size Mermaid tolerates); any new format Revision — this
change touches only tool output, not the format corpus.

## Impact

- Affected specs (openspec capabilities): **new capability `graph-views`**
  (delta under `specs/graph-views/` in this change).
- Affected format corpus: **none** — no `specs/*.md` file changes; no new
  Reference Typing row; no Revision bump.
- Affected code: `src/graph.rs` (canonical-id normalization in edge
  extraction or projection; native dot/mermaid emission; wiring
  projection), `src/main.rs` (`--format edges|dot|mermaid` and `--view`
  flags on the `Graph` command; new `guide --json` subcommand),
  `src/guide.rs` (JSON serialization of the closed value sets; explain
  topic), `src/acset/schema.rs` (the Schema value the schema view
  derives from — see D4), new `scripts/graph_views.py`, `justfile`
  (`docs-graphs` recipe),
  `docs/src/` (one view page consuming generated includes),
  `tests/cli.rs` (projection fixtures: empty corpus, single-intent
  corpus, violation-bearing corpus).
