# compile-totality

## Invariant

For every spec that has passed lint and coverage, `compile_spec` returns either:

- `Ok(Compiled)` with non-empty `toml`, `props`, and `tla`; or
- `Err(CompileError)` with a non-empty `stage` and `message`.

It never returns a partial result (e.g. TOML without TLA), and every failure
names exactly one stage (`precondition_satisfied`, `fragment_extraction`,
`model_to_tla`, `no_semantic_drift`, `compile_preserves_ids`, ...).

## Why it matters

`compile.compile_is_total` is the contract that lets `spk compile` report a
single labeled failure instead of writing a half-set of artifacts. The `Compiled`
type has no partial variant, so the invariant is largely type-enforced; its
value is as a regression alarm on the construction path and a check that error
stages stay named.

## Check mode

`assert`. For each generated spec, match on the result and check the
non-emptiness or the stage/message non-emptiness.

## Public API / seam

`compile::compile_spec` (`src/compile.rs:1980-2050`), `Compiled` (`:1746-1765`),
`CompileError` (`:1767-1776`). `write_artifacts_into` (`:1900`) is excluded: it
can fail on I/O after a successful compile, which is a different contract.

## Generator sketch

- Linted-and-covered specs with all three families.
- Malformed variants that should fail at a named stage: no states; a `**py:**`
  fragment (no emitter); a law-row fragment without a case body; a guard
  fragment; an unresolved helper; a helper cycle; an unknown marker tag.
- For `Ok`, assert all three strings non-empty. For `Err`, assert stage and
  message non-empty and that stage is one of the documented labels.

## Predicted at HEAD `c76e2b0c`

**GREEN — gate.** The only construction path returns all fields from one
`Compiled` literal; every failure is a `CompileError`. The generated property
`compile_completes_or_fails_cleanly` (out of scope) already encodes this; its
fixtures are fixed examples of the same shape.

This property adds little beyond a type-level guarantee; it is catalogued
because the compile spec claims it and because a future "streaming" emitter
could reintroduce partial results.

## What would make it red

- A refactor that writes artifacts incrementally inside `compile_spec` and can
  return early after TOML.
- An empty-but-present artifact being treated as success while a downstream
  consumer expects content. The current contract accepts an empty
  `//! (no Property rows...)` props file as non-empty; a spec with no
  properties still produces a non-empty header. An empty `tla` is impossible
  because `model_to_tla` always writes a module header.
- A panic on malformed input instead of `Err` (the property should catch this
  with a process-isolated or `catch_unwind` harness).

## Existing instrumentation

Generated `compile_completes_or_fails_cleanly` in `specodelic/compile_props.rs`
(out of scope). It is a fixture-driven property, so its distribution is narrow.

## Assumptions and Open Questions

- [ASSUMPTION] "Non-empty" means the emitted string is non-zero-length, not
  semantically substantive. A prose-only artifact would pass.
- None that change the invariant.