# Subagent brief: specodelic-9o1h — compose kernel outcomes into the citation algebra (E2 F2)

You are a pi subagent in repo `/var/home/sasha/para/areas/dev/gh/charly/specodelic`
(Rust CLI `specodelic`, alias `spk`). An orchestrator has claimed ticket
`specodelic-9o1h` and is hands-off: you own the implementation end-to-end.
The orchestrator verifies with real gates — never claim work the diff doesn't
contain.

## Why this ticket

Ticket `specodelic-9o1h` (P2, experiment E2 finding F2): the citation algebra
resolves evidence against **executable** outcomes only, so
`[[exec]] ∧ [[kernel]]` evaluates `unknown` ("missing target … no same-run
evidence") even when both claims hold; Rule 4 of `aggregate_required_claims`
then turns the file into `exploration_only` — it can never go clean. Measured
in `.wai/projects/specodelic/research/2026-11-12-4und-experiment-e2-kernel-coverage-report.md`
§5 ("Citation algebra measurement") and §8 F2. The ticket requires a design
note before authors mix claim kinds — the orchestrator's decision of record is
below; record it as your `wai add design` artifact in the design phase.

Current code, for orientation:
- `src/citation_corpus.rs`: `apply_corpus_resolution(specs, runs)` builds
  `exec_maps` per file from report statuses (excluding citation rows) and calls
  `resolve_corpus_citations(specs, &exec_maps)`. `Resolver::resolve_target`
  consults, in order: `exec_outcomes` → citation rows (dependency order,
  cycle-detecting) → property rows (always unknown) → missing target.
- `src/orchestrate.rs::run_claim_gated_model_check`: backend pass →
  `apply_corpus_resolution` → `kernel::evaluate_corpus_claims` →
  `merge_corpus_statuses` → `aggregate_required_claims`. Kernel claims are
  appended only at the merge, after citations were already resolved.
- `src/kernel.rs::evaluate_corpus_claims` evaluates from `&[Spec]` alone (no
  artifact dependency) — it can run earlier.
- `src/commands/model_check.rs` consumes the same shared
  `run_claim_gated_model_check` path; both views must agree.

## Design decision of record (orchestrator, 2026-11-13) — implement this

1. **Evidence union:** a citation target resolves against same-run EXEC
   outcomes *and* corpus-static KERNEL claim outcomes. Keep the existing
   precedence order in `resolve_target`: exec → kernel → citation row
   (dependency order) → property-row unknown → missing. Exec and kernel sets are
   disjoint (a row carries exactly one evaluator), so the order only documents
   intent.
2. **Evaluate kernel claims first:** move `kernel::evaluate_corpus_claims`
   before citation resolution in `run_claim_gated_model_check` and pass the
   per-file kernel statuses into `apply_corpus_resolution` /
   `resolve_corpus_citations` as an added evidence map (smallest signature
   change that keeps the resolver testable from `tests/citation_algebra.rs`).
   Never re-evaluate kernel claims per citation.
3. **TLC backend:** kernel claims are `unknown` with the unsupported reason;
   citations citing them stay `unknown` (Kleene, never pass). No special case.
4. **One-way composition:** the closed kernel grammar contains no `[[..]]`
   references, so kernel→citation cannot cycle. Citation→citation dependency
   and cycle semantics are unchanged.
5. **No aggregate change:** `aggregate_required_claims` Rule 4 is untouched.
   The acceptance flip is that `[[exec]] ∧ [[kernel]]` now verifies and the
   file can report `no_counterexample` when every required claim discharges.
6. **Status identity unchanged:** `merge_corpus_statuses` still appends each
   kernel claim exactly once; a citation row citing a kernel claim is its own
   row. `verify` consumes the persisted outcome unchanged.
7. **Docs:** state cross-evaluator composition in the corpus (prose in
   `openspec/specs/model_check/spec.md` citation requirement and/or
   `openspec/specs/kernel/spec.md`; no new table rows or `#### Scenario:`
   blocks that lack a contract). `just lint-specs` / `openspec validate` stay
   green.

## What to build (TDD, red → green → refactor)

1. **RED** — tests that fail on current HEAD (extend `tests/citation_algebra.rs`
   and/or `tests/cli/citation_resolution.rs`; CLI-level outcome assertions can
   go in `tests/cli/model_check.rs`):
   - same-file `[[exec]] ∧ [[kernel]]`: both verify → the citation row is
     `verified`, every required claim verified, outcome `no_counterexample`
     (this is the F2 acceptance case; on HEAD it is `unknown` →
     `exploration_only`);
   - kernel claim `unknown` (e.g. an unresolvable literal or a dangling
     atom) → citing citation `unknown` with a composed reason that names the
     kernel claim cause; outcome `exploration_only`;
   - kernel claim `counterexample` → citing citation `counterexample`; Rule 2
     names the citation row, no invented state trace;
   - qualified cross-file citation `[[otherfile.kernelrow]]` composes the same
     way;
   - TLC/unsupported backend leg: kernel claim unsupported → citing citation
     unknown (composed reason), never verified;
   - regression pins: exec-only citations unchanged; citation-of-citation
     ordering/cycle reasons unchanged; missing-target and property-row reasons
     unchanged; kernel claims still appear exactly once in
     `invariant_statuses`.
2. **GREEN** — implement the evidence union + evaluation reordering.
3. **Refactor** — keep `resolve_corpus_citations` callable in isolated tests
   (an empty kernel map must behave exactly as today).
4. **Non-vacuity** — prove the acceptance test fails with the implementation
   reverted (stash is fine), and that the E2 shape `[[exec]] ∧ [[kernel]]`
   flips from `unknown`/`exploration_only` to `verified`/`no_counterexample`
   end-to-end through the real binary.
5. **Both views** — prove the CLI `model-check --json` and the orchestrate
   stage report the same statuses/outcome for one fixture.

## Hard rules

- No softened or vacuous claims: every new test must fail without the fix.
- Do not re-implement any evaluator in the citation algebra; it only consumes
  statuses.
- If the composition as specified is unsound (a genuine semantic conflict,
  e.g. a kernel claim that can cite), STOP and report with evidence — do not
  invent a different design silently.

## Scope guard

Allowed: `src/citation_corpus.rs`, `src/orchestrate.rs`, `src/kernel.rs` (only
if the evidence hand-off needs it), `openspec/specs/model_check/spec.md` and
`openspec/specs/kernel/spec.md` (prose clarification only),
`tests/citation_algebra.rs`, `tests/cli/citation_resolution.rs`,
`tests/cli/model_check.rs`, `tests/common/**` (fixtures), `.wai/projects/specodelic/**`
(your own artifacts only).

Never edit: `src/model_check.rs` (pinned hotspot; the fix must not need it),
`src/lint/**`, `openspec/specs/specodelic/spec.md` (a genuine format gap is
STOP-and-report), other corpus files, `openspec/changes/**`, `.wai/resources/**`,
`.wai/pipeline-runs/**`, `.beads/**`, `packs/`, `specs/`, `pretender.toml`,
`.genesis/tools.toml`.

## Orientation

- `export WAI_PROJECT=specodelic`; `export TMPDIR=/var/tmp/specodelic-xch1`
  (`mkdir -p` it); `bd show specodelic-9o1h`; `bd heartbeat specodelic-9o1h`
  before long phases.
- Read: E2 report §5 + §8 F2; `src/citation_corpus.rs` (`Resolver`,
  `resolve_target`, `resolve_corpus_citations`); `src/orchestrate.rs`
  `run_claim_gated_model_check`; `src/kernel.rs` `evaluate_corpus_claims`,
  `merge_corpus_statuses`, `aggregate_required_claims`;
  `src/compile.rs::extract_model_ir` (`guard_citations`, `guard_kernel`);
  `openspec/specs/model_check/spec.md` (citation requirements);
  `tests/citation_algebra.rs`, `tests/cli/citation_resolution.rs`.
- Start your executor pipeline:
  `wai pipeline start tdd-ro5 --topic="specodelic-9o1h: compose kernel outcomes into the citation algebra"`
  Export the printed `WAI_PIPELINE_RUN=<tdd-ro5 run id>` and advance ONLY via
  `wai pipeline next`; never hand-edit run YAMLs; never advance the
  orchestrator run
  (`epic-orchestrator-2026-11-13-specodelic-9o1h-compose-kernel-outcomes-into-the-citation-algebra-e2-f2`).
- `cargo fmt` before commits; `pretender check` green; targeted tests during
  red/green, full battery at the end.

## Repo facts (boilerplate — applies to every ticket)

- **Commit hygiene** (standing, every commit): before ANY `git commit`, run
  `git status --short` and stage only files YOU authored
  (`git add <paths>`, never bare `git add`); unstage foreign files.
  Attribute the message only to what the diff contains.
- **Output discipline**: every command emits through
  `genesis::guide::Output::emit`; errors carry a remediation hint.
- **beads**: issues live in `.beads/issues.jsonl`; the orchestrator runs
  `bd close` (you do not). Do not commit `.beads/**` yourself.
- **Do NOT push** — the orchestrator verifies and pushes.
- **Do NOT run `wai close`** — the orchestrator owns session close.
- **ah/espectacular**: read-only over `openspec/`; contract TOMLs for
  change-phase scenarios are orphans until archive — author them in the
  archive commit, never mid-change.
- **New source files** need Purpose/Responsibilities/Rationale headers
  (file-headers skill convention).
- **Non-interactive shells**: use `-f`/`-rf`/`-y` flags; commands may be
  aliased with `-i` and will hang on prompts.

## Report format (end with this — the orchestrator verifies against it)

## Report

**Commits**
- `<hash>` <message> — <what it does, one line>

**Design decisions**
- evidence map shape + signature changes + where kernel evaluation moved

**Behavior**
- F2 probe `[[exec]] ∧ [[kernel]]`: before/after status + file outcome
- negative legs (kernel unknown / counterexample / TLC unsupported)

**Gates run**
- targeted citation/kernel tests → <result>
- `just test` → <result>
- `ah check --run-tests --all-tests` → <result>
- `openspec validate --all --strict` → <result>
- `just lint-specs` / `just ci` → <result>
- non-vacuity proof (revert run) → <result>

**Deviations** (or "none")
- <any scope/plan deviation and why>

**Next**
- <exact next action for the orchestrator, or "ticket complete">