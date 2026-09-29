# Design: add-model-check

## Context

`specs/model_check.md` is the spec of record: a backend-neutral run
state machine (`not_run → running → clean | counterexample_found |
timed_out`) with constraints `checker_invoked`, `exhaustive_within_bound`,
`counterexample_is_minimal`, `counterexample_names_violated_invariant`,
`backend_identified`, `no_counterexample_feeds_verify`,
`rerun_on_model_change`. Its Notes pin the backends: **stateright**
(default, embedded Rust crate, BFS ⇒ minimality by construction,
`target_max_depth`/`timeout`/`target_state_count` as the bound knobs) and
**TLC** (opt-in JVM reference engine). The compiled artifacts exist:
`ModelIR` (states/transitions-with-prose-guards/emits), a byte-stable
`.tla` module, a TOML doc, and proptest scaffolding whose predicates are
`todo_predicate!` placeholders.

The format corpus has **no executable predicate language**: guard and
predicate cells are prose (optionally with `[[ref]]` links). Anything
that "executes" them today either treats them as comments (`.tla`
emission) or panics at runtime (proptest scaffolding).

## Goals / Non-Goals

- Goals: the `model_check` step as a real CLI arm over the envelope; a
  native default backend that honestly runs; run reports `verify` can
  consume with staleness; `backend_identified` attribution.
- Non-Goals: executing prose predicates (no predicate language exists —
  Decision 3); TLC (ug3); orchestration (8kk); re-compiling inside
  model-check (anti-goal).

## Decisions

### Decision 1 — ticket split: nx7 = step + native backend; ug3 = TLC

`specodelic-nx7`'s description predates the mp1 row-6 decision ("pick ONE
of TLC/Alloy") — superseded by `model_check.md`'s Notes. Scope now:
nx7 ships the step machinery (state machine, CLI, bound flags, run
report, staleness) **plus the native stateright backend**; `specodelic-ug3`
shrinks to the TLC opt-in backend (JVM subprocess over the emitted
`.tla`, `-depth` as the stated bound, missing binary = ERROR, never
`no_counterexample`) and cross-backend report comparability. Both
tickets get updated descriptions at implementation time.

### Decision 2 — Bound shape

`Bound { max_depth: u32, max_states: Option<u64>, timeout: Option<Duration> }`,
CLI flags `--max-depth` (default 100), `--max-states`, `--timeout-secs`.
The report always restates the applied bound (`exhaustive_within_bound`:
"the bound is part of the report"). These map 1:1 onto stateright's
`target_max_depth` / `target_state_count` / `timeout` checker knobs named
in `model_check.md`'s Notes — the earlier "Bound type shape" question
from the mp1 row-6 session is resolved by this mapping.

*Post-review amendments (rule-of-5, 2026-09-28):* (1) stateright's knobs
are `NonZeroUsize`, so a `0` flag value is clamped to `1` rather than
silently ignored; (2) `depth_reached == cap` is ambiguous ("stopped at
cap" vs "the space ends at cap") — when the depth cap is the only
constraint, a confirmation re-run at `cap+1` resolves it: completing
below `cap+1` proves the space ends at depth ≤ cap and the run reports
`no_counterexample`; hitting `cap+1` too reports `timed_out`. Runs with
a simultaneous state/time cap stay conservative (`timed_out` on any
reached cap), since the confirmation could itself be truncated.

### Decision 3 — what the native backend checks (HITL — needs approval)

There is nothing executable to violate yet: guards/predicates are prose.
Three options:

- **A (recommended): abstract dynamics, honest clean.** stateright
  interprets the `ModelIR` as a program-counter model — state = the
  automaton's state value, actions = transitions, always enabled
  (identical semantics to the committed `.tla` emission, guards carried
  as annotations). The run explores exhaustively within the bound and
  reports `no_counterexample` (or `timed_out`). No user invariant is
  executed, and the report says so (`invariants_checked: []` — honest,
  `checker_invoked` still holds: a real run happened). nx7's MUST leg
  "a synthetic spec with a violated invariant yields a counterexample
  trace naming the invariant" is **deferred** to the predicate-fragment
  decision and amended on the ticket at approval.
- **B: minimal guard fragment now.** Give `[[file.c]]`-only guards an
  executable meaning (e.g. transition enabled iff `c` "holds") — but
  nothing in the corpus parses into it, so it is dead machinery with
  specodelic.md-revision implications. Rejected for v0.
- **C: user-supplied Rust predicates.** A `checks/` companion or inline
  Rust in spec files — a real format change (Revision bump) that makes
  counterexamples genuinely executable, but far larger than nx7.
  Candidate for a future proposal; row 7 on `specodelic-mp1`.

Under A, `counterexample_found` remains a real state machine outcome the
report schema carries (and TLC will populate in ug3); the native v0
backend simply never emits it. Tests pin that honesty: a spec with
prose invariants yields `no_counterexample` + `invariants_checked: []`,
never a fabricated violation.

*Post-review amendment (rule-of-5, 2026-09-28):* Option A additionally
required an artifact-consistency guard — the run interprets the live
spec's IR, so the on-disk `.tla` is parsed (StateValues set + Next
disjunct ids) and compared against the IR before any run; a mismatch is
a labeled `stale_artifact` error with a `spk compile` hint, closing the
mixed-(IR, artifact) hole `checker_invoked` forbids.

### Decision 4 — run reports persist; staleness by content hash

Each run writes `<stem>.check.json` next to compile's artifacts:
`{backend: {engine, version}, bound, outcome, violated_invariant_id?,
trace?, artifact_sha256}` where `artifact_sha256` is the hash of the
compiled `.tla` module the run consumed. `no_counterexample_feeds_verify`
+ `rerun_on_model_change` then become checkable facts: verify (1pv)
re-hashes the current artifact and treats a stored clean report as
`undetermined` when the hash differs. In-memory only would make
`blocks_run_to_completion`-style "most recent run" unimplementable for
1pv.

### Decision 5 — stateright as a direct dependency

`stateright = "<current>"` in `[dependencies]`. The version constant is
surfaced in the report's `backend.version` (engine attribution) via
`stateright::VERSION`-equivalent or the crate's own version string —
pinned so two backends' reports are attributable (`backend_identified`).

## Risks / Trade-offs

- **MUST amendment** (Decision 3) weakens nx7's original contract →
  surfaced explicitly at this approval gate, tracked as an mp1 row.
- stateright's API surface (async checker, port binding for its web UI)
  must be used in offline `Checker` mode only → mitigation: integration
  test asserts no listener; keep to the `checker(seed)` exploration API.
- `.check.json` grows the committed-artifact set → same rationale as the
  other artifacts (byte-stable, diff-visible reruns).

## Migration Plan

Additive: new module + dependency + one CLI arm switch + one more
artifact per compiled file. Rollback = drop the arm; nothing existing
changes shape.

## Open Questions

- Decision 3 option (A/B/C) — the approval gate decides; A recommended.
- Default `--max-depth` (100 proposed) — cheap to change.
