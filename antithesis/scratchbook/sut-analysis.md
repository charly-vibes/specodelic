---
sut_path: /var/home/sasha/para/areas/dev/gh/charly/specodelic
commit: c76e2b0c
updated: 2026-11-18
external_references:
  - path: /var/home/sasha/para/areas/dev/gh/charly/reports/hegel-eval-2026-11-16/README.md
    why: Suite evaluation and property baseline that scopes this run to tests-only Hegel adoption and names specodelic's three law clusters.
  - path: /var/home/sasha/para/areas/dev/gh/charly/specodelic/.wai/resources/research/2026-11-18-antithesis-skills-method-inventory.md
    why: Read order, method lessons ("claims are leads, not facts"), and the non-goals that keep the Antithesis harness machinery out of this single-crate pass.
  - path: https://hegel.dev/
    why: Engine under evaluation as the executor for these properties; its harness, state-machine, and generator model shape the check modes below.
---

# SUT analysis — `specodelic` (binary `spk`)

## Scope of this pass

This scratchbook covers **the crate's own laws**, at library seams, in four
clusters:

1. rename algebra (`src/rename.rs`): identity, composition, collision refusal,
   kind preservation;
2. acset writer apply / span-failure round-trip (`src/acset/writer.rs`);
3. merge verdict laws (`src/merge.rs`): symmetry, collisions, blast radius,
   modify/delete, relint gate;
4. compile determinism (`src/compile.rs`).

Explicitly **out of scope**: the generated proptest artifacts under
`specodelic/*_props.rs` and the `spk verify` gate. Those stay on proptest — the
generated artifacts are product codegen, and `verify` runs them in a scratch
crate with a hardcoded `proptest = "1"` dependency. Nothing here proposes
changing either.

## What the system is

A single Rust crate (library plus `spk` binary) that parses Markdown spec files
into a typed IR, lints and indexes them, and compiles them into TOML, TLA+, and
proptest artifacts. The four seams in scope are **pure data transformations with
no I/O** except where noted:

| Seam | Entry point | Input → output | I/O |
|---|---|---|---|
| Parser | `spec::parse_str` (`src/spec.rs:216`) | `&str` → `Spec` with byte spans | none |
| Rename | `rename::run` / `rename::run_via_writer` (`src/rename.rs:65,83`) | `&[(PathBuf, String)]`, old, new → `RenameOutcome` (write-set + optional removal) | none |
| Writer | `acset::writer::emit` (`:56`), `apply` (`:234`) | source + parsed `Spec` + edit → text or `WriteSet` | none |
| Merge | `merge::run` (`src/merge.rs:140`) | three `(rel path, text)` trees → `MergeReport` | none |
| Compile | `compile::compile_spec` (`src/compile.rs:1980`) | `&Spec` → `Compiled { toml, props, tla, model_ir, warnings }` | none |
| Compile write | `compile::write_artifacts_into` (`:1900`) | filesystem writes + `rustfmt` subprocess | yes |

`Spec` (`src/spec.rs:99`) holds the Intent, three table families
(`constraints`, `states`, `transitions`, `properties`), and every wiki-link in a
**structured** field. Spans (`Span { start, end }`, `src/spec.rs:31`) are
recorded for the frontmatter id, row id cells, every table column's cell, state
bullet heads, and each `[[…]]` occurrence. Spans are skipped in serialization,
so `spk parse --json` is stable.

## State and concurrency model

- **No shared mutable state.** Every in-scope function takes references and
  returns new values. `RenameOutcome`/`WriteSet` are data; applying them is the
  caller's transaction (docs at `src/rename.rs:176-190` and
  `src/acset/writer.rs:196-209`).
- **No threads in the law paths.** The one thread spawn in the in-scope files is
  `compile::rustfmt_clean`'s stdin feeder (`src/compile.rs:1946-1975`), used only
  by `write_artifacts_into`, not by `compile_spec`.
- **Deterministic containers.** The in-scope modules use `Vec` for document
  order and `BTreeMap`/`BTreeSet` for indexes (`src/merge.rs:17`,
  `src/compile.rs` throughout). A grep for `HashMap`/`HashSet`/`rand`/clock
  calls in the four modules returns nothing.
- **Filesystem state exists only at the compile-write and CLI boundaries.**
  Tempdir fixtures are needed for `tests/merge_flow.rs`-style end-to-end runs,
  but the library properties can be in-memory.

This means the Antithesis fault classes (partitions, process kills, clock
skew, race interleavings) have no purchase on the in-scope laws. The useful
generator is **input shape**, not fault injection. That is the central
deployment-topology conclusion, recorded in `deployment-topology.md`.

## Detailed architecture at the seams

### Rename

`run` delegates wholesale to `run_via_writer` (`src/rename.rs:65-73`). Flow:

1. `validate_shape(new_id)` — rejects empty/whitespace/`[]|#` (`:354-366`).
2. Identity short-circuit `old_id == new_id` returns an empty write-set
   (`:88-96`). Note this runs **after** shape validation, so the identity law is
   stated only for shape-valid ids.
3. `plan_rename` (`:195-286`) parses every file the parser accepts, finds the
   unique owner (Intent id or `intent.row`), scans all defined ids for a
   collision, and enforces the row-namespace law (`new_id` must start with
   `intent.` for row renames).
4. Per file: checklists use `checklist::rewrite_text`; parsed specs use
   `writer::apply(Edit::Rename { old, new, local })`; unparsable files fall back
   to `rewrite_text` (`:373-406`), a link-only line rewrite.
5. It collects writes/removals, then `verify_would_be_repo` (`:288-352`)
   re-parses the would-be corpus and checks **dangling links** (`graph::build`)
   plus checklist referential integrity. A failure returns
   `RenameError::VerifyFailed` and produces no outcome — atomicity by
   construction.

The writer realizes the row rename's local rewrite in `local_id_repls`
(`writer.rs:378-419`), which calls `collect_cell_repls` (`:435-457`) over **every
column of every row**, not only id-bearing columns. That is the seam behind the
predicted red `rename-kind-preserved`.

### Writer

- `emit` (`writer.rs:56-60`) calls `verify_spans` and then returns
  `source.to_string()`. It never re-serializes. Therefore the identity half
  (`emit(parse(x)) == x`) is **near-tautological**: the only possible failure is
  `verify_spans` rejecting the parse, not the emission differing. The
  informative content is span coherence (does each recorded span slice exactly
  the parsed element?).
- `verify_spans` (`:64-174`) checks the frontmatter id span, every row id and
  cell span, transition id/from/to and cell spans, and link spans (inner text
  trimmed equals the parsed target). Mismatch or out-of-bounds is
  `spec.span_failure` with a non-empty remediation.
- `apply` (`:234-333`) short-circuits identity, validates shape, verifies spans,
  collects replacements from three families (wiki links, intent id, local id
  cells/bullets), rejects overlapping spans, splices, then runs
  `verify_roundtrip` (`:506-631`). `verify_roundtrip` reparses and checks intent
  realization, stale/unrealized state bullets, transitions and **row counts**
  whose cells equal the local id, plus link-target multiset equality.
- The roundtrip check counts rows, not columns. A local id equal to a keyword
  such as `invariant` or `unit` rewrites every matching `kind` cell; the before
  and after row counts can still balance, so the check does not catch it.
- Identity `apply` returns before `verify_spans`; a stale parse plus a no-op edit
  is accepted as an empty write-set. This is consistent with
  `apply(id, x) == x` but means `apply` is not a span gate on the no-op path.

### Merge

`merge::run` (`merge.rs:140-186`) loads three tips (`Tip { specs, ids, texts,
broken }`), then:

1. unparsable files → `unparsable` finding (failed);
2. `collision_findings` (`:191-221`): ids in both tips are collisions if newly
   minted (absent from base) or edited on both branches (regardless of whether
   the two edits agree);
3. `textual_conflict_findings` (`:223-247`): same path, both differ from base,
   and differ from each other;
4. `blast_radius_findings` (`:249-267`): `touched_ids` (all ids in files whose
   text changed, plus deletions) → mixed closure via
   `acset::query::blast_radius` per tip → non-empty intersection ⇒
   `needs_review`;
5. `rename_replay_findings` (`:269-308`): a branch removed an id, the other
   branch still links to it (or a child) ⇒ `needs_review`;
6. `union_tree` (`:310-380`): A wins initially; B is taken where A lacks the
   path unless A's deletion wins; edits beat untouched; **edited-on-one +
   deleted-on-other** is flagged as `textual_conflict` and the edited version
   stays for relint;
7. `relint_findings` (`:382-420`): parse + `lint::lint_corpus` + dangling on the
   union tree ⇒ `relint_failure` (failed).

Verdict precedence (`:170-182`): any of `id_collision`, `relint_failure`,
`unparsable`, `textual_conflict` ⇒ `failed`; no findings ⇒ `merged`; otherwise
(blast radius, rename replay) ⇒ `needs_review`.

**Asymmetry found by inspection:** for an add/add of the *same path* with
different text, `union_tree` is not symmetric — it keeps whichever branch is
passed as `a`, because the base has no entry for the path. No add/add finding
kind exists, so the only observable difference is the relint finding produced by
whichever text wins. This is the mechanism behind the predicted red
`merge-verdict-symmetry`.

### Compile

`compile_spec` (`compile.rs:1980-2050`) validates fragments, requires a
non-empty state set, then calls `constraints_to_toml` (`:58`), `extract_model_ir`
(`:1141`), `properties_to_proptest` (`:1301`), and `model_to_tla` (`:1618`). It
then enforces `no_semantic_drift` (TOML reparse/re-emit byte equality) and
`compile_preserves_ids` (structured id sets across TOML, `// id:` comments, and
the ModelIR) before returning. These two checks are internal gates, so their
catalogued properties are **near-tautological** for successful compiles.

Determinism sources checked by inspection:

- all document-order emission follows `Vec` order from the parser;
- all indexes (`cells`, `emits`, `constraint_exprs`, artifact-id sets, collision
  maps) are `BTreeMap`/`BTreeSet`;
- generator helpers are emitted in first-reference order over that same
  deterministic block order; duplicate fn names get deterministic numeric
  suffixes (`:1308-1319`);
- `rustfmt` is invoked only by `write_artifacts_into`, so `compile_spec` has no
  subprocess or environment input.

Predicted: `compile-artifact-determinism` **GREEN** at HEAD.

## Failure-prone areas (ranked)

1. **Any-cell rewriting in the writer's local rename** — rewrites `kind`
   (and any other) columns whose value equals the local id. No generator in the
   existing suite draws such ids. Predicted corruption, not refusal.
2. **Merge add/add same-path asymmetry** — verdict depends on argument order
   when relint differs on the two candidate texts; no `add/add` finding kind.
3. **Merge verdict is only as strong as `union_tree`** — relint sees a tree the
   tool does not return, so callers cannot inspect what was linted.
4. **Span fidelity is the writer's whole premise** — parser drift produces
   `spec.span_failure` (safe) or, if the roundtrip oracle is blind, silent cell
   corruption (see 1).
5. **Generator adequacy** — the existing handwritten proptests fix local ids to
   `r{n}`/`renamed{n}` and use constant `Just(...)` strategies in generated
   artifacts. A green run is evidence about those distributions only.
6. **Compile environment** — `write_artifacts_into` output depends on the
   installed `rustfmt`; `compile_spec` itself is pure. Any determinism property
   must state which seam it covers.

## Assumptions

- [ASSUMPTION] "Well-formed corpus" for rename/merge properties means: parses,
  ids unique, all structured links resolve. The rename implementation does not
  check the pre-state; the merge tool is designed for lint-clean branches.
- [ASSUMPTION] The generated `specodelic/*_props.rs` and `spk verify` remain on
  proptest; no property here requires touching them.
- [ASSUMPTION] Hegel can run library-level tests in `tests/` with only a
  dev-dependency; the crate's regular `proptest = "1"` dependency is untouched.

## Open Questions

- Should `collect_cell_repls` be restricted to id-bearing columns, or is
  rewriting every cell that equals the local id an intentional "any-cell" rule?
  The rename spec says "structured id cells"; the code says every column. This
  is the product decision behind `rename-kind-preserved`. (needs human input)
- Is an add/add same-path conflict in merge intended to be a `textual_conflict`
  (git marks it) or silently take one side? The current union tree takes `a`
  with no finding. (partial: code path identified; spec is silent)
- Does `rename_naturality`'s `compile(rename(I)) == rename(compile(I))` case
  have a defined artifact-rename operation? No such function exists in
  `src/rename.rs` or `src/compile.rs`; this is why the catalog does not include
  a compile-commutation property. (partial: searched both modules; no seam)