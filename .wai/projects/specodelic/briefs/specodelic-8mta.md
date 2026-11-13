# Subagent brief: specodelic-8mta — resolve-check kernel id literals (E2 F1)

You are a pi subagent in repo `/var/home/sasha/para/areas/dev/gh/charly/specodelic`
(Rust CLI `specodelic`, alias `spk`). An orchestrator has claimed ticket
`specodelic-8mta` and is hands-off: you own the implementation end-to-end.
The orchestrator verifies with real gates — never claim work the diff doesn't
contain.

## Why this ticket

Ticket `specodelic-8mta` (P2, from experiment E2 finding F1): kernel id
literals are compared as opaque strings with no resolution check, so an
unqualified or typo'd literal yields a **verified-vacuous** claim instead of
`unknown`. Measured live in E2 probe round 1:
`¬(∃ t ∈ Transition: t.to == both_go)` is `verified` because `both_go` never
matches any qualified id (`probe.traffic.both_go`). That contradicts the
corpus guarantee `kernel.kernel_status_three_valued` / `kernel.kernel_decidable`
(`unknown` is honest; no path coerces it to pass or counterexample).

Current code, for orientation:
- `src/kernel.rs:769` — `Term::Id(id) => TermVal::Id(id.clone())`; no check.
- `KernelEnv::from_specs` already builds per-object membership (`members`) and
  the acset `Instance`; `kind_index` qualifies row ids corpus-wide.
- `Atomic::Reachable` seeds go through `query::forward_closure`, which already
  errors on unknown seeds → unknown with a labeled reason (tests:
  `tests/kernel_grounding.rs` `reachable_*`, `tests/kernel_status.rs`).
  The target id (`to`) is only a set-membership test — an unresolvable target
  currently yields `counterexample`, which is the same honesty gap.
- Corpus blast radius: the four `**kernel:**` claims in
  `openspec/specs/specodelic/spec.md` use bound vars, projections, `⊥`, and
  object cardinalities only — no bare id literals. Their verdicts MUST NOT
  change.

## Design decision of record (orchestrator, 2026-11-13) — implement this

**Resolve-check id literals at evaluation time → `unknown` with a labeled
reason.** Do NOT implement the alternative lint-advisory candidate from the
ticket: a warning cannot stop a verified-vacuous claim, and the corpus already
promises the honest status.

- `Term::Id(id)` resolves iff `id` names a declared node in the evaluation
  corpus (any object's membership: Intent ids are file ids, row ids are
  file-qualified). Add an `O(log n)` lookup (e.g. a `BTreeSet<String>` built
  in `from_specs`) — never a linear scan per evaluation.
- Unresolvable literal → `TermVal::Unknown` plus a labeled reason that names
  the literal and the rule, e.g.
  ``id literal `both_go` resolves to no declared id in this corpus — id
  literals must be file-qualified (e.g. `file.row`)``.
  Thread the reason out of `term_value` (change its signature to carry
  `Option<String>`) so `eq`/`cmp` surface the literal's cause instead of the
  generic "mismatched or indeterminate values" text.
- `Atomic::Reachable(from, to, m)`: resolve-check `to` as well; an
  unresolvable target → unknown with the labeled reason naming it. Keep the
  existing unknown-seed path (forward_closure) and its reason.
- Kleene discipline unchanged: `¬unknown` stays unknown, a counterexample
  still dominates `∧`, resolved literals keep their exact current verdicts.
- No new atomic, no grammar change, no parser change: this is evaluation
  semantics only.

## What to build (TDD, red → green → refactor)

1. **RED** — add tests that fail on current HEAD:
   - unqualified literal in `∃`/`¬∃` equality → `unknown` (NOT verified, NOT
     counterexample) with a reason naming the literal;
   - qualified-but-typo'd literal → `unknown`, reason names it;
   - resolved literal comparison → unchanged verdict (regression pin);
   - `reachable` with an unresolvable target → `unknown` + labeled reason;
   - Kleene: the unresolved literal under `¬` stays unknown; a counterexample
     operand still dominates `∧`.
   Prefer `tests/kernel_grounding.rs` (or a new `tests/kernel_literals.rs`);
   CLI-level status/reason assertions can extend `tests/cli/model_check.rs`.
   `src/kernel.rs` has no in-module test block for this area — do not add one
   if an integration file fits.
2. **GREEN** — implement the resolve-check + reason threading.
3. **Docs** — `src/guide.md` grammar summary gets one sentence: id literals
   must name a declared (file-qualified) id; an unresolvable literal
   evaluates `unknown` with a labeled reason. In
   `openspec/specs/kernel/spec.md`, clarify under the existing
   `Three-valued evaluation status` requirement (prose sentence or one
   `#### Scenario:` block — if you add a scenario block, `ah check` must stay
   green). Do NOT add table rows, do not touch other corpus files.
4. **Verify non-vacuity** — prove at least one test fails against the unfixed
   code path (stash/`git stash` is fine, or reason from the RED run) and that
   the E2 probe shape (`¬(∃ t: t.to == both_go)`) now reports unknown.

## Hard rules

- No softened assertions; no `true`/placeholder predicates; no suppression of
  the existing generic reasons; no changing verdicts for resolvable literals.
- If you find a *second*, genuinely different vacuity source, report it — do
  not silently widen scope.
- If the declared semantics cannot be implemented as described, STOP and
  report with evidence.

## Scope guard

Allowed: `src/kernel.rs`, `src/guide.md`, `tests/kernel_grounding.rs`,
`tests/kernel_status.rs`, `tests/kernel_grammar.rs`, `tests/common/kernel_corpora.rs`,
`tests/common/kernel_fixtures.rs`, `tests/cli/model_check.rs`,
`openspec/specs/kernel/spec.md` (prose/scenario clarification only),
`.wai/projects/specodelic/**` (your own artifacts only).

Never edit: `src/model_check.rs` (pretender-pinned), `src/lint/**`,
`openspec/specs/compile/spec.md`, `openspec/specs/specodelic/spec.md`, other
corpus files, `openspec/changes/**`, `.wai/resources/**`, `.wai/pipeline-runs/**`,
`.beads/**`, `packs/`, `specs/`, `pretender.toml`, `.genesis/tools.toml`.

## Orientation

- `export WAI_PROJECT=specodelic`; `export TMPDIR=/var/tmp/specodelic-xch1`
  (`mkdir -p` it); `bd show specodelic-8mta`; `bd heartbeat specodelic-8mta`
  before long phases.
- Read: `.wai/projects/specodelic/research/2026-11-12-4und-experiment-e2-kernel-coverage-report.md`
  §3.2 and §8 F1; `src/kernel.rs` (parser `term`, `KernelEnv`, `term_value`,
  `eq`, `cmp`, `atomic`, `eval_scope`); `openspec/specs/kernel/spec.md`;
  `src/guide.md` kernel grammar summary; the reachable/unknown tests above.
- Start your executor pipeline:
  `wai pipeline start tdd-ro5 --topic="specodelic-8mta: resolve-check kernel id literals"`
  Export the printed `WAI_PIPELINE_RUN=<tdd-ro5 run id>` and advance ONLY via
  `wai pipeline next`; never hand-edit run YAMLs; never advance the
  orchestrator run
  (`epic-orchestrator-2026-11-13-specodelic-8mta-resolve-check-kernel-id-literals-e2-f1-vacuous-verified`).
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
- resolution lookup + reason threading shape

**Behavior**
- E2 probe shape before/after (status + reason)
- corpus verdicts before/after (must be identical)

**Gates run**
- `cargo test --test kernel_grounding` (and any new file) → <result>
- `just test` → <result>
- `ah check --run-tests --all-tests` → <result>
- `openspec validate --all --strict` → <result>
- `just lint-specs` / `just ci` → <result>

**Deviations** (or "none")
- <any scope/plan deviation and why>

**Next**
- <exact next action for the orchestrator, or "ticket complete">