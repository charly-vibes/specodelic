---
tags: [pipeline-run:epic-orchestrator-2026-11-13-specodelic-1nhe-law-kind-per-case-fragment-bodies, pipeline-step:land-gates]
---

# Subagent brief: specodelic-1nhe — per-case fragment bodies for law-kind property rows

You are a pi subagent in repo `/var/home/sasha/para/areas/dev/gh/charly/specodelic`
(Rust CLI `specodelic`, alias `spk`). An orchestrator has claimed ticket
`specodelic-1nhe` and is hands-off: you own this format Revision end-to-end.
The orchestrator verifies with real gates — never claim work the diff doesn't
contain.

## Orientation (do this first)

- `export WAI_PROJECT=specodelic`; `export TMPDIR=/var/tmp/specodelic-xch1` (`mkdir -p` it).
- `bd show specodelic-1nhe` — the ticket scope + anti-goals are authoritative.
- `bd heartbeat specodelic-1nhe` before long phases (the claim lease TTL is short).
- Read in order: `openspec/specs/specodelic/spec.md` (Revisions 13, 15, 16, 20;
  `law_requires_cases`; the `format_revision` / `append_only_variants` rows),
  `openspec/specs/compile/spec.md` (`properties_to_proptest`,
  `predicate_fragment_opt_in`, `fragment_language_closed`,
  `fragment_law_rejected`, `rust_back_compat`, `mention_not_extraction`, and the
  executable property rows `law_property_compiles_required_cases`,
  `law_fragment_labeled`), `openspec/specs/linter-coverage/spec.md` (the
  `law_cases` rule), `openspec/specs/AGENTS.md` (revision discipline).
- Code: `src/spec.rs` (`law_case_labels`), `src/compile.rs` (`FRAGMENT_MARKER`,
  `fragment_positions`, `fragment_of_strict`, `validate_fragments`,
  `blocks_for_row`, `write_block`, `required_law_cases`), `src/verify.rs`
  (`blocks_from_source`), `src/lint/mod.rs` (law_cases), `src/migrate.rs`
  (`REVISIONS`, `LATEST_REVISION`), `src/guide.rs` (`FORMAT_REVISION`),
  `src/guide.md`.
- Start your executor pipeline and follow its prompts literally:
  `wai pipeline start tdd-ro5 --topic="specodelic-1nhe: law-kind per-case fragment bodies"`
  Then `export WAI_PIPELINE_RUN=<the tdd-ro5 run id it prints>` and advance
  ONLY via `wai pipeline next`. Never hand-edit any `.wai/pipeline-runs/*.yml`,
  and never advance or touch the orchestrator's epic run
  (`epic-orchestrator-2026-11-13-specodelic-1nhe-law-kind-per-case-fragment-bodies`).

## Deliverable — spec first, then RED → GREEN → REFACTOR

A format Revision (next number: **21**) that lets a `kind = "law"` Property row
carry one executable fragment body per named case, plus the tool implementation.
Spec edits land before implementation; tests are written failing first.

Design requirements (decide the exact grammar, but every point must hold):

- **Case labels remain the enumeration of record.** The machine-findable
  `**name:**` form in the predicate cell still enumerates cases; the
  identity + associativity floor is untouched. Bodies form a case-labelled map
  keyed by those labels: one body per case, no single-body-per-row semantics.
- **Labeled failures, never silent markers.** A fragment not associated with a
  case label, a duplicate body for one case, an unknown case label on a body, or
  a body on a non-law row under the new grammar are labeled extraction failures
  naming the rule and the offending case.
- **Emission.** `properties_to_proptest` emits one block per case (unchanged
  cardinality) with that case's body verbatim; a case with no body keeps the
  honest `todo_predicate!` placeholder. `verify`'s partial-pass-is-failure rule
  (`both_gates_required`) must not be weakened — never emit a passing stub.
- **Grammar consistency.** The closed tag set, fragment-position rule,
  hygiene/helper rules, and `mention_not_extraction` apply per body. Existing
  cells must compile byte-identically (`rust_back_compat`). The current
  "at most one executable-fragment marker per cell" wording must be amended for
  law rows in the same Revision — no invariant may silently contradict the new
  grammar.
- **lint/compile agreement.** Both sides must read cases through one shared
  parser (the `law_case_labels` seam in `src/spec.rs` or its successor). Trap:
  today's regex `\*\*([a-zA-Z][a-zA-Z _-]*?):\*\*` matches `**rust:**` as a case
  named `rust` — the fragment marker must not be counted as a case label. A law
  row with a body but no labels still fails `law_cases`; an empty label set
  still falls back to the identity/associativity floor.
- **Revision bookkeeping (all of it).** New `## Revision 21` heading in
  `openspec/specs/specodelic/spec.md`; `FORMAT_REVISION` =
  `"specodelic.md Revision 21"` (`src/guide.rs`); `LATEST_REVISION = 21` + a new
  `RevisionEntry` (`step: None`) in `src/migrate.rs`; `openspec/STATUS.md`
  revision log; `openspec/CHANGELOG.md` top entry; `src/guide.md` primer note if
  it describes law cases / fragments. Refresh the `AGENTS.md` managed block with
  `spk init --force` (the `doctor` block-revision check) and **revert the
  `.genesis/tools.toml` no-op churn** — never commit it.
- New constraint rows need deriving properties (`coverage`); new ids must reach
  the file's own intent (`single_root_reachable`); the four-layer shape holds.
- If you author an `openspec/changes/<id>/` proposal for this Revision, inspect
  it with `openspec validate`, and archive only via `just archive-change
  id=<id>` — never `openspec archive` directly (the companion re-attaches the
  layer sidecar). espectacular stays read-only over `openspec/`.

## Explicit non-goals

- Do NOT migrate the corpus law rows (`openspec/specs/specodelic/spec.md`
  `rename_naturality`, `openspec/specs/kinds/spec.md` `kind_shape_naturality`) —
  that is lf3's follow-up after this lands.
- Do NOT extend the `verify-specs` scope beyond `[compile, packs]`.
- No checker re-implementation in fragments; no `Just(name)`/`true`/trivial
  scaffolds; no weakening of any existing invariant to make the gate pass.

## Scope guard

Allowed: `openspec/specs/specodelic/spec.md`, `openspec/specs/compile/spec.md`,
`openspec/specs/linter-coverage/spec.md` (only if `law_cases` wording changes),
`src/spec.rs`, `src/compile.rs`, `src/lint/**`, `src/verify.rs` (only if block
metadata parsing must change), `src/migrate.rs`, `src/guide.rs`, `src/guide.md`,
`openspec/CHANGELOG.md`, `openspec/STATUS.md`, `AGENTS.md` (managed block only),
`tests/**`, regenerated `specodelic/*_props.rs` / `*.check.json`,
`openspec/changes/<new change>/`.

Never edit: other corpus files, `.wai/resources/**`, `.wai/pipeline-runs/**`,
`.beads/**`, `packs/`, `specs/`, `pretender.toml`, `.genesis/tools.toml` churn.

## Verification

- RED first: failing tests pin each acceptance point (per-case extraction,
  per-case emission, placeholder fallback, labeled failures, label/marker
  interaction, back-compat).
- Then green, then refactor; keep commit atoms clean (`cargo fmt` before each).
- Targeted tests during red/green only; full battery before reporting:
  `just ci` (includes `verify-specs`, `model-check-specs`, openspec validate,
  pretender), `ah check --run-tests --all-tests` (the `--run-tests` form selects
  by worktree diff — use `--all-tests`), `just lint-specs`, `just verify-specs`.
- Commit hygiene: `git status --short` before every commit; stage only your own
  files; never bare `git add .`.
- Do NOT push. Do NOT run `bd close` or `wai close`. Do NOT edit another run's
  YAML.

## Report format (end with this — the orchestrator verifies against it)

## Report

**Commits**
- `<hash>` <message> — <what it does, one line>

**Design of record**
- <the grammar chosen, in 2–4 lines; the Amendment wording on the retired/rewritten rows>

**Gates run**
- `just ci` → <result>
- `ah check --run-tests --all-tests` → <result>
- `just lint-specs` / `just verify-specs` → <result>
- `openspec validate --all --strict` → <result>

**Deviations** (or "none")
- <any scope/plan deviation and why>

**Next**
- <exact next action for the orchestrator, or "ticket complete">
