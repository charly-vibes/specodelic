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
`[[C-foo]]` (no dot in the target) is skipped as metasyntactic in every
file: dotless spellings are format prose, never references. A
`spec.md` file under its capability directory carries its real id
(`openspec/specs/ge-cli/spec.md` → `id: ge.cli` — Revision 18) and
cites its own rows file-qualified: `[[ge.cli.c1]]`.

**Resolution algorithm** (for dotted ids — file ids routinely contain
namespace dots): exact file id first, then every dot split from the
LAST to the FIRST: `prefix` must be a known file id and the remainder must be a
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
- `verify` requires `no_counterexample`, `properties_pass`, and
  every opted-in invariant claim verified — prose-only invariants stay
  explicitly unchecked and are never counted as verified. Verification
  is bounded model checking of the claims the spec opts into; it is
  not a substitute for the application's own test suite.
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

- Specodelic half: YAML frontmatter (a real `id` from the naming law,
  `kind: intent`, an EARS `statement`) plus the `## Constraints`,
  `## Model`, and `## Properties` tables.
- Openspec half: `## Purpose`, `## ADDED Requirements`, and a sibling
  `## Requirements` section with identical text. A `## MODIFIED
  Requirements` delta is dual-format the same way — same id law, same
  mirror. A file carrying both delta kinds mirrors every delta
  requirement in `## Requirements`: `linter.requirement_drift`
  compares per requirement, so the natural mirror (all delta
  requirements, delta order) satisfies the rule.

**Naming law** (Revision 18): openspec hard-requires the filename
`spec.md`, and the id law derives a `spec.md` file's expected id from
its PARENT DIRECTORY name — `openspec/specs/ge-cli/spec.md` declares
`id: ge.cli` (the `-` ⇔ `.` mapping). There is no `id: spec`: every
dual-format file carries its real id, wiki-refs resolve corpus-wide
(deltas compose like any corpus), and two files claiming the same
intent id are refused on command paths (`duplicate_corpus_identity`).

**Enforcement**: `linter.dual_format_valid` — a file carrying
`## ADDED Requirements` or `## MODIFIED Requirements` must pair it
with the sibling `## Requirements` section (the id is the naming
law's business); a capability spec under
`openspec/specs/` must be dual-format or CI fails.
`linter.requirement_drift` (enforced by `spk lint` itself) fails when
the two requirement sections drift apart (blank lines and trailing
space ignored). Dual-format files trace to their own intent with
the file's real id: `traces_to: [[<file-id>]]` is the intended
constraint → intent edge.

**Migration** (plain openspec file → compliant):

1. Add the frontmatter: the real id derived by the naming law
   (`spec.md` under its capability directory → parent-dir name;
   anything else → stem), `kind: intent`, one EARS `SHALL` statement
   condensing the requirements.
2. Derive the tables: one `invariant` Constraint per MUST the
   requirements imply, a Model covering the scenarios, one unit
   Property deriving from each constraint (the coverage rule).
3. Mirror `## Requirements` (identical text to every carried delta
   section's requirements — `## ADDED Requirements` and, when
   present, `## MODIFIED Requirements` alike); a plain capability
   spec keeps only `## Requirements`.
4. Gates: `spk lint <file>`, `openspec validate --all --strict`, and
   the section-sync check must all pass.

<!-- topic: packs -->
## Domain packs — first-class vocabulary extension (Revision 14)

A domain pack is a four-layer spec file with `kind: profile` frontmatter
whose manifest is exactly six per-facet closed tables: `## Sections`,
`## Kinds`, `## References`, `## Checkers`, `## Floors`, `## Requires`.
The manifest declares exactly what the pack introduces; the structural
rule is the append-only `pack_shape` lint finding. The corpus spec is
`packs.md`.

- **Discovery** is a corpus scan — every `kind: profile` file in the
  workspace is a candidate pack (anchored at the git toplevel, so a
  subdirectory lint still finds workspace packs). No config file, no
  registry outside the corpus.
- **Opt-in is advisory-first, triggered by vocabulary use**: a file
  using vocabulary a discovered pack declares gets that pack's checkers
  run for it (per-pack attribution, longest-prefix matching). An
  explicit `uses` column (set-valued, targets the pack's frontmatter id)
  upgrades to declared enablement and enables pinning/skew checks.
  Declared vocabulary used with no pack discovered or declared is a
  labeled failure finding naming the candidate pack and both
  remediations.
- **Lifecycle**: the pack's Model carries `draft` → `published` →
  `deprecated`; `pack_shape` must pass in every state; draft packs'
  findings name the draft status; deprecated packs stay advisory with
  the deprecation named. The pack-authored `## Requires` table pins the
  base format revision (and pack deps); skew against the workspace
  corpus revision is a warnings-channel advisory, never silent, never
  failing.
- **Self-exemption**: a pack file is never a consumer — its own manifest
  rows trigger no checkers and produce no orphan findings.
- **Base closed sets freeze by policy** from Revision 14 on: pack
  vocabulary is pack-qualified (`bioimage.tolerance`), never global —
  two packs claiming the same mechanism is legitimate; collision exists
  only at vocabulary level.

<!-- topic: conform -->
# conform

`spk conform <file> --oracle <corpus.jsonl>` evaluates an EXTERNAL
oracle's recorded traces against a spec — the compiled Model's declared
transition relation and the executable claims cited by its guards. It
is READ-ONLY: conform classifies, it never generates, never writes
artifacts, and never advances the lifecycle. The command-path input
gate applies: the file must be lint-clean with current compiled
artifacts (conform never re-compiles), and the scenario corpus must
fully parse — a refused invocation emits ZERO verdict records.

## The oracle corpus — JSONL schema

One JSON object per line, each `{id, setup?, trace}`; unknown fields
are refused, never silently ignored (`deny_unknown_fields` — the
refusal names the offending field and line, e.g. a misspelled
`observes` comes back as "unknown field `observes`, expected `action`
or `observations`"). Missing `id` and duplicate ids are refused too
(the report is keyed by scenario id).

```json
{"id": "confirm-happy", "setup": {"state": "held"},
 "trace": [{"action": "confirm", "observations": {"status": "confirmed"}}]}
```

- `id` — unique across the corpus.
- `setup` — opaque EXCEPT its `state` entry: the value must name a
  declared Model state — the initial state anchoring the trace to the
  States list. No `setup`, or a `state` value no declared Model state
  declares, leaves no Model element anchoring the trace →
  `underspecified` (the no-anchor reason). Every other `setup` key is
  a free-variable name, opaque to conform.
- `trace` — the ordered steps, each `{action, observations?}`:
  - `action` — a declared transition id/name; matching is string
    identity after trimming (mechanical, never semantic).
  - `observations` — a MAP of free-variable name → declared state or
    effect id; each value is a string or an array of strings. The
    field is deliberately named `observations` — never `observes` —
    to avoid colliding with the format's typed reference field of the
    same name (design D3 of add-conform).

## The verdict taxonomy — closed values with decision rules

Every trace receives exactly one verdict (totality); the values are
never coerced into a binary pass/fail. Rules, in evaluation order per
step:

| Verdict | Decision rule |
|---------|---------------|
| `forbidden` | Positive contradiction: a declared transition exercised from a state it is not declared from, or a state observation naming a state other than the transition's declared target — the compiled Model forbids the step/outcome (`contradiction_forbidden_any_mode`, in ANY invocation mode). Also absence of coverage when `--closed-world` was declared (`closed_world_forbidden_recorded`). |
| `underspecified` | No declared Model element or claim covers the trace: an undeclared action, an observation naming no declared state or effect, or no/undeclared initial state. Open world by default (`uncovered_trace_never_forbidden`) — absence of coverage is never forbidden without the explicit `--closed-world` declaration. |
| `unsupported` | A covering claim's evaluator kind exists in the format but has no emitter in this run (`**py:**`/`**ts:**`) — the reason names the kind (`unsupported_evaluator_kind`). |
| `unknown` | A covering claim's expr is prose — not interpretable by this run; prose never implies a permitted/forbidden judgment (`prose_claim_not_interpretable`, naming the claim). |
| `permitted` | The walk explains the trace through declared transitions AND every covering claim is executable (`**rust:**` fragment, `**kernel:**` expression, or a pure citation expression like `[[a]] ∧ [[b]]`) with its guard enabled each step. An empty trace from a declared state is explainable: `permitted` with no steps. |
| `agree` | Structural-only mode ONLY (`--structural-only`): the step sequence conforms to the declared transition relation while a covering guard's expr is prose — the reason STILL names the uninterpretable guard (`structural_agree_prose_guard_unchecked`), so the record never claims the prose invariant was checked. Default mode is unchanged: prose guards stay `unknown`. |

Order matters: the first prose or unsupported covering claim decides —
a trace covered by BOTH a prose claim and an executable claim is
`unknown` (or `agree` under `--structural-only`), never `permitted`.

## `permitted` requires executable claims

Out of the box, a fully prose spec can never yield `permitted`: every
guard's cited invariant is prose, so every fully-covered trace is
`unknown` and the run validates only Model-relation explainability.
The path to `permitted` opts each cited invariant into an executable
evaluator — a `**rust:**` fragment, a `**kernel:**` expression, or a
pure citation expression resolved by the citation algebra. That is a
cross-feature interaction (conform × kernel markers × citation
algebra): citation-expr constraints are NOT "just prose with links" —
they are executable. `**py:**`/`**ts:**` do not count: no emitter
exists in this run, so a covering claim of those kinds yields
`unsupported`.

## No `emits` column → non-target observations are `underspecified`

A Model without an `emits` column declares no effects, so an
observation value that is not the exercised transition's target state
names no declared state or effect → `underspecified`. Declaring
`emits` on states (citing `effect` constraints — topic `kinds`) is
what makes non-target observations coverable.

## The kernel grammar (folded summary)

A `**kernel:**` marker opts a claim into the min-expr kernel — the
closed, decidable bounded logic over acset instances I(k):

```text
expr    := unary ('∧' unary)*
unary   := '¬' unary | primary
primary := '∀' var '∈' Object ':' expr
         | '∃' var '∈' Object ':' expr
         | '(' expr ')'
         | 'resolves(m)' | 'unique(m)' | 'acyclic(m)'
         | 'reachable(a, b, m)'
         | term ('==' | '<' | '<=' | '>' | '>=') term
term    := '|' Object '|' | Int | '⊥' | var | Id | var.morphism
```

Quantifier objects are the schema's objects (`Constraint`, `Intent`,
`Property`, `State`, `Transition`); anything outside the closed set is
refused with `kernel_grammar_closed`, naming the offending token and
the full closed atomic set. Every atomic names its proven grounding
(graph traversal, acset traversal, model_check bounded evaluation),
so evaluation is total over finite instances and `unknown` never
coerces to pass.
