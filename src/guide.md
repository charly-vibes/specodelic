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
   reads the lifecycle; the guard cell cites a Constraint with a
   file-qualified wiki-link (topic `references`).
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
structure. Inside a table cell a literal `|` must be escaped as `\|`
(e.g. a union type `int \| float \| str`) — the parser splits cells on
unescaped pipes.
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
`ears_syntax`; so does a statement matching no pattern.

<!-- topic: kinds -->
# kinds

Intent `kind` — one of: {{intent_kinds}}.

Constraint row `kind` (`constraint_kind_closed`) — one of:
{{constraint_kinds}}.

- `invariant` — a claim the machine checks; the only kind of Constraint
  a transition `guard` may cite (a guard may also cite a State — the
  "has reached state X" pattern).
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

**Refs are file-qualified**: the general shape is
`[[<file-id>.<row-id>]]`. A bare-text cell (`derives_from: C-foo`)
produces no reference at all — the property reports as orphaned. A bare
`[[C-foo]]` (no dot in the target) is skipped as metasyntactic — except
in a dual-format delta (`id: spec`), where a bare target naming one of
the file's own rows resolves to that row: `[[c1]]` is exactly
`[[spec.c1]]` there, because the file is self-contained and the bare
form has one possible meaning. A dotless target naming no own row stays
metasyntactic in every file. A dual-format delta cites its own rows
with either spelling — `[[c1]]` or `[[spec.c1]]` — in `derives_from`
and in transition `guard` cells; files outside the delta always use the
file-qualified form.

**Resolution algorithm** (for dotted ids — file ids routinely contain
namespace dots): exact file id first, then a bare-local row (own file,
`id: spec` files only), then every dot split from the LAST to the
FIRST: `prefix` must be a known file id and the remainder must be a
row, the `model.state`/`model.transition` anchor, or `row.member` —
the row is the single segment right after the split, everything after
it is a member path (row ids are never dotted). Last-dot wins, so
`[[linter.frontmatter.has_id]]` resolves as file `linter.frontmatter`,
row `has_id`, while `[[specodelic.model.state]]` resolves as file
`specodelic`, anchor `model.state`.

Each reference field is a typed foreign key (`ref_kind_compatible`):

{{reference_typing}}

Notable consequences: a transition guard cites an `invariant` Constraint
or a State; a state's `emits` cites only `effect` Constraints;
`satisfies` is an outbound pointer to a contract published elsewhere
and is not a reachability edge; `observes` points from a consuming row
at an effect Constraint — a declared observable — and, like
`satisfies`, is not a reachability edge and joins no acyclic set
(mutual cross-file observation is well-formed).

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
cite an `invariant` Constraint or a State. Every state must appear as a `from`
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
<!-- topic: dual-format -->
# dual-format

Where specodelic meets **OpenSpec**: this repo manages its engineering
change workflow in openspec (proposals → tasks → archive), and every
change delta is also a lint-clean specodelic file. One markdown file,
two parsers, requirement text authored once.

A **dual-format file** carries both grammars:

- Specodelic half: YAML frontmatter (`id: spec`, `kind: intent`, an
  EARS `statement`) plus the `## Constraints`, `## Model`, and
  `## Properties` tables.
- Openspec half: `## Purpose`, `## ADDED Requirements`, and a sibling
  `## Requirements` section with identical text. A `## MODIFIED
  Requirements` delta is dual-format the same way — same id law, same
  mirror; the mirror rules treat the two delta sections identically.

**Naming law**: openspec hard-requires the delta filename `spec.md`,
so dual-format files declare `id: spec` (the `-` ⇔ `.` mapping makes
that the legal id). Uniqueness is per-file — any number of `id: spec`
files coexist. Deltas stay self-contained: `[[wiki-refs]]` resolve
only within the file; domain semantics are cited by prose path
(`specs/<name>.md`), never wiki-link.

**Enforcement**: `linter.dual_format_valid` — a file carrying
`## ADDED Requirements` or `## MODIFIED Requirements` must declare
`id: spec` and pair it with the
sibling `## Requirements` section; a capability spec under
`openspec/specs/` must be dual-format or CI fails.
`linter.requirement_drift` (enforced by `spk lint` itself) fails when
the two requirement sections drift apart (blank lines and trailing
space ignored). Self-contained deltas trace to their own intent with
the file's own frontmatter id: `traces_to: [[spec]]` is the intended
constraint → intent edge for a `id: spec` file.

**Migration** (plain openspec file → compliant):

1. Add the frontmatter: `id: spec`, `kind: intent`, one EARS `SHALL`
   statement condensing the requirements.
2. Derive the tables: one `invariant` Constraint per MUST the
   requirements imply, a Model covering the scenarios, one unit
   Property deriving from each constraint (the coverage rule).
3. Mirror `## Requirements` (identical text to `## ADDED
   Requirements`); a plain capability spec keeps only
   `## Requirements`.
4. Gates: `spk lint <file>`, `openspec validate --all --strict`, and
   the section-sync check must all pass.
