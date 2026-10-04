# Design: add-acset-core

## Context

The `acset.zip` proposal (six intent specs: `acset.schema`, `acset.instance`,
`acset.morphism`, `acset.pushout`, `acset.query`, `acset.writer` + a
`theory.md` patch) was reviewed under a Universal Rule-of-5 pass
(2026-10-03, TypeSafe-verified findings). Verdict: **NEEDS_REVISION** —
adopt the low-risk core as phased slices; the writer and pushout are
capability changes that get their own proposals later. This change is
slice 1+2.

## Rule-of-5 findings and how this change resolves them

| id | finding (verified) | resolution here |
|----|--------------------|-----------------|
| CORR-001 (HIGH, conf 0.97) | `acset.pushout`'s `universal_property` is declared kind=invariant but quantifies over all instances — undecidable per the repo's model_check discipline | **Deferred with fix**: recorded in the pushout ticket `specodelic-9um` — re-tier as a law in Properties (or state a finite bound). Not in scope here. |
| CORR-002 (HIGH, conf 0.94) | `acset.morphism.dangling_preserved` ("an edit never resolves … a dangling reference") contradicts `acset.pushout.rename_replay_resolves_dangling` (a repair) | **Deferred with fix**: recorded in the pushout ticket `specodelic-9um` — scope `dangling_preserved` to pure rename edits and add a reconciliation note in the pushout spec. Not in scope here. |
| CORR-003 (MEDIUM) | `src/merge.rs:122-130` builds a private adjacency despite `specs/merge.md`'s "Depends on graph.md, not a local reachability walk" | **Resolved by this change**: merge's blast-radius queries migrate onto the shared closure primitive; the private map is deleted in the parity phase. |
| CLAR-001 (MEDIUM) | No CLI surface named for any acset module | **Decision**: acset-core is **internal-only** — no new subcommand, flag, or exit-code path. The writer/pushout changes must name their surfaces when proposed. |
| EDGE-001 (MEDIUM) | `acset.writer.width_padding_policy` permits misaligned tables with no lint rule to notice | Not in scope (writer deferred); noted in the writer ticket `specodelic-p5b`. |
| EDGE-002 (HIGH, conf 0.87) | `acset.pushout` gluing is unspecified for `None` (dangling) entries | **Deferred with fix**: recorded in the pushout ticket — None glues with None per morphism and reports in the dangling report; None-vs-defined is not a conflict. |
| EDGE-003 (MEDIUM) | `add-graph-views` derives the schema view from `guide`'s closed value sets; `acset.schema` introduces a second source for the same table | **Handed off**: ticket `specodelic-hya` re-points the schema view at the `Schema` value once this change lands (or adds the cross-check). |

## Key decisions

1. **Internal-only core.** The three modules (`acset_schema`, `acset_instance`,
   `acset_query` under `src/acset/`) are consumed by `graph`, `merge`, and
   `refactor`. No envelope-visible behaviour changes; error handling keeps
   the existing per-command surfaces (specs/errors.md contract applies at
   the consuming commands, unchanged).
2. **Parity gates are the migration mechanism.** Each module lands behind
   a fixture+property parity test against the walk it replaces
   (`adapter_graph_equivalent` for the builder vs `graph::build`;
   `parity_with_existing` for closures vs `graph.rs`/`merge.rs` local
   walks). The old path is deleted only after parity holds across the
   whole corpus — no flag day.
3. **Schema drift is lint-visible.** `schema_matches_typing_table` is a
   lint-time check, not a test-time one, mirroring how the format's other
   closed value sets are policed.
4. **`theory.md` patch deferred.** The fixed-schema-instances section is
   true of this change too, but it ships with the pushout change (which
   carries the pushout/category semantics) to keep this change
   tool-scoped.
5. **No GAT engine.** Deliberately fixed, finite schema (the proposal's
   own note): schema changes only under a Revision of `specs/specodelic.md`
   (append-only discipline, `specs/specodelic.md` `append_only_variants`).

## Risks

- **Parity mismatch on typing violations** — the add-graph-views proposal
  cited 38 typing violations, but the current corpus reports **0** (re-
  verified via `spk graph -j`, 2026-10-03, after orphan-labeling and
  Revision 15 landed); the builder must still reproduce violation
  classification exactly. Mitigation: a fixture corpus containing each
  violation CLASS before any code — pin classes, never a count.
- **Duplicate-id corpora** — the old path is first-wins and cannot fail
  (`or_insert`, `pub fn build -> GraphReport`); the builder must resolve
  collisions identically and surface them only as a collision report,
  or the parity gate's scope is unsatisfiable (Rule-of-5 CORR-001 on
  this proposal, fixed in the delta).
- **Duplicated edge loops collapse** (`graph.rs:345`/`:438`) — behaviour
  differences hide in annotation details. Mitigation: byte-identical
  `spk graph --json` snapshots before/after.
