# Change: Define verification claim gates and report compatibility

## Why

At 29f81ef, a stored per-claim counterexample or unknown can coexist with
aggregate no_counterexample and final verified. This is a policy gap:
verify currently consumes the aggregate model outcome, while the active
kernel proposal introduces additional claim statuses. Specify the contract
before changing the acceptance rule. Incomplete kernel/citation evaluation
is owned by add-min-expr-kernel tasks 3.6–3.8.

## What Changes

- **BREAKING:** opted-in invariant claims (Rust, kernel, citation) must all
  be discharged before a file can be verified; prose-only invariants are
  explicitly unchecked and do not become executable by implication.
- Versioned claim reports bind statuses to the exact structured corpus
  scope and compiled inputs. Old reports require a rerun, not guessed
  success. Model-check and verify expose the same claim counts and blockers.
- Scope command evaluation explicitly: dual-format id: spec files run one
  file per invocation with separate output directories; combined inputs fail
  with a split-run hint. Multi-file dual-format lint remains supported.
- Document lint, exploration, claim verification, and application-test
  binding separately; synchronize README, docs status, project guidance,
  and the embedded guide with implemented capabilities at release.

## Impact

New additive capability: verification-claims. Domain contract updates at
implementation: specs/model_check.md, specs/verify.md, specs/specodelic.md
with required revision discipline; src/model_check.rs, src/verify.rs,
src/orchestrate.rs, src/commands/model_check.rs, src/human.rs and CLI tests.
No competing model-check delta is authored here: add-min-expr-kernel owns
that active capability snapshot. Integrate that change first, then archive
this additive capability. Reconcile domain docs before either release
claims the new aggregate policy. No source or deployed spec changes in
this proposal stage.

## Sequencing and ownership

Pending approval. New policy work has no existing implementation ticket;
create separate feature and refactoring tickets from the cycles in tasks.md
after proposal approval. Kernel integration precedes this work. Both must
precede specodelic-gch corpus migration. Coordinate property adequacy with
add-py-fragment-emission (l8l); this change alone does not claim that a
passing constant-generator test provides broad input coverage.
