# Graph views — derived diagrams of the corpus

Every diagram on this page is **derived**: rendered at docs build time by
`just docs-graphs` from this repo's own corpus — never committed, never
hand-edited. That is the whole point: **views are never more current or
more correct than the graph artifact they were rendered from.** If a diagram and the corpus disagree, the corpus wins
and the diagram is stale — regenerate it (`just docs-graphs`), don't fix
it.

The embedded primer (`spk explain graph-views`) carries the same
taxonomy and the one-pipe render recipes for your own corpora.

## Wiring — producer→consumer connections

`spk graph specs --view wiring --format mermaid`: the file-level
projection of `constraints.satisfies` edges — which spec file's
constraints consume which extension points. Self-loops are dropped; a
corpus with no cross-file wiring renders a labeled `no_wiring` note,
never a silently clean diagram.

```mermaid
{{#include views/wiring.md}}
```

## Per-file state machines

`graph_views.py states`: one Mermaid flowchart per owning file,
grouped by subgraph. States stay distinct, guard edges render dashed,
typing violations render as annotated red elements, and fan-in counts
distinct sources.

```mermaid
{{#include views/states.md}}
```

## Traceability — file dependencies

`graph_views.py trace`: every edge collapsed to its owning intent,
cross-file dependencies deduped, fan-in annotated per intent.

```mermaid
{{#include views/trace.md}}
```

## The typing diagram — revision-labeled

`graph_views.py schema`: the Reference Typing table (which reference
field may point at what), rendered from the versioned acset Schema
export (`spk guide --schema --json`) alone. The label carries the format
revision the export was generated at (`format_revision`) — a diagram
labeled `Revision 17` was rendered from a `Revision 17` binary.

```mermaid
{{#include views/schema.md}}
```

## Rebuilding the views

The views live in `docs/src/views/` (gitignored — build-time artifacts,
never committed; if they disagree with the corpus, the corpus wins and
they get regenerated). `just docs-graphs` regenerates all four before
the docs build; the same commands by hand:

```bash
spk graph specs --format edges > edges.tsv
spk graph specs --json > graph.json
spk guide --schema --json > schema.json
python3 scripts/graph_views.py schema schema.json
python3 scripts/graph_views.py states edges.tsv --graph graph.json
python3 scripts/graph_views.py trace edges.tsv --graph graph.json
spk graph specs --view wiring --format mermaid > wiring.md
```

For your own corpora, the one-pipe render recipes are in
`spk explain graph-views`.
