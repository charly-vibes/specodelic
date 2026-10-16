# Decision: graph-view rendering boundary — external gently pipe (Option A)

- **Date**: 2026-10-16
- **Ticket**: specodelic-4al1 (P4 decision; parent context: gently-leverage
  evaluation session, issue-review remediation batch)
- **Status**: **ADOPTED — Option A: the external pipe is the sanctioned
  render path.** `spk graph` stays render-free (add-graph-views D8
  unchanged). Option B (`--render` backed by a gently-core dependency) is
  parked, revisit-only-on-evidence.

## Options considered

- **Option A — external pipe** (`spk graph <corpus> --format dot |
  gently --as ascii|html`): **ADOPTED**. Zero new dependencies; the text
  projections stay byte-stable and are the only artifact `spk` guarantees;
  gently can evolve (and break) on its own release cadence without
  touching our gate; matches the D8 rationale and upstream Graph::Easy
  tool culture. Cons (two binaries for one view, no styled end-to-end
  guarantee) are acceptable for a docs/diagnostics seam.
- **Option B — `spk graph --render ascii|html` via gently-core**: **PARKED**.
  A render flag couples release cadences and would break the D8
  byte-stability rationale (rendered layouts change with gently versions;
  text projections do not). gently's own CLI wiring is also incomplete for
  some formats (boxart). No current user pain justifies the coupling.

## Facts that decided it

1. D8 already names `graph-easy` as the external render target; gently is
   the family's native implementation of exactly that seam (Rust port,
   parses DOT + Graph::Easy text, renders ascii/html/txt/dot).
2. Bidirectional dogfooding exists: gently already runs `spk` over its own
   `ge.*` corpus; both tools benefit from the pipe staying external.
3. The concrete consumer (graph views in the docs site + llms.txt) is
   satisfiable at the pipe level inside `just docs-graphs` — no in-process
   need for a renderer.

## Consequences

- `spk explain graph-views` and the docs recipes document the pipe
  (specodelic-qzw8); the ASCII/HTML view embeds are built by the docs
  recipe, not by `spk` (specodelic-f7yi).
- A future revisit requires evidence the two-binary seam hurts real users
  (e.g. an agent workflow that cannot invoke a second binary) — not a
  refactor ticket filed on spec.
- gently's render bugs (gently-n0d, gently-206 closed in 0.2.2; residual
  gently-hpd for per-file DOT) are upstream concerns, not specodelic
  gate failures — the docs recipe degrades gracefully (views without
  ASCII variant still build).
