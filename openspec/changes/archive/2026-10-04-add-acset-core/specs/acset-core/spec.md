# acset-core Specification

## Purpose

One place for the format's schema and one way to walk it. Today the
Reference Typing table lives as `NodeKind`/`kind_index` and match arms in
`src/graph.rs`; `src/merge.rs` builds a private adjacency map despite
`specs/merge.md` mandating reuse of `graph.md`'s blast-radius query; and
`fan_in`/`fan_out` are built in two duplicated edge loops. This capability
replaces those with a `Schema` value, a typed instance builder, and
closure primitives — with edge-for-edge parity gates so no behaviour
changes. The rigorous reading (attributed C-sets over the small fixed
category) ships with the later pushout change; nothing here needs it.

## Constraints

| id | kind | expr | traces_to |
|----|------|------|-----------|
| objects_closed | invariant | `the schema's objects are exactly {Intent, Constraint, State, Transition, Property}; a sixth object is added only under a new Revision heading of specs/specodelic.md (append-only discipline)` | [[acset.core]] |
| morphisms_typed | invariant | `every morphism has a name, exactly one source object, and exactly one target object; the pair (source, name) is unique across the schema` | [[acset.core]] |
| typing_table_is_data | invariant | `allowed_targets(m) is read from the Schema value for every morphism m — never decided by a per-field match arm; adding a reference field is adding one row` | [[acset.core]] |
| refinement_declared | invariant | `a typing rule that depends on a row's own kind column (emits targets effect Constraints only; satisfies targets extension_point only; observes targets effect only) is declared as a refinement predicate on the morphism's target, never inlined into the checker` | [[acset.core]] |
| endo_acyclicity_flagged | invariant | `an endo-morphism (supersedes; derives_from between laws) carries an explicit flag saying whether cycles through it are forbidden; acyclicity is checked generically for flagged morphisms only` | [[acset.core]] |
| schema_matches_typing_table | invariant | `when the lint target carries the format doc, the Schema value equals its Reference Typing table (specs/specodelic.md) row for row, reported as a lint finding on divergence — the document and the code cannot drift; a corpus without the format doc is out of the gate's scope (no-op, never fabricated expected rows)` | [[acset.core]] |
| canonical_order | invariant | `objects and morphisms are enumerated in a canonical sorted order, so every artifact derived from the Schema is byte-stable` | [[acset.core]] |
| ids_interned | invariant | `each object's id-set is a dense index 0..n-1 with a bidirectional map to the file-qualified string id; a duplicate file-qualified id is resolved first-wins and recorded in the builder's collision report — identical to the existing graph builder's or_insert behavior — and a lint-clean corpus carries none (no_duplicate_claim fires upstream)` | [[acset.core]] |
| morphisms_partial | invariant | `each schema morphism is stored as a vector of optional indices, None meaning an unresolved reference; the builder never drops an unresolved reference at parse time` | [[acset.core]] |
| typing_enforced_at_build | invariant | `a resolved reference whose target object or sub-kind violates the Schema's allowed_targets is recorded as a typing violation and is not stored as a morphism value — the behaviour of specs/graph.md's edge_kind_matches_typing` | [[acset.core]] |
| adapter_total | invariant | `from_specs is total on parsed corpora: every Link of every spec is stored as a morphism value, recorded as dangling, or recorded as a typing violation — none is silently dropped` | [[acset.core]] |
| adapter_graph_equivalent | invariant | `for every corpus the existing graph builder accepts, the edge set derived from the instance equals the edges of the existing graph builder, edge for edge — the parity gate that retires the old path without a flag day` | [[acset.core]] |
| build_deterministic | invariant | `index assignment follows sorted file-qualified id order; two builds of the same corpus serialize byte-identically regardless of input file order` | [[acset.core]] |
| cells_carried | invariant | `non-reference cells (expr, statement, generator, predicate, kind cells) are carried as attribute columns keyed by the same indices and are never inspected by the builder — consistent with specs/specodelic.md's prose_untouched` | [[acset.core]] |
| seeds_exist | invariant | `every seed id is an id of the instance; an unknown seed is rejected, not ignored` | [[acset.core]] |
| scope_by_morphism_set | invariant | `a query names the set M of schema morphisms to follow; restricting M to supersedes yields exactly the supersedes subgraph, on which cycle detection is closure-based (specs/specodelic.md's supersedes_acyclic)` | [[acset.core]] |
| single_traversal_primitive | invariant | `forward_closure and backward_closure over a seed set and a morphism set M are the only traversal primitives; graph, merge, and refactor are expressed through them with no private adjacency map` | [[acset.core]] |
| closure_terminates | invariant | `closure on a finite instance terminates and visits each node at most once, including when M contains a cycle` | [[acset.core]] |
| dangling_not_followed | invariant | `an unresolved reference is not an edge: traversal skips it, the dangling report lists it, and it contributes to no reachability result` | [[acset.core]] |
| blast_radius_defined | invariant | `blast_radius(I, touched) == the mixed transitive closure — from every reached node, both the morphism values are followed forward and the references backward, i.e. the fixpoint of X ↦ backward_closure(forward_closure(X)); the backward closure of the touched ids alone, plus the targets reached forward from them, is NOT the definition (a backward-reached dependent's own forward reach joins the radius) — the definition in specs/graph.md's blast_radius_is_transitive_closure, pinned by parity against merge.rs's pre-migration walk` | [[acset.core]] |
| fan_in_is_preimage_size | invariant | `fan_in(x) == the number of pairs (s, m) with the morphism value at s defined and equal to x` | [[acset.core]] |
| results_ordered | invariant | `query results are sorted by file-qualified id, so output is deterministic` | [[acset.core]] |
| parity_with_existing | invariant | `for every corpus, query-derived blast_radius, fan_in, and supersedes cycles equal the values computed by the pre-migration walks in graph.rs and merge.rs — the parity gate that lets the private adjacency maps be deleted` | [[acset.core]] |

## Model

### States
- `proposed`
- `approved`
- `implemented`
- `archived`

### Transitions

| id | from | to | guard |
|----|------|----|-------|
| approve | proposed | approved | `proposal reviewed and approved by the maintainer` |
| implement | approved | implemented | `all tasks.md items complete; parity gates hold over the corpus; just ci and openspec validate --strict pass` |
| archive | implemented | archived | `just archive-change id=add-acset-core ran with the dual-format recipe` |

## Properties

| id | kind | derives_from | generator | predicate |
|----|------|--------------|-----------|-----------|
| sixth_object_rejected | unit | [[acset.core.objects_closed]] | `schema_with_extra_object("Action")` | `check(schema) == failed` |
| duplicate_morphism_rejected | unit | [[acset.core.morphisms_typed]] | `schema_with_two_morphisms_same_source_and_name()` | `check(schema) == failed` |
| duplicate_id_first_wins_parity | unit | [[acset.core.ids_interned]] | `corpus_with_two_rows_same_qualified_id()` | `node_kinds(builder(corpus)) == node_kinds(existing kind_index(corpus))` — first-wins, and the collision report names the id |
| typing_read_from_schema | unit | [[acset.core.typing_table_is_data]] | `schema_with_one_added_reference_field()` | `typing_violation(edge_over_new_field) is decided with no change to checker code` |
| endo_cycle_detected | unit | [[acset.core.endo_acyclicity_flagged]] | `supersedes_cycle_a_b_a()` | `check(instance) == failed` |
| unflagged_endo_cycle_tolerated | unit | [[acset.core.endo_acyclicity_flagged]] | `law_derives_from_law_self_edge()` | `check(instance) == passed` — only flagged morphisms are cycle-checked |
| schema_drift_detected | unit | [[acset.core.schema_matches_typing_table]] | `schema_missing_one_row_of_the_typing_table()` | `check(schema, typing_table) == failed` |
| index_roundtrip | unit | [[acset.core.ids_interned]] | `arbitrary_corpus()` | `name_of(object, index_of(object, id)) == id` |
| emits_refinement_enforced | unit | [[acset.core.refinement_declared]] | `emits_edge_to_invariant_constraint()` | `check(edge) == failed` — the effect-only rule comes from the declared refinement |
| order_is_canonical | unit | [[acset.core.canonical_order]] | `arbitrary_schema_built_from_two_declaration_orders()` | `schemas_equal_as_values()` — enumeration order never depends on declaration order |
| dangling_is_a_value | unit | [[acset.core.morphisms_partial]] | `corpus_with_one_unresolved_link()` | `morphism_value(source) == None` and the build reports no error |
| no_link_dropped | unit | [[acset.core.adapter_total]] | `arbitrary_corpus()` | `card(stored) + card(dangling) + card(violations) == card(links)` |
| graph_parity | unit | [[acset.core.adapter_graph_equivalent]] | `arbitrary_corpus_accepted_by_graph_build()` | `edges(from_specs(c)) == the pre-migration builder's edges` |
| forbidden_edge_not_stored | unit | [[acset.core.typing_enforced_at_build]] | `traces_to_pointing_at_a_state()` | `morphism_value(source) == None` and `card(typing_violations) == 1` |
| rebuild_is_byte_stable | unit | [[acset.core.build_deterministic]] | `arbitrary_corpus(), arbitrary_file_order()` | `serialize(from_specs(shuffle(c))) == serialize(from_specs(c))` |
| attributes_untouched | unit | [[acset.core.cells_carried]] | `row_whose_expr_contains_wiki_link_syntax_in_prose()` | `attribute(row, "expr") == original_cell` |
| unknown_seed_rejected | unit | [[acset.core.seeds_exist]] | `closure_over_unknown_seed()` | `check(query) == failed` |
| supersedes_scope_exact | unit | [[acset.core.scope_by_morphism_set]] | `corpus_with_supersedes_and_traces_edges()` | `closure(M == supersedes) contains no traces edge` |
| no_private_adjacency | unit | [[acset.core.single_traversal_primitive]] | `grep_for_adjacency_map_construction()` | `no adjacency-map construction exists outside acset::query` |
| closure_on_cycle_terminates | unit | [[acset.core.closure_terminates]] | `cyclic_morphism_set()` | `closure(seed) terminates and visits each node at most once` |
| closure_laws_hold | unit | [[acset.core.single_traversal_primitive]] | `arbitrary_instance(), arbitrary_seed_set(), arbitrary_morphism_set()` | **contains:** `S ⊆ closure(S)` **idempotence:** `closure(closure(S)) == closure(S)` **monotonicity:** `S ⊆ T implies closure(S) ⊆ closure(T)` |
| dangling_not_followed_in_traversal | unit | [[acset.core.dangling_not_followed]] | `corpus_with_one_dangling_link()` | `closure(seed) skips the dangling endpoint and the dangling report lists it` |
| blast_radius_parity | unit | [[acset.core.blast_radius_defined]] | `arbitrary_corpus_with_two_branch_tips()` | `blast_radius(query) == blast_radius(pre-migration merge walk)` |
| fan_in_matches_preimage | unit | [[acset.core.fan_in_is_preimage_size]] | `corpus_with_one_shared_target()` | `fan_in(shared_target) == 2` |
| results_sorted_by_id | unit | [[acset.core.results_ordered]] | `query_over_reverse_insertion_order()` | `output == sorted(output)` |
| walk_parity_fan_and_cycles | unit | [[acset.core.parity_with_existing]] | `arbitrary_corpus()` | `closure-derived fan_in, fan_out, and supersedes cycles == the pre-migration walks' values` |

## ADDED Requirements

### Requirement: Schema-as-data typing
The acset-core SHALL hold the format's fixed finite schema — the five
objects (Intent, Constraint, State, Transition, Property) and their typed
reference morphisms — as a single `Schema` value read by one generic
checker, so the Reference Typing table of `specs/specodelic.md` is
evaluated from data instead of per-field match arms in `src/graph.rs`;
adding a reference field is adding one row.

#### Scenario: Typing read from schema
- **WHEN** a new reference field is added as one schema row
- **THEN** typing decisions for that field come from the Schema value with no change to checker code

#### Scenario: Schema drift detected at lint time
- **WHEN** the Schema value diverges from specs/specodelic.md's Reference Typing table on any row
- **THEN** lint reports the divergence naming the mismatched row

#### Scenario: Endo-acyclicity only where flagged
- **WHEN** a cycle passes through an unflagged endo-morphism such as derives_from between laws
- **THEN** it is not reported, while cycles through flagged endo-morphisms such as supersedes are reported

### Requirement: Typed instance builder
WHEN a parsed corpus is loaded, the acset-core SHALL build a typed
instance — an interned, dense id-set per schema object and a partial
function per morphism — in which an unresolved reference is a
representable value (None) that is never dropped and never a parse error.

#### Scenario: Dangling is a value
- **WHEN** a corpus contains one link that resolves to no id
- **THEN** the instance stores that morphism value as None and the build succeeds

#### Scenario: No link dropped
- **WHEN** a parsed corpus is built into an instance
- **THEN** every link is stored, recorded as dangling, or recorded as a typing violation — the three counts sum to the link count

#### Scenario: Forbidden edge not stored
- **WHEN** a corpus contains a reference whose target object is not in the morphism's allowed targets
- **THEN** the instance records a typing violation and stores no morphism value for it

#### Scenario: Graph parity edge for edge
- **WHEN** a corpus accepted by the existing graph builder is built into an instance
- **THEN** the derived edge set equals the existing builder's edges exactly

#### Scenario: Byte-stable rebuild
- **WHEN** the same corpus is built with input files in two different orders
- **THEN** both serializations are byte-identical

#### Scenario: Duplicate id parity
- **WHEN** a corpus contains two rows with the same file-qualified id (a corpus the existing builder accepts first-wins)
- **THEN** the builder resolves the same way and records the collision in its collision report — never a hard failure the old path did not emit

#### Scenario: Empty corpus builds an empty instance
- **WHEN** the input directory contains zero spec files
- **THEN** the instance is empty and well-formed, closures over it are empty, and parity with the existing builder holds

### Requirement: Single traversal primitive
The acset-core SHALL expose forward and backward closure over a seed set
and a chosen set of schema morphisms as its only traversal primitives,
through which graph fan-in, merge blast radius (`specs/merge.md`), and
the refactor advisor (`specs/refactor.md`) are expressed with no private
adjacency map — and query-derived results SHALL equal the pre-migration
walks for every corpus.

#### Scenario: Parity with existing walks
- **WHEN** closure-derived blast radius, fan-in, and supersedes cycles are computed over any corpus
- **THEN** they equal the values the pre-migration graph.rs and merge.rs walks compute

#### Scenario: Unknown seed rejected
- **WHEN** a closure is requested over a seed id that is not in the instance
- **THEN** the query fails with a remediation hint rather than ignoring the seed

#### Scenario: Dangling not followed
- **WHEN** traversal reaches an unresolved reference
- **THEN** it is skipped for reachability, listed in the dangling report, and contributes to no result

#### Scenario: Supersedes-scoped cycles
- **WHEN** M is restricted to supersedes and a cycle exists in the supersedes subgraph
- **THEN** closure-based cycle detection over that scope reports it, matching the existing supersedes_acyclic behaviour

## Requirements

### Requirement: Schema-as-data typing
The acset-core SHALL hold the format's fixed finite schema — the five
objects (Intent, Constraint, State, Transition, Property) and their typed
reference morphisms — as a single `Schema` value read by one generic
checker, so the Reference Typing table of `specs/specodelic.md` is
evaluated from data instead of per-field match arms in `src/graph.rs`;
adding a reference field is adding one row.

#### Scenario: Typing read from schema
- **WHEN** a new reference field is added as one schema row
- **THEN** typing decisions for that field come from the Schema value with no change to checker code

#### Scenario: Schema drift detected at lint time
- **WHEN** the Schema value diverges from specs/specodelic.md's Reference Typing table on any row
- **THEN** lint reports the divergence naming the mismatched row

#### Scenario: Endo-acyclicity only where flagged
- **WHEN** a cycle passes through an unflagged endo-morphism such as derives_from between laws
- **THEN** it is not reported, while cycles through flagged endo-morphisms such as supersedes are reported

### Requirement: Typed instance builder
WHEN a parsed corpus is loaded, the acset-core SHALL build a typed
instance — an interned, dense id-set per schema object and a partial
function per morphism — in which an unresolved reference is a
representable value (None) that is never dropped and never a parse error.

#### Scenario: Dangling is a value
- **WHEN** a corpus contains one link that resolves to no id
- **THEN** the instance stores that morphism value as None and the build succeeds

#### Scenario: No link dropped
- **WHEN** a parsed corpus is built into an instance
- **THEN** every link is stored, recorded as dangling, or recorded as a typing violation — the three counts sum to the link count

#### Scenario: Forbidden edge not stored
- **WHEN** a corpus contains a reference whose target object is not in the morphism's allowed targets
- **THEN** the instance records a typing violation and stores no morphism value for it

#### Scenario: Graph parity edge for edge
- **WHEN** a corpus accepted by the existing graph builder is built into an instance
- **THEN** the derived edge set equals the existing builder's edges exactly

#### Scenario: Byte-stable rebuild
- **WHEN** the same corpus is built with input files in two different orders
- **THEN** both serializations are byte-identical

#### Scenario: Duplicate id parity
- **WHEN** a corpus contains two rows with the same file-qualified id (a corpus the existing builder accepts first-wins)
- **THEN** the builder resolves the same way and records the collision in its collision report — never a hard failure the old path did not emit

#### Scenario: Empty corpus builds an empty instance
- **WHEN** the input directory contains zero spec files
- **THEN** the instance is empty and well-formed, closures over it are empty, and parity with the existing builder holds

### Requirement: Single traversal primitive
The acset-core SHALL expose forward and backward closure over a seed set
and a chosen set of schema morphisms as its only traversal primitives,
through which graph fan-in, merge blast radius (`specs/merge.md`), and
the refactor advisor (`specs/refactor.md`) are expressed with no private
adjacency map — and query-derived results SHALL equal the pre-migration
walks for every corpus.

#### Scenario: Parity with existing walks
- **WHEN** closure-derived blast radius, fan-in, and supersedes cycles are computed over any corpus
- **THEN** they equal the values the pre-migration graph.rs and merge.rs walks compute

#### Scenario: Unknown seed rejected
- **WHEN** a closure is requested over a seed id that is not in the instance
- **THEN** the query fails with a remediation hint rather than ignoring the seed

#### Scenario: Dangling not followed
- **WHEN** traversal reaches an unresolved reference
- **THEN** it is skipped for reachability, listed in the dangling report, and contributes to no result

#### Scenario: Supersedes-scoped cycles
- **WHEN** M is restricted to supersedes and a cycle exists in the supersedes subgraph
- **THEN** closure-based cycle detection over that scope reports it, matching the existing supersedes_acyclic behaviour
