# Verification improvement proposals — review handoff

Base: 29f81ef, package 0.5.2. User requested a general evaluation, a
Rule-of-5 review including proposals/tickets, then creation of change
proposals. This session authors proposals and tracking metadata only;
implementation approval is still required.

## Evidence and interpretation

- A two-state spec with a passing Rust invariant and the false kernel
  claim `**kernel:** |State| == 999` lints/compiles, but its model-check
  report omits the kernel claim; verify returns verified. Kernel-only
  input instead reports exploration_only and verify rejects it.
- A citation to a passing invariant via a qualified ID reports unknown;
  the equivalent bare local citation reports verified. A negated bare
  citation reports counterexample while the aggregate remains
  no_counterexample and verify returns verified.
- Runtime integration is missing despite completed library-level kernel
  tests. This does not mean the unfinished kernel proposal was deployed.
- Existing verify consumes the aggregate model outcome. Mixed-claim
  aggregation is a specification decision, separate from identity and
  runtime-integration defects. Do not silently reinterpret all prose as
  required executable claims.
- D5 already assigns generator vocabulary to the first Python emitter.
  External scenario/test binding exists; pending kernel.binding is an
  opaque constraint carrier with different scope. Executable guards stay
  deferred pending a data-carrying state space.

The preceding Rule-of-5 review completed five passes. TypeSafe supported
three HIGH findings above its 0.8 threshold; qualified-citation resolution
and graph raw-ID wording remained REVIEW_REQUIRED at 0.75 and 0.44 despite
direct supporting evidence. The user authorized drafting improvements
after receiving those qualifications. Drafts propose concrete resolutions;
they do not imply approval of implementation or deployment.

## Proposals and ordering

1. Amend `add-min-expr-kernel`: new tasks 3.6–3.8 explicitly own canonical
   citation resolution, complete-corpus kernel evaluation, command/report
   parity, and separately tracked refactoring. Full compile/model-check
   snapshots retain existing requirements for the verbatim archive path.
2. New `define-verification-claim-gates`: proposed required/unchecked claim
   classification, aggregate truth table, versioned scope-bound evidence,
   old-report rerun policy, and accurate assurance documentation. Requires
   kernel integration; precedes corpus migration.
3. New `add-py-fragment-emission`, existing l8l: D5 bounded generator data
   grammar, named project definitions, Rust/Python strategy mappings,
   PropertiesRunner adapter, complete mixed-language results, legacy
   placeholder adequacy gate, mandatory kernel fixture agreement and a
   runnable application-test binding example. Depends on kernel/36n and
   the claim policy. The generator opt-in and stricter verification are
   proposed decisions with explicit compatibility/migration consequences.
4. New `add-ts-fragment-emission`, existing aby: reuse the above domains,
   manifest and gate, with project-local fast-check/Vitest execution.
   Depends on l8l; no new TypeScript kernel evaluator is claimed.
5. Amend `add-graph-views`, existing gre: raw edges retain intent IDs and
   qualified row IDs; only file-level views collapse to owning intents.
   New task 1.7 pins two distinct states in one file.

Compile capability overlays MUST deploy in order: kernel → Python →
TypeScript. Rebase each full snapshot against deployed requirements before
implementation/archive. The capability mirror tool requires synchronized
delta/Requirements sections; merged overlapping Pure widening requirements
retain all scenario identities under one requirement. The subsequent review
fix narrows artifact byte preservation to unmarked legacy generators; marked
generator cells deliberately switch to typed values.

## Tracker handling

Expanded l8l/aby descriptions and acceptance criteria as proposal tracking
entries, marked design/proposal-review. They are not atomic implementation
issues; after approval split feature cycles and refactoring into separate
tickets. Linked aby as depending on l8l. Added proposal notes to gre, 36n,
gch, bf5 and lf3. Closed txo/7ga history was not rewritten. Policy and
corrective integration tickets must be created and wired after approval;
gch's notes explicitly prohibit claiming migration before prerequisites.

The installed bd uses an existing `.beads/embeddeddolt` store despite the
tracked no-db setting. Exported only this session's seven changed issue
records back to `.beads/issues.jsonl`; preserved other lines and wrapper
fields. The tracked JSONL remains the project source of truth. No tracker
configuration, migration, remote sync, push or deployment was attempted.

## Quality ledger

Changed — three new proposal suites, two amended existing proposal suites,
seven existing issue records, and this research/handoff note. No runtime
source, tests, deployed capabilities or domain corpus were changed.

Verified — `openspec validate --all --strict --no-interactive`: 25 passed,
0 failed. `spk lint openspec`: 50 files, 0 blocking findings, 18 advisory
warnings. `python3 scripts/check_section_sync.py openspec`: passed.
`ah check`: no structural issues; contract execution skipped as the command
reports. `git diff --check`: passed. No runtime test rerun was needed for
proposal-only changes. Initial duplicate Pure widening headings were
merged before successful validation.

Review — self-review of scope, retained requirements, archive sequencing,
test shapes, runner ownership, backward compatibility and cross-ticket
dependencies. No sub-agents used.

Risks — drafts require maintainer review, particularly required-claim policy,
report migration and generator syntax. Proposed scenario contracts bind to
real tests during implementation; no nonexistent test bindings were added.
Strict proposal validation is not implementation or execution evidence.

Next — review these drafts, then create atomic implementation/refactor issues
and enforce the stated prerequisites. Keep parser-adapter work in existing
3a8 and revision migration in 75m; no duplicate proposals were created for
those already-owned concerns.

## Proposal-review fixes (2026-10-06)

The user requested fixes for all four findings in
`2026-10-06-ro5-proposal-review.md`. All four are addressed in proposal
artifacts and pending test shapes:

- Byte stability now covers unmarked legacy generators. Formerly accepted
  marked cells intentionally change; migrate predicate types, recompile,
  rerun model-check and verify. Both downstream compile mirrors agree.
- Choice branches must unify recursively after alias expansion. No tagged
  unions or coercion in v0. Rust/Python/TypeScript predicate types and direct,
  nested, aliased and empty-list fixture expectations are explicit.
- Graph schema transport is `spk guide --schema --json`, a versioned direct
  projection of canonical Schema, with sorted fields, refinements and flags.
  Producer/consumer perturbation tests and malformed-export failures precede
  implementation; the Python renderer needs no Rust-source parsing.
- Dual-format id: spec command inputs run singly in separate output
  directories. Combined inputs fail before writes; lint retains file-local
  multi-file support. Scope tests reject reports swapped between files with
  identical claim names but different invariant contents.

Existing proposal tracking entries remain pending; no implementation issue
is closed and no implementation task is marked complete. The review report
contains the resolution ledger and validation results.

## Ticket creation completed — 2026-10-07

The user authorized ticket creation after the review fixes. The full index,
coverage mapping, dependencies and quality ledger are in
`2026-10-06-improvement-ticket-breakdown.md`. Created 40 tickets, reused
seven existing work records, and added the missing blocker to specodelic-lf3.
Every pending task now has a ticket owner. Earlier instructions to create
these tickets after review are fulfilled; implementation remains undone.
