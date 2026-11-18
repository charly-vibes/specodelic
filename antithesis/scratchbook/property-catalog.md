---
sut_path: /var/home/sasha/para/areas/dev/gh/charly/specodelic
commit: c76e2b0c
updated: 2026-11-18
external_references:
  - path: /var/home/sasha/para/areas/dev/gh/charly/reports/hegel-eval-2026-11-16/README.md
    why: Baseline property list for specodelic (rename algebra, writer identity, merge verdicts) and the tests-only adoption boundary this catalog refines.
  - path: /var/home/sasha/para/areas/dev/gh/charly/specodelic/.wai/resources/research/2026-11-18-antithesis-skills-method-inventory.md
    why: Discovery/evaluation method, claims-are-leads discipline, and the non-goal of importing the Antithesis harness for single-crate work.
  - path: https://hegel.dev/
    why: Candidate executor; its generator and state-machine model determine which check mode each property can use.
---

# Property catalog — `specodelic` crate laws

## Scope and method

This is a **single-SUT** pass over one Rust crate. The SUT is the library, not
the generated artifacts: every property below calls a public or crate-visible
Rust seam directly. The generated `specodelic/*_props.rs` and the `spk verify`
gate are **out of scope** and stay on proptest.

The crate claims these guarantees in
`openspec/specs/rename/spec.md`, `openspec/specs/acset-writer/spec.md`,
`openspec/specs/merge/spec.md`, and `openspec/specs/compile/spec.md`. Following
`validating-claims.md`, each claim is a **property to test**, never a fact
asserted as holding. Where the implementation contains an internal check of the
same predicate, the entry is marked **gate** (near-tautological for successful
results); the value then lies in the generator reaching the failure side.

## Check modes (local equivalents — no Antithesis SDK is present)

| Mode | Meaning here |
|---|---|
| `assert` | One unconditional assertion per generated case, in-process. The `Always` equivalent. |
| `always` | The invariant is checked after every step of a generated operation sequence (rename/apply composition), so an intermediate violation cannot be averaged away. |
| `sometimes` | At least one generated case must satisfy the predicate. This is generator **non-vacuity**, not a product liveness claim: it fails when the generator cannot reach the interesting state. |
| `reachable` | A workload path must reach a code location, asserted by an instrumented run. Unused in this catalog: there is no Antithesis workload in scope, and `sometimes` covers generator reachability. |

No property uses an Antithesis SDK assertion. The catalog is the input to a
future `/antithesis properties` import or to Hegel tests in `tests/`; the
recommendation in `recommendation.md` covers the latter.

## Portfolio summary

| Slug | Cluster | Type | Priority | Check mode | Seam | Predicted at `c76e2b0c` |
|---|---|---|---|---|---|---|
| `rename-identity-noop` | rename | Safety | P1 | assert | `rename::run` | **GREEN** |
| `rename-composition-associative` | rename | Safety | P1 | always | `rename::run` | **GREEN** (link-closed corpora) |
| `rename-collision-refused` | rename | Safety | P0 | assert | `rename::run` | **GREEN** |
| `rename-kind-preserved` | rename | Safety | P0 | assert | `rename::run` → `writer::apply` | **RED** |
| `rename-write-set-realizes-target` | rename | Safety | P1 | assert | `rename::run` | **GREEN** |
| `rename-writer-parity` | rename | Safety | P2 | assert | `rename::run` vs `run_via_writer` | **INVALIDATED** (self-comparison) |
| `rename-success-reachable` | rename | Reachability | P2 | sometimes | `rename::run` | **GREEN** |
| `writer-emit-identity-byte-exact` | writer | Safety | P2 | assert | `writer::emit` | **GREEN** (near-tautological) |
| `writer-span-coherence` | writer | Safety | P1 | assert | `writer::emit` / `verify_spans` | **GREEN** |
| `writer-apply-faithful-roundtrip` | writer | Safety | P1 | assert | `writer::apply` | **GREEN** (gate) |
| `writer-untouched-bytes-preserved` | writer | Safety | P0 | assert | `writer::apply` | **GREEN** |
| `writer-span-failure-labeled` | writer | Safety | P1 | assert | `writer::emit` / `apply` | **GREEN** |
| `writer-edit-composition` | writer | Safety | P1 | always | `writer::apply` | **GREEN** |
| `merge-verdict-symmetry` | merge | Safety | P0 | assert | `merge::run` | **RED** |
| `merge-collision-correctness` | merge | Safety | P1 | assert | `merge::run` | **GREEN** |
| `merge-blast-radius-intersection` | merge | Safety | P1 | assert | `merge::run` | **GREEN** |
| `merge-modify-delete-flagged` | merge | Safety | P1 | assert | `merge::run` | **GREEN** |
| `merge-relint-gates-verdict` | merge | Safety | P1 | assert | `merge::run` | **GREEN** |
| `merge-textual-conflict-flagged` | merge | Safety | P1 | assert | `merge::run` | **GREEN** |
| `merge-needs-review-reachable` | merge | Reachability | P2 | sometimes | `merge::run` | **GREEN** |
| `compile-artifact-determinism` | compile | Safety | P1 | assert | `compile::compile_spec` | **GREEN** |
| `compile-totality` | compile | Safety | P2 | assert | `compile::compile_spec` | **GREEN** (gate) |
| `compile-id-preservation` | compile | Safety | P2 | assert | `compile::compile_spec` | **GREEN** (gate) |
| `compile-no-semantic-drift` | compile | Safety | P2 | assert | `compile::compile_spec` | **GREEN** (gate) |
| `compile-artifact-stem-collision-refusal` | compile | Safety | P2 | assert | `compile::artifact_stem_collision` | **GREEN** |

Two predicted reds: `rename-kind-preserved` (writer rewrites non-id cells) and
`merge-verdict-symmetry` (union tree is order-dependent on add/add). Both are
**source-derived predictions**, not executed failures.

## Rename

### rename-identity-noop

| | |
|---|---|
| **Type** | Safety |
| **Priority** | P1 |
| **Property** | For every parseable corpus `I` and every shape-valid id `a`, `rename::run(I, a, a)` returns `Ok` with an empty write-set and no removal, and the corpus bytes are unchanged. |
| **Check mode** | `assert` — universal over generated corpora; the identity law is an `Always`, checked per case. |
| **Seam** | `rename::run` (`src/rename.rs:65`), early return at `:88-96`. |
| **Predicted** | **GREEN**. `validate_shape` runs first; identity returns before any parse or plan. |
| **Existing instrumentation** | Partial: `identity_rename_writes_nothing` (`src/rename.rs:467-477`) is one fixed example. |
| **Open Questions** | None that change the invariant. The law is scoped to shape-valid ids because validation precedes the identity check. |

### rename-composition-associative

| | |
|---|---|
| **Type** | Safety |
| **Priority** | P1 |
| **Property** | For a link-closed corpus and collision-free fresh ids `a→b→c` of the same owner family, applying `rename(I,a,b)` then `rename(I',b,c)` yields byte-identical files to `rename(I,a,c)` (writes and removal included). |
| **Check mode** | `always` — the equation is checked after each step of the two-step sequence. |
| **Seam** | `rename::run` (`:65`), `plan_rename` (`:195`), `writer::apply` (`writer.rs:234`). |
| **Predicted** | **GREEN** on link-closed corpora. Caveat: a pre-existing *dangling* link that aliases the intermediate id can make the two-step and one-step results differ; that corpus violates the link-closed precondition. |
| **Existing instrumentation** | Missing at this generality; `writer` has a c1→c2→c3 composition test (`tests/acset_writer_apply.rs:361-384`) and `rename` has none. |
| **Open Questions** | Does the product intend composition for corpora that are not link-closed? The current verify gate accepts such inputs until a rename introduces a new resolution. (partial: mechanism identified in `plan_rename`/`verify_would_be_repo`; no spec statement) |

### rename-collision-refused

| | |
|---|---|
| **Type** | Safety |
| **Priority** | P0 |
| **Property** | If `new_id` is already defined anywhere in the parsed corpus (Intent id, or qualified `intent.row`), `rename::run` returns `Err(Collision { holder, .. })` and no write-set; the corpus is untouched. |
| **Check mode** | `assert`. |
| **Seam** | `plan_rename` collision scan (`src/rename.rs:241-264`). |
| **Predicted** | **GREEN**. Every spec's `defined_ids()` is scanned and qualified before the owner is rewritten. |
| **Existing instrumentation** | Partial: `collision_rejected` is a spec row; the in-module tests cover invalid shapes and checklists, not generated collisions. |
| **Open Questions** | Should checklist mapped ids count as id holders? They are references, not definitions, so excluding them is consistent with `defined_ids`. (resolved by inspection: checklists never define ids) |

### rename-kind-preserved

| | |
|---|---|
| **Type** | Safety |
| **Priority** | P0 |
| **Property** | A row rename changes the row's `id` cell and every id-bearing cell/bullet/link that names the local id, and **no other cell of any row**. In particular, every row's `kind` cell (Constraints: `invariant`/`advisory`/`effect`; Properties: `unit`/`law`) is byte-identical before and after. |
| **Check mode** | `assert` over generated local ids, including ids equal to a kind keyword. |
| **Seam** | `writer::apply` → `local_id_repls` (`writer.rs:378-419`) → `collect_cell_repls` (`:435-457`), reached from `rename::run`. |
| **Predicted** | **RED**. `collect_cell_repls` rewrites *every column* whose parsed value equals the local id. If the local id is `invariant`, every constraint `kind` cell with value `invariant` in the owner file is rewritten to the new local id. `verify_roundtrip` counts rows (not columns) whose cells equal the id, and the before/after counts can balance, so the rename is accepted. |
| **Existing instrumentation** | Missing. Existing generators fix ids to `r{n}`/`renamed{n}` and kind cells to `invariant`, so the case is outside the distribution (`tests/acset_writer_apply.rs:409-478`). |
| **Open Questions** | Is "any cell equal to the local id" intentional (rename's any-cell rule) or should the rewrite be restricted to id-bearing columns (`id`, `traces_from`, `derives_from`, `guard`, `from`, `to`)? The spec names structured id fields; the code applies to all columns. (needs human input) |

### rename-write-set-realizes-target

| | |
|---|---|
| **Type** | Safety |
| **Priority** | P1 |
| **Property** | If `rename::run` returns `Ok`, then applying its writes/removal yields a corpus where: `new_id` is defined; no structured link or id cell still names `old_id`; every link resolves; and an Intent rename's file path is the new id's filename mapping. |
| **Check mode** | `assert`. |
| **Seam** | `rename::run` (`:83`), `verify_would_be_repo` (`:288-352`). |
| **Predicted** | **GREEN**. The verify gate reparses and checks zero dangling plus checklist resolution before returning the write-set. |
| **Existing instrumentation** | Partial: `rename_writer_parity`/snapshot tests pin bytes for fixed fixtures; no generated apply-and-reparse oracle. |
| **Open Questions** | None. This property deliberately excludes `kind` preservation, which is catalogued separately because the gate does not check it. |

### rename-writer-parity

| | |
|---|---|
| **Type** | Safety |
| **Priority** | P2 |
| **Property** | `rename::run` and `rename::run_via_writer` produce byte-identical write-sets for every corpus and rename. |
| **Check mode** | `assert` (would be), but **invalidated at HEAD**. |
| **Seam** | `src/rename.rs:65-73`; `tests/rename_writer_parity.rs`. |
| **Predicted** | **INVALIDATED — vacuous green**. `run` delegates wholesale to `run_via_writer`, so the existing proptest compares a function with itself. The independent oracle that remains is the checked-in byte snapshot (`tests/snapshots/rename_baseline/`). |
| **Existing instrumentation** | The property test exists but has no discriminating power; the snapshots still work. |
| **Open Questions** | Does the hand-wired path exist anywhere still? The fallback inside `run_via_writer` (`rewrite_text`) is not an alternative top-level driver. (resolved by inspection: no second driver) |

### rename-success-reachable

| | |
|---|---|
| **Type** | Reachability (generator non-vacuity) |
| **Priority** | P2 |
| **Property** | At least one generated corpus/rename pair is accepted with a non-empty write-set (row rename and Intent rename). |
| **Check mode** | `sometimes`. |
| **Seam** | `rename::run`. |
| **Predicted** | **GREEN** with a corpus generator that includes the editable families; **RED** with a generator that only produces unparseable or colliding cases. |
| **Existing instrumentation** | Missing as a generator-adequacy guard. |
| **Open Questions** | None. |

## Writer

### writer-emit-identity-byte-exact

| | |
|---|---|
| **Type** | Safety |
| **Priority** | P2 |
| **Property** | For every parser-accepted source `x`, `emit(x, parse(x)) == x` byte for byte, including CRLF and table padding. |
| **Check mode** | `assert`. |
| **Seam** | `writer::emit` (`writer.rs:56-60`). |
| **Predicted** | **GREEN — near-tautological.** `emit` returns `source.to_string()` and never re-serializes. The property can only fail if `verify_spans` wrongly rejects an accepted parse or a future edit changes the implementation to rebuild text. |
| **Existing instrumentation** | Existing proptest `identity_emission_exact` (`tests/acset_writer_identity.rs:143-208`). |
| **Open Questions** | None. The value of this property is as a regression alarm on `emit` becoming a serializer, not as evidence about emission. |

### writer-span-coherence

| | |
|---|---|
| **Type** | Safety |
| **Priority** | P1 |
| **Property** | Every span recorded by the parser resolves in bounds to exactly the bytes of the parsed element it denotes: frontmatter id, row id cell, every cell span, transition id/from/to, and each `[[…]]` inner text (trimmed). `emit`/`apply` never panics on a span. |
| **Check mode** | `assert`. |
| **Seam** | `writer::verify_spans` (`writer.rs:64-174`). |
| **Predicted** | **GREEN** over parser-accepted corpora; this is the substantive half of the writer's identity gate. |
| **Existing instrumentation** | Existing proptest reach it indirectly through `emit`; `tests/acset_writer_spans.rs` checks presence, not generated coherence. |
| **Open Questions** | Are there accepted shapes with a span recorded for a *trimmed* value while the raw span includes padding? By inspection the parser records the trimmed cell content span, and the identity generator's padded cases imply coherence. (partial: code + fixtures, not exhaustively measured) |

### writer-apply-faithful-roundtrip

| | |
|---|---|
| **Type** | Safety |
| **Priority** | P1 |
| **Property** | If `apply(Edit::Rename{..}, path, x, parse(x))` returns `Ok`, then reparsing the emitted text yields exactly the renamed instance: the Intent id is the new one, the local id is replaced where it stood, no stale local id survives in states/transitions/table cells, and the link-target multiset equals the expected rewrite. |
| **Check mode** | `assert`. |
| **Seam** | `writer::apply` (`:234-333`), `verify_roundtrip` (`:506-631`). |
| **Predicted** | **GREEN — gate.** `apply` performs this check before returning `Ok`; the property restates the gate for observability. It is blind to the `kind`-column corruption catalogued as `rename-kind-preserved`. |
| **Existing instrumentation** | Existing proptest `roundtrip_faithful` (`tests/acset_writer_apply.rs:409-492`), with a narrow id distribution. |
| **Open Questions** | Should the roundtrip oracle count *columns* rather than rows, so non-id cell corruption is caught? (needs human input; same decision as `rename-kind-preserved`) |

### writer-untouched-bytes-preserved

| | |
|---|---|
| **Type** | Safety |
| **Priority** | P0 |
| **Property** | For any successful edit, every byte outside the replaced spans is identical: prose, expressions, predicates, CRLF terminators, trailing whitespace, and table padding. The emitted text differs only in the cells/links the edit names. |
| **Check mode** | `assert`. |
| **Seam** | `writer::apply` splice (`:420-433`), `splice_replacements`. |
| **Predicted** | **GREEN**. The writer splices replacement strings into the source and never re-serializes. |
| **Existing instrumentation** | Partial: fixed tests `crlf_survives_edit`, `prose_survives_edit`, `padding_not_realigned` (`tests/acset_writer_apply.rs:107-183`); no generated oracle. |
| **Open Questions** | None. This is the law that makes the writer worth having. |

### writer-span-failure-labeled

| | |
|---|---|
| **Type** | Safety |
| **Priority** | P1 |
| **Property** | If the source no longer matches the parse (mutated bytes, truncated source, out-of-bounds span) `emit`/`apply` returns `Err` with label `spec.span_failure`, a detail naming the element, and a non-empty remediation; it never panics and never returns doctored text. |
| **Check mode** | `assert`. |
| **Seam** | `verify_spans` (`:64-174`), `span_failure` (`:178-188`). |
| **Predicted** | **GREEN**. Bounds are checked with `str::get`; mismatches return the labeled error. |
| **Existing instrumentation** | Fixed tests `span_failure_label_asserted_on_{disagreement,out_of_bounds}` (`tests/acset_writer_identity.rs:225-254`). |
| **Open Questions** | `apply`'s identity short-circuit returns `Ok` before `verify_spans`, so a stale parse plus a no-op edit is accepted. Is that intended? It follows `apply(id,x) == x`. (resolved by inspection: no corruption possible, but it means the no-op path is not a span gate) |

### writer-edit-composition

| | |
|---|---|
| **Type** | Safety |
| **Priority** | P1 |
| **Property** | For chained renames whose intermediate id is fresh, `apply(g, parse(apply(f, x)))` equals `apply(compose(g,f), x)` byte for byte, for local and Intent renames. |
| **Check mode** | `always` — equality after each step. |
| **Seam** | `writer::apply` (`:234`). |
| **Predicted** | **GREEN**. Replacements are span-local and the second step reparses, so offsets are refreshed; existing fixed test `edits_compose` uses c1→c2→c3. |
| **Existing instrumentation** | Partial: one fixed example (`tests/acset_writer_apply.rs:361-384`); no generated pairs. |
| **Open Questions** | Should composition be stated across owner families (Intent then row)? That is not a single edit and has no `compose` definition. (resolved by inspection: restrict to same-family renames) |

## Merge

### merge-verdict-symmetry

| | |
|---|---|
| **Type** | Safety |
| **Priority** | P0 |
| **Property** | `merge::run(base,a,b)` and `merge::run(base,b,a)` agree on verdict and on the multiset of finding kinds, after normalizing the branch label in messages. |
| **Check mode** | `assert`. |
| **Seam** | `merge::run` (`:140-186`), `union_tree` (`:310-380`), `relint_findings` (`:382-420`). |
| **Predicted** | **RED**. For an add/add of the same path with different text and different ids, `union_tree` keeps whichever branch is passed as `a` when the base has no entry. No finding kind covers add/add, so the relint result — and therefore the verdict — can differ by argument order. |
| **Existing instrumentation** | Missing. `tests/merge_flow.rs` covers collision, rename replay, modify/delete, and a clean merge, but not add/add or symmetry. |
| **Open Questions** | Should add/add be a `textual_conflict` (git marks it), or should `union_tree` refuse to pick a side? The spec is silent. (needs human input) |

### merge-collision-correctness

| | |
|---|---|
| **Type** | Safety |
| **Priority** | P1 |
| **Property** | An id defined on both tips is flagged `id_collision` iff it was absent from the base or was edited on both branches; an id inherited unchanged from the base is never flagged. |
| **Check mode** | `assert`. |
| **Seam** | `collision_findings` (`:191-221`). |
| **Predicted** | **GREEN**. The branch conditions exactly implement the stated rule. |
| **Existing instrumentation** | Fixed subprocess tests: `id_collision_across_branches_is_flagged`, clean merge (`tests/merge_flow.rs:104-145`). |
| **Open Questions** | Two branches editing an inherited id to the *same* new text are flagged as a collision ("edited on both branches to identical definitions"). This matches the constraint text but may surprise users. (resolved by inspection: spec requires identity with the ancestor) |

### merge-blast-radius-intersection

| | |
|---|---|
| **Type** | Safety |
| **Priority** | P1 |
| **Property** | If the mixed blast radii of the two branches' touched ids intersect, the report contains `blast_radius_intersection` and the verdict is at least `needs_review`; disjoint radii produce no such finding. |
| **Check mode** | `assert`. |
| **Seam** | `touched_ids` (`:93-122`), `blast_radius` (`:125-136`), `query::blast_radius` (`src/acset/query.rs:184-203`). |
| **Predicted** | **GREEN** for the canonical rename-replay scenario: a new reference on B reaches the renamed id through the forward half of the mixed closure, intersecting A's touched/deleted id. |
| **Existing instrumentation** | Partial: `textually_clean_semantic_conflict_is_flagged` (`tests/merge_flow.rs:180-219`) is one fixed scenario. |
| **Open Questions** | `touched_ids` is file-level (all ids in a changed file), not row-level. That over-approximates the radius deliberately. (resolved by inspection: documented at `:90-92`) |

### merge-modify-delete-flagged

| | |
|---|---|
| **Type** | Safety |
| **Priority** | P1 |
| **Property** | If one branch edits a base file and the other deletes it, the report contains a `textual_conflict` (so the verdict is `failed`) and the edited text remains in the union tree for relint; a delete against an untouched counterpart is not a conflict. |
| **Check mode** | `assert`. |
| **Seam** | `union_tree` (`:310-380`). |
| **Predicted** | **GREEN**. Both directions are handled symmetrically in the second deletion loop. |
| **Existing instrumentation** | Fixed tests in both directions (`tests/merge_flow.rs:220-266`). |
| **Open Questions** | None. |

### merge-relint-gates-verdict

| | |
|---|---|
| **Type** | Safety |
| **Priority** | P1 |
| **Property** | If the union tree fails to parse, fails `lint_corpus`, or has a dangling reference, the verdict is `failed` regardless of any `needs_review` findings. |
| **Check mode** | `assert`. |
| **Seam** | `relint_findings` (`:382-420`), verdict precedence (`:170-182`). |
| **Predicted** | **GREEN**. `relint_failure` is in the failed set and the union tree is always linted. |
| **Existing instrumentation** | Partial: the clean-merge test only asserts the empty-findings side. |
| **Open Questions** | The linted tree is not returned in `MergeReport`, so a caller cannot inspect what was linted. (needs human input if the API is expected to expose it) |

### merge-textual-conflict-flagged

| | |
|---|---|
| **Type** | Safety |
| **Priority** | P1 |
| **Property** | A file edited on both branches, differently from the base and differently from each other, yields a `textual_conflict` finding and a `failed` verdict. |
| **Check mode** | `assert`. |
| **Seam** | `textual_conflict_findings` (`:223-247`). |
| **Predicted** | **GREEN**. |
| **Existing instrumentation** | Covered indirectly by the modify/delete tests; no same-path both-edited test. |
| **Open Questions** | Add/add (base has no path) is excluded by the base lookup; see `merge-verdict-symmetry`. (partial: code path identified) |

### merge-needs-review-reachable

| | |
|---|---|
| **Type** | Reachability (generator non-vacuity) |
| **Priority** | P2 |
| **Property** | At least one generated triple produces a `needs_review` verdict with a `blast_radius_intersection` or `rename_replay` finding. |
| **Check mode** | `sometimes`. |
| **Seam** | `merge::run`. |
| **Predicted** | **GREEN** with a triple generator that varies file ownership and links; **RED** if the generator only produces identical or disjoint trees. |
| **Existing instrumentation** | Missing as a generator-adequacy guard. |
| **Open Questions** | None. |

## Compile

### compile-artifact-determinism

| | |
|---|---|
| **Type** | Safety |
| **Priority** | P1 |
| **Property** | For every lint-clean spec, two calls to `compile_spec` on equal inputs produce byte-identical `toml`, `props`, and `tla`, and compiling a reparsed copy of the same source produces the same bytes. |
| **Check mode** | `assert`. |
| **Seam** | `compile::compile_spec` (`:1980-2050`). |
| **Predicted** | **GREEN**. All iteration is `Vec` document order or `BTreeMap`/`BTreeSet`; no clock, randomness, or environment input. |
| **Existing instrumentation** | Missing as a repeated-call property; the generated corpus has fixed expectations. |
| **Open Questions** | Is environment-dependent determinism required for `write_artifacts_into`? Its `.rs` bytes depend on the installed `rustfmt` version, so a cross-environment byte-identity claim would need an explicit rustfmt pin or the fallback path. (partial: code path; no product statement) |

### compile-totality

| | |
|---|---|
| **Type** | Safety |
| **Priority** | P2 |
| **Property** | `compile_spec` either returns all three non-empty artifacts or a `CompileError` with a non-empty `stage` and `message`; never a partial result. |
| **Check mode** | `assert`. |
| **Seam** | `compile_spec` (`:1980-2050`), `Compiled` (`:1746-1765`). |
| **Predicted** | **GREEN — gate** for successful results: the return type has no partial variant. |
| **Existing instrumentation** | Generated `compile_completes_or_fails_cleanly` (out of scope). |
| **Open Questions** | None. |

### compile-id-preservation

| | |
|---|---|
| **Type** | Safety |
| **Priority** | P2 |
| **Property** | Every id in `spec.defined_ids()` appears unchanged in at least one artifact: TOML constraint ids or `source`, `// id:` lines in props, or the ModelIR's state/transition/emit ids. |
| **Check mode** | `assert`. |
| **Seam** | `compile_spec` id-set check (`:2016-2045`). |
| **Predicted** | **GREEN — gate**. A miss returns `Err(stage = compile_preserves_ids)`. |
| **Existing instrumentation** | Generated `id_preservation_holds` (out of scope). |
| **Open Questions** | None. |

### compile-no-semantic-drift

| | |
|---|---|
| **Type** | Safety |
| **Priority** | P2 |
| **Property** | Re-parsing the emitted TOML and re-emitting it yields byte-identical TOML. |
| **Check mode** | `assert`. |
| **Seam** | `compile_spec` (`:2000-2015`). |
| **Predicted** | **GREEN — gate**. Field order is the serde struct order; a mismatch returns `Err(stage = no_semantic_drift)`. |
| **Existing instrumentation** | Generated `roundtrip_stable` (out of scope). |
| **Open Questions** | None. |

### compile-artifact-stem-collision-refusal

| | |
|---|---|
| **Type** | Safety |
| **Priority** | P2 |
| **Property** | Two distinct files whose artifact stems collide are refused with a message naming both paths and a remediation hint; repeated collection of the same canonical path is not a collision; the result is independent of input order. |
| **Check mode** | `assert`. |
| **Seam** | `artifact_stem_collision` (`:1825-1853`), `canonical_artifact_path` (`:1821`). |
| **Predicted** | **GREEN**. A `BTreeMap` keyed by stem records the first claimant; canonical identity suppresses repeats. |
| **Existing instrumentation** | Fixed unit tests/generated properties cover both collision and repeat cases; no order-permutation property. |
| **Open Questions** | None. |

## Invalidated / excluded entries

- **`rename-writer-parity`** — catalogued above as invalidated at HEAD because
  `run` delegates to `run_via_writer`. The existing proptest is retained as a
  compile-time guard on the delegation, not as evidence of parity.
- **`compile-naturality` (`compile(rename(I)) == rename(compile(I))`)** — not
  catalogued. The rename spec states this case, but there is no
  artifact-rename operation against which to evaluate the right-hand side:
  `rename` parses Markdown specs, and generated `*.toml`/`*.tla`/`*_props.rs`
  are not specs. Testing it would first require defining that operation. This is
  a documented exclusion, not a green property. (partial: searched
  `src/rename.rs` and `src/compile.rs`; no seam)

## Assumptions and open questions at catalog level

- [ASSUMPTION] All properties run against in-memory `(path, text)` corpora; no
  filesystem is required except where a test chooses to use a tempdir.
- [ASSUMPTION] "Link-closed corpus" means every structured `[[…]]` target
  resolves. Rename properties that claim byte-level composition depend on it.
- [ASSUMPTION] Predicted reds are source-derived and unexecuted; the repo's
  build/test gates were not run for this documentation pass.
- Open: should the merge report expose the union tree (or its digest) so relint
  and symmetry can be checked directly? (needs human input)
- Open: should the rename spec's "structured id cells" be narrowed to an
  explicit column list? (needs human input; it decides `rename-kind-preserved`)

Investigation pointers: the union-tree question is examined in
`properties/merge-relint-gates-verdict.md` and
`properties/merge-verdict-symmetry.md`; the column-list question is examined in
`properties/rename-kind-preserved.md`; the excluded compile-naturality entry
was checked in `src/rename.rs` and `src/compile.rs` (no artifact-rename seam).