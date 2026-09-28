# Change: Emit the TLA+ module from the compiled ModelIR (complete `model_to_tla`)

## Why

`specs/compile.md` mandates that `Compile` always emits a TLA+ module —
"the human-reviewable, engine-portable form of the compiled model" that
TLC consumes directly (beads `specodelic-mp1` row 6 decided the backend:
native stateright default, TLC opt-in, Alloy dropped). The
`add-compile-functor` change shipped the backend-neutral `ModelIR` seam
and deferred emission to this follow-up; `specodelic-lnq` stays open
until the model artifact ships. The IR extraction and its tests are done,
so the emitter is the only remaining piece.

## What Changes

- Add `model_to_tla(spec) -> String` to `src/compile.rs`: translate the
  `ModelIR` into one TLA+ module per `specs/compile.md`'s
  `model_to_tla` — each State becomes a value in the state variable's
  range, each Transition becomes one disjunct of the `Next` action, and
  the `emits` mapping becomes an `Output` function whose domain is
  exactly the emitting states (never a default or null entry)
- Guards and `emits` expressions are spec prose, not TLA+ — they are
  carried verbatim in per-disjunct/per-entry comments while the emitted
  disjunct itself is valid TLA+ over the state variable (scaffolding
  philosophy: same as the proptest artifacts; semantic checking is
  `nx7`'s domain)
- `Compile` now produces the model artifact: `Compiled.tla`, written as
  `<stem>.tla` next to the TOML and proptest artifacts (byte-stable,
  committed), included in the envelope
- Tests: unit tests for disjunct count and `Output` domain per
  `compile.md`'s properties (`tla_disjunct_count_matches`,
  `output_function_covers_emitting_states_only`), plus integration
  updates for the third artifact

## Impact

- Affected specs: `compile` (new OpenSpec capability delta; domain spec
  of record remains `specs/compile.md` — this delta mirrors its
  `model_to_tla` constraint, it does not reinterpret it)
- Affected code: `src/compile.rs` (emitter + tests), `src/main.rs`
  (write `<stem>.tla`), regenerated artifacts under `specodelic/`
- Beads: completes the model-emission phase of `specodelic-lnq` (then
  the ticket closes); unblocks `specodelic-nx7`'s TLC path
