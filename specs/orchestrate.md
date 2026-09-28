---
id: orchestrate
kind: intent
checked_against_core: clear
statement: "WHEN a repo's files have each reached `parsed`, THE orchestrator SHALL run every Checker Ownership checker in dependency order and then drive `compile`, `model_check`, and `verify` in sequence, halting at the first stage that fails."
---

# Orchestrator

`STATUS.md` §4's last prioritized item: something that actually runs the
pipeline `specodelic.md` describes, the way `ddl` orchestrates the rest of
that tool ecosystem. Every stage already has its own spec —
`linter-frontmatter.md` through `linter-schema_shape.md` plus
`linter-coverage.md` for `lint`, `compile.md` for `compile`,
`model_check.md` for `model_check`, `verify.md` for `verify` — but nothing
until now specified the thing that calls them in the right order, on the
right inputs, and stops (or doesn't) when one of them fails. This file is
that spec, not a reimplementation of any stage's own logic: it owns
sequencing and gating, and defers every actual check to the file that
already owns it.

## Constraints

| id                                  | kind      | expr                                                                                                                                          | traces_to |
|---------------------------------------|-----------|------------------------------------------------------------------------------------------------------------------------------------------------|-----------|
| lint_gate_matches_checker_ownership   | invariant | `orchestrator reports lint success iff every terminal node of specodelic.md's Checker Ownership table reports passed — never a subset or a superset of that table` | [[orchestrate]] |
| dependency_respecting_skip            | invariant | `a checker is invoked only after every checker listed in its Checker Ownership 'Depends on' column has reported passed; if a dependency reports failed, its dependents are skipped (not invoked, not reported as failed) rather than run against input the failed checker hasn't validated` | [[orchestrate]] |
| independent_branches_run_regardless   | invariant | `two checkers with no dependency relation between them (referential_integrity→graph_shape→model_shape vs. ears_syntax vs. schema_shape) each run and report independently — one branch failing never skips or blocks the other` | [[orchestrate]] |
| compile_gate_matches_coverage         | invariant | `orchestrator advances to model_check iff linter.coverage reports passed — exactly specodelic.md's compile guard ([[specodelic.coverage]] ∧ [[specodelic.law_requires_cases]]), not a looser or stricter check` | [[orchestrate]] |
| stage_order_fixed                     | invariant | `compile is never invoked before every file's lint stage reports passed; model_check is never invoked before compile reports compiled; verify is never invoked before model_check reports model_checked` | [[orchestrate]] |
| external_completeness_never_gates     | invariant | `linter.external_completeness runs only if the repo declares a checklist, and its outcome (passed/failed/not_applicable) never blocks or delays lint, compile, model_check, or verify` | [[orchestrate]] |
| deterministic_rerun                   | invariant | `running the orchestrator twice against an unchanged repo produces byte-identical reports` | [[orchestrate]] |

## Model

### States
- `idle`
- `lint_stage`
- `compile_stage`
- `model_check_stage`
- `verify_stage`
- `succeeded`
- `failed`

### Transitions

| id                | from                | to                  | guard                                                                                              |
|-------------------|---------------------|---------------------|-------------------------------------------------------------------------------------------------------|
| start_lint        | idle                | lint_stage          | `every file in the repo has independently reached parsed` — see [[linter.frontmatter]]                |
| lint_ok           | lint_stage          | compile_stage       | [[orchestrate.lint_gate_matches_checker_ownership]] ∧ [[orchestrate.dependency_respecting_skip]] ∧ [[orchestrate.independent_branches_run_regardless]] |
| lint_fail         | lint_stage          | failed              | `¬lint_ok.guard`                                                                                       |
| compile_ok        | compile_stage       | model_check_stage   | [[orchestrate.compile_gate_matches_coverage]]                                                          |
| compile_fail      | compile_stage       | failed              | `¬compile_ok.guard`                                                                                    |
| model_check_ok    | model_check_stage   | verify_stage        | [[orchestrate.stage_order_fixed]] ∧ `model_check.md's model_checked/no_counterexample distinction resolved` |
| model_check_fail  | model_check_stage   | failed              | `¬model_check_ok.guard`                                                                                |
| verify_ok         | verify_stage        | succeeded           | [[orchestrate.stage_order_fixed]] ∧ `verify.md's both_gates_required holds`                            |
| verify_fail       | verify_stage        | failed              | `¬verify_ok.guard`                                                                                     |

## Properties

| id                                  | kind | derives_from                                        | generator                                                                  | predicate                                                                                     |
|----------------------------------------|------|----------------------------------------------------------|----------------------------------------------------------------------------------|---------------------------------------------------------------------------------------------------|
| upstream_failure_skips_dependents      | unit | [[orchestrate.dependency_respecting_skip]]              | `repo_where(linter.frontmatter: fails)`                                          | `invocation_count(linter.referential_integrity) == 0` — never run against unvalidated input        |
| independent_branch_failure_isolated    | unit | [[orchestrate.independent_branches_run_regardless]]     | `repo_where(linter.ears_syntax: fails, linter.referential_integrity_branch: passes)` | `report(linter.graph_shape) == passed` — the passing branch's own outcome is unaffected             |
| compile_blocked_on_partial_lint        | unit | [[orchestrate.stage_order_fixed]]                       | `repo_where(five_of_six_checkers_pass, one_still_pending)`                       | `invocation_count(compile) == 0`                                                                    |
| external_completeness_failure_ignored  | unit | [[orchestrate.external_completeness_never_gates]]       | `repo_where(all_seven_checkers_and_pipeline_pass, declared_checklist: fails)`     | `orchestrate reaches succeeded`                                                                      |
| rerun_idempotent                       | unit | [[orchestrate.deterministic_rerun]]                     | `run_orchestrator_twice_against_unchanged_repo()`                                | `report(run_1) == report(run_2)`                                                                     |
| clean_repo_succeeds                    | unit | [[orchestrate.lint_gate_matches_checker_ownership]]     | `arbitrary_repo_that_independently_passes_lint_compile_model_check_verify()`      | `orchestrate reaches succeeded`                                                                      |
| coverage_failure_holds_compile         | unit | [[orchestrate.compile_gate_matches_coverage]]           | `repo_where(linter.coverage: failed, every_other_checker_and_pipeline_stage: passed)` | `orchestrate halts at compile_stage` — never reaches model_check_stage, and by exactly the coverage checker's verdict, not a looser or stricter one |

## Notes

`checked_against_core: clear` (see `AGENTS.md`'s convention). As with
`rename.md` and `linter-external_completeness.md`, this file's constraints
are local to what *sequencing and gating* specifically require — none of
them assert anything about spec-file content that `specodelic.md` itself
should own.

**Scope boundary: this file drives the four-stage pipeline, not
`rename`.** `rename.md`'s `(old_id, new_id)` operation is invoked on
demand by a person or tool, never automatically as part of a
lint/compile/model_check/verify run — folding it in here would make an
ordinary pipeline run capable of silently mutating the repo it's checking,
which is exactly the kind of implicit side effect `AGENTS.md`'s workflow
rules exist to prevent. If a repo's CI wants "verify, then rename some
stale id, then re-verify" as one recipe, that's a policy composed *of*
`orchestrate` and `rename` calls by whatever invokes both — not a fact
either file should assert about the other.

**Why `model_check_ok`/`verify_ok` use a plain-text clause alongside a
bracketed one.** `stage_order_fixed` only says *when* `model_check`/
`verify` may start, not what result they must report to count as success
— that's `model_check.md`'s `model_checked`/`no_counterexample`
distinction (`CLAR-003`) and `verify.md`'s `both_gates_required`,
respectively, neither of which is itself phrased as a single Constraint
row this file could point a `guard` at. Rather than restate those files'
internal logic as a new orchestrate-local constraint (duplicating logic
this file doesn't own), the guard names the fact in prose and leaves the
actual check to the file that already owns it — the same "same claim,
different altitude, don't restate the logic" discipline
`linter-referential_integrity.md`'s Notes describe for `rename_naturality`.

**Where `graph.md`, `refactor.md`, and `merge.md` fit.** None of the three
are part of this file's own four-stage pipeline. This file needs no
`refactor_advisory_never_gates`-style invariant of its own: `refactor.md`'s
finding is emitted via `emits` (`kind == effect`), and
`[[specodelic.ref_kind_compatible]]`'s existing typing already makes any
`effect`- or `advisory`-kind Constraint ineligible as a `guard` target,
tested generically by `[[specodelic.advisory_cannot_gate]]` — nothing
here can reference `refactor.md`'s output as a guard even by mistake, so
asserting it again would only restate an already-proven fact. Contrast
`external_completeness_never_gates` above, which *is* still needed: that
checker's own constraints are `invariant`-kind, so its non-gating status
holds only because no transition below happens to cite it — a wiring fact
this file must keep asserting, not a typing fact guaranteed forever the
way `refactor.md`'s is. `graph.md` is a standing derived artifact other
files query, not a pipeline stage, and `merge.md` runs at merge time,
outside a single repo's own lint→verify run, the same way `rename.md` is
explicitly out of scope above. This orchestrator stays scoped to the four
stages `specodelic.md` already defines; it doesn't grow a fifth or sixth
stage just because new files exist to query.

**`graph.md`'s reference graph and this file's Checker Ownership table are
two different graphs, not one generalized into the other.** Checker
Ownership's "Depends on" column is tool-execution-order metadata — which
linter must finish before another starts — fixed at design time and never
derived from any spec file's content. `graph.md`'s adjacency structure is
content-reference edges (`traces_to`/`guard`/`emits`/etc.) mechanically
extracted from spec files. They happen to look similar (both are small
dependency graphs over roughly the same set of files) but answer different
questions, and folding Checker Ownership into `graph.md` — replacing this
table with a `graph.md` query — would conflate tooling sequencing with
content dependency. Flagged here specifically because the resemblance
makes that an easy mistake to make later, not because either file is
unclear on its own.

**With this file, every item `STATUS.md` §4 had prioritized (P0 through
the old P2) is now specced**: the compile/verify pipeline, the rename
tool, external completeness, and this orchestrator. What's left in §4 is
the lower-severity, already-flagged loose ends (`CLAR-001` through
`CLAR-003`, `EXCL-001`, and the acknowledged `no_prose_field_parsed` shape
mismatch) — none of which block one another or this file, and none of
which were newly surfaced by writing it.

**Open question, `Needs Human Review`:** this file assumes "every file in
the repo has independently reached `parsed`" as `start_lint`'s
precondition, i.e. per-file parsing happens outside the orchestrator's own
Model. Whether the orchestrator should own driving `draft → parsed` for
each file too (making it a true single entry point) or stay scoped to
`parsed → verified` as specified here is a design choice with different
consequences for what "running the orchestrator on a `draft`-only repo"
even means — left open rather than assumed.
