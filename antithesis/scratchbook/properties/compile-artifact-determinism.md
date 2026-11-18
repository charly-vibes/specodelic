# compile-artifact-determinism

## Invariant

For every lint-clean spec `s`:

1. `compile_spec(&s)` called twice returns byte-identical `toml`, `props`, and
   `tla`;
2. `compile_spec(&parse(compile_source(s)))` returns the same three bytes,
   where `compile_source` is the source text the spec was parsed from;
3. the `model_ir` is structurally equal across calls;
4. `warnings` is equal (order included).

In short: compile is a pure function of the parsed spec, with no clock,
randomness, environment, or hash-map iteration in the output.

## Why it matters

`no_semantic_drift` and `compile_is_total` are about content; determinism is
what makes artifact hashing, staleness checks, and `verify`'s scratch-crate
invalidation sound. A single `HashMap` iteration or timestamp would make two
identical compiles produce different bytes, defeating every digest-based gate
downstream.

## Check mode

`assert`, over generated lint-clean specs. Run `compile_spec` twice in the same
process and compare all four fields.

## Public API / seam

`compile::compile_spec` (`src/compile.rs:1980-2050`); emitters
`constraints_to_toml` (`:58`), `properties_to_proptest` (`:1301`),
`model_to_tla` (`:1618`), `extract_model_ir` (`:1141`).
`write_artifacts_into` (`:1900`) is a **different** seam and is excluded from the
byte-identity claim because it pipes the props artifact through `rustfmt`
(`:1946-1975`), whose version is an environment input.

## Generator sketch

- Specs with 1–3 constraints, 1–3 states (some with `emits`), 1–3 transitions,
  1–3 properties (unit and law), several generators (some fixture names, some
  arbitrary), and optional `**rust:**` fragments and `## Fragments` helpers.
- Vary row order only where semantically valid: states' first entry is the
  initial value, so state order is content, not noise. Do not assert
  order-independence over a semantics-preserving permutation unless the spec
  defines one.
- Compile twice; assert byte equality. Reparse the original source and compile
  again; assert equality.

## Predicted at HEAD `c76e2b0c`

**GREEN.** By inspection:

- document-order emission walks `Vec`s populated by the parser;
- `cells`, `emits`, `emits_values`, `constraint_exprs`, artifact-id sets, and
  collision maps are `BTreeMap`/`BTreeSet`;
- helper generators are emitted in first-reference order over a stable block
  order; duplicate fn names get deterministic numeric suffixes
  (`:1308-1319`);
- `toml::to_string` follows serde struct field order (`ConstraintsDoc`);
- no clock/random/environment calls in `compile_spec`.

A grep for `HashMap`/`HashSet`/`rand`/`SystemTime` in `src/compile.rs` returns
nothing.

## What would make it red

- Introducing a `HashMap` for `cells`, `emits`, or generator names.
- Embedding a timestamp, build-id, hostname, or absolute source path in an
  artifact. `model_to_tla` embeds `spec.intent.id` and the module name derives
  from the artifact stem (path-dependent), so compiling the *same content* at a
  different path is **not** expected to be byte-identical. The property must
  state: same spec **and same path**.
- Emitting in `HashSet` iteration order for duplicate-fn disambiguation.
- `helper_warnings` collecting from a set without sorting.
- Calling `rustfmt` inside `compile_spec` (currently only in
  `write_artifacts_into`).

## Existing instrumentation

Missing as a repeated-call property. The generated corpus has fixed expected
bytes, which is consistent with determinism but does not test it. The compile
spec's `no_semantic_drift` only re-emits TOML, not props/TLA.

## Assumptions and Open Questions

- [ASSUMPTION] Artifact stems are path-derived (`artifact_stem`,
  `src/compile.rs:1854-1871`), so the property compares compiles of the same
  `Spec` value (including `path`), not merely the same semantic content.
- Open: should `write_artifacts_into` be deterministic across machines? It is
  not, because rustfmt formatting is versioned. If the answer is yes, the
  artifact must be emitted already-formatted or rustfmt must be pinned.
  (needs human input)

## Investigation Log

- Examined: `compile_spec` (`src/compile.rs:1980-2050`), all four emitters, the
  helper/duplicate-name paths, and `rustfmt_clean` (`:1946-1975`).
- Found: `compile_spec` has no subprocess, clock, random, or hash-map iteration;
  `write_artifacts_into` is the only rustfmt caller. Artifact stems are
  path-derived, so the property must compare equal `Spec`s including `path`.
- Not done: no repeated-call run; the conclusion is an inspection result.