# specodelic — Status & Plan

*Read this file first, on its own, with no other context. It is written to
be self-sufficient: a reader who has never seen the conversation that
produced this project should be able to understand what it is, what
exists, and what to do next from this document alone.*

---

## 1. What this project is

`specodelic` is the project: a Rust CLI (`spk`) for a markdown-based
specification format for software features (and a corpus of specs, written
in that format, that specify the CLI itself — see USAGE.md §0 for this
project/format/tool/subject distinction), designed so that one file gives
you four things at once:

1. **Something a human can read** — plain prose intent, EARS-style
   requirement statements, ordinary markdown.
2. **Something a machine can lint** — because every structured field lives
   in YAML frontmatter or a markdown table with a fixed column schema, not
   free text.
3. **Something you can refactor automatically** — every id that's
   *defined* appears once, in a table cell; every place that id is *used*
   appears as a `[[wiki-link]]`. A rename tool changes the definition and
   find/replaces every link — safely, because links and definitions are
   syntactically distinguishable.
4. **Something you can simulate before writing code** — the `Model`
   section of a spec is a state machine (states + guarded transitions)
   that compiles to TLA+, so a model checker backend (stateright by
   default, TLC opt-in — see `model_check.md`) can find
   deadlocks, unreachable states, or violated invariants before any
   implementation exists. The same guards compile to property-based test
   (`proptest!`-style) generators, so verification and PBT come from the
   *same* source data instead of being written twice.

### The four layers, present in every spec file

| Layer | Where it lives in the file | Purpose |
|---|---|---|
| **Intent** | YAML frontmatter (`id`, `kind`, `statement`) + opening prose | Human-readable purpose; `statement` must match one of the 5 EARS patterns |
| **Constraints** | A `## Constraints` table: `id \| kind \| expr \| traces_to` | Invariants, formalized enough to check, each tracing to an Intent |
| **Model** | `## Model` — `### States` (a list) and `### Transitions` (a table: `id \| from \| to \| guard`) | A finite state machine; guards reference Constraints by file-qualified `[[wiki-link]]` (`[[<file-id>.<row-id>]]` — bare ids do not resolve) |
| **Properties** | A `## Properties` table: `id \| kind \| derives_from \| generator \| predicate` | PBT-style checks, one `kind = "law"` variant that requires associativity/identity cases |

### The core design decisions and why

- **One feature = one markdown file.** Keeps the reference graph
  acyclic-by-construction at the file level and matches this ecosystem's
  existing convention (one tool = one repo).
- **`[[wiki-link]]` references, not prose references.** This is what makes
  rename-refactoring mechanical: find every `[[old_id]]`, replace with
  `[[new_id]]`, done. No natural-language matching required.
- **Frontmatter + tables only are parsed; prose is never inspected.**
  The linter must never branch on the content of a `rationale` or free-text
  paragraph — only on structured fields. This keeps linting deterministic
  and keeps the format from trying to formalize writing quality (a
  separate, already-solved problem — see §2).
- **Data-oriented, Clojure-flavored bias.** Every section is the same
  underlying shape (rows of a map with an `id` and a `kind` column),
  differentiated by `kind` rather than by bespoke per-section parsers —
  one generic table-extractor and one reference-resolver serve every
  section. Ids are namespaced keys (`order.cancel.refund_bounded`), not
  nominal types. This is a deliberate echo of `clojure.spec`: validate
  plain data with predicates, don't model concepts as class hierarchies.
- **Extension is append-only.** New cases are added as new table rows,
  never edited into an existing `match`/`switch`-shaped structure. This is
  what makes certain code-smell categories *impossible to write*, not just
  linted against (see §2).

### The formal guarantees, in plain terms

- The five kinds are a **fixed schema**: every row in every spec file is
  one of exactly `Intent`, `Constraint`, `State`, `Transition`, `Property`.
- The reference fields (`traces_to`, `derives_from`, `guard`, `from`/`to`,
  `supersedes`, `emits`) are **typed foreign keys** — each must point at a
  specific kind of row, never any kind (see the Reference Typing table in
  `specodelic.md`).
- **One spec file is a document conforming to that schema** — the same
  relationship a JSON file has to its JSON Schema.
- **Linting = schema validation + referential integrity + coverage**: no
  dangling reference, and every constraint has a matching test.
- **Refactoring = safe rename**: renaming an id updates its definition and
  every reference to it, with a guarantee that nothing breaks — the same
  guarantee an IDE's "rename symbol" gives for source code.
- **Cross-file ids are qualified names**: `order.cancel.refund_bounded` is
  `file.local_id`, the same idea as a qualified name in any module system.

None of this needs anything beyond the above to *use* the format. It has a
precise mathematical restatement (categories, functors, natural
transformations, the Grothendieck construction) so the guarantees above
are provable facts rather than ad hoc prose rules — that restatement lives
in full, exactly once, in **`theory.md`**; every other file, including
this one, links there instead of repeating it.

---

## 2. Where this came from: the diagnostician mapping

The format's specific irrepresentability rules were derived by taking a
public list of code-smell "diagnosticians" (invalid-states,
composability, modularity, rigidity, mutability, error-handling,
specification-evaluation, testability-implementability) and asking, for
each: *can the pathology this flags be made ungrammatical in the schema,
rather than merely lint-flagged after the fact?* Concretely:

| Pathology | Made irrepresentable by |
|---|---|
| Boolean blindness / ad-hoc state machines | No boolean column type exists; states must be named variants |
| Unconstrained optionality | `guard` is a required, non-empty field on every transition |
| Switch/case rigidity (OCP violation, *within* one file's own governed set) | Extension points are open, append-only tables — a new case is an insert, not an edit |
| Switch/case rigidity (OCP violation, *across* files — a consumer extends your interface without ever editing it) | An `extension_point`-kind Constraint publishes the contract; a consumer's own file points `satisfies` back at it (`specodelic.md` Revision 7, `USAGE.md` §2.6) — a different mechanism from the row above, not a restatement of it |
| Side-effect entanglement | Guards (pure) and actions (effectful) are syntactically distinct row kinds; actions can't appear inside a guard expression |
| Flattened generic errors | Failure constraints must be a tagged variant, not a free-text `error` string |
| Missing algebraic laws | A `kind = "law"` property requires associativity/identity cases as a schema-level requirement, not a suggestion |
| God object / cyclic dependency | One file = one feature id; the `traces_to`/`derives_from` graph must be a DAG (checked) |

One diagnostician category — weak phrases, passive voice, vague temporals
in prose (`testability-implementability-evaluator`) — was explicitly kept
**out of scope**. That's a prose-quality judgment, and the format's own
rule is that prose is never parsed. It's a separate tool that should run
on the compiled `statement` field, not something this schema should try to
absorb.

---

## 3. Inventory — what exists right now

All files are markdown, in `specodelic`'s own format, describing
`specodelic` and its linter using themselves (a self-hosting spec, the
same way a self-hosting compiler compiles its own source).

| File | Kind / id | Describes | Status |
|---|---|---|---|
| `specodelic.md` | `specodelic` | The format itself: its own constraints, its own lifecycle (`draft → parsed → linted → compiled → model_checked → verified`), the Reference Typing table, the Checker Ownership table | **Done — Revision 14** (frontmatter kind set now `{intent, profile}`, `uses` reference field — the domain-pack mechanism, see `packs.md`) |
| `kinds.md` | `kinds` | Canonical field set and closed `kind`-column value set for each of `𝒦`'s five objects (Intent, Constraint, State, Transition, Property) | **Done — Revision 4** |
| `USAGE.md` | — | How to point the four layers at a real domain: quick-start, pattern catalog (sealed enumerations, Moore output, multi-implementation conformance, staged/lazy evaluation, extended law cases, consumer-extended contracts, event-sourced logs, empirical runtime bounds), and a migration guide from artifact-per-purpose formats (e.g. OpenSpec) | Living document |
| `theory.md` | — | Every category-theoretic claim in this repo, stated once, each paired with a plain-language restatement; every other file links here instead of restating the math locally | Done (Changelog #24) |
| `graph.md` | `graph` | Mechanically derived, queryable reference graph over every typed edge in the repo — blast-radius (transitive closure) queries, never re-derived by other tools independently | Done (Changelog #25) |
| `refactor.md` | `refactor` | Non-gating tidy-first advisor: flags a node with high, unrelated fan-in (or a changeset touching only part of a node) as a split candidate, via `graph.md` queries | Done (Changelog #25) |
| `merge.md` | `merge` | Semantic conflict detection across two divergent branches (id collisions, rename-vs-new-reference splits) that a textual git merge can't see, via `graph.md` and `rename.md` | Done (Changelog #25) |
| `compile.md` | `compile` | The `Compile` functor: `Constraints → TOML`, `Model → TLA+`, `Properties → proptest!`, plus totality/id-preservation/round-trip guarantees | Done |
| `model_check.md` | `model_check` | What actually running the model checker means: bounded, re-runnable, minimal-counterexample reporting, backend-pluggable (stateright default, TLC opt-in); clarifies `model_checked` vs `no_counterexample` | Done |
| `verify.md` | `verify` | Executes `compile.md`'s proptest! blocks and combines the result with `model_check.md`'s clean/counterexample outcome into `specodelic.md`'s `verified` gate | Done |
| `rename.md` | `rename` | The rename/refactor tool itself: a single `(old_id, new_id)` request's own lifecycle (validate → apply → verify), and the local invariants (id-availability, filename-matching, atomicity, kind-preservation, prose non-interference) that make `rename_naturality` a law of an actual process | Done |
| `linter-external_completeness.md` | `linter.external_completeness` | Optional, per-repo check that every item in a declared external checklist has an explicit `covered`/`waived` mapping to a real constraint or property, or a stated rationale — mechanizes "nothing was silently unconsulted," not "the repo is complete" | Done |
| `orchestrate.md` | `orchestrate` | The top-level orchestrator: drives `parsed → verified` in stage order, gating each stage on the prior stage's exact `specodelic.md` guard, skipping dependents of a failed checker while independent branches keep running | Done |
| `packs.md` | `packs` | The first-class domain-pack mechanism (Revision 14): `kind: profile` pack artifact with six manifest tables, corpus-scan discovery, advisory-first opt-in, `uses` edges, lifecycle + revision-skew advisories, honest-empty checking | **Done — implemented** (`src/packs.rs`; `linter.pack_shape` / `orphan_vocabulary` / `skew_advisory`) |
| `../packs/data-lineage.md` | `data.lineage` | The first standard pack (artifact, not a spec-of-record): typed `## Data` section, `data.dataset`/`data.artifact`/`data.environment` kinds, `produced_by`/`consumed_by` lineage fields, `data.lineage.closure` checker (honest-empty), `data.artifact` provenance floor, opaque `binding` (bridge-never-absorb) | **Done — shipped** (add-data-lineage-pack; the bioimage pilot consumes it via `uses`) |
| `../packs/numeric-predicates.md` | `numeric.predicates` | The second standard pack (artifact): typed `## Quantities` section, `numeric.quantity`/`numeric.bound`/`numeric.tolerance` kinds, `measured_by` outbound-leaf field, tolerance case-label floor (`bound`, `against`), opaque `unit`/`domain` (bridge-never-absorb); vocabulary prose-safe (no bare-English tokens) | **Done — shipped** (add-numeric-predicates-pack; the bioimage pilot consumes it via `uses`) |
| `linter-frontmatter.md` | `linter.frontmatter` | First gate: frontmatter has `id`/`kind`/`statement`, `id` matches filename | Done |
| `linter-referential_integrity.md` | `linter.referential_integrity` | No duplicate ids, no dangling `[[refs]]`, refs point at a kind-compatible target | Done |
| `linter-graph_shape.md` | `linter.graph_shape` | `traces_to`/`derives_from` graph is a DAG, every row reachable from its own file's Intent | Done |
| `linter-model_shape.md` | `linter.model_shape` | Every transition has a guard, states/transitions reference each other validly, no boolean columns in the model | Done |
| `linter-ears_syntax.md` | `linter.ears_syntax` | `statement` matches an EARS pattern; ids don't encode two capabilities or universal quantifiers | Done |
| `linter-schema_shape.md` | `linter.schema_shape` | Variant tables only grow across revisions; the parser never branches on prose content | Done |
| `linter-coverage.md` | `linter.coverage` | Every constraint has a deriving property; every law has its required cases | Done |
| `linter-observability.md` | `linter.observability` | Every effect Constraint is the target of ≥1 `observes` reference from a different row; unobserved effects are warned (advisory, exit 0 — never gating in Revision 1); derives external boundaries from published `extension_point` contracts | Done |
| `errors.md` | `errors` | The cross-cutting error contract: error-kind envelopes with `ok:false`, the 0/1/2 exit-code mapping, one file-id-namespaced labeled error per failing stage with a remediation hint — published as `extension_point` rows every tool file points `satisfies` at; every failure terminal emits a labeled, falsifiable error Constraint. Enforcement routed three-tiered: existing checkers today, `linter-failure_shape` when implemented, pipeline fixtures for runtime rows | Done (Changelog #66–#67) |
| `linter-failure_shape.md` | `linter.failure_shape` | The error contract's tier-2 checker: mute failure terminals, per-file error-label collisions, and typed-negation guard totals against the recorded carve-out list — all graph-decidable; spec-only (no Checker Ownership row until implemented, design D6) | Spec-only; implementation ticket filed |
| `CHANGELOG.md` | — | Append-only, chronological record of what changed, across all files | Living document |
| `AGENTS.md` | — | Standing operating instructions for any agent working in this repo | Living document |
| `STATUS.md` (this file) | — | Status, plan, and self-contained primer | Living document |

**Every checker file in `specodelic.md`'s Checker Ownership table now
exists.** `linted` (the join point of all six leaf checkers) and
`compile`'s gate (`linter.coverage`) are both fully specified end to end.

**The corpus lints clean.** As of CHANGELOG #30 every constraint in the
corpus has a deriving property: `spk lint specs` reports 18 files, zero
findings, exit 0 — the tool enforces its own description of itself
without exemptions.

**Lint findings are self-describing, and the format guide is embedded.**
Every finding carries a stable `linter.<name>` `rule_id` and a one-line
`rule_semantics` (from the same `src/lint.rs` rule table the linter emits
from), so an agent can read a violation without repo access;
`spk explain` serves the embedded primer (`spk explain lint-rules`
renders that same catalog), and `--version --json` reports
`format_revision` (`specodelic.md Revision 8`) so a consumer can detect
corpus drift against their installed binary.

### Revision log for `specodelic.md`

- **Revision 1**: initial constraint/model/property list, written before
  any checker was decomposed.
- **Revision 2**: folded in four gaps that decomposing the checkers into
  separate files surfaced by inspection (not by any mechanism yet — see
  the open problem in §4, P2): `id_matches_file`, `ref_kind_compatible` (with
  an explicit Reference Typing table), `single_root_reachable`, and the
  `every_state_used`/`every_transition_valid` pair. Also replaced the
  flat seven-clause `lint` guard with a reference to the Checker
  Ownership table, since the flat conjunction was itself an instance of
  the God-transition / God-object pathology from §2, just written in a
  guard field instead of a class.
- **Revision 3**: a Rule-of-5 review (`rule-of-5-universal`) of the full
  corpus found `id_matches_file` was CRITICAL-broken — its hyphen→dot
  reversal wasn't a well-defined function, since hyphens were also
  standing in for underscores within a segment, and applying the rule
  literally would have made the linter reject 5 of its own 7 checker
  files. Fixed by reserving `-` for the namespace dot only and preserving
  `_` literally in filenames. **Five files were renamed** as part of this
  fix (see the corrected inventory table below). Also corrected two
  documentation-drift bugs the same review found: a miscounted claim in
  `linter-coverage.md` ("seven checker files" — the Checker Ownership
  table had six then, and coverage wasn't a row in it; the table has
  eight gating rows today — see §1's reading order), and stale
  "not yet written" notes in `specodelic.md` left over from before
  `linter-schema_shape.md` and `linter-coverage.md` existed.
- **Revision 4**: writing `kinds.md` surfaced that a Constraint/Property
  row's own `kind` column had no closed set anywhere. Added
  `constraint_kind_closed` and `property_kind_closed`, both enforced by
  `linter-schema_shape.md` the same session.
- **Revision 5**: checked against ten features from an unrelated polyglot
  tool ecosystem. Seven needed no schema change (non-gating signals,
  confidence-scored properties, feedback loops, cross-language adapter
  contracts — see `specodelic.md`'s own Revision 5 for the reasoning on
  each). Two did: `advisory` added to `Constraint.kind` (non-gating by
  typing, since `guard` narrows to `kind == invariant` only), and
  `supersedes` added as a self-typed Reference Typing field so a row can
  declare what it replaces, checked as its own acyclic graph independent
  of `traces_to`/`derives_from`.
- **Revision 6**: checked against a second unrelated domain (a lazy,
  category-theoretic data library). Most of what looked missing was an
  existing pattern aimed at the wrong section — now written up on its own
  in `USAGE.md` rather than repeated per-checker. One real addition:
  `emits`, an optional field on `State` (Reference Typing: `State →
  Constraint, kind == effect`), giving a Model the output half of a Moore
  machine that had nowhere to live before. One simplification:
  `append_only_variants`, `kind_field_extensible`, and
  `reference_field_extensible` — the same "grows only, only via Revision"
  rule, discovered three times with wordings that had already drifted out
  of sync — collapsed into one statement of `append_only_variants`
  covering all three id-sets it governs. One latent wording bug fixed in
  `linter-referential_integrity.md`: `ref_kind_compatible`'s `expr` named
  three reference fields by hand and had already gone stale (missing
  `supersedes`); reworded to read the Reference Typing table generically,
  matching what the implementation was already doing.
- **Revision 14**: the extension mechanism became first-class — domain
  packs. `frontmatter_valid`'s kind set grows `{intent}` → `{intent,
  profile}` (a `profile` frontmatter marks a pack file) and the Reference
  Typing table gains `uses` (Constraint, any file → Intent of a
  `kind: profile` file, set-valued, outbound leaf). The full pack
  contract lives in `packs.md`; base closed sets freeze by policy from
  this Revision on — pack vocabulary is per-pack, never global.

---

## 4. What's NOT specced yet — prioritized

Ordered by dependency (earlier items block later ones) and by how
structurally central the gap is, not just by when it was noticed.

### Done — the compile/verify pipeline
`Compile`, `model_check`, and `verify` — `specodelic.md`'s
`linted → compiled → model_checked → verified` lifecycle — are now fully
specified: `compile.md` (the `Set^𝒦 → TOML`/`TLA+`/`proptest!` functor),
`model_check.md` (what running the checker against `compile.md`'s output
means, and the `model_checked` vs. `no_counterexample` distinction —
CLAR-003), and `verify.md` (executing `compile.md`'s proptest! blocks and
combining that with `model_check.md`'s outcome into one gate). The
`verified` verdict is now REACHABLE, not merely honest: specodelic.md
Revision 15 (specodelic-rjb, 2026-10-03) adds executable predicate
fragments — the `**rust:**` opt-in marker whose fragment compiles verbatim
into the proptest artifact and executes as a scratch-crate BFS invariant —
so a file whose predicate and invariant fragments pass verifies for real;
files without fragments behave exactly as before (honest
exploration_only / properties_failed).

Verification claims (2026-10, define-verification-claim-gates): every
opted-in invariant claim — executable `**rust:**` fragments, kernel
expressions, and whole-cell citations — must be verified before a file
verifies; prose-only invariants stay explicitly unchecked, claim reports
are versioned and bound to the structured corpus scope, and the JSON,
human, and persisted report views name the same blockers and counts.
Verification is bounded model checking of opted-in invariant claims —
not a substitute for the application's own test suite.

### Done — the rename/refactor tool
`rename_naturality` appeared as a property in three different files
(`specodelic.md`, `linter-referential_integrity.md`,
`linter-graph_shape.md`'s `topo_sort_naturality`) before the tool itself —
its own Intent, states, constraints, and properties — was written up as a
feature in its own right. `rename.md` now specifies it: a single
`(old_id, new_id)` request's own lifecycle, ending in a `verify` step that
re-runs `linter.referential_integrity` and `linter.graph_shape` before
reporting `passed`. One `Needs Human Review` item opened there: whether a
batch (whole-namespace) rename is one atomic transaction or `n`
independent ones — not decided yet.

### Done — external completeness checking
`linter-external_completeness.md` closes this. It doesn't make internal
consistency into completeness — that gap is irreducible from inside the
framework, and the file's own Notes say so plainly. What it mechanizes
instead: given a declared external checklist, every item must carry an
explicit human claim — `covered` (mapped to a real constraint/property id)
or `waived` (with a stated rationale) — so the checklist can never be
silently unconsulted, even though whether a given mapping is *semantically
correct* stays outside what any checker here can verify. It's optional per
repo and doesn't gate `linted`/`compiled`/`verified`; see its Notes for why.
The manifest format was decided (mp1 row 10, 2026-09-29): a checklist is a
dedicated `*.checklist.md` artifact OUTSIDE `𝒦` — flat `## Items` list plus
a `## Mapping` table with exactly `item`/`status`/`mapped_ids`/`rationale`
columns; presence of the file declares the checklist. All five
`linter.external_completeness` rules are implemented (specodelic-b15);
`mapping_naturality` is enforced by the rename tool (specodelic-4d5):
`spk rename` rewrites every checklist's `mapped_ids` cells (bare and
`[[…]]` spellings alike) and its verify gate re-runs
`covered_maps_resolve` over the post-rename corpus — a missed cell is
rejected before any byte is written, never a silent dangle.

### Done — orchestration
`orchestrate.md` closes this. A top-level `idle → lint_stage →
compile_stage → model_check_stage → verify_stage → succeeded/failed`
driver that calls the six Checker Ownership checkers in dependency order
(skipping a checker's dependents, not failing them, when a dependency
fails; letting independent branches run and report regardless of each
other), then `linter.coverage`, `compile.md`, `model_check.md`, and
`verify.md` in sequence, gating each stage transition on the exact guard
`specodelic.md` already specifies for it — never a looser or stricter
check invented at the orchestration layer. `linter.external_completeness`
runs when a checklist is declared but never gates any stage. Explicitly
out of scope: driving `rename` — that stays a separate, on-demand
operation a pipeline run never triggers implicitly. One `Needs Human
Review` item opened: whether the orchestrator should also own each file's
own `draft → parsed` transition, or stay scoped to `parsed → verified` as
specified.

**Every item this section had prioritized (the old P0 through P2) is now
specced.** What's left below are the lower-severity, already-flagged loose
ends — none block each other or anything above.

### Done — the reference graph, tidy-first advisor, and merge check
Three files, building on each other: `graph.md` turns the cross-file
reference graph into a derived, queryable artifact instead of something
only reconstructible by hand; `refactor.md` uses it to flag a node with
high, unrelated fan-in as a tidy-first split candidate, always non-gating
(`kind == effect`, already excluded from every `guard` by
`specodelic.md`'s existing typing — no new "never gates" invariant was
needed, see `orchestrate.md`'s Notes); `merge.md` uses it to catch what a
textual git merge can't see — id collisions and a rename left dangling by
a reference minted on a different branch — before reporting a merge
complete. Three `Needs Human Review` items opened; two resolved
2026-09-30:
- `graph.md`: whether `linter-referential_integrity.md` and
  `linter-graph_shape.md` should be refactored to query this artifact
  internally instead of independently re-deriving reachability.
- `refactor.md`: RESOLVED (HITL `specodelic-mp1` row 3): unrelated
  fan-in keys off the id namespace ("shares no namespace segment below
  root" stands as written); physical directory placement rejected.
- `merge.md`: RESOLVED (HITL `specodelic-mp1` row 2): "a human has
  explicitly approved" means an explicit, attributable, recorded
  operator decision — mechanism-agnostic floor, no gate or role
  assumed. See `merge.md`'s decision-of-record Notes paragraph.

### Deferred follow-ups of the language-neutral property binding (Rev 16)
The tag-widening change landed the closed set `{rust, py, ts}` with
`**rust:**` as the only executable tag; two follow-ups are deferred of
record and must land as their own changes when picked up:
- **`add-py-fragment-emission`** (and its `add-ts-fragment-emission`
  sibling) — the per-language emitters behind `PropertiesRunner`. The
  change that lands a first emitter MUST land the generator-vocabulary
  decision (design D5: language-neutral core vocabulary + per-project
  domain generators) in the same change, not before and not after.
  Until then every compiled strategy stays `Just(const)` — vacuous
  `properties_pass` once lf3 wires verify green (see
  `openspec/research/2026-10-04-generator-coverage-gap/synthesis.md`).
- **`specodelic-lf3`** — wire `spk verify specs` into `just ci` after
  the corpus migrates `todo_predicate!` stubs to executable fragments.

### One acknowledged loose end inside an existing file — RESOLVED
`linter-schema_shape.md`'s `no_prose_field_parsed` was flagged as not
fitting the generator/predicate shape ("verified by code audit, not by a
runnable PBT case"). Decided of record 2026-09-30 (HITL ticket
`specodelic-mp1` row 5): the claim is mechanically testable by a
planted-prose differential parse, so it will be rewritten as a runnable
`unit` property (lands with `specodelic-cxq`) and **no `audit` kind is
added** — `kinds.md` Revision 6 records the dissolution; `kind ∈ {unit,
law}` stays closed.

### One guideline, three instances (not three unrelated items)
`CLAR-001`, `CLAR-002`, and `CLAR-003` were originally filed separately;
they're the same class of problem — two names sharing a token differ in
what they actually guarantee, without a disambiguating rename — now
stated once as a standing guideline in `AGENTS.md` #6. Listed together
rather than under three headings, since fixing one likely wants the same
disambiguating-rename treatment applied to all three in one Revision:
- **CLAR-001**: the constraint id `coverage` (in `specodelic.md`) and the
  checker id `linter.coverage` (a whole file) are easy to conflate on a
  skim — plausibly what caused the "seven checker files" miscount that
  Revision 3 fixed. Consider renaming the constraint to
  `constraint_coverage`.
- **CLAR-002**: "kind" is overloaded — `𝒦`'s five objects (Intent,
  Constraint, State, Transition, Property) vs. the `kind` column that
  Constraint rows (`invariant`) and Property rows (`unit`/`law`) each
  separately carry. Not resolved yet: the fix is a column rename and
  should go through `rename_naturality` rather than be done by hand.
- **CLAR-003**: `specodelic.md`'s lifecycle state `model_checked` sounds
  like "the model was checked and holds," but its guard (`model_present`)
  only confirms a Model section exists; whether a run happened and found a
  counterexample are separate facts (`model_check.checker_invoked`,
  `specodelic.no_counterexample`), already spelled out in `model_check.md`'s
  Notes but not fixed by renaming the state.
- **EXCL-001**: a related but distinct shape (a missing `kind`, not a
  confusable name) — RESOLVED by dissolution 2026-09-30 (`specodelic-mp1`
  row 5): no `audit` kind; the single motivating claim becomes a runnable
  `unit` property instead. See `kinds.md` Revision 6 and the "One
  acknowledged loose end" section above.

---

## 5. How to resume this in a fresh session

1. Read `AGENTS.md` first — standing rules that apply regardless of which
   task you're picking up.
2. Read `specodelic.md` in full — it is the root of everything else.
3. Read `kinds.md` — the canonical definition of the five objects every
   other file in this repo instantiates.
3a. Read `USAGE.md` if the task ahead is writing a spec for a piece of
   software rather than another checker file — it has the pattern catalog
   (worked out against a real external domain) for the cases that don't
   obviously fit the four-layer shape at first glance.
4. Read `compile.md`, `model_check.md`, and `verify.md` — together they
   specify `specodelic.md`'s whole `linted → compiled → model_checked →
   verified` lifecycle.
5. Skim `CHANGELOG.md` for the chronological "what happened and why," if
   the reasoning behind a past decision isn't clear from the file itself.
6. Read the eight gating checker files (the Checker Ownership table's
   rows) in ownership order: `frontmatter → referential_integrity →
   graph_shape → model_shape → failure_shape`, plus the parallel
   branches `ears_syntax` and `schema_shape` (not sequential
   continuations), and the join leaf `coverage` (step 7 reads it with
   the tooling it feeds).
7. Read `linter-coverage.md`, then `rename.md`,
   `linter-external_completeness.md`, and `orchestrate.md` — the tooling
   layer built on top of the checker files, in that order (each depends on
   facts the ones before it establish).
8. Every item §4 once prioritized (compile/verify pipeline, rename,
   external completeness, orchestration) is now specced. Pick the next
   thing to work on from the lower-severity loose ends below
   (`CLAR-001` through `CLAR-003`, `EXCL-001`, the acknowledged
   `no_prose_field_parsed` shape mismatch, or either file's own `Needs
   Human Review` notes) unless the person names something else.
9. When writing a new spec file, follow the same self-check every prior
   file used: does every constraint trace to an intent, does every
   property derive from a constraint, does the model's guard set match
   what `linter-model_shape.md` requires? For a domain spec specifically,
   also check `USAGE.md` §2 before concluding something doesn't fit — most
   things that look unsupported are an existing pattern aimed at the wrong
   section. If a new file surfaces a genuine gap in `specodelic.md`
   itself, fold it back in as a new Revision, the way Revision 2 did,
   rather than letting gaps accumulate unaddressed across files.
