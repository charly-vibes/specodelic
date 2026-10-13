# Theory

*Not a spec file — no frontmatter, exempt from the four-layer shape, same
category as `AGENTS.md`/`STATUS.md`/`USAGE.md`/`CHANGELOG.md`. This is
where every category-theoretic claim in this repo lives, in full, exactly
once. Every other file states its guarantees in plain terms and links here
for the rigorous version, rather than restating or paraphrasing the math —
paraphrase drift is exactly how `append_only_variants`,
`kind_field_extensible`, and `reference_field_extensible` ended up as the
same rule stated three slightly different ways (`specodelic.md` Revision
6). One home for the theory prevents that happening to this vocabulary
too.*

*Links into this file use single-bracket markdown links
(`[naturality](theory.md#naturality-safe-rename)`), never `[[double-bracket]]`
wiki-links — this file carries no frontmatter `kind`, so it is not a valid
target for any typed reference field, and a future linter resolving
`[[...]]` against real `𝒦`-typed rows would treat a link here as
dangling. Links to this file only ever appear in prose (Notes, rationale),
never in a structured field (`expr`, `guard`, `predicate`), consistent
with `prose_untouched`.*

---

### Schema (𝒦)
**Plain:** A fixed, closed set of five record types — Intent, Constraint,
State, Transition, Property — the only kinds of thing a spec file's rows
can be.
**Rigorous:** `𝒦` is a small fixed category whose objects are exactly
these five kinds. Nothing else in this repo needs the categorical
statement; every other file can just say "one of the five kinds" or "the
schema."

### Typed foreign keys (generating morphisms)
**Plain:** Every reference field (`traces_to`, `derives_from`, `guard`,
`from`/`to`, `supersedes`, `emits`) is a foreign key that must point at a
specific kind of row, never any kind — the same shape as a database
foreign key with a `CHECK` constraint on the target table.
**Rigorous:** These fields are `𝒦`'s generating morphisms, each with a
fixed source/target typing. The Reference Typing table in `specodelic.md`
is that typing written out explicitly; `ref_kind_compatible` is the
invariant that enforces it.

### Document instance (functor and copresheaf)
**Plain:** One spec file is one document that conforms to the schema —
the same relationship a JSON file has to its JSON Schema, or a row has to
a table definition.
**Rigorous:** A spec file is a functor `I : 𝒦 → Set` (a copresheaf) — it
assigns a set of ids to each kind and a real function to each reference
field.

### Well-formedness (linting)
**Plain:** Linting is schema validation plus referential integrity (no
dangling foreign key) plus a coverage rule (every Constraint has a
matching Property) — nothing more exotic than what any strict schema
validator already does.
**Rigorous:** Linting checks that the functor `I` is well-formed: no
dangling reference means every generating morphism's image is total;
"every constraint has a test" means a particular morphism is surjective.

### Naturality (safe rename)
**Plain:** Renaming an id updates its definition and every place it's
referenced, with a guarantee that nothing breaks — the same guarantee an
IDE's "rename symbol" gives you, just for markdown tables instead of
source code.
**Rigorous:** A rename is a natural transformation `η : I ⇒ I'` between
two functor instances; naturality is the commuting square that formally
states the rename didn't silently break a reference. See
`rename_naturality` (`specodelic.md`) and `rename.md`.

### Namespacing (Grothendieck construction)
**Plain:** A cross-file id like `order.cancel.refund_bounded` is just a
qualified name — `file.local_id` — the same idea as a qualified name in
any module system (`package.Class.method`).
**Rigorous:** Cross-file ids form a Grothendieck construction: the dotted
id is a point in the total space `∫I` over the indexing category of
feature files; the short local id used inside one file is its fiber
coordinate.

### Interface-contract tests (algebraic laws)
**Plain:** Some properties aren't a single example — they're a contract:
if you claim your operation is a "law" of a certain shape, a fixed list of
test cases becomes mandatory (the same way implementing `Comparable`
obligates you to test reflexivity and transitivity, not just one
comparison).
**Rigorous:** A `kind: law` Property is a claimed algebraic law.
`law_requires_cases` sets the floor (identity, associativity); a specific
law (an adjunction, a functor) may require further named cases (unit,
counit, naturality, triangle identity) listed in its own predicate. See
`USAGE.md` §2.5 for the worked example and case names.

### State with output (Moore machine)
**Plain:** A state that not only transitions but also hands back a value
— the same idea as a reducer function returning `(newState, output)`
instead of just `newState`.
**Rigorous:** A state's optional `emits` field gives the state machine the
output-function half of a Moore machine (states, transitions, and a
function from state to output). `emits` must resolve to a Constraint with
`kind == effect` (see the Reference Typing table).

### Limit over a dependency diagram (independent checks joined)
**Plain:** When several independent checks must *all* pass before you can
move on, and some depend on others finishing first, that's just a
dependency graph with a single "all done" gate at the end — the same
shape as a build system's DAG of tasks.
**Rigorous:** `linted` is a limit over the Checker Ownership dependency
diagram — the product of all eight gating checkers' `passed` states, subject to the
arrows between them; `lint` fires only when every terminal node reports
`passed`, not on a fixed sequence.

### Additive-only evolution
**Plain:** An enum, a variant list, or a set of allowed reference-field
targets can only gain new members over time — nothing is ever removed,
renamed in place, or silently redefined. Same discipline as protobuf/Avro
schema evolution rules ("never remove a field, never renumber").
**Rigorous:** `append_only_variants` states this once, generally, for any
id-set governed by `𝒦`: it grows only under a new Revision heading, never
silently, and a narrowing is permitted only in the same Revision that
introduces the split it depends on, and only if it invalidates nothing
previously valid.

### Affected graph (transitive closure / blast radius)
**Plain:** Everything upstream or downstream of a proposed change, found by
walking the reference graph — the same thing a monorepo build tool means
by "affected targets" (Bazel's `rdeps()`, Nx's affected-graph).
**Rigorous:** The forward and backward transitive closure of edges
reachable from a node, computed within the free category generated by
`𝒦`'s typed morphisms. See `graph.md`'s `blast_radius_is_transitive_closure`.

### Confluence (merging two divergent rewrites)
**Plain:** When two people each make a valid, independent edit starting
from the same file, and those edits need to be combined into one result
that's still valid — not just "the text merges cleanly," but "the rules
the file must obey still hold after combining them." A rename on one side
and a brand-new reference to the old name on the other is the case that
looks fine textually and isn't.
**Rigorous:** Two divergent instances `I_A`, `I_B` derived from a common
`I` by independent rewrites are a **critical pair** if their rewrites
don't commute; the merge is a **pushout** of `I_A` and `I_B` over `I`, and
confluence is the property that this pushout is itself a well-formed
instance of `𝒦`, resolving the critical pair rather than producing a
broken result. See `merge.md`'s `rename_replayed_onto_foreign_edits`.

| Plain term (use everywhere else) | Rigorous term (defined above) |
|---|---|
| schema / the five kinds | `𝒦` |
| typed foreign key | generating morphism |
| document instance | functor / copresheaf |
| well-formedness check | functor well-formedness |
| safe rename | naturality / natural transformation |
| qualified name | Grothendieck construction |
| interface-contract test cases | algebraic law (identity/associativity/…) |
| state with output | Moore machine |
| all-must-pass dependency gate | limit over a diagram |
| grows-only enum | additive-only / append-only variants |
| affected/impacted nodes | transitive closure (blast radius) |
| combined valid result from two divergent edits | confluence / pushout |

## Note on "kind" overloading

Two unrelated things are both called "kind" elsewhere in this repo:
`𝒦`'s five objects (which table a row belongs to), and the `kind` *column*
that Constraint and Property rows separately carry (`invariant`/`unit`/
etc.). This file uses "kind" only for the first sense and "sub-kind" or
"tag" for the second, for clarity — this is the same confusion tracked as
`CLAR-002` in `STATUS.md`, unresolved there pending a column rename via
`rename_naturality`.
