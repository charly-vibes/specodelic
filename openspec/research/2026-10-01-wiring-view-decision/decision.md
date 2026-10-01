# Decision — declared-dataflow wiring view (specodelic-5qj)

**Date:** 2026-10-01 · **Ticket:** specodelic-5qj (exploration; no production
code in this ticket) · **Reviewed:** Rule-of-5, two passes; HIGH findings
TypeSafe-verified (jev-1.13.0, confidence 0.84–0.96, measured FP rate 0%).

## Decision

**Fold into `add-graph-views` before its implementation**, with the output
layer reclassified:

1. **`spk graph` gains native text projections** —
   `--format json|dot|mermaid|edges` × `--view states|wiring|trace|schema`.
   DOT and mermaid are plain-text string templates; emitting them from Rust
   needs **zero external crates** (same class as the already-planned
   `--format edges` TSV). The JSON envelope stays canonical.
2. **Rendering stays 100 % external** — the tool depends on no graphing/viz
   library. The happy path is one standard pipe:
   `spk graph --view wiring --format dot specs/ | dot -Tsvg -o wiring.svg`.
   Other documented consumers: `graph-easy` (ASCII terminal), `mmdc`,
   viz-js/d3-graphviz, the graphviz filter family (`acyclic`, `tred`).
3. **Guidance ships with the binary** — a `spk explain graph-views` primer
   topic teaching the one-pipe recipes (the embedded-AIX pattern; works
   offline for consumers). The repo-local `scripts/graph_to_dot.jq`
   prototype retires to a CI fixture asserting dot-emission parity.
4. **Visual grammar** (any backend must mirror it): solid = state machine,
   dashed = guards, bold = `states.emits`, dotted = traceability, red
   dashed = dangling refs/violations — a view is never silently cleaner
   than the graph artifact (D3 unchanged).

## Why (review trail)

- The initial "JSON-only, bridge via jq filter" answer failed
  self-containment for the actual distribution model: the filter lives only
  in this repo, is referenced by zero docs, and is absent from the installed
  `spk 0.3.0` (CORR-005, mechanically verified).
- The category error (DRAFT-003, verified): "no viz deps" rejects *rendering
  libraries*, not *output text projections*. Conflating them manufactured
  the three-pipe ceremony.
- An earlier candidate backend (ascii-dag, in-terminal rendering) is dropped:
  it requires Rust 1.92 vs repo MSRV 1.88 (mechanically verified) and is
  unnecessary under native projections — the terminal niche stays with
  `graph-easy`, documented, not depended upon.

## Sample outputs (this directory, `samples/`)

| File | What it shows |
|---|---|
| `specodelic-wiring.dot/.svg` | file-level `satisfies` wiring, this corpus — 8 producer→consumer pairs, `linter → errors` dominates (51 rows) |
| `bajan-wiring.dot/.svg` | same projection over bajan — 4 inter-file pairs; **note:** the ticket's contingency ("bajan wiring empty until ac8") is stale, bajan now carries typed satisfies edges |
| `compile-full.dot/.svg` | full-graph projection of `specs/compile.md` (all edge kinds, red dangling edges visible in file-scoped scope) |

Corpus facts at decision time: this repo 21 files / 639 edges / 99
`satisfies`; bajan 7 files / 231 edges / 4 `satisfies` / 1 violation.

## Known gap carried into implementation

The envelope carries a node *count* but no node list — isolated nodes
(nodes with zero edges) vanish from any rendered view. The edges projection
(or envelope data) must enumerate nodes explicitly, and empty views must be
*labeled* (`no_transitions`, `no_wiring`), never silently clean — the
`exploration_only ≠ clean` principle applied to views.
