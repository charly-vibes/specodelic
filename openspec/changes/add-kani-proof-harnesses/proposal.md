# Change: Kani proof harnesses for specodelic internals

## Why

Specodelic's verification ladder (docs/src/verification-boundaries.md, rungs 1–5)
certifies *user* corpora, but the tool's own decision-carrying internals — the
Kleene three-valued citation evaluator, the guard-expr parser, the
`linter.acyclic` cycle finder, the tiered `single_root_reachable`
classification — are guarded today only by sampled tests. `spk model-check`
already proves corpus claims by bounded exploration; the same honesty
discipline should apply to the code that decides verdicts. A survey of the
formal-verification tool landscape (quint.sh guide, 2026-11) graded Kani
MIT-licensed, cargo-native (`cargo kani`), and the only candidate whose
harness shape fits these pure-function internals without a forked toolchain
(Verus/Prusti/Creusot struck on toolchain invasiveness and license). Provenance:
grading table with license sources captured in
`.wai/resources/research/2026-11-18-quint-fvt-tools-grading.md`
(Stateright A+/A+, Kani A/A).

## What Changes

- New dev capability: a Kani harness suite over specodelic's bounded
  pure-function internals, run via `cargo kani` behind a `just kani` recipe —
  explicitly NOT in `just ci` (Kani runtimes are too slow for the per-commit
  gate; pre-push and nightly are the run points).
- Harness families landed in TDD red→green slices, each asserting a law the
  code already claims in doc-comments or specs but nothing machine-checks:
  1. Kleene algebra laws on `evaluate_citation` — commutativity,
     associativity, counterexample-dominance, Verified identity, ¬¬
     involution. The domain is a 3-value enum, so this is an UNBOUNDED
     complete proof, not a bounded one.
  2. `parse_citation_expr` soundness — no-panic on arbitrary cells (the
     parser hand-slices byte offsets) and the RO5U F-1 rule (any parsed expr
     has bracket-free ids; malformed cells stay prose).
  3. Differential oracle for `find_cycles` / `find_supersedes_cycles` — the
     real DFS vs a naive exponential checker written in the harness, equal
     cycle sets for arbitrary graphs within the unwinding bound. This is the
     engine behind `linter.acyclic`, which gates every commit in this repo.
  4. `reachability_findings` tier totality — every row lands in exactly one
     tier (own-file reachable / cross-file-only warn / hard-fail) on bounded
     spec graphs; nothing falls through, nothing double-reported.
- All harnesses live in in-crate `#[cfg(kani)]` modules (design D1), never
  compiled into release builds; harness families 5–7 from the survey
  (string-scanning panic-freedom, helper-expansion termination, model-check
  verdict honesty) and the guard-expression round-trip renderer (a
  production-code addition, excluded by this change's
  zero-production-change stance) are deliberately deferred to follow-up
  changes.

## Capabilities

### New Capabilities

- `proof-harnesses`: the durable obligation that specodelic's
  verdict-deciding internals carry machine-checked laws — which internals,
  which laws, the run points (`just kani`), and the honesty rule that a
  bounded harness result is never reported as an unbounded proof.

### Modified Capabilities

(none — no existing requirement changes; `compile`, `linter-graph_shape`,
`model_check` requirements are untouched; their *evidence* strengthens.)

## Impact

- Affected specs: new `openspec/specs/proof-harnesses/spec.md` (via this
  change's delta); no existing capability spec gains or loses requirements.
- Dual-format protocol: NOT touched (the deployed `proof-harnesses` spec will
  be dual-format per the standing protocol, added through the sanctioned
  archive recipe; the delta here stays a plain openspec delta).
- Lint rules: NOT touched — no new `linter.*` rule; `spk lint` behavior is
  byte-identical.
- Affected code: `src/compile.rs` (harness modules only — zero production-code
  changes expected; a red harness counterexample against current code would
  be a finding to file, not silently "fix" in this change), `justfile` (`just kani`
  recipe; `Cargo.toml` unchanged — none expected per design D6),
  `docs/src/verification-boundaries.md` (a tool-trust row).
- Out of scope: Verus/Prusti/Creusot adoption (struck in grading), harness
  families 5–7 (follow-ups), any backend integration for user corpora
  (Apalache/Lean are separate changes on the track-2 roadmap), CI gate
  changes to `just ci`.
