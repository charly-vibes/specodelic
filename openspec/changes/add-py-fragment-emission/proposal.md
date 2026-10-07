# Change: Execute Python properties with real generated inputs

## Why

The format accepts py intent but compile deliberately rejects it until an
emitter exists. Existing Rust scaffold generators are Just(name), so passing
predicates exercise a constant string. Archived language-neutral-property-
binding decision D5 requires the first Python emitter and generator-vocabulary
decision in the same change. This proposal supplies the missing plan for
existing ticket specodelic-l8l; it does not invent another runner registry.

## What Changes

- Add an explicit language-neutral generator opt-in, a bounded vocabulary,
  and project-local named data definitions; emit real proptest and Hypothesis
  strategies from the same generator IR, with homogeneous choice branches.
- **BREAKING generator opt-in:** cells marked **gen:** now produce typed
  values even if the old compiler accepted them as constant-name scaffolds.
  Unmarked legacy artifact bytes stay stable; marked inputs require predicate
  migration, recompilation and fresh verification evidence.
- Emit and execute Python unit-kind property predicates through a
  PropertiesRunner adapter; retain existing law-fragment rejection until a
  per-case executable law syntax is separately specified.
- **BREAKING verification policy:** legacy constant placeholder generators
  remain compilable but are labeled inadequate input evidence and cannot
  satisfy properties_pass. Parameterless executable unit tests remain valid.
- Preserve exactly-once row/case accounting across mixed Rust/Python files,
  native shrinking, bounded execution, staleness checks, and failure hints.
- Provide a runnable workflow example and activate kernel fixture agreement
  against the Python reference evaluator; no language-portable predicate
  translation is inferred from Python/Rust fragments with arbitrary code.

## Impact and sequencing

Owner: existing specodelic-l8l (currently a placeholder). Pending approval.
Depends on add-min-expr-kernel including corrective integration and phase 4
(36n), then define-verification-claim-gates. New capabilities:
property-generators and python-properties; a full compile capability snapshot
modifies the availability rule only after the kernel compile delta lands.
Do not archive both compile overlays independently against the same base.
Rebase against the deployed compile capability before applying this change.

Implementation touches src/compile.rs, src/verify.rs, src/orchestrate.rs,
src/commands/corpus.rs, src/commands/model_check.rs, src/main.rs, new focused
emitter/runner modules, CLI tests, and format/domain docs. Generator syntax
requires a format Revision; update literals, artifact provenance and examples
in the implementation change. Keep espectacular read-only over openspec and
out of specs/ enforcement. No runtime code or deployed docs change now.

specodelic-lf3 may wire corpus verify only after migrating its properties to
adequate generators or meaningful parameterless tests; do not turn a green
constant-generator run into the adequacy criterion. TypeScript follows as
add-ts-fragment-emission (aby). Separate refactoring tickets from features.
