# Design

## Context

The harness targets are four bounded pure-function surfaces: the Kleene
three-valued citation evaluator and guard-expr parser (`src/compile.rs`), the
reference-cycle finder behind `linter.acyclic` (`src/lint/graph.rs`,
`src/graph.rs`), and the tiered reachability classifier
(`src/lint/graph.rs`). All are enum-and-slice code with hand-managed byte
offsets — no unsafe, no I/O — which is exactly the shape Kani proves
cheaply. Kani installs via `cargo install --locked kani-verifier` and
compiles the crate with `#[cfg(kani)]` code visible; nothing ships in
release builds. See proposal.md — Why for motivation and tool grading.

## Goals / Non-Goals

**Goals:**

- Every law the four surfaces claim in doc-comments becomes a named,
  machine-checked harness obligation with an owning spec row.
- The suite is one command (`just kani`), slow-but-regular (pre-push,
  nightly), and structurally unable to leak into `just ci` or release builds.
- Honest scope labels per family: exhaustive where the domain is finite,
  bounded where unwinding limits apply.

**Non-Goals:**

- No user-corpus backend work (Apalache/Lean/Quint — the track-2 roadmap,
  separate changes).
- Harness families 5–7 (string-scanning panic-freedom, helper-expansion
  termination, model-check verdict honesty) and the guard-expression
  round-trip renderer — follow-up changes.
- No git-hook installation: `core.hooksPath` is beads-owned in this repo
  (sibling-tool constraint, AGENTS.md); `just kani` stays convention-run.
- No production-code changes *planned by this change* — a red harness is a
  finding, handled per D5.

## Decisions

- **D1 — Harness placement: `#[cfg(kani)]` modules inside the crate**
  (`src/compile/kani.rs` beside the existing `compile/tests.rs`; a
  `src/lint/graph/` submodule for the linter side — matching the existing
  module split). The targets are
  `pub(crate)`; a separate verification crate would force widening
  visibility for testability. Alternatives: separate kani crate (rejected —
  visibility churn), `tests/` integration harnesses (rejected — cannot reach
  `pub(crate)` items).

- **D2 — Bound policy, stated per family.** The value-domain algebra harness
  is exhaustive with NO unwinding bound (domain is a 3-value enum; the laws
  are checked over `kani::any` value tuples — a complete proof). Structured
  expression laws, parser, and graph harnesses carry explicit unwind bounds
  (expression depth ≤ 4; graphs ≤ 7 nodes / ≤ 12 edges; parser cells ≤ 128
  bytes) recorded as named constants in the harness module and echoed in the
  result label. Bounds start conservative and grow only with measured
  runtime; each family carries a stated wall-clock budget in the harness
  module doc, and a family that exceeds its budget shrinks its bound before
  it is dropped. Family 5 (reachability totality) targets the pure
  tier-classification core over minimal hand-built `Spec` values — arbitrary
  full-`Spec` generation is explicitly out of scope as the family's heaviest
  hidden cost.

- **D3 — Differential oracles live in the harness, never in production
  code.** The naive exponential cycle checker is written inside the
  `#[cfg(kani)]` module; the production engine is unaware of it. This keeps
  the oracle from becoming a second engine to maintain.

- **D4 — Run points: recipe + convention, no hooks.** `just kani` wraps a
  locked, per-crate `cargo kani` invocation;
  documentation names pre-push and nightly as the run points. The recipe
  guards its toolchain: when `kani-verifier` is absent or its pinned release
  disagrees with the installed rustc, it fails with a remediation hint
  (install command + pinned release) per the genesis error discipline —
  never a bare toolchain backtrace. Hook wiring is
  forbidden by the pretender constraints (AGENTS.md sibling-tool rules).

- **D5 — Red harnesses are findings, not silent fixes.** If any harness
  surfaces a counterexample against current code, the counterexample is
  filed as a beads issue with the minimized input; the fix is a separate
  refactor ticket (tidy-first discipline). This change lands only when all
  harnesses are green or their reds are filed and dispositioned.

- **D6 — Zero dependency footprint.** Kani is a channel-installed tool, not
  a Cargo dependency; `#[cfg(kani)]` items compile only under the Kani
  toolchain. `Cargo.toml` changes: none expected. Release-build purity is
  checked by a task (build and grep for harness symbols).

## Risks / Trade-offs

- **Kani/rustc version coupling** — Kani tracks specific rustc versions;
  a toolchain bump can break the suite until Kani catches up. Mitigation:
  pin the Kani release in docs; the suite is off the commit gate, so drift
  never blocks routine work.
- **Runtime cost growth** — harness unwinding bounds are the budget knob
  (D2); if a family exceeds its runtime budget, the bound shrinks before the
  harness is dropped.
- **False confidence from bounds** — bounded graph results are weaker than
  they feel; `honest_scope` labeling and the spec's bounded-result
  requirement keep the claim honest.
- **Maintenance drag** — internals changing under fixed harnesses break the
  suite; that friction is the feature (it forces law-conscious refactors),
  and bounds/constants live in one place per module to keep updates cheap.
