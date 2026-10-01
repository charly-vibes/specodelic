# graph_to_dot.jq — project an `spk graph -j` envelope into Graphviz DOT.
#
# Usage:  spk graph -j <path> | jq -rf scripts/graph_to_dot.jq
# Pipe the DOT into any dot consumer — none are dependencies of spk:
#   ... | dot -Tsvg -o wiring.svg        # image (graphviz)
#   ... | graph-easy                     # ASCII/boxart terminal view
#   ... | acyclic | dot -Tsvg            # graphviz filter family
#
# Edge-kind styling (visual grammar, mirrored by any future backend):
#   transitions.from/to   solid    — the state machine
#   transitions.guard     dashed   — guard annotation
#   states.emits          bold     — output function
#   constraints.traces_to
#   properties.derives_from
#                         dotted   — traceability
#   dangling references   red dashed edge to a red box — a view is never
#                         silently cleaner than the graph artifact

def kind_of: .kind | sub("^[^.]+\\."; "");

def style_of:
  { "from":         "",
    "to":           "",
    "guard":        ", style=dashed",
    "emits":        ", penwidth=2",
    "traces_to":    ", style=dotted",
    "derives_from": ", style=dotted, color=gray50" }[kind_of] // "";

def edge_lines:
  [ .data.edges[]
    | kind_of as $k
    | style_of as $s
    | "  \"\(.from)\" -> \"\(.to)\" [label=\"\($k)\"\($s)];" ]
  | unique
  | join("\n");

def dangling_lines:
  [ .data.dangling[]
    | capture("^(?<src>.+) → \\[\\[(?<tgt>.+)\\]\\]$")
    | "  \"\(.src)\" -> \"\(.tgt)\" [style=dashed, color=red];\n" +
      "  \"\(.tgt)\" [shape=box, color=red, style=dashed];" ]
  | unique
  | join("\n");

[
  "digraph spec {",
  "  rankdir=LR;",
  "  node [shape=ellipse, fontsize=10];",
  edge_lines,
  dangling_lines,
  "}"
]
| join("\n")
