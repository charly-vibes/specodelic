# Subagent brief: specodelic-d8c — enforce Reference Typing (ref_kind_compatible) as a lint rule

You are a pi subagent in repo `/var/home/sasha/para/areas/dev/gh/charly/specodelic`
(Rust CLI `specodelic`, alias `spk`). An orchestrator has claimed ticket
`specodelic-d8c` and is hands-off: you own the implementation end-to-end.
The orchestrator will verify with real gates — never claim work the diff
doesn't contain.

## Orientation (do this first)

- `export WAI_PROJECT=specodelic`
- Read: `bd show specodelic-d8c`; `openspec/specs/specodelic/spec.md`
  (Reference Typing table — the normative target kinds); 
  `openspec/specs/linter-referential_integrity/spec.md` (owning checker file);
  `src/lint.rs` (rule table: `rule_id` + `rule_semantics` pattern);
  `openspec/AGENTS.md` (change conventions); `just archive-change` recipe in
  the justfile.
- `wai search "ref kind"` and `wai search "reference typing"` — accumulated
  patterns first.
- Start and execute the executor pipeline literally:
  `wai pipeline start tdd-ro5 --topic="specodelic-d8c: enforce Reference Typing (ref_kind_compatible) as a lint rule"`
  Advance ONLY via `wai pipeline next` — never hand-edit the run YAML under
  `.wai/pipeline-runs/`.
- Set `export TMPDIR=/var/tmp/specodelic-xch1` (mkdir -p it) — MANDATORY for
  all cargo invocations in this repo.

## What to build

Enforce the five-field Reference Typing table as a real lint rule. Today the
table is promised by the rule catalog and `src/guide.rs`, but no rule checks
target kinds (verified on v0.5.2: guard→advisory-constraint, state
emits→invariant-constraint, constraint traces_to→constraint all lint clean).

1. **Openspec change first**: scaffold `openspec/changes/add-ref-kind-enforcement/`
   (proposal.md, design.md, tasks.md, delta on
   `linter-referential_integrity` — the checker file that owns "refs point at
   a kind-compatible target"; add a `#### Scenario:` per delta convention).
   Design decision to record in design.md: severity (error vs advisory) and
   corpus-conformance strategy (see 4).
2. **RED**: failing tests first — violation fixtures per typed field
   (guard citing a non-invariant constraint, emits from a non-failure state to
   a non-effect constraint, traces_to at kind-incompatible targets,
   derives_from at non-constraint targets, supersedes/satisfies/observes/uses
   mismatches — one fixture family per table row), asserting the labeled
   finding (`linter.ref_kind_compatible` + rule_semantics + remediation hint).
   GREEN: implement in `src/lint.rs` following the existing rule-table
   pattern; emit through `genesis::guide::Output::emit`.
3. **Corpus must stay green**: `just lint-specs` (dogfooding) is a gate.
   The ticket documents corpus rows that will violate once enforced
   (e.g. guard→advisory constraints vs invariant-only doc). Prefer
   conforming the corpus rows; widen a typing-table row ONLY with recorded
   justification in design.md (that also needs a specodelic.md delta row in
   the same change). If a corpus-lint pin test exists (precedent s8fg /
   commit 4cb4075, "corpus lint pin 47->48"), update it as data-only.
4. **Dual-format layer**: after tasks.md is fully ticked, deploy the spec
   delta ONLY via `just archive-change add-ref-kind-enforcement` — never
   hand-edit `openspec/specs/`. Author `.espectacular/` contract TOMLs for
   the change's scenarios in the archive commit (they are orphans until
   archive). Then refresh the AGENTS.md managed rule catalog with
   `spk init --force` and check `git diff AGENTS.md` is catalog-only.
5. **Gates before your report**: `just test` green; `just ci` green (full
   gate); `ah check` no NEW findings; `openspec validate --all --strict`
   clean; `just lint-specs` clean. During iteration use `just test-smart`.

## Hard scope guard

- Allowed files: `src/lint.rs`, `src/**` (rule plumbing as needed),
  `tests/**`, `openspec/changes/add-ref-kind-enforcement/**`,
  `openspec/specs/**` ONLY via the `just archive-change` recipe output,
  `AGENTS.md` ONLY via `spk init --force`, corpus `openspec/specs/<dir>/spec.md`
  rows ONLY when conforming them to the typing table (cite each in the
  change's tasks.md).
- Never edit: `.espectacular/**` except new contract TOMLs for YOUR change's
  scenarios in the archive commit, `pretender.toml`, `specs/` (domain corpus),
  `.wai/resources/**`, `.beads/hooks/**`, this brief's template, run YAMLs.
- Respect shrink-only ratchets: run `pretender check` before committing; if a
  pinned source file breaches, move new tests to `tests/` rather than raising
  entries. `src/model_check.rs` is pinned — new tests go in `tests/`.

## Repo facts (boilerplate — applies to every ticket)

- **Commit hygiene** (standing, every commit): before ANY `git commit`, run
  `git status --short` and stage only files YOU authored
  (`git add <paths>`, never bare `git add`); unstage foreign files.
  Attribute the message only to what the diff contains.
- **Output discipline**: every command emits through
  `genesis::guide::Output::emit`; errors carry a remediation hint.
- **beads**: issues live in `.beads/issues.jsonl`. Do NOT claim/close/update
  the ticket — the orchestrator owns bd state for this run.
- **Do NOT push** — the orchestrator verifies and pushes.
- **Do NOT run `wai close`** — the orchestrator owns session close.
- **ah/espectacular**: read-only over `openspec/` except the sanctioned
  `just archive-change` recipe.
- **New source files** need Purpose/Responsibilities/Rationale headers
  (file-headers skill convention).
- **Non-interactive shells**: use `-f`/`-rf`/`-y` flags; commands may be
  aliased with `-i` and will hang on prompts.

## Report format (end with this — the orchestrator verifies against it)

## Report

**Commits**
- `<hash>` <message> — <what it does, one line>

**Gates run**
- `just test` → <result>
- `ah check` → <result>
- `openspec validate --all --strict` → <result>

**Deviations** (or "none")
- <any scope/plan deviation and why>

**Next**
- <exact next action for the orchestrator, or "ticket complete">
