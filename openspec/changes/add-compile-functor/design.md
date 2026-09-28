# Design: add-compile-functor

## Context

`specs/compile.md` is the spec of record; this change implements it
verbatim. Current state: `Compile` is a shared stub arm in `main.rs`;
parsing (`spec.rs`), linting (`lint.rs`), and graph (`graph.rs`) exist.
The corpus is fully covered, so the precondition gate can hold
end-to-end today.

## Goals / Non-Goals

- Goals: one `Compile` invocation → three artifacts (TOML, TLA+ text,
  proptest! source text), gated on linted+covered, ids preserved,
  round-trip stable, labeled failures.
- Non-Goals: running TLC/Alloy or executing the proptests (`nx7`/`1pv`);
  orchestration (`8kk`); parsing prose (`prose_untouched`); minting,
  dropping, or altering ids.

## Decisions

- **Decision: artifacts as text, emitted both in the envelope and to
  disk.** The TLA+ module and proptest! sources are strings; TOML is
  also kept as string (plus a parsed echo for validation). The envelope's
  `data` carries all three plus per-file stats; `--out-dir` (default
  `compiled/`) writes `<stem>.toml`, `<stem>.tla`, `<stem>_props.rs`.
  Rationale: `nx7` consumes the `.tla` text, `1pv` the `.rs` text;
  disk artifacts make the outputs inspectable and testable without a
  process boundary.
- **Decision: precondition gate = internal lint.** `compile` runs the
  existing `lint` pass on the file and refuses when any issue exists —
  the coverage rule is the binding one today, and the corpus is clean.
  Rationale: implements `precondition_satisfied` without inventing a
  second lint implementation.
- **Decision: proptest emission as compilable scaffolding, not
  pseudo-code translation.** Each Property row emits a `proptest!` block
  whose input strategy is a call into a small generated helper module
  (`spec_gen` trait + per-generator functions) and whose assertion body
  invokes the predicate verbatim as the body of the check closure. Where
  a predicate is not valid Rust (the corpus's predicates are
  pseudo-code), the emitted block carries the predicate as the closure
  body with the row's generator/predicate quoted in the file header
  comment — the artifact compiles because unresolved predicates route
  through a `todo_predicate!` macro that fails at the *verify* stage
  (`1pv`), not at compile-artifact emission. Rationale: `compile.md`'s
  MUST is "a Properties row yields a proptest! block that compiles";
  making arbitrary spec prose into sound Rust is `verify`'s problem
  domain, and a labeled, structured TODO keeps `no_semantic_drift`
  checkable (round-trip over the emitted text, not over predicate
  semantics).
- **Decision: laws expand at emission time.** A law-kind property emits
  one block per required case (identity, associativity, …) by reading
  the case names from the property's own predicate per
  `law_requires_cases` — mirroring `law_property_compiles_required_cases`.

## Risks / Trade-offs

- Proptest scaffolding may feel indirect → mitigated by emitting the
  spec's own generator/predicate text into the artifact header so `1pv`
  can work from the source of truth.
- `main.rs`'s stub arm is shared by `ModelCheck`/`Verify` → split the
  arm; only `Compile` moves, the other two keep the stub (their tickets
  own their arms).
- Round-trip stability (`no_semantic_drift`) needs a parse of emitted
  TOML only (TLA+/Rust are not re-parsed by this tool) → round-trip is
  defined over the TOML artifact, matching `compile.md`'s
  `re-parsing a compiled artifact`.

## Migration Plan

Additive: no existing behavior changes; the stub arm narrows. Rollback =
revert the enum-arm split.

## Open Questions

- Should `--out-dir` default to `compiled/` in-repo (gitignored) or
  require an explicit flag? (Default proposed: `compiled/`, added to
  `.gitignore`.)
- `Output` function encoding for `emits` states: TLA+ function-of-state
  vs. per-state constant — propose function-of-state per
  `compile.md`'s `model_to_tla` wording.
