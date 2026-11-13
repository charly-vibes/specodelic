# Subagent brief: specodelic-3632 — the four seams blocking lf3's verify gate

You are a pi subagent in repo `/var/home/sasha/para/areas/dev/gh/charly/specodelic`
(Rust CLI `specodelic`, alias `spk`). An orchestrator has claimed ticket
`specodelic-3632` and is hands-off: you own this work end-to-end. The
orchestrator verifies with real gates — never claim work the diff doesn't
contain.

## Why this ticket

`specodelic-lf3` run 4 (commit `004398e1`) migrated 43 of 47 rows in
`openspec/specs/specodelic/spec.md` + `openspec/specs/kinds/spec.md` to
executable `**rust:**` predicates. Four rows stay on the honest
`todo_predicate!` placeholder because the tool has no seam to decide them, so
`just verify-specs` is still scoped to `[compile, packs]`:

1. `kinds` `constraint_row_extra_field_rejected` — an extra Constraint column
   (`guard`) is parsed without a finding.
2. `kinds` `state_row_with_extra_field_rejected` — an extra State field
   (`kind`) is parsed without a finding.
3. `kinds` `subkind_addition_without_revision_rejected` — a diff adding a
   subkind value with no new `## Revision` heading is not decided (no
   revision-diff reader).
4. `specodelic` `silently_shrunk_id_set_rejected` — a governed id-set member
   removed with no Revision heading is not decided (same reader).

## Phase A — design first (before any red test)

Record a `wai add design` artifact deciding:

- **Row-shape enforcement semantics.** `openspec/specs/kinds/spec.md` already
  declares `constraint_row_shape` (base fields `{id, kind, expr, traces_to}`
  plus typed reference columns declared in `specodelic.md`'s Reference
  Typing — currently `satisfies`, `observes` — plus active-pack fiber kinds),
  `state_row_shape` (`{id, emits?}`), `transition_row_shape`,
  `property_row_shape`. Decide where the rule lives (a new
  `linter.row_shape_closed` rule, or a widened `linter.constraint_kind_closed`
  family), its `rule_id`, and its `rule_semantics` string. It MUST read the
  field set from the schema-as-data (Reference Typing columns + pack kinds),
  never a hardcoded list that can drift. The corpus documentation for a new
  rule id is part of the change (extend
  `openspec/specs/linter-schema_shape/spec.md` with a constraint + property,
  or cite the existing kinds constraints — do not invent a rule the corpus
  cannot explain).
- **Revision-diff semantics.** `append_only_variants`
  (`openspec/specs/specodelic/spec.md`) and `kind_field_extensible`
  (`kinds.md`) speak of id-sets across revisions `r < r'`. Decide what the
  checker reads: e.g. a pure function over a `(prior, current)` text pair that
  parses both, computes the governed sets (States ids; Constraint/Property
  kind value-sets; the Reference Typing field set; kinds.md's subkind sets),
  and fails when a member is removed or a subkind added without a new
  `## Revision N` heading in `current`. Name the public seam the property
  fragments will call.
- **If the claim as written is unsound or undecidable**, STOP and report with
  evidence. Do NOT reword/retire a corpus property row or invent semantics
  the corpus does not state — that comes back to the orchestrator.

## Phase B — implement, red → green

- Implement the seams + the lint rule(s); add the corpus constraint/property
  rows that document the new rule (four-layer shape, refs resolve, coverage
  holds). `src/lint/mod.rs`'s RULE_TABLE and `rule_semantics` are the catalog;
  `just check-explain-topic-docs` / `just lint-baseline` / `just doc-examples`
  may need the corresponding doc updates.
- Migrate the four property cells to `**rust:**` predicates calling the real
  seams over the existing fixtures (`src/lib.rs` `fixtures::specs`, extend as
  needed; `tests/specodelic_fixtures.rs` proves fixtures are real inputs).
- Regenerate the compiled artifacts canonically
  (`cargo run -q -- compile <CORPUS_DIRS> --out-dir specodelic/` and
  `cargo run -q -- model-check <CORPUS_DIRS>`); never hand-edit them.
- Extend the `verify-specs` recipe scope to
  `[openspec/specs/compile/spec.md, openspec/specs/packs/spec.md,
  openspec/specs/specodelic/spec.md, openspec/specs/kinds/spec.md]` ONLY when
  all four rows are executable and every block passes (properties-only
  verdict; never weaken `both_gates_required`). Prove the gate fails on a
  deliberately broken fragment and that a mutation of a new predicate fails
  (non-vacuity); restore green.

## Hard rules

- No checker re-implementation in fragments; no `true`/`Just(name)`/trivial
  scaffolds; no softened predicates; no gate extension before all four rows
  pass. If a row still cannot be honestly expressed, leave its
  `todo_predicate!`, do NOT extend the gate, and report.
- Do NOT modify `openspec/specs/compile/spec.md`, `specodelic.md`'s existing
  rules (a genuine format gap is STOP-and-report), `src/law.rs`, or any other
  ticket's files.

## Scope guard

Allowed: `src/lint/**`, `src/spec.rs` (only if the revision-pair seam needs a
parser addition), `src/lib.rs` (fixtures additions), `openspec/specs/kinds/spec.md`
+ `openspec/specs/specodelic/spec.md` (the four property cells and, if the
design says so, new rule documentation; Notes only otherwise),
`openspec/specs/linter-schema_shape/spec.md` (new rule documentation),
`justfile` (verify-specs scope), `tests/**`, `specodelic/*_props.rs` /
`specodelic/*.check.json` / `specodelic/*.toml` regenerated artifacts,
`openspec/changes/<new change>/` + `just archive-change id=<id>` if you author
a change proposal, `openspec/CHANGELOG.md`/`STATUS.md` only if the change
warrants an entry.

Never edit: `openspec/specs/compile/spec.md`, other corpus files,
`.wai/resources/**`, `.wai/pipeline-runs/**`, `.beads/**`, `packs/`, `specs/`,
`pretender.toml`, `.genesis/tools.toml` churn.

## Orientation

- `export WAI_PROJECT=specodelic`; `export TMPDIR=/var/tmp/specodelic-xch1`
  (`mkdir -p` it); `bd show specodelic-3632`; `bd heartbeat specodelic-3632`
  before long phases.
- Read: the four rows + their scenarios; `kinds.md`'s shape constraints and
  `specodelic.md`'s `append_only_variants`/Reference Typing; `src/lint/rules.rs`,
  `src/lint/mod.rs` (RULE_TABLE), `src/lint/observability.rs`; `src/spec.rs`
  (row cells, `Spec`); `src/fixtures`/`src/lib.rs` `fixtures::specs`;
  `tests/specodelic_fixtures.rs`; `justfile` `verify-specs`; the lf3 run-4
  review `.wai/projects/specodelic/reviews/2026-11-13-ro5u-review-…`.
- Start your executor pipeline:
  `wai pipeline start tdd-ro5 --topic="specodelic-3632: lint+revision seams for the four blocked rows"`
  Export the printed `WAI_PIPELINE_RUN=<tdd-ro5 run id>` and advance ONLY via
  `wai pipeline next`; never hand-edit run YAMLs; never advance the
  orchestrator run
  (`epic-orchestrator-2026-11-13-specodelic-3632-lint-revision-seams-for-the-four-blocked-lf3-rows`).
- `cargo fmt` before commits; `pretender check` green; targeted tests during
  red/green only.

## Report format (end with this — the orchestrator verifies against it)

## Report

**Commits**
- `<hash>` <message> — <what it does, one line>

**Design decisions**
- row-shape rule id + field-set source of truth
- revision-diff seam signature + what it reads

**Rows**
- the four rows: <executable / still placeholder + why>

**Gates run**
- `just verify-specs` → <result incl. scope, broken-fragment + mutation proofs>
- `just ci` → <result>
- `ah check --run-tests --all-tests` → <result>
- `just lint-specs` / `openspec validate --all --strict` → <result>

**Deviations** (or "none")
- <any scope/plan deviation and why>

**Next**
- <exact next action for the orchestrator, or "ticket complete">