---
tags: [pipeline-step:research, pipeline-run:epic-orchestrator-2026-11-13-specodelic-lf3-scoped-verify-migration]
---

# LF3 feasibility — `packs` property rows

**Ticket:** specodelic-lf3 (scoped-first) · **run:** 2026-11-13 ·
**Method:** row-by-row classification of every Property row in
`openspec/specs/packs/spec.md` against the runner as it exists at HEAD.

## Verdict

| metric | value |
|--------|-------|
| property rows | 16 |
| expressible now | **0** |
| needs runner harness | **16** |

**No row of this file is honestly expressible today.** Every row is a
claim about the tool's own behavior (the linter, the compiler, the pack
scanner, the verify gates) or about corpus content — never a closed-form
predicate over generated values. See
[the summary](./2026-11-13-lf3-feasibility-summary.md) for the runner
measurements behind this verdict and the corpus-wide fallback sweep.

## Blockers

| code | meaning |
|------|---------|
| **T** | tool dependency — the row asserts the tool's own checker behavior; an honest executable form must CALL the tool |
| **F** | corpus/fixture access — the row asserts something about corpus content; the fragment would have to read the corpus (`std::fs` is banned by `fragment_hygiene`) or use a fixture API |
| **G** | generator vocabulary — the row needs varied generated values; today a named generator compiles to `Just(name)` — one constant input per property |
| **L** | law-case fragments — a law-kind row cannot carry a `**rust:**` fragment at all (`fragment_law_rejected`), so each named case needs its own body (a format Revision) |

## Row-by-row

| # | row id | kind | generator (fixture named) | predicate (gist) | verdict | blockers | why |
|---|--------|------|---------------------------|------------------|---------|----------|-----|
| 1 | `manifest_round_trips` | unit | `pack_file_parsed_then_manifest_read()` | `declared entries equal parsed manifest rows — no drift, no pros | needs-harness | **T** | the pack-file parser must round-trip the six manifest tables |
| 2 | `discovery_finds_all_packs` | unit | `corpus_with_n_profile_files()` | `scan reports exactly n packs, order-independent` | needs-harness | **T** | the corpus scanner must discover exactly n packs |
| 3 | `subdir_lint_discovers_workspace_packs` | unit | `lint_subdirectory_with_pack_at_workspace_root()` | `pack discovered, its vocabulary checks run, zero orphan finding | needs-harness | **T** | the linter must discover a workspace-root pack from a subdirectory |
| 4 | `no_pack_no_change` | unit | `corpus_linted_before_and_after_mechanism()` | `lint findings byte-identical for a corpus using no pack vocabul | needs-harness | **T** | lint findings must be byte-identical with and without pack vocabulary |
| 5 | `overlapping_vocabulary_activates_both` | unit | `two_packs_declare_identical_vocabulary_and_a_file_uses_ | `both packs' checkers run; findings attributed per pack` | needs-harness | **T** | both packs' checkers must run and findings must be attributed per pack |
| 6 | `orphan_finding_names_pack` | unit | `corpus_using_vocabulary_without_pack()` | `failure finding names the candidate pack and both remediations, | needs-harness | **T** | the orphan-vocabulary finding must name the candidate pack and both remediations |
| 7 | `uses_edge_typing` | unit | `file_with_uses_edge_to_profile_id()` | `graph edge typed uses resolves to the profile id; wrong-kind ta | needs-harness | **T** | the graph builder must type a `uses` edge |
| 8 | `pack_self_activation_absent` | unit | `pack_file_linted_alone()` | `its checkers report an empty checked-set; no orphan findings fr | needs-harness | **T** | the pack's own checkers must report an empty checked-set |
| 9 | `skew_is_warning_not_failure` | unit | `pack_pinned_to_older_revision()` | `exit 0 with warning-channel advisory naming the pack's base pin | needs-harness | **T** | the skew advisory must ride the warning channel with exit 0 |
| 10 | `empty_vocabulary_honest` | unit | `pack_enabled_without_instances()` | `checker reports empty checked-set; no fabricated findings` | needs-harness | **T** | the checker must report an empty checked-set, never fabricated findings |
| 11 | `narrowing_rejected` | unit | `pack_removing_previously_declared_kind()` | `pack_shape fails naming the removed entry and the append-only l | needs-harness | **T** | the pack-shape checker must fail a narrowed manifest |
| 12 | `profile_file_is_four_layer` | unit | `pack_parsed_as_standard_spec()` | `all four layers parse from the pack file alone — no sidecar art | needs-harness | **T** | the four-layer parser must parse a profile file alone |
| 13 | `pack_vocabulary_qualified` | unit | `two_packs_claim_same_mechanism()` | `both packs' declared vocabulary resolves without cross-pack col | needs-harness | **T** | two packs claiming one mechanism must resolve without collision |
| 14 | `base_sets_unchanged_by_packs` | unit | `guide_sets_compared_across_pack_enable()` | `INTENT_KINDS and Reference Typing sets identical with and witho | needs-harness | **T** | the guide's base closed sets must be identical with and without packs |
| 15 | `lifecycle_states_parsed` | unit | `pack_model_section_parsed()` | `each lifecycle state yields exactly its declared checking behav | needs-harness | **T** | the pack model section must parse into lifecycle states |
| 16 | `fiber_kinds_typeable` | unit | `base_table_kind_column_with_active_fiber_kind()` | `the base closed-set walkers accept the pack-qualified kind when | needs-harness | **T** | the base closed-set walkers must accept a pack-qualified fiber kind |

## What would unblock this file

1. **Tool-dependency seam** — the scratch crate must be able to depend on
   the `specodelic` lib (`src/verify.rs` `SCRATCH_MANIFEST` is
   `proptest`-only today), so a fragment can call `specodelic::lint::lint_corpus`
   / `specodelic::compile::*` on a fixture instead of re-implementing them.
2. **Fixture API** — the generators in this file name *fixtures*
   (`spec_file_with(transition.to_not_in_states: true)`,
   `row_belonging_to_a_sixth_section("Actions")`, …). Nothing in the tree
   turns a generator name into a concrete spec/row today.
3. **Generator vocabulary** — a named generator yields `Just(name)`
   (`src/compile.rs:1323`), i.e. exactly one input per property. Owned by
   specodelic-jwjj.

