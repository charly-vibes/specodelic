# Change: Implement `spk model-check` — the model_check step with the native stateright backend

## Why

`specodelic.md` names `model_check` as the `compiled → model_checked`
transition, and `specs/model_check.md` specifies what a run means
(state machine, bounds, minimal counterexamples, backend attribution) —
but the CLI arm is a stub (`src/main.rs`:"not yet implemented"). The
backend decision is made (beads `specodelic-mp1` row 6: stateright
embedded default, TLC opt-in, Alloy dropped), so the step can be built
against the compiled artifacts `add-compile-functor` + `add-tla-emitter`
now ship. Beads ticket: `specodelic-nx7`.

## What Changes

- Add `src/model_check.rs`: the run state machine (`not_run → running →
  clean | counterexample_found | timed_out`) and a `RunReport` envelope
  payload — outcome, backend `{engine, version}`, the stated bound,
  violated-invariant id + minimal trace when found
- Native default backend: stateright (embedded crate) *interprets* the
  compiled `ModelIR` — program-counter state, transitions as actions,
  matching the committed `.tla` module's semantics — no codegen, no
  external binary
- CLI: `spk model-check <files> [--max-depth N] [--max-states N]
  [--timeout-secs T]` over the envelope; precondition reuses compile's
  gate (artifact must exist — never re-compiles, per `model_check.md`'s
  contract)
- Run reports persist as `<stem>.check.json` next to compile's artifacts
  so `verify` (`specodelic-1pv`) can consume "most recent run against the
  *current* compiled artifact" with a staleness check
  (`rerun_on_model_change`)
- **Re-scoped MUST** (needs approval): the corpus language has no
  executable predicate semantics — guards and predicates are prose
  (compile's proptest scaffolding panics via `todo_predicate!` at
  execution). v0's native backend therefore explores the automaton
  dynamics exhaustively within the bound and reports
  `no_counterexample`/`timed_out`; the counterexample leg of
  `nx7`'s MUST awaits the predicate-fragment decision (new mp1 row).
  See design.md Decision 3.
- TLC stays opt-in and out of v0 — it becomes `specodelic-ug3`'s
  remaining scope (JVM subprocess, `-depth` bound, ERROR — never
  `no_counterexample` — when the binary is missing)

## Impact

- Affected specs: `model-check` (new OpenSpec capability delta; domain
  spec of record remains `specs/model_check.md` — the delta mirrors its
  constraints, it does not reinterpret them)
- Affected code: new `src/model_check.rs` + `stateright` dependency;
  `src/main.rs` ModelCheck arm (shared with sibling pipeline tickets);
  artifacts under `specodelic/` gain `<stem>.check.json`
- Beads: implements `specodelic-nx7` (amended MUST); `specodelic-ug3`
  re-scoped to the TLC backend; unblocks `specodelic-1pv` (verify)
- New external dependency: `stateright` (embedded model checker)
