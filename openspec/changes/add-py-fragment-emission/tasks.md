# Tasks: Python properties and generator vocabulary

Reviewed and ticketed at the user’s request; owner epic specodelic-l8l.
Each feature cycle and separate TIDY ticket is mapped below. Implementation
has not begun; keep feature and refactoring commits separate. New source files carry intent headers.

## 1. Authors get real generated values
- [ ] 1.1 RED: compile grammar/domain fixtures listed in design D1 and D2;
      prove a marked generator does not emit the legacy constant name while
      an unchanged unmarked input retains legacy artifact bytes. Cover
      homogeneous and mixed-type choices, nested/empty lists and aliases;
      pin v0 types and rejection before any artifact emission.
- [ ] 1.2 GREEN: generator parser, typed IR, registry resolution and Rust
      strategy emission with recursive type unification; labeled failures
      and unchanged unmarked legacy bytes.
- [ ] 1.3 TIDY: separate ticket/commit extracts shared generator metadata.

## 2. Python properties execute and fail meaningfully
- [ ] 2.1 RED: CLI compile→verify fixture with py unit predicate; assert
      passing domain and failing/shrunk counterexample attribution.
- [ ] 2.2 GREEN: Python emitter, artifact manifest, pytest/Hypothesis adapter,
      deterministic artifacts and structured per-block results.
- [ ] 2.3 TIDY: separate ticket/commit consolidates emitter helpers.

## 3. Partial execution cannot pass
- [ ] 3.1 RED: mixed-language failure, missing runner, timeout descendant,
      missing artifact, stale registry, legacy placeholder and explicit
      singleton/parameterless fixtures with exact verdict expectations.
- [ ] 3.2 GREEN: complete dispatch, adequacy gate, source/registry staleness,
      process-tree cleanup and actionable diagnostics; no auto-install.
- [ ] 3.3 TIDY: separate ticket/commit shares reporting without changing gates.

## 4. Shared kernel claims agree across evaluators
- [ ] 4.1 RED: reuse 36n fixture expectations for Python reference evaluation;
      assert all v0 atomics, Kleene operators and boundary statuses agree.
- [ ] 4.2 GREEN: reference adapter and mandatory CI agreement job; document
      promotion to a named-case Property law when both evaluators exist.
- [ ] 4.3 TIDY: separate ticket/commit removes duplicated fixture plumbing.

## 5. Demonstrate scope and deploy contracts
- [ ] 5.1 RED→GREEN: runnable resumable-job example including one broken
      application test via existing external binding; document stage limits.
- [ ] 5.2 Update format/domain specs, Revision literals, compiled artifacts,
      capability matrix and SUMMARY. Publish legacy-generator and formerly accepted marked-cell migration.
- [ ] 5.3 Rebase the full compile delta after kernel archive; preserve all
      deployed Requirements and scenarios. Author actual scenario contracts.
- [ ] 5.4 Strict validation, section sync, corpus lint, ah checks and just ci;
      archive only via the dual-format recipe. Coordinate lf3 corpus gate.

## Implementation ticket map

Created at the user’s request after the proposal review fixes. Checkboxes
remain unchecked until the linked behavior is implemented and verified.
Dependencies are recorded in beads; this table maps scope, not completion.

| Tasks | Ticket | Outcome |
|-------|--------|---------|
| 1.1, 1.2 | `specodelic-l8l.1` | Marked generators give Rust predicates real typed values |
| 1.3 | `specodelic-l8l.2` | Keep generator metadata identical across emitters |
| 2.1, 2.2 | `specodelic-l8l.3` | Python unit properties pass or report a shrunk failure |
| 2.3 | `specodelic-l8l.4` | Keep Rust and Python artifact emission consistent |
| 3.1, 3.2 | `specodelic-l8l.5` | Partial or inadequate property execution cannot verify |
| 3.3 | `specodelic-l8l.6` | Keep property failures identical across reporting paths |
| 4.1, 4.2 | `specodelic-l8l.7` | Rust and Python agree on every declared kernel fixture |
| 4.3 | `specodelic-l8l.8` | Keep backend agreement fixtures shared without skipped cases |
| 5.1 | `specodelic-l8l.9` | A runnable example distinguishes spec checks from application tests |
| 5.2, 5.3, 5.4 | `specodelic-l8l.10` | Python verification retains legacy contracts and migration guidance |
