# Using specodelic (formerly spec-format)

*Read this after `AGENTS.md` and `specodelic.md`, before writing a spec
for a piece of software (a "domain spec") rather than for this repo's own
linter. Everything here is about how to point the four existing layers —
Intent, Constraints, Model, Properties — at a real feature, including a
few shapes that don't look like they fit until you see where they go.*

---

## 0. Terminology: four words that are easy to confuse

Prose governance only — the parser never reads any of this
(`prose_untouched`), so these conventions are enforced by review and the
sweep in the acceptance criteria of the tracking ticket, not by a lint
rule.

| Term | Means | Example |
|---|---|---|
| **the format** | the markdown schema/language itself: frontmatter fields, the four layer tables, wiki-links, the 5 kinds. Defined by `specodelic.md`, `kinds.md`, `theory.md`. | "states are named variants, not booleans" — a rule *of the format* |
| **the tool (spk)** | the `specodelic` CLI that lints/compiles/checks files written in the format. Specified by the `linter-*.md`, `compile.md`, … corpus. | `spk lint` reports `linter.frontmatter` findings |
| **the subject** | whatever a given spec file is *about*. For this corpus the subject is the tool itself (dogfooding); for a domain spec it's whatever system you're specifying. Declared by a file's `statement` + prose — deliberately **not** a frontmatter field, hence never machine-checkable. | `orchestrate.md`'s subject is the pipeline; `order-cancel.md`'s is the refund policy |
| **spec file** | one document conforming to the format | `specs/orchestrate.md` |
| **the corpus** / `specs/` | the directory of spec files this repo maintains | "the corpus is dogfooding" |
| **spec-format** | the project's former name. Appears only in provenance contexts (rename history, CHANGELOG); never in current prose | "renamed from `spec-format` on import (2026-09-28)" |

Rules of thumb:

- Bare **"specodelic"** names the project/repo only. "the specodelic
  format", "a specodelic spec file", "the specodelic CLI" are fine
  (adjectival); "the specodelic" as a bare noun for the format or the
  tool is the violation this section exists to prevent.
- "spec" is likewise four senses — file, format, corpus, legacy name —
  and bare "the spec" is best avoided in prose for the same reason.
- **Subject-relative litmus test:** a constraint `expr` that mentions the
  checker or the pipeline is layer conflation *iff the file's subject is
  not the tool*. `orchestrate.md`'s `lint_gate_matches_checker_ownership`
  mentioning checkers is correct — its subject *is* the pipeline. A
  domain spec about a refund policy mentioning `spk` would be conflation.
- What the format contributes to a domain spec is narrow and structural:
  `no_boolean_columns` forces states like `done/partial/failed` to be
  named variants, `append_only_variants` governs growing that set, and
  `guard_required` forces each race/handoff to be an explicit guarded
  transition. Everything else about a subject's semantics is ordinary
  spec-writing, expressible *in* the format but not supplied *by* it.

## 1. Quick start: writing a new domain spec

Every file is the same four sections, in this order, whether it's a
five-line CRUD endpoint or `specodelic.md` itself. Save the example
below as `order-cancel.md` — the filename stem must equal the frontmatter
id with `.` mapped to `-` (`order.cancel` → `order-cancel.md`; `_` is
literal in both):

```markdown
---
id: order.cancel
kind: intent
statement: "WHEN a customer cancels a paid order, THE system SHALL refund
  the payment in full within 5 business days."
---

# Order Cancellation

One paragraph of prose: why this exists, what it's not (the parser never
reads this paragraph — see `prose_untouched` in `specodelic.md`).

## Constraints

| id              | kind      | expr                                              | traces_to        |
|------------------|-----------|--------------------------------------------------|-------------------|
| refund_bounded    | invariant | `refund_amount == paid_amount`                    | [[order.cancel]] |
| refund_timely     | invariant | `refund_issued_at - cancelled_at <= 5 business days` | [[order.cancel]] |
| refund_traces_resolve | invariant | `**kernel:** resolves(traces_to)`          | [[order.cancel]] |
| refund_guard_reaches | invariant | `**kernel:** reachable(order.cancel.refund, order.cancel.refund_bounded, guard)` | [[order.cancel]] |

## Model

### States
- `active`
- `cancel_requested`
- `refunded`

### Transitions

| id       | from              | to        | guard                                            |
|----------|-------------------|-----------|---------------------------------------------------|
| request  | active            | cancel_requested | `customer submitted a cancel request`      |
| refund   | cancel_requested  | refunded  | [[order.cancel.refund_bounded]] ∧ [[order.cancel.refund_timely]] |

## Properties

| id              | kind | derives_from                    | generator                    | predicate                          |
|------------------|------|----------------------------------|-------------------------------|--------------------------------------|
| full_refund_only | unit | [[order.cancel.refund_bounded]] | `arbitrary_paid_order()`      | `refund(cancel(order)) == order.paid_amount` |
| refund_within_term | unit | [[order.cancel.refund_timely]] | `arbitrary_paid_order()`    | `days_between(cancelled_at, refund_issued_at) <= 5` |
| traces_resolve_prop | unit | [[order.cancel.refund_traces_resolve]] | `arbitrary_paid_order()` | `every constraint trace resolves` |
| guard_reaches_prop | unit | [[order.cancel.refund_guard_reaches]] | `arbitrary_paid_order()` | `the refund transition's guard reaches the bounded invariant` |
```

The two `**kernel:**` rows are the example's executable slice — the
min-expr kernel's closed atomic grammar (`add-min-expr-kernel`):
`resolves(traces_to)`
asserts every `traces_to` reference in the file resolves, and
`reachable(…, guard)` asserts the `refund` transition's guard actually
cites the bounded invariant. The kernel evaluates these over the
file's own rows and references — the domain-data cells above them
(`refund_amount == paid_amount`, the days-between bound) stay informal
strings, exactly as §4 below says: kernel semantics never outruns the
instances the format itself stores, and `spk model-check` reports each
kernel claim's status (`verified` here) in its run report.

That's the whole shape. The rest of this guide is about the parts that
don't obviously fit — a closed set of node types, a machine that produces
output, several interchangeable implementations, a lazy pipeline, a
contract a consumer extends from outside your file, an append-only event
log, an empirical runtime bound — each of which turned out to be an
existing pattern applied somewhere non-obvious, not a reason to invent a
new section.

**Before finishing any new file**, run the same self-check every checker
file in this repo already used (`STATUS.md` §5, step 9): does every
Constraint trace to this file's Intent, does every Constraint have a
deriving Property, does the Model's guard set match what
`linter-model_shape.md` requires, and — new as of this guide — does any
part of the domain need one of the patterns in §2 before it's forced into
the wrong table.

---

## 2. Pattern catalog

These are not new mechanisms. Each is the existing four-layer schema
pointed at something it wasn't obviously built for, worked out by checking
a real domain (a lazy, category-theoretic data library) against the
format end to end. Use these instead of inventing a fifth section.

### 2.1 A closed, exhaustively-matched set of node types ("sealed AST")

**Don't** put this in Model/States. States are a *lifecycle* — one thing
moving through phases over time. A closed set of value constructors (an
expression grammar's node kinds, a tagged union's variants) is not a
lifecycle; nothing is "transitioning" between `Literal` and `BinOp`.

**Do** put it in the Constraints table, one row per constructor, each an
`invariant` stating that constructor's shape:

```markdown
| id              | kind      | expr                                                          | traces_to  |
|------------------|-----------|----------------------------------------------------------------|-------------|
| expr_literal      | invariant | `Literal node has exactly field {value: Scalar}`                | [[expr]]   |
| expr_column       | invariant | `Column node has exactly field {name: str}`                     | [[expr]]   |
| expr_binop        | invariant | `BinOp node has exactly fields {left: Expr, right: Expr, op: Op}` | [[expr]] |
```

"Sealed" — closed except by deliberate revision — is exactly what
`append_only_variants` already guarantees for any 𝒦-governed id-set: this
constructor list can only grow, and only under a new Revision heading in
this file. You get exhaustiveness-by-construction for free; you don't
need to declare it separately.

### 2.2 A state machine that produces output (Moore machine)

If a state needs to say what it *emits*, not just what it transitions to,
give it an `emits` field pointing at an `effect`-kind Constraint:

```markdown
### States
- `seeded`
- `accumulating` `emits: [[reducer.partial_sum]]`
- `finalized` `emits: [[reducer.total]]`

## Constraints

| id            | kind   | expr                                  | traces_to     |
|----------------|--------|------------------------------------------|----------------|
| partial_sum     | effect | `output == running_total_so_far`          | [[reducer]]   |
| total           | effect | `output == sum(all_seen_values)`          | [[reducer]]   |
```

A state with no `emits` is a plain automaton state — both are valid in the
same Model. Don't add `emits` to every state out of habit; only the ones
whose whole point is what they hand back.

### 2.3 Several interchangeable implementations of one contract

**Don't** try to parametrize a single Model over "which backend." **Do**
reuse the Checker Ownership shape from `specodelic.md`: one shared Intent
file, and N implementation files that each own their own Constraints and
Properties but all `traces_to` the same Intent:

```
storage.abstract.md      — the Intent + Constraints every backend must satisfy
storage.in_memory.md     — traces_to [[storage.abstract]], its own Model + Properties
storage.arrow.md         — traces_to [[storage.abstract]], its own Model + Properties
```

"All backends conform" is then the same thing `linted` already is for this
repo's eight gating checkers: a limit over a dependency diagram, `passed` only when
every leaf reports `passed`. No new object in `𝒦`, no new table — the
proof this works is that it's already running, right now, as this repo's
own lint pipeline.

### 2.4 Build now, execute later (laziness / staged evaluation)

Split it into two states with a guarded transition between them, the same
way `compile.md` separates `extracting` from `emitting`:

```markdown
### States
- `unmaterialized`
- `materialized`

### Transitions

| id           | from            | to             | guard                                      |
|--------------|-----------------|-----------------|-----------------------------------------------|
| materialize  | unmaterialized  | materialized    | `.collect() or .compute() was explicitly called` |
```

Building the expression graph is everything that happens *in*
`unmaterialized`; nothing there is a state of its own unless it needs its
own `emits` or its own guarded sub-transitions.

### 2.5 A law beyond identity/associativity

`law_requires_cases` in `specodelic.md` is a **floor**: every `law`
property needs at least identity and associativity, but nothing stops it
from requiring more, named in its own predicate:

```markdown
| id            | kind | derives_from        | generator                          | predicate |
|----------------|------|-----------------------|--------------------------------------|-------------|
| sigma_delta_pi | law  | [[poco.adjoint_triple]] | `arbitrary_schema(), arbitrary_functor()` | **identity:** `...`  **associativity:** `...`  **unit:** `η: Id ⇒ Δ∘Σ`  **counit:** `ε: Σ∘Δ ⇒ Id`  **triangle:** `(ε∘Σ)∘(Σ∘η) == id_Σ` |
```

Don't wait for a schema change to add unit/counit/naturality/triangle-
identity cases to an adjunction or functor law — add them to the case list
directly, the way `rename_naturality` already added a third case beyond
the two the floor requires.

**Idempotency needs no new case name, just a predicate**, the same
floor-plus-cases shape:

```markdown
| id          | kind | derives_from      | generator            | predicate |
|--------------|------|-----------------------|----------------------|-------------|
| retry_safe   | law  | [[api.rate_limit]]   | `arbitrary_request()` | **identity:** `...`  **associativity:** `...`  **idempotent:** `apply(apply(x)) == apply(x)` |
```

**A monad's laws are the same floor plus its own named cases** — not a
different mechanism from the adjunction example above, just a different
domain wearing it:

```markdown
| id           | kind | derives_from         | generator                          | predicate |
|---------------|------|--------------------------|---------------------------------------|-------------|
| result_monad  | law  | [[result.bind_contract]] | `arbitrary_value(), arbitrary_kleisli_fn()` | **identity:** `bind(pure(x), f) == f(x)`  **associativity:** `bind(bind(m,f),g) == bind(m, x -> bind(f(x),g))`  **left_identity:** `bind(pure(x), f) == f(x)`  **right_identity:** `bind(m, pure) == m` |
```

The floor's `identity`/`associativity` cases already *are* two of a
monad's required laws under different traditional names — a monad law
property doesn't need four independent cases, it needs the floor plus
`left_identity`/`right_identity` named explicitly so a reader unfamiliar
with monad terminology can still see the floor is satisfied.

### 2.6 A contract a consumer extends from outside your file (Open/Closed, protocol-style)

Julia multiple dispatch on a package-owned type hierarchy, and any purely
internal "closed set of cases," are just §2.1 again — a sealed enumeration
keyed on type instead of on a tagged value. This pattern is for the
*other* shape: Clojure-style protocols/multimethods, or a generic function
you expect downstream packages to extend for their own types, where the
whole point is that a consumer can conform **without ever editing your
file** — the expression-problem move, and the actual mechanism behind the
Open/Closed Principle (closed for modification, open for extension).

**Don't** try to enumerate conformers in your own file, and don't add a
reachability edge pointing back at them. You can't ever know the full set
— that's not a gap to work around, it's the same kind of unboundedness
`model_check.md` already admits for infinitely-deep recursive structures,
just spread across files and time instead of depth. A schema that
pretended to close over "all conformers, including ones not yet written"
would be claiming something it can't check.

**Do** publish the contract as an `extension_point`-kind Constraint in
your own file — this is a communication artifact, not an enforcement one:

```markdown
| id                | kind             | expr                                                                                          | traces_to |
|--------------------|------------------|----------------------------------------------------------------------------------------------------|-----------|
| symmetry_contract   | extension_point  | `∀ T conforming: symmetry(T)::SymmetryTag, canonical_order(T)::Vector{Int}, sign(T, perm)::Int`      | [[canon]] |
```

A consumer who wants to conform writes their **own** ordinary spec file,
with one Constraint row's `satisfies` field pointing back at your
`extension_point` row:

```markdown
| id                | kind      | expr                                                          | traces_to        | satisfies                         |
|--------------------|-----------|-------------------------------------------------------------------|--------------------|--------------------------------------|
| my_tag_symmetry     | invariant | `symmetry(MyTag) == :cyclic; canonical_order sorts by weight`      | [[my_extension]]  | [[xact.canon.symmetry_contract]]  |
```

That row is reachable the ordinary way, through its own `traces_to`
pointing at *its own* file's Intent — `satisfies` is an extra outbound
pointer sitting beside it, exactly the way `guard` and `emits` already
coexist with `traces_to` without disturbing reachability. Nothing in your
file changes, needs updating, or even needs to know the consumer's file
exists. `ref_kind_compatible` checks the consumer's `satisfies` row
resolves to a real, correctly-typed `extension_point` — same as any other
typed reference — but that only confirms they pointed at a real contract,
not that their `expr` is actually faithful to it; that's the same
human/test-suite judgment call every other `invariant` already rests on.

Two things this needs *nothing new* for, worth knowing so you don't
over-build it: `guard`'s Reference Typing admits only an invariant
Constraint (or a State citation, specodelic.md Revision 12), so an
`extension_point` row (like `advisory` and `effect`) is still excluded
from gating any transition, by the same existing typing fact — don't
add a fresh "extension points can't gate" invariant, cite the existing
one (§`AGENTS.md` #3a). And don't
reach for `single_root_reachable`'s carve-out machinery either — there
isn't one, because there's nothing to carve out.

When the consumer is an **external checker** (an outside test suite
claiming one of your invariants), the same open/closed shape applies
one level out, through the `kernel.binding` claim path: your Constraint
row may carry a `kernel.binding` column, and compile extracts the cell
as an opaque string into the compiled Constraints artifact — verbatim,
never parsed, validated, or interpreted. The checker claims the
constraint on its own side, by binding its tests through its
contract-TOML `flags` (`[[tests.cargo]]`/`[[tests.pytest]]`/
`[[tests.shell]]`); specodelic never learns the checker's language —
the bridge carries the claim, it never absorbs the checker (design D4
of `add-min-expr-kernel`; the name is namespaced against pack binding
columns). Don't invent a registry of checker languages or a parser for
binding contents — an opaque carried string is the whole contract.

### 2.7 An append-only event log with current state as a derived projection (event sourcing)

The format already does this to itself — `append_only_variants` plus
`supersedes` mean a Revision history never overwrites a prior row, and
"is this row current" is computed (`superseded(x) ⟺ ∃ y. y.supersedes ∋
x`), never stored as a boolean anywhere (`no_boolean_columns` covers it
for free, per `specodelic.md`'s own Notes on
`no_stored_superseded_flag`). Point that same mechanism at a domain
whose whole design is an event log instead of mutable rows — an
audit trail, a ledger, a CRDT-backed document history:

**Don't** model "current balance" or "current document state" as a State
in the Model section that gets overwritten on each event. A State that's
silently replaced on every transition, with the old value discarded, is
exactly the mutable-row shape `append_only_variants` was built to make
impossible for the format's own tables — the same shape should be avoided
in a domain being specified with it.

**Do** model each event kind as a §2.1 sealed Constraint (`deposited`,
`withdrawn`, `reversed` — an append-only, exhaustively-matched set of
event shapes), and current state as a Property whose `predicate` folds
over the log rather than a State the Model stores directly:

```markdown
## Constraints

| id          | kind      | expr                                                    | traces_to  |
|--------------|-----------|-------------------------------------------------------------|-------------|
| deposited     | invariant | `Deposited event has exactly field {amount: Money}`          | [[ledger]] |
| withdrawn     | invariant | `Withdrawn event has exactly field {amount: Money}`          | [[ledger]] |
| reversed      | invariant | `Reversed event has exactly field {target_event_id: EventId}` | [[ledger]] |

## Properties

| id               | kind | derives_from       | generator            | predicate                                                             |
|-------------------|------|-------------------------|----------------------|----------------------------------------------------------------------|
| balance_is_a_fold  | unit | [[ledger.deposited]]   | `arbitrary_event_log()` | `balance(log) == fold(log, 0, apply_event)` — never a field assigned to directly |
```

A revision, correction, or reversal is a *new* event appended to the log
(§2.1's constructor set grows, exactly as `append_only_variants` already
requires), never an edit to a past event — the same discipline this
repo's own `CHANGELOG.md` follows for itself ("past entries are never
edited — a correction gets a new entry"). Model/States is still the right
place for a genuinely different concept: the *processing* lifecycle of
one event (`received → validated → applied → acknowledged`) is a normal
finite-state lifecycle per §2.1's own distinction between a lifecycle and
a closed value shape; it's the *ledger's* current balance that's a fold
over Constraints-typed events, not a State.

### 2.8 An empirical runtime bound (latency/throughput/memory SLA)

**Don't** route this through the model checker. `model_check.md` is
explicit that unbounded checking never terminates, and — the reason this
case needs its own note rather than just citing that line — TLA+
verify discrete state reachability against a specified model, not wall-
clock behavior against real hardware. "This state machine always responds
within 50ms" isn't a deeper version of `no_counterexample`, it's a
different verification method entirely (a load test, not a state-space
search), and treating it as a Model-section guard would silently promise
a proof no model checker here can give.

**Do** treat it as a threshold Property, the same mechanism Revision 5
already established for confidence-scored checks — `expr`/`predicate` are
already unparsed strings, so a threshold needs no new field:

```markdown
| id              | kind | derives_from          | generator                                          | predicate                                        |
|------------------|------|----------------------------|---------------------------------------------------------|-----------------------------------------------------|
| eval_latency_slo  | unit | [[reply.eval_contract]]   | `fixed_benchmark_corpus(n=10_000 evals, warm_process)` | `p99_latency(corpus) ≤ 50ms`                       |
| concurrency_ceiling_holds | unit | [[reply.resource_limits]] | `load_test(concurrent_clients=64, duration=60s)` | `no request exceeds max_concurrent_evals ⟹ queued, never dropped` |
```

**One property genuinely different in kind from every other row in this
table, worth stating so it isn't mistaken for one:** a passed `acyclic_traces`
check is a permanent fact about a graph; a passed `eval_latency_slo`
benchmark is a fact about *this run, on this hardware, at this commit* —
re-running it can legitimately produce a different result with no
regression having occurred, the identical "re-runnable, not a one-time
verified fact" framing `model_check.md` already uses for its own checker
output. Record the benchmark environment (hardware, OS, load level) as
rationale prose beside the row rather than a structured field — per
`prose_untouched`, the parser never reads it, but a human comparing two
runs across commits needs it to know whether a changed number reflects a
real regression or a different machine.

**Design-time resource bounds are not this pattern and need no threshold
Property at all.** A configurable ceiling like REPLy's `max_message_bytes`
or `max_eval_time_ms` is a declarative fact about the contract itself
("requests over N get rejected"), not an empirical measurement — it's an
ordinary `invariant` Constraint with the violation case as a named
terminal state (`timeout`, `concurrency-limit-reached`), no different from
any other guarded transition. Reach for this section only when the claim
is actually *measured* against a corpus or a load scenario, not merely
*declared* as a configured limit.

---

### 2.11 Declaring what your domain adds (domain packs)

When your domain needs vocabulary the core format doesn't have — an
optional section, an extra `kind` value, a new outbound reference field,
per-kind required case labels — declare it as a **domain pack**
(`specodelic.md` Revision 14, full contract in `packs.md`): a four-layer
spec file with `kind: profile` whose six manifest tables
(`## Sections`, `## Kinds`, `## References`, `## Checkers`, `## Floors`,
`## Requires`) declare exactly what the pack introduces.

| id        | kind      | expr                                                          | traces_to   | uses                        |
|-----------|-----------|----------------------------------------------------------------|-------------|-----------------------------|
| foo.uses.bar | invariant | `foo's rows may carry bar's declared vocabulary`              | [[foo]]     | [[bar]] [[baz]]             |

Three facts govern the mechanism:

- **Opt-in is advisory-first and triggered by use.** A file that uses
  vocabulary a discovered pack declares gets that pack's checkers run
  for it (advisory-first); an explicit `uses` column (set-valued, like
  `traces_to`) upgrades to declared enablement, which is what pinning
  and revision-skew checks read. Files using no pack vocabulary lint
  byte-identically to before.
- **The pack is discovered by corpus scan** — every `kind: profile`
  file in the workspace is a candidate pack; no config file, no
  registry. Declared vocabulary used with no pack discovered or
  declared is a labeled failure finding naming the candidate pack and
  both remediations (enable/declare it, or fix the vocabulary).
- **Base closed sets are frozen.** `profile` and `uses` are the only
  core growth (Revision 14); everything else a pack introduces is
  pack-qualified (`bioimage.tolerance`) so independent packs coexist
  instead of colliding.

---

## 3. Migrating an existing spec into specodelic

This maps most naturally from artifact-per-purpose formats (OpenSpec-style
`proposal.md` / `design.md` / `specs/*.md` / `tasks.md`), but the same
four questions apply no matter what you're migrating from.

| Existing artifact | Ask | Goes to |
|---|---|---|
| Proposal's "why" | Can this be stated as one EARS-pattern sentence? | Frontmatter `statement` |
### 2.9 Declaring what must be observable (`observes`)

An `effect` Constraint says a state *emits* something (§2.2). It says
nothing about whether anyone is supposed to be watching. If a behavior's
output is part of the contract — a metric, an event feed, a finding a
consumer surfaces — say so with an `observes` column on the row that
consumes the output, pointing back at the effect:

```markdown
## Constraints

| id         | kind      | expr                        | traces_to     | observes              |
|------------|-----------|-----------------------------|---------------|-----------------------|
| eff_out    | effect    | `output == {value}`          | [[publisher]] |                       |
| watch      | invariant | `output_seen == value`       | [[publisher]] | [[publisher.eff_out]] |
```

`observes` is one-directional and cross-file: the observer's file points
at the publisher's effect and needs no other link between the files. It
is an extra outbound pointer beside `traces_to` — it never replaces the
row's own reachability, joins no acyclic set (mutual cross-file
observation is fine), and typing allows only `kind == effect` targets.
Every effect that *nothing* observes is reported as an advisory warning
(`linter.observability`, exit 0 — never a lint failure): if the output
is genuinely part of the contract, add an observer; if it was scaffolding,
delete the effect. That warning is the format asking "who is this output
for?" before the system ships with a firehose nobody attached a hose to.

### 2.10 Derived parallelism — the accumulator's algebra licenses the architecture

Whether a batch/stream processor can run in parallel is not an
assertion, it is a *derivation* from the accumulator's algebra — and
since Revision 13 every tier of that derivation is a machine-checkable
declaration, not prose: the law row's named cases (`**name:**` labels,
§2.5) are enforced by `linter.law_cases` (the identity and associativity
floor is a minimum) and compile expands one proptest block per case.
The worked example at `docs/src/examples/batch-resume.md` carries this
table end to end; its shape:

| the accumulator's algebra — each tier a named case | the architecture the law row licenses |
|---|---|
| deterministic step, replayable emissions (`unit` properties, §2.2's `emits`) | parallel **map** over any partition — no algebra required at all |
| results form a monoid (`**identity:**` + `**associativity:**`) | + chunked parallel **reduce**; merge in tree order |
| + commutative (`**commutativity:**`) | + workers need no ordering; any shuffle, any reduce tree |
| + idempotent merge (`**idempotence:**` — a semilattice) | + concurrent workers checkpoint freely; duplicates vanish |
| results not a monoid (running average, ordered state machine) | reduce is inherently sequential — still streamable, never chunkable; *say so here* rather than discovering it in production |

The point of doing this in the format instead of a design doc: the
tiers are `law` rows deriving from a stated invariant (e.g.
`reduce_sound_iff_monoid`), so a claimed tier whose case is missing
fails `lint` before it fails in production, and the map side needs none
of the algebra — only determinism (`map_is_parallel_unconditional`).

| Proposal's "what", design decisions | Is this a hard requirement, or a stated-but-non-gating design principle? | Constraints table — `invariant` if it must hold to pass `lint`/`verify`; `advisory` if it's a design intent that should be visible but never blocks a transition (see §2.1 of `specodelic.md`'s Revision 5) |
| A closed set of request/response/node types the design defines | Is anything actually transitioning, or is this just an enumeration of shapes? | If enumeration only → Constraints table, one row per shape (§2.1 above). If it's genuinely a lifecycle → Model/States |
| Design's lifecycle / workflow diagram | Are these states of one thing over time? | Model — States + Transitions, each transition's guard citing the Constraint(s) that must hold |
| Tasks.md acceptance criteria, "must produce X given Y" | Is this a checkable input→output claim? | Properties table — `unit` kind, one row per scenario; promote to `kind: law` only if it's an algebraic property (identity, associativity, or a case set from §2.5) rather than a single example |
| Design's "alternative backends" / "works with any of X" | Multiple things must satisfy the same contract, and *you* own every implementation | Split into one shared-Intent file + N implementation files (§2.3) |
| Design's "third parties can extend this" / a protocol, multimethod, or plugin interface | Consumers you'll never see must satisfy a contract, and their files aren't yours to enumerate | An `extension_point` Constraint in your file; each consumer's own file points `satisfies` back at it (§2.6) |
| Design's "this must be observable" / a metric, event feed, or surfaced finding | Does something consume the output, or is it a firehose nobody watches? | The effect Constraint (§2.2) plus an `observes` column on each consuming row — unobserved effects get an advisory warning (§2.9) |
| Design's "materialized lazily" / "computed on demand" | A build phase and an execute phase are genuinely different | Two states + a guarded transition (§2.4) |
| Design's "audit trail" / "event log" / "append-only ledger" / "CRDT history" | Is "current state" actually a stored, overwritten field, or a fold over past events? | Events as a §2.1 sealed Constraint set; current state as a Property whose predicate folds the log (§2.7) |
| Design's "must handle N req/s" / "p99 under Xms" / a load-test or benchmark requirement | Is this measured against a corpus/load scenario, or just a configured ceiling on the contract? | Measured → threshold `unit` Property with a benchmark generator (§2.8, re-runnable, not a permanent fact). Configured ceiling → ordinary `invariant` Constraint with the violation as a named terminal state |

**Worked micro-example** — migrating an OpenSpec-style proposal fragment:

> *proposal.md*: "Add rate limiting to the API. Requests over the limit
> get a 429. The limit is configurable but defaults to 100/min. This is a
> stopgap; we'd like adaptive limiting eventually but that's out of scope."

Maps to:

```markdown
---
id: api.rate_limit
kind: intent
statement: "WHEN a client exceeds the configured request rate, THE API
  SHALL respond 429 for further requests until the window resets."
---

## Constraints

| id                 | kind      | expr                                          | traces_to           |
|---------------------|-----------|------------------------------------------------|-----------------------|
| limit_enforced       | invariant | `requests_in_window > limit ⟹ response == 429`  | [[api.rate_limit]]  |
| default_limit        | invariant | `default(limit) == 100 per minute`              | [[api.rate_limit]]  |
| adaptive_limiting     | advisory  | `limit could vary with observed load`           | [[api.rate_limit]]  |
| rate_traces_resolve  | invariant | `**kernel:** resolves(traces_to)`               | [[api.rate_limit]]  |
| exceed_guard_reaches | invariant | `**kernel:** reachable(api.rate_limit.exceed, api.rate_limit.limit_enforced, guard)` | [[api.rate_limit]]  |

## Model

### States
- `open`
- `limited`

### Transitions

| id     | from    | to       | guard                              |
|--------|---------|----------|------------------------------------|
| exceed | open    | limited  | [[api.rate_limit.limit_enforced]]  |
| reset  | limited | open     | `the rate window resets`           |

## Properties

| id               | kind | derives_from                        | generator             | predicate                            |
|-------------------|------|-------------------------------------|------------------------|---------------------------------------|
| over_limit_is_429 | unit | [[api.rate_limit.limit_enforced]]   | `arbitrary_request()`  | `response == 429`                     |
| default_is_100    | unit | [[api.rate_limit.default_limit]]    | `arbitrary_config()`   | `default(limit) == 100 per minute`    |
| adaptive_is_advisory | unit | [[api.rate_limit.adaptive_limiting]] | `arbitrary_config()` | `varying(limit) never blocks a response` |
| rate_traces_resolve_prop | unit | [[api.rate_limit.rate_traces_resolve]] | `arbitrary_request()` | `every constraint trace resolves` |
| exceed_guard_reaches_prop | unit | [[api.rate_limit.exceed_guard_reaches]] | `arbitrary_request()` | `the exceed transition's guard reaches the enforced limit` |
```

The two `**kernel:**` rows are this example's executable slice, the
same shape the quick-start shows: `resolves(traces_to)` and
`reachable(…, guard)` evaluate over the file's own rows and references
(the min-expr kernel, `add-min-expr-kernel`), and `spk model-check`
reports each one's status in its run report — the domain-data cells
above them stay informal strings per §4.

The "stopgap... adaptive eventually" aside becomes the `advisory` row —
recorded, traceable, and non-gating by typing rather than left as a
sentence a linter can't see. Nothing about it can ever end up in a
`guard` field (`specodelic.md`'s Reference Typing table forbids it), so
it can never accidentally become a hard requirement by a later transition
citing it.

---

## 4. What *not* to force in

- **Don't invent a table for something already covered.** Before adding
  anything, check whether it's actually one of §2's patterns first — most
  things that look like a missing feature turn out to be an existing one
  aimed at the wrong section. (Left uncounted deliberately here, after
  this being stale once already — see `CHANGELOG.md` #28 — check §2's own
  headings for the current count rather than trusting a number restated
  in prose.)
- **`expr`/`generator`/`predicate` stay informal strings.** Formalizing a
  categorical law so a machine can execute it (rather than a human read
  it) is a theorem-proving project, not a specodelic extension — see
  `specodelic.md` Revision 6's closing note. Write the law clearly in
  prose-math; don't try to make the schema parse it.
- **The model checker is bounded.** `model_check.md` is explicit that
  unbounded checking never terminates. A Model section is for a
  *finite-state lifecycle* (a reducer's phases, a request's lifecycle) —
  it is not how you verify a property over an unboundedly deep recursive
  structure (an arbitrarily nested expression tree, an arbitrarily long
  pipe chain). That belongs in the Properties table instead, as a `law` or
  `unit` property with a generator that samples arbitrary depth/length —
  PBT doesn't need to terminate exhaustively the way model checking does.
- **If a domain still doesn't fit after checking §2**, that's a real gap —
  fold it back into `specodelic.md` as a new Revision the same session,
  per `AGENTS.md`, rather than working around it locally in one file.
