<!--
Embedded Specodelic format primer — rendered by `spk explain`.

Not a spec file: no frontmatter, lives in `src/`, never scanned by the
linter. Closed value sets are NOT listed here — `{{placeholder}}` slots
render from the constants in `src/guide.rs` so the primer cannot
disagree with the enforced sets. Topics are delimited by
`<!-- topic: <id> -->` markers; ids are append-only.
-->

<!-- topic: format -->
# format

Specodelic is a four-layer markdown specification format: **one file is
one spec**, machine-linted and machine-compiled.

A spec file is:

1. **Intent** — a YAML frontmatter block (`id`, `kind`, `statement`).
   The `statement` must match one of the five EARS patterns (topic
   `ears`).
2. **Constraints** — a `## Constraints` markdown table with fixed
   columns (`id`, `kind`, `expr`, `traces_to`). Each row is one named,
   machine-checkable claim.
3. **Model** — a `## Model` section with a `### States` bullet list and
   a `### Transitions` table (`id`, `from`, `to`, `guard`). The machine
   reads the lifecycle; the guard cell cites a Constraint by `[[id]]`.
4. **Properties** — a `## Properties` table (`id`, `kind`,
   `derives_from`, `generator`, `predicate`). Every Constraint needs a
   deriving Property (`coverage`).

Closed sets (Intent kinds, Constraint kinds, Property kinds) render in
topic `kinds`; the typed reference fields render in topic `references`.

File naming law: the frontmatter `id` equals the filename stem with
`-` ⇔ `.` — `linter-graph_shape.md` declares `id: linter.graph_shape`.
`_` is literal in both.

Prose is never parsed (`prose_untouched`): rationale and description
text stay free-form. Only frontmatter and the fixed-schema tables carry
machine-checkable content, so the linter and compiler inspect
structured fields exclusively.

<!-- topic: ears -->
# ears

The Intent `statement` must match exactly one of the five EARS
patterns, with an imperative `SHALL`:

| Pattern            | Shape                                                  |
|--------------------|--------------------------------------------------------|
| Ubiquitous         | `THE <system> SHALL <response>`                        |
| Event-Driven       | `WHEN <trigger> THE <system> SHALL <response>`         |
| State-Driven       | `WHILE <state> THE <system> SHALL <response>`          |
| Unwanted-Behavior  | `IF <condition> THEN THE <system> SHALL <response>` (also `WHEN <trigger> THEN … SHALL`) |
| Optional-Feature   | `WHERE <feature> IS INCLUDED THE <system> SHALL <response>` |

Conformance is shape-only grammar checking — word choice inside the
slots is never judged. A statement without `SHALL` fails
`ears_statement`; so does a statement matching no pattern.

<!-- topic: kinds -->
# kinds

Intent `kind` — one of: {{intent_kinds}}.

Constraint row `kind` (`constraint_kind_closed`) — one of:
{{constraint_kinds}}.

- `invariant` — a claim the machine checks; the only kind a transition
  `guard` may cite.
- `advisory` — a claim checked and reported but never gating; it can
  never guard a transition, by typing not by convention.
- `effect` — an output claim; the only kind a State `emits` field may
  reference (the Moore-machine output of that state).
- `extension_point` — a published contract a consumer file points at
  outbound via `satisfies`; the consumer's row keeps its own ordinary
  `traces_to`.

Property row `kind` (`property_kind_closed`) — one of: {{property_kinds}}.

- `unit` — one proptest block from the predicate.
- `law` — the predicate expands into one block per named `**case:**`
  label; `law_requires_cases` demands at least `identity` and
  `associativity` — a floor, not a ceiling.

Closed sets grow only under a new `## Revision` heading, never silently
(`append_only_variants`).

<!-- topic: references -->
# references

Structured cells reference other specs with `[[wiki-links]]` — a target
may be a file id (`specodelic`), a row id (`specodelic.model_present`),
a section anchor (`model.state`), or a row member
(`specodelic.model.parse`). Every structured-field reference must
resolve somewhere in the corpus (`total_refs`).

Each reference field is a typed foreign key (`ref_kind_compatible`):

{{reference_typing}}

Notable consequences: a transition guard cites only `invariant`
Constraints; a state's `emits` cites only `effect` Constraints;
`satisfies` is an outbound pointer to a contract published elsewhere
and is not a reachability edge.

<!-- topic: lifecycle -->
# lifecycle

The Model section declares the states a spec artifact moves through
and the transitions between them. The canonical pipeline lifecycle:

`draft → parsed → linted → compiled → model_checked → verified`

- `parse` requires valid frontmatter (`frontmatter_valid`,
  `id_matches_file`)
- `lint` is the join point of the linter checkers — it requires every
  owned checker to pass
- `compile` requires `coverage` and `law_requires_cases`
- `model_check` requires a complete Model section (`model_present`)
- `verify` requires `no_counterexample` and `properties_pass`

Every transition's `guard` must be non-null (`guard_required`) and may
only cite `invariant` Constraints. Every state must appear as a `from`
or `to` in at least one transition (`every_state_used`), and every
transition's endpoints must be declared states
(`every_transition_valid`).

<!-- topic: lint-rules -->
# lint-rules

Lint findings are self-describing: each carries its rule id (e.g.
`linter.ears_syntax`) and a one-line semantics string, so the error
itself is the documentation. The catalog below is generated from the
same table the linter uses — it enumerates exactly the rule ids this
binary can emit.

{{lint_rules}}