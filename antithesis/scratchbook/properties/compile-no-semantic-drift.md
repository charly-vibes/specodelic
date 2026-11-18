# compile-no-semantic-drift

## Invariant

For every successful compile, parsing the emitted TOML into `ConstraintsDoc`
and re-serializing it with `toml::to_string` yields bytes identical to the
emitted TOML.

## Why it matters

`compile.no_semantic_drift` is the claim that the compile functor is stable
under its own output: a consumer can re-emit an artifact without churn. This is
what makes artifact diffs meaningful and keeps the TOML a canonical
representation rather than a pretty-printed view that changes on every pass.

## Check mode

`assert`. The implementation already runs this check internally; the property
restates it and exercises generated content shapes.

## Public API / seam

`compile::compile_spec` (`src/compile.rs:2000-2015`), `ConstraintsDoc` /
`ConstraintToml` (`:31-56`), `constraints_to_toml` (`:58-61`).

## Generator sketch

- Specs with 0–3 constraints, including a `kernel.binding` cell (serialized only
  for invariant-kind rows), escaped pipes in `expr`, empty `traces_to`, and
  unusual-but-valid strings.
- Compile; parse the TOML with `toml::from_str::<ConstraintsDoc>`; re-emit;
  compare bytes.
- Also compare against a fresh `constraints_to_toml(&parse(source))` to ensure
  the round trip is not merely self-consistent.

## Predicted at HEAD `c76e2b0c`

**GREEN — gate.** Field order is the serde struct order; `binding` is skipped
when `None`, so pre-binding artifacts stay byte-identical. A mismatch returns
`Err(stage = "no_semantic_drift")` before `Compiled` is constructed.

The scope is TOML only. Props and TLA have no reparse/re-emit path, so a
"no drift" claim over those artifacts would need a different oracle.

## What would make it red

- Adding a field to `ConstraintToml` without a stable serde order or with a
  skip/rename rule that makes parse→emit lossy.
- Serializing a `binding` for non-invariant rows, then dropping it on parse
  (the `kernel_binding_of_row` filter would make the round trip lossy).
- A `toml` crate upgrade that changes formatting of a value (e.g. multiline
  strings). The gate would catch it as an `Err`, turning every compile red
  rather than silently drifting.
- Escaped pipes: `constraints_doc` unescapes `expr` before serialization. If
  the unescape is not idempotent, re-emission from the parsed doc could differ.

## Existing instrumentation

Generated `roundtrip_stable` (out of scope) checks compile twice on the source,
not the TOML reparse/re-emit. The internal gate is the real check.

## Assumptions and Open Questions

- [ASSUMPTION] TOML is the only artifact with a canonical re-emission claim.
- None that change the invariant.