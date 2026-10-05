# Generator-coverage gap — the vacuous-`Just(name)` window and the vocabulary direction

**Date:** 2026-10-04 · **Inputs:** external survey *"Type Systems and Property Testing"*
(Idris 2 QTT type-level PBT, Target/LiquidHaskell refinement-guided sampling,
underapproximate coverage types, QuickChick inductive generator derivation, DBAS
mixed embeddings) mapped against the repo; Rule-of-5 review of that mapping
(converged Stage 4; 3 HIGH findings TypeSafe-verified, measured FP rate 0%;
one finding — CORR-001 — below gate at 0.7, re-confirmed mechanically against
`specodelic/compile_props.rs:22-80`); review of the two in-flight openspec changes.

This note captures the *why* for a future change proposal. It is deliberately
**not** a spec delta — see §4 for why premature corpus edits would be
self-defeating.

---

## 1. The actual state (not hypothetical)

The article's coverage-types warning — *"a generator that returns a single
hardcoded constant may satisfy an over-approximating refinement type, yet fail
as a test generator"* — is the repo's as-built condition, not a latent risk:

- Every compiled strategy in `specodelic/compile_props.rs` (`spec_gen`, lines
  22–80) is `Just("<name>".into())` — one constant String per generator cell.
- Every predicate without a `**rust:**` marker is a `todo_predicate!`
  placeholder that fails honestly at execution.
- Revision 15 gave **predicates** an executable opt-in (`**rust:**` fragments,
  `predicate_fragment_opt_in` + four rejection rows + `fragment_hygiene`).
  **Generators have no analogous mechanism**: `properties_to_proptest`
  (specs/compile.md) turns the generator cell into the block's input strategy
  verbatim, and verbatim means a constant call.

Consequence: once a predicate opts in, its block tests exactly **one input**.
`properties_pass` on such a block is a vacuous pass.

## 2. The vacuous-gate window (lf3 interaction)

`specodelic-lf3` (open, P3) wires `spk verify specs` into `just ci`, blocked on
migrating the corpus's 18 `todo_predicate!` cells to executable `**rust:**`
fragments. After lf3 lands, the verify gate goes **green with all blocks
running constant generators** — the gate closes without anything meaningful
passing through it. The generator-vocabulary work is the missing half of lf3.
No issue owns that half today; this note is the pointer until the proposal
exists.

## 3. Relation to the in-flight change (D5)

`add-language-neutral-property-binding` already names and defers this exact
problem: *"the vacuous-`Just(name)`-generator problem deserves its own change"*
(proposal, Deferred section), with design **D5** recording the direction: a
small **language-neutral core vocabulary** (`int(range)`, `string`, `list(...)`,
`one_of(...)`) declared in-format, plus named project-level registries, landing
*with* the first non-Rust emitter.

D5 also **corrects** the investigation's first instinct (a `rust:`-analogue
fragment in generator position). Vocabulary-as-data beats fragment-generators
on both review findings:

- **Hygiene (EDGE-002):** vocabulary elements are data, not code — no arbitrary
  Rust construction to sanitize; statically lintable. The fragment option
  survives as the bespoke escape hatch inside the hybrid.
- **Coverage check tractability:** a closed vocabulary has enumerable
  structure, so a generator-completeness check can be structural over
  vocabulary elements — the same shape as `every_law_has_cases`'s
  `**name:**` label machinery — with **no runtime input observation, no SMT,
  no port of the article's coverage-type system**. The article's full
  under-approximate machinery is not KISS for file-shaped domains (no
  meaningful "boundary case" oracle); the repo's existing label-enumeration
  pattern captures most of the value at a fraction of the cost.

## 4. Scope decisions (KISS/YAGNI, from the review)

| Change | Verdict |
|---|---|
| Generator realness mechanism (D5 hybrid vocabulary) | **Keep** — the vacuous pass is actual, and there is no escape hatch: `compile_props.rs` is generated ("edit there, not here") |
| Generator-completeness check | **Keep, slimmed** — structural over vocabulary/labels, in the linter or as verify.md invariant; *not* `linter.coverage` extension (its `coverage_is_computable` invariant is AST-only; runtime realized-input coverage is a category error there) |
| Boundary-biased sampling (Target-style) | **Drop** — proptest already shrinks and range-samples; sparse-numeric-domain optimization, wrong shape for this corpus |
| Stateful proptest strategies from the Model table (QuickChick/Idris 2 pattern) | **Defer** — `model_check` already explores the state space exhaustively within a stated bound; redundant coverage until some spec's Model exceeds that. Trigger to revisit: a Model too large for exhaustive checking |
| Soundness/completeness property rows on `compile.md` | **Fold into the future change** — its new constraints need deriving properties anyway (`every_constraint_covered`) |

## 5. Why this note, not a spec edit (EDGE-001 recursion)

Premature corpus edits trigger the repo's own coverage law: any new constraint
row needs a deriving property, which compiles to a `Just(const)` block subject
to the very gap being fixed. The corpus also describes the tool as-built
(`just lint-specs` must stay clean); vocabulary constraints cannot be written
until the grammar they ride on (the in-flight change's closed tag set) is
merged. Spec edits belong **inside** the future change's openspec delta,
authored when sequenced.

## 6. Sequencing

```
1. add-language-neutral-property-binding  (grammar widening + pytest exemplar) — in flight
2. specodelic-lf3                         (predicate migration, wire verify)    — filed
3. generator-vocabulary change            (D5 hybrid + realness + case check)   — this note; NOT filed
```

Step 3's proposal should also carry a grill item inherited from the lf3
anti-goal ("never weaken the partial-pass-is-failure"): whether blocks with
non-executable or constant generators should be **labeled** in the verify
report (an `exploration_only`-style honesty marker, mirroring
`model_check.md`'s state) rather than silently green during the window between
lf3 and step 3. Labeling is honesty, not weakening — but it is lf3's call, not
this note's.

## 7. Decision of record / next step

When step 1 (and ideally lf3) land: author the openspec change proposal
carrying D5's hybrid vocabulary, the generator-realness compile path, the
structural case-check, and the §4 fold-ins — then create the bd ticket per the
`specodelic-gre` pattern (ticket references the approved change). Until then
this note is the durable pointer for the vacuous-gate window opened by lf3.