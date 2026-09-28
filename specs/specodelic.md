---
id: specodelic
kind: intent
statement: "THE specodelic SHALL represent a feature's intent, constraints, state model, and properties as one parseable markdown file conforming to a fixed schema."
---

# Specodelic

A feature's specification is a markdown file with YAML frontmatter, a fixed
set of tables, and wiki-link references between them. This file describes
that format's own constraints, lifecycle, and properties — it is a spec
written in the specodelic, about the specodelic.

## Constraints

| id                      | kind      | expr                                                                    | traces_to        |
|--------------------------|-----------|----------------------------------------------------------------------------|-------------------|
| frontmatter_valid        | invariant | `frontmatter has id, kind, statement; kind ∈ {intent}`                     | [[specodelic]]  |
| unique_id                | invariant | `∀ row ∈ file: unique(row.id)`                                             | [[specodelic]]  |
| total_refs               | invariant | `∀ ref ∈ file: resolves(ref)` — no dangling `[[...]]`                      | [[specodelic]]  |
| acyclic_traces           | invariant | `the traces_to/derives_from graph is a DAG`                                | [[specodelic]]  |
| guard_required           | invariant | `∀ transition: guard != null`                                              | [[specodelic]]  |
| ears_statement           | invariant | `intent.statement matches one of the 5 EARS patterns`                      | [[specodelic]]  |
| one_capability_per_row   | invariant | `no row.id joins two intents via "and"/"or"`                               | [[specodelic]]  |
| coverage                 | invariant | `∀ constraint: ∃ property. property.derives_from == constraint.id`         | [[specodelic]]  |
| law_requires_cases       | invariant | `∀ property where kind == "law": has(associativity) and has(identity)` — a floor, not a ceiling: a given law may require further named cases (e.g. unit, counit, naturality, triangle identity) in addition, listed in that property's own predicate | [[specodelic]]  |
| no_boolean_columns       | invariant | `schema defines no bool column type; states are named variants only`       | [[specodelic]]  |
| append_only_variants     | invariant | `∀ id-set S governed by 𝒦 — a variant-table's ids (chiefly States), a Constraint/Property row's own `kind` value-set, or the Reference Typing table's field set — across revisions r < r': S(r) ⊆ S(r'), grown only under a new Revision heading, never silently; an existing member's typing may narrow only in the same Revision that introduces the kind-split it depends on, and only if the narrowing invalidates nothing valid at r` | [[specodelic]]  |
| prose_untouched          | invariant | `parser never inspects rationale/description content`                      | [[specodelic]]  |
| model_present            | invariant | `[[model.state]] and [[model.transition]] sections both exist`             | [[specodelic]]  |
| no_counterexample        | invariant | `model checker (TLC/Alloy) finds no violated invariant`                    | [[specodelic]]  |
| properties_pass          | invariant | `all compiled proptest! blocks pass`                                       | [[specodelic]]  |
| id_matches_file          | invariant | `frontmatter.id == replace(stem(path), "-", ".")` — `-` maps only to the namespace dot; `_` is preserved literally within a segment. E.g. `linter-graph_shape.md` declares `id: linter.graph_shape` | [[specodelic]]  |
| ref_kind_compatible      | invariant | `∀ ref: target.kind ∈ allowed_targets(field)` — see Reference Typing below | [[specodelic]]  |
| single_root_reachable    | invariant | `∀ constraint/property/state/transition row in file: reachable(row, file's own intent row)` | [[specodelic]]  |
| every_state_used         | invariant | `∀ state: state appears as from or to in ≥ 1 transition`                   | [[specodelic]]  |
| every_transition_valid   | invariant | `∀ transition: from ∈ states and to ∈ states`                              | [[specodelic]]  |
| constraint_kind_closed   | invariant | `∀ Constraint row: row.kind ∈ {invariant, advisory, effect, extension_point}` — see [[kinds.constraint_row_shape]] | [[specodelic]]  |
| property_kind_closed     | invariant | `∀ Property row: row.kind ∈ {unit, law}` — see [[kinds.property_row_shape]] | [[specodelic]]  |
| supersedes_acyclic       | invariant | `the directed graph formed by supersedes edges alone (Constraint→Constraint, Property→Property) contains no cycle` | [[specodelic]]  |

### Reference Typing

Which `kind` a reference field may point at — every reference field here
is a [typed foreign key](theory.md#typed-foreign-keys-generating-morphisms).
`ref_kind_compatible` is this table read as a constraint.

| Field         | Appears on  | Must resolve to |
|---------------|-------------|-------------------|
| `traces_to`   | Constraint  | Intent            |
| `derives_from`| Property    | Constraint        |
| `guard`       | Transition  | Constraint, kind == `invariant` only — an `advisory` Constraint can never gate a transition, by typing, not by convention |
| `from` / `to` | Transition  | State             |
| `supersedes`  | Constraint, Property | same kind as the row it appears on (Constraint→Constraint, Property→Property) |
| `emits`       | State       | Constraint, kind == `effect` only — a state's declared Moore-machine output; optional and absent on a state with no output of its own |
| `satisfies`   | Constraint, any file | Constraint, kind == `extension_point` only — one-directional, outbound from a *consumer's* file to a contract published elsewhere, possibly in a file the consumer's author never edits or fully reads. The row carrying `satisfies` still has its own ordinary `traces_to` pointing at its own file's Intent, so `single_root_reachable` needs no carve-out: `satisfies` is an extra outbound pointer, exactly like `guard` or `emits`, not a second reachability edge |

## Model

### States
- `draft`
- `parsed`
- `linted`
- `compiled`
- `model_checked`
- `verified`

### Transitions

| id          | from          | to            | guard                                                                                              |
|-------------|---------------|---------------|-------------------------------------------------------------------------------------------------------|
| parse       | draft         | parsed        | [[specodelic.frontmatter_valid]] ∧ [[specodelic.id_matches_file]]                                     |
| lint        | parsed        | linted        | `∀ checker ∈ checker_ownership: checker.state == passed` — see Checker Ownership below                  |
| compile     | linted        | compiled      | [[specodelic.coverage]] ∧ [[specodelic.law_requires_cases]]                                          |
| model_check | compiled      | model_checked | [[specodelic.model_present]]                                                                          |
| verify      | model_checked | verified      | [[specodelic.no_counterexample]] ∧ [[specodelic.properties_pass]]                                    |

### Checker Ownership

`linted` is not one flat conjunction — it is the join point of five
independently-specified checker files, each its own instance of this same
format (a spec-of-a-linter-check, verifying a spec). This table replaces
the constraint-by-constraint guard with a reference to where each
constraint is actually enforced, and the partial order those checkers run
in.

| Checker file                    | Owns constraints                                                                 | Depends on                        |
|-----------------------------------|--------------------------------------------------------------------------------------|--------------------------------------|
| `linter-frontmatter.md`           | `frontmatter_valid`, `id_matches_file`                                               | — (first gate)                       |
| `linter-referential_integrity.md` | `unique_id`, `total_refs`, `ref_kind_compatible`                                      | `linter-frontmatter.md`              |
| `linter-graph_shape.md`           | `acyclic_traces`, `single_root_reachable`, `supersedes_acyclic` | `linter-referential_integrity.md`    |
| `linter-model_shape.md`           | `guard_required`, `model_present`, `every_state_used`, `every_transition_valid`, `no_boolean_columns` | `linter-graph_shape.md`              |
| `linter-ears_syntax.md`           | `ears_statement`, `one_capability_per_row`                                            | `linter-frontmatter.md` (parallel to the above branch) |
| `linter-schema_shape.md`          | `append_only_variants`, `prose_untouched`, `constraint_kind_closed`, `property_kind_closed` | `linter-frontmatter.md` (parallel to the referential/graph/model branch) |

`linted` is an [all-must-pass dependency gate](theory.md#limit-over-a-dependency-diagram-independent-checks-joined)
over this diagram: `lint` fires only when every terminal node reports
`passed`, not when constraints are checked in some fixed sequence; the two
branches after `linter-frontmatter.md` (referential→graph→model, and
ears-syntax / schema-shape) run independently of each other.

## Properties

| id                    | kind | derives_from                            | generator                                       | predicate                                                                                                                                          |
|------------------------|------|--------------------------------------------|----------------------------------------------------|--------------------------------------------------------------------------------------------------------------------------------------------------------|
| referential_integrity  | unit | [[specodelic.total_refs]]                | `arbitrary_spec_file()`                           | `∀ ref ∈ file: lookup(ref) ≠ None`                                                                                                                      |
| acyclicity             | unit | [[specodelic.acyclic_traces]]             | `arbitrary_spec_repo()`                           | `is_dag(traces_to_graph(repo))`                                                                                                                          |
| rename_naturality      | law  | [[specodelic.unique_id]]                  | `arbitrary_spec_repo(), arbitrary_id_rename()`    | **identity:** `rename(I, a, a) == I`  **associativity:** `rename(rename(I,a,b), b,c) == rename(I,a,c)`  **naturality:** `compile(rename(I)) == rename(compile(I))` |
| filename_convention    | unit | [[specodelic.id_matches_file]]            | `(id, mismatched_filename)` pairs                 | `check(id, filename) == failed`                                                                                                                          |
| typing_enforced        | unit | [[specodelic.ref_kind_compatible]]        | `guard_field_pointing_at_a_property_row()`        | `check(file) == failed`                                                                                                                                  |
| own_intent_reachable   | unit | [[specodelic.single_root_reachable]]      | `spec_repo_with(disconnected_constraint_cluster: true)` | `check(repo) == failed`                                                                                                                             |
| model_well_formed      | unit | [[specodelic.every_transition_valid]]     | `spec_file_with(transition.to_not_in_states: true)` | `check(file) == failed`                                                                                                                                |
| advisory_cannot_gate   | unit | [[specodelic.ref_kind_compatible]]        | `guard_field_pointing_at_an_advisory_constraint()` | `check(file) == failed`                                                                                                                                  |
| supersedes_cross_kind_rejected | unit | [[specodelic.ref_kind_compatible]] | `constraint_row_with(supersedes: a_property_row_id)` | `check(file) == failed`                                                                                                                                |
| supersedes_cycle_rejected | unit | [[specodelic.supersedes_acyclic]]     | `spec_repo_with(supersedes_cycle: length ≥ 2)`     | `check(repo) == failed`                                                                                                                                  |
| no_stored_superseded_flag | unit | [[specodelic.no_boolean_columns]]     | `schema_with(explicit is_superseded: bool column)` | `check(schema) == rejected` — supersession status is derived (`superseded(x) ⟺ ∃ y. y.supersedes ∋ x`), never stored                                     |
| emits_cannot_target_non_effect | unit | [[specodelic.ref_kind_compatible]] | `emits_field_pointing_at_an_invariant_constraint()` | `check(file) == failed` — same shape as `advisory_cannot_gate`, mirrored onto the other direction of the kind-split |
| satisfies_wrong_kind_rejected | unit | [[specodelic.ref_kind_compatible]] | `satisfies_field_pointing_at_an_invariant_constraint()` | `check(file) == failed` — same shape as `advisory_cannot_gate` and `emits_cannot_target_non_effect`, mirrored onto the third kind-typed reference field |
| extension_point_needs_no_reachability_carveout | unit | [[specodelic.single_root_reachable]] | `constraint_row_with(satisfies: an_extension_point_in_another_file, traces_to: own_file_intent)` | `check(row) == passed` — `satisfies` adds no reachability obligation; the row is reachable the ordinary way, through its own `traces_to` |
| frontmatter_missing_field_rejected | unit | [[specodelic.frontmatter_valid]] | `frontmatter_missing(one of id, kind, statement)` | `check(file) == failed` |
| guardless_transition_rejected | unit | [[specodelic.guard_required]] | `transition_row_with(empty guard cell)` | `check(file) == failed` |
| non_ears_statement_rejected | unit | [[specodelic.ears_statement]] | `statement_outside_the_five_ears_patterns("the system should maybe work")` | `check(file) == failed` |
| multi_intent_join_rejected | unit | [[specodelic.one_capability_per_row]] | `row_id_joined_to_two_intents_via_and()` | `check(file) == failed` |
| coverage_gap_flagged | unit | [[specodelic.coverage]] | `spec_file_with(constraint_with_no_deriving_property: true)` | `lint(file) reports rule == failed` |
| law_missing_case_rejected | unit | [[specodelic.law_requires_cases]] | `law_property_missing("associativity")` | `check(file) == failed` |
| orphan_state_rejected | unit | [[specodelic.every_state_used]] | `state_appearing_in_no_transition()` | `check(file) == failed` |
| missing_model_section_rejected | unit | [[specodelic.model_present]] | `spec_file_without_states_or_transitions()` | `check(file) == failed` |
| invariant_violation_blocks_verify | unit | [[specodelic.no_counterexample]] | `model_with_a_violated_invariant()` | `check(model_check) == failed` |
| failing_proptest_blocks_verify | unit | [[specodelic.properties_pass]] | `compiled_proptest_block_that_fails()` | `check(verify) == failed` |
| prose_does_not_affect_lint | unit | [[specodelic.prose_untouched]] | `two_specs_identical_except_rationale_prose()` | `lint(a) == lint(b)` — differing prose never changes a lint result |
| silently_shrunk_id_set_rejected | unit | [[specodelic.append_only_variants]] | `(r, r')` where a 𝒦-governed id-set member was removed with no Revision heading | `check(revisions) == failed` |
| constraint_kind_out_of_set_rejected | unit | [[specodelic.constraint_kind_closed]] | `constraint_row_with(kind: "made_up")` | `check(file) == failed` |
| property_kind_out_of_set_rejected | unit | [[specodelic.property_kind_closed]] | `property_row_with(kind: "audit")` | `check(file) == failed` |

## Notes

This file is itself a [document instance](theory.md#document-instance-functor-copresheaf)
of its own schema (`Intent`, `Constraint`, `State`, `Transition`,
`Property`). Running this file through its own `lint` transition should
produce zero findings — if it doesn't, the format and its own description
of itself disagree, the same way a self-hosting compiler that fails to
compile its own source has a bootstrapping bug rather than an ordinary
bug.

One thing the format cannot check about itself: whether `no_boolean_columns`
is *itself* satisfiable without a boolean-shaped escape hatch somewhere in
the tooling (e.g. a linter's own pass/fail result). That's outside `𝒦` —
a property of the compiler, not of any instance it compiles — flagged as
Needs Human Review rather than silently assumed.

## Revision 2

Decomposing the linter into `linter-frontmatter.md`,
`linter-referential_integrity.md`, `linter-graph_shape.md`, and
`linter-model_shape.md` surfaced four gaps in Revision 1's constraint list,
now resolved and folded in above:

- **`id_matches_file`** — resolved as a structural rule, not left open:
  a file's declared `id` must equal its filename with hyphens read as
  namespace dots (`linter-frontmatter.md` ⇒ `linter.frontmatter`). Applies
  uniformly to every spec file, since every file's frontmatter `kind` is
  `intent` regardless of whether it sits at the top of the feature tree or
  describes one narrow checker.
- **`ref_kind_compatible`** — resolved by writing out `𝒦`'s morphism
  typing explicitly (see Reference Typing table) instead of leaving it as
  an unstated assumption a checker would have had to invent.
- **`single_root_reachable`** — resolved as reachability from the file's
  *own* declared intent specifically, not any intent in the repo, since the
  looser version would silently accept a cross-feature reference filed
  under the wrong id.
- **`every_state_used` / `every_transition_valid`** — folded in as
  prerequisites for `model_present` to mean anything downstream, since a
  dangling `to` field would otherwise reach the TLA+/Alloy generator
  undetected.

The `lint` transition's guard was also restructured: the flat seven-clause
conjunction from Revision 1 was itself an instance of the God-transition
pathology `modularity-diagnostician` flags in code, just written in a
guard field instead of a function body. It's replaced with a reference to
the Checker Ownership table, so `linted` is now legible as a limit over an
explicit dependency diagram rather than an opaque AND of everything at
once — the fix is the same move as extracting a God Object into cohesive
collaborators, applied to a guard expression instead of a class.

Since this revision, `linter-ears_syntax.md`, `linter-schema_shape.md`, and
`linter-coverage.md` were all written; none surfaced further gaps in the
constraint list above (see each file's own Notes section).

## Revision 3

A Rule-of-5 review (`rule-of-5-universal`) of the full corpus found one
CRITICAL correctness bug in Revision 2's `id_matches_file` constraint, plus
documentation drift left over from before the last three checker files
existed. Both fixed here:

- **`id_matches_file` was not invertible.** The original rule ("hyphens
  read as namespace dots") assumed a filename's hyphens could be
  unambiguously mapped back to dots, but hyphens were also standing in for
  underscores within a segment (`linter-graph-shape.md` for id
  `linter.graph_shape`). `dots_to_dashes⁻¹` doesn't exist as a function
  when the forward map isn't injective. Fixed by reserving `-`
  exclusively for the namespace dot and preserving `_` literally in
  filenames; five files were renamed to match:
  `linter-referential_integrity.md`, `linter-graph_shape.md`,
  `linter-model_shape.md`, `linter-ears_syntax.md`,
  `linter-schema_shape.md`. Without this fix, the linter as specified
  would have rejected five of its own seven checker files — a
  bootstrapping failure the format's own Notes explicitly claims should
  never happen.

## Revision 4

Writing `kinds.md` — the standalone definition of `𝒦`'s five objects,
`STATUS.md` §4's P0 item — surfaced one gap: nothing in Revision 3's list
required a Constraint row's or Property row's own `kind` column to come
from a closed set. `frontmatter_valid` only ever checked the file-level
`kind == "intent"`; a Property row with `kind: "made_up"` had no
constraint anywhere rejecting it. Added `constraint_kind_closed`
(`kind ∈ {invariant}`) and `property_kind_closed` (`kind ∈ {unit, law}`),
each tracing to the corresponding row-shape constraint now specified in
`kinds.md`. Enforcement landed the same session: `linter-schema_shape.md`
now owns both (see its updated Checker Ownership row above).
- **Stale "not yet written" notes removed** from the Checker Ownership
  table and this section, left over from before `linter-schema_shape.md`
  and `linter-coverage.md` existed.

## Revision 5

Checking this format against ten features a real polyglot tool ecosystem
needed (non-gating quality signals, confidence-scored checks, scenario
supersession, cross-language adapter contracts, self-reported-vs-verified
claims, versioned shared dependencies, an ad hoc "needs human review"
escape hatch, feedback/regression loops, priority-ordered fallback
installs, and multi-target compilation) found that seven of the ten were
already representable with the mechanisms above and needed no schema
change — writing them down for the record, so the next reviewer doesn't
re-open them:

- **Non-gating quality signals** and **"needs human review" as structure,
  not prose** turned out to be the same gap, and it closes by finally
  exercising `kind_field_extensible` on `Constraint.kind` the way it was
  already exercised on `Property.kind`: adding `advisory` above. An
  advisory Constraint carries no obligation on any transition, for free,
  once `guard`'s Reference Typing row is narrowed to `kind == invariant`
  (see below) — non-gating by typing, not by the checker remembering to
  ignore it.
- **Confidence-scored / statistical properties** need no new mechanism:
  a Property's `predicate` is already an unparsed boolean-valued
  expression (`prose_untouched` only forbids the parser reading rationale
  prose, not expr/predicate fields). A threshold like
  `precision(check(corpus)) ≥ 0.95` is an ordinary unit Property with a
  fixed-corpus generator; the scoring algorithm behind the threshold is
  the feature's own concern, not the format's.
- **Feedback/regression loops** and **priority-ordered fallback steps**
  need no new mechanism either: States/Transitions is an open table, not
  a closed enum, so `revise: verified → draft` or a guard expressing
  "try A, else B, else C" via mutually-exclusive guard clauses both fit
  the existing Model section without touching `𝒦`.
- **Cross-language adapter/plugin contracts** don't need a sixth object
  in `𝒦` — they need their own file, the same way `compile.md`,
  `model_check.md`, and `verify.md` already got their own files instead
  of growing `specodelic.md`'s Model section.
- **Self-reported-vs-verified claims** and **versioned shared
  dependencies** both turned out to be downstream of the one real
  structural gap below, once supersession makes a row's provenance and
  lineage traceable instead of overwritten.

One gap needed an actual addition, generalized rather than special-cased:

- **`supersedes`.** `append_only_variants` already guarantees a
  superseded row is never deleted — but nothing let a *newer* row declare
  what it replaces, so retirement had to be said in prose or not at all.
  Rather than add a bespoke "supersession" concept, `supersedes` is
  specified as one more row in the Reference Typing table — structurally
  identical to `traces_to`/`derives_from`, just self-typed (Constraint→
  Constraint, Property→Property). Its cycle-freedom is checked as its own
  `supersedes_acyclic` invariant rather than folded into `acyclic_traces`,
  for the same reason Revision 2 broke the `lint` guard into independent
  checkers instead of one larger conjunction: lineage and intent-tracing
  are different graphs answering different questions, and merging them
  would recreate the God-invariant shape Revision 2 removed. Whether a
  row is "current" is never stored — it's computed
  (`superseded(x) ⟺ ∃ y. y.supersedes ∋ x`) — so no boolean column is
  needed and `no_boolean_columns` covers it with no changes of its own.
- **`reference_field_extensible`.** Adding `supersedes` exposed that the
  Reference Typing table itself had no stated growth path, unlike the
  `kind` columns (`kind_field_extensible`, in `kinds.md`). The same
  append-only-by-Revision discipline now applies to the table as a whole,
  not just to the value sets inside it — the general rule this codifies
  is that a Revision's job is to widen `𝒦`'s typing, in either dimension,
  never to leave a second unwritten one beside the first.

Enforcement cost was uneven, and it's worth recording why: `linter-
referential_integrity.md`'s `ref_kind_compatible` needed **zero code
changes** for either `supersedes` or the narrowed `guard` row, because it
was already written to read `allowed_targets(field)` from this table
rather than hardcoding it per field — the Reference Typing table is its
only input. `linter-schema_shape.md` and `linter-graph_shape.md` were not
so lucky: `reference_field_extensible` and `supersedes_acyclic` needed
genuinely new local checks (an append-only-table diff, and a second small
DAG check kept independent of `acyclic` rather than unioned into it — see
each file's own Notes), which are now implemented in both files rather
than left pending.

**Self-correction:** `reference_field_extensible`'s first draft in this
Revision forbade retyping an existing reference field at all — which its
own paragraph above immediately violates, by narrowing `guard` from any
Constraint to `kind == invariant` only. The rule above is the corrected
version: a narrowing is permitted in the same Revision that introduces
the kind-split it depends on, and only when it invalidates nothing that
was previously valid (true here, since no `advisory` Constraint — and
therefore no guard pointing at one — could have existed before this
Revision introduced the kind itself).

## Revision 6

Checked this format against a second, unrelated domain — a pure-Python,
lazy, pipe-composed data library whose operations are grounded in category
theory (an adjoint triple, a closed expression grammar, Moore-machine
reducers). Unlike Revision 5, most of what looked missing here turned out
to already be one Revision-5 pattern applied a second time, or an existing
mechanism nobody had pointed at yet. Following this file's and
`AGENTS.md`'s own instruction to fold gaps back the same session rather
than let them accumulate, both are done here, and the general workflow
is now written up on its own in **`USAGE.md`** — read that file before
writing a *domain* spec (one describing a piece of software, not this
repo's own linter), since everything below assumes its vocabulary
(sealed enumeration, staged lifecycle, conformance-by-limit).

**Simplified, not extended — three rules become one.** `kind_field_extensible`
(`kinds.md`), `reference_field_extensible` (Revision 5, above), and the old
narrow wording of `append_only_variants` were the same rule discovered
three times: *an id-set governed by `𝒦` only grows, and only under a new
Revision heading.* `append_only_variants` above now states that once, for
all three cases (variant-table ids, a `kind` column's value-set, and the
Reference Typing table's field set); `reference_field_extensible` is
retired as a separate row, and `kinds.md`'s `kind_field_extensible` is
reworded to cite this constraint rather than restate its logic (see that
file's own Revision). This is the same move Revision 2 made collapsing a
flat `lint` guard into the Checker Ownership table: one stated rule with
several instances, not several hand-maintained copies that can drift out
of sync with each other — which they had (Revision 5's version of this
rule already didn't match `kind_field_extensible`'s wording word-for-word,
despite saying the same thing).

**One real addition: `emits`, giving `State` an output.** A Moore machine
is states, transitions, *and* an output function of state — the third part
had nowhere to live, since `kinds.md`'s `state_row_shape` fixed State to
exactly `{id}`. Rather than add a sixth `𝒦` object, this reuses exactly
the move Revision 5 validated on `guard`: kind-split `Constraint` again
(`{invariant, advisory, effect}` — one more `kind_field_extensible`-governed
widening, now expressed via `append_only_variants`), then add one row to
Reference Typing: `emits | State | Constraint, kind == effect only`. The
new kind lives on `Constraint`, not on `State` or `Transition`, because a
State's output is a claim *about a value*, the same shape every other
Constraint already makes — it needs no new row shape, only a new place a
State may point at one. `emits` is optional: a state with no declared
output is an ordinary automaton state, not a Moore state, and both are
valid. `linter-referential_integrity.md`'s `ref_kind_compatible` needed
**zero code changes** for this either, for the same reason Revision 5
noted for `supersedes` — except its *wording* needed a fix genuinely found
this session: it named `traces_to`/`derives_from`/`guard` explicitly,
which was already stale (it didn't mention `supersedes`, added last
Revision) and would have gone stale again with `emits`. Reworded there to
read the Reference Typing table generically, matching what the check
already does in practice.

**Confirmed representable with no schema change, written up in `USAGE.md`
as patterns rather than repeated here:** a spec author's own closed,
exhaustively-matched enumeration (a "sealed AST") is a Constraints-table
id-set under the same generalized `append_only_variants` rule — it belongs
there, not in Model/States, because it's a closed *value* shape, not a
*lifecycle*; a build-then-execute staging (laziness) is two ordinary
guarded states, the same shape `compile.md`'s `extracting → emitting`
already uses; conformance across multiple interchangeable implementations
is the Checker Ownership pattern read backwards — N files each owning a
slice of one shared Intent, joined as a limit — already fully built and
running on this repo, just not yet named as a reusable pattern anywhere;
and `law_requires_cases`'s two required cases were already a stated floor,
now said explicitly in its own row above, so an adjunction or functor law
needing unit/counit/naturality/triangle-identity cases adds them to its
own property rather than waiting on a schema change.

**Deliberately left alone:** `expr`/`generator`/`predicate` remain
unparsed strings. Revision 5 already noted this in passing for confidence
thresholds; it's restated here as a considered boundary, not an oversight
— turning categorical laws into something a checker executes rather than
a human reads is a different tool (a theorem prover), not a wider lint
schema, and is out of scope for the same reason `no_prose_field_parsed`'s
audit-vs-generator mismatch is flagged `Needs Human Review` rather than
forced into a shape it doesn't fit.

## Revision 7

Checked this format against a third kind of gap: Julia-style multiple
dispatch and, more sharply, Clojure-style protocols/multimethods, where a
*consumer* extends a generic function or interface for their own type
from a file the original author never edits and may never read — the
expression-problem pattern, and the concrete mechanism behind the Open/
Closed Principle (closed for modification, open for extension). Every
existing pattern in `USAGE.md` §2 assumes one file eventually enumerates
its whole id-set (`append_only_variants` grows a set *inside the file that
owns it*); an open-world conformer breaks that assumption structurally,
not by omission — the same honest boundedness `model_check.md` already
admits for unbounded recursive structures, here across *files* instead of
*depth*.

**Scope deliberately narrowed before specifying anything:** an earlier
draft of this Revision added a two-way `implements` edge and a
`single_root_reachable` carve-out so the origin file could enumerate its
own conformers. Discarded — verifying *unknown, not-yet-written* code was
never a goal this format's own boundedness rules could honestly support,
and every previous Revision that tried to overreach past what's checkable
(`no_prose_field_parsed`'s parser-audit case, the categorical-law
theorem-proving boundary) ended up walked back to a narrower, honest
claim. The kept scope: this format publishes a **contract an extension
must satisfy**, typed and lint-checkable as a well-formed statement; it
never claims to verify who or what actually implements it. That's a
communication artifact, not an enforcement one — the same distinction
`no_boolean_columns`'s Notes already draws between what `𝒦` can check
about itself and what it can't.

**One new Constraint kind, one new one-directional reference field, both
minimal:**

- **`extension_point`** — `constraint_kind_closed` widens from
  `{invariant, advisory, effect}` to `{invariant, advisory, effect,
  extension_point}`, one more `kind_field_extensible`-governed widening
  (`kinds.md` gets the matching Revision). An `extension_point` row's
  `expr` states the method signature(s) and behavioral expectation a
  conformer must provide — e.g. `∀ T conforming: symmetry(T)::SymmetryTag,
  canonical_order(T)::Vector{Int}` — the same field every other Constraint
  kind already has, just aimed outward at code the origin file's author
  doesn't write.
- **`satisfies`** — one new row in the Reference Typing table: a
  Constraint on *any* file may point `satisfies` at an `extension_point`
  Constraint on any other file, declaring "this row is my conformance to
  that published contract." Structurally identical to `traces_to` (a
  typed, outbound, one-directional pointer), deliberately *not*
  bidirectional like `supersedes` — the origin file gets no inbound edge
  and no obligation to know its conformers exist, by construction rather
  than by convention.

**Two things this addition needs *nothing* new for, checked explicitly so
the next reviewer doesn't re-open them (`AGENTS.md` #3a):**

- **`guard` already excludes `extension_point` from gating.** `guard`'s
  Reference Typing row already reads `Constraint, kind == invariant only`
  — `advisory`, `effect`, and now `extension_point` are all excluded by
  the same existing typing fact, not by a new "cannot gate" invariant.
  Exactly the `advisory_cannot_gate` precedent `AGENTS.md` #3a calls out:
  check the type system before adding a restatement.
- **`single_root_reachable` needs no carve-out.** A Constraint row
  carrying `satisfies` is still an ordinary row of its *own* file's
  Constraints table, with its own ordinary `traces_to` pointing at that
  file's own Intent — `satisfies` is an extra outbound pointer sitting
  beside `traces_to` on the same row, exactly the way `guard` and `emits`
  already coexist with it without affecting reachability. No forest, no
  second root, no change to the existing invariant at all — this is the
  concrete payoff of narrowing scope before specifying: the two-way
  `implements` draft *would* have needed a carve-out; the one-way
  `satisfies` doesn't.
- **`linter-referential_integrity.md` needs zero code changes**, the same
  claim Revision 5 made for `supersedes` and Revision 6 made for `emits`,
  for the same reason: `ref_kind_compatible` already reads
  `allowed_targets(field)` generically from the Reference Typing table
  rather than naming fields by hand (Revision 6 fixed the one place its
  *wording* still hadn't caught up to that). A fourth typed field is not a
  fourth special case there.

**What this deliberately still doesn't do:** there is no way to ask
"do all current and future conformers satisfy this contract" as a single
checkable fact, and there won't be one — that question is unbounded across
files and time the same way an arbitrarily-nested expression tree is
unbounded across depth (`USAGE.md` §4), and forcing a false sense of
completeness onto it would be a worse outcome than leaving it explicitly
unclaimed. What *is* checkable, per conformer, is entirely ordinary:
`ref_kind_compatible` confirms a consumer's `satisfies` row resolves to a
real `extension_point` row of the right kind; whether that consumer's own
`expr` is actually faithful to the contract it names is the same kind of
human/test-suite judgment call every other `invariant` row already relies
on — `satisfies` adds *traceability* of intent to conform, not proof of
conformance.

**Note on an existing, narrower use of "Open/Closed" in this repo:**
`STATUS.md` §2 already lists "Switch/case rigidity (OCP violation)" as
solved by append-only variant tables — that's OCP *within* one file's own
governed id-set (a new case is an insert, not an edit to an existing
`match`). This Revision's `extension_point`/`satisfies` is OCP in the
*other*, harder sense — extension by a party who never touches the origin
file at all. Both are real and both now have a home; they shouldn't be
read as the same mechanism solving the same problem twice, which is
exactly the kind of same-token-different-guarantee confusion `AGENTS.md`
#6 already tracks (`CLAR-001` through `CLAR-003`) — flagged here rather
than silently left for a future reviewer to notice on their own.
