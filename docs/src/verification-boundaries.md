# Verification boundaries — what a green check actually buys you

Every specodelic command can exit 0. An exit 0 from `spk lint` and an exit
0 from `spk verify` are **not the same guarantee**, and the toolchain never
collapses them into one `PASS` label: each stage attests something
different, and each stage's verdict is honest about what it did *not*
check. This page names the ladder rung by rung, then states the known
limitations up front — each one backed by a command you can run right now.

The canonical one-line statement of the widest boundary — **verification
is not application testing** — lives in the repo
[README](../../README.md); this page applies that discipline stage by
stage and does not restate it.

## The guarantee ladder

Read the ladder from the bottom up. Each rung presupposes every rung
below it — a file cannot be `matches-oracle` without first being
`checked-within-bounds`, and so on. A file's honest status is the
*highest rung it has evidence for*, never a single undifferentiated PASS.

| # | Rung | Gate | What passing assures |
|---|------|------|----------------------|
| 1 | **Lint-clean** | `spk lint specs` | The file is format-valid: all eight gating checkers pass |
| 2 | **Well-modeled** | lint's model checkers + a non-empty Model section | The behavior is stated as a state machine that is present and internally coherent |
| 3 | **Checked-within-bounds** | `spk compile`, then `spk model-check` | Bounded exploration of the compiled model found no violation within the stated bound — or honestly reported it could not finish |
| 4 | **Matches-oracle** | `spk verify` | The compiled property blocks executed and passed, and every invariant claim the spec opted into is verified |
| 5 | **Better-design** | a `counterexample_found` from `spk model-check` | The checker refuted a claim — and the counterexample is often the design lesson, not a tool failure |

### Rung 1 — lint-clean

`spk lint specs` exits 0: the file passes the join of the eight gating
checkers — frontmatter, EARS syntax, schema shape, coverage wiring,
graph shape, model shape, referential integrity, external completeness
(each has its own page under *Lint Rules*). `spk explain lifecycle`
defines this rung: *"`lint` is the join point of the linter checkers —
it requires every owned checker to pass."*

**Does not assure:** that the constraints are true, or that the
constraint list is complete. See the coverage blind spot below — this is
the rung where a MISSING constraint hides most comfortably.

### Rung 2 — well-modeled

The file carries a Model section that is present and coherent: a States
list and a Transitions table (`linter.model_present` — *"empty-but-present
beats absent"*), every transition guarded (`guard_present`), every state
used, every endpoint declared (`linter.model_shape`). This is the
precondition `spk model-check` imposes — *"`model_check` requires a
complete Model section (`model_present`)"* (`spk explain lifecycle`).

**Does not assure:** that the model says anything — an empty-but-present
model lints. And a coherent model can still be the *wrong* model; only
rung 3 puts it under exploration.

### Rung 3 — checked-within-bounds

`spk compile` (which *"`only runs on a file that has passed both lint and
coverage`"* — `specs/compile.md`, `precondition_satisfied`) emits the TOML,
TLA+, and proptest artifacts; `spk model-check` explores the compiled
model's reachable state space within a stated depth bound. A *clean*
verdict means no counterexample exists **within the bound**;
`spk model-check --help` is explicit that *exhaustive runs report
`exploration_only`, never `no_counterexample`* — a completed exploration
is evidence the space ends within the bound, never that the model holds
(`specs/model_check.md`).

**Does not assure:** anything beyond the stated bound, and — because the
program-counter model carries no data binding — nothing about values, only
about state names and transitions.

### Rung 4 — matches-oracle

`spk verify` executes the compiled proptest blocks and gates on the
model-check outcome. `spk explain lifecycle` states the full gate: *"`verify`
requires `no_counterexample`, `properties_pass`, and every opted-in
invariant claim verified — prose-only invariants stay explicitly unchecked
and are never counted as verified."* A `verified` verdict is the top of
what the toolchain can attest about a spec.

**Does not assure:** application correctness. The claims verified are
exactly the ones the spec opted into; prose invariants are not checked at
all. See the README's *Verification is not application testing* — a
`verified` spec is an audited specification, not a tested system.

### Rung 5 — better-design

When `spk model-check` returns `counterexample_found`, the ladder's
highest-value rung may not be "fix the model" but "fix the claim". The
[worked example](examples/worked-example.md) shows the precedent: the
author's executable invariant swore a rejection state was unreachable,
and the checker found a two-step trace reaching it — the rejection was
reachable *by design*, so the over-claiming row was the bug. As that
example puts it: *"The checker was the design critic; the counterexample
was the design lesson."*

**Does not assure:** that every counterexample is a design lesson — but a
refuted claim always names the violated invariant, so the diagnosis starts
from the named row, not from a diff.

## Known limitations

Each limitation below cites a runnable command; run it yourself — the
outputs shown were captured at v0.8.0.

### Coverage cannot detect a missing constraint

`spk explain lint-rules` prints the checker's one-directional semantics:

```text
| `linter.coverage` | every constraint must have a deriving property (`∃ property.derives_from == <constraint>`) |
```

Every constraint present must have a property — but nothing can tell you
a constraint *should* exist and doesn't. `specs/linter-coverage.md` says
this in so many words: *"a coverage check can only tell you every
constraint present has a property; it cannot tell you a constraint is
missing, which is exactly the class of gap the first four checkers caught
by inspection rather than by any mechanism this framework has yet."*
Coverage is a completeness check over what you wrote, not over what the
domain needs.

### `**py:**` and `**ts:**` fragments are not executable

The format grammar accepts both tags (`specs/compile.md`,
`fragment_language_closed` — the closed set is `{rust, py, ts}`), and a
`**py:**` fragment in a predicate cell lints clean. But no emitter has
shipped: at HEAD, `**rust:**` is the only executable tag. Save this
minimal spec (the whole file — same shape as the worked example):

```markdown
---
id: demo.py_spec
kind: intent
statement: "THE demo spec SHALL compile under py emission"
---

## Constraints

| id | kind | expr | traces_to |
|---|---|---|---|
| demo_py_c1 | invariant | `the demo spec compiles under py emission` | [[demo.py_spec]] |

## Model

### States
- `Idle`
- `Done`

### Transitions

| id | from | to | guard |
|---|---|---|---|
| t1 | Idle | Done | [[demo.py_spec.demo_py_c1]] |

## Properties

| id | kind | derives_from | generator | predicate |
|---|---|---|---|---|
| demo_py_p1 | unit | [[demo.py_spec.demo_py_c1]] | `proptest::collection::vec(any::<u8>(), 0..8)` | **py:** len(xs) <= 8 |
```

`spk lint demo.py_spec.md` passes clean — and then `spk compile
demo.py_spec.md` fails labeled, never a silent fall-through to Rust
emission:

```text
{"ok":false,"envelope_kind":"error","data":{"failed":[{"file":"demo.py_spec.md",
"message":"row `demo_py_p1`: no emitter for **py:** fragments — executable
emission for py is deferred of record (follow-up change:
add-py-fragment-emission); **rust:** is the only executable tag at this
Revision (no_emitter_labeled_failure, specodelic.md Revision 16)",
"stage":"compile.emission_failure"}]}}
```

The tag set is closed on purpose — an accepted tag without an emitter
would silently fall through to Rust, the vacuous outcome the closed set
exists to prevent (`specs/compile.md`, `no_emitter_labeled_failure`).

### Pipeline verbs vary in implementation maturity

Every specced command ships (`spk --help` — the README keeps the count
honest), but "ships" is not "uniform". What the binary actually does at
HEAD:

- **`spk refactor` advises — it never edits.** `spk refactor --help`
  opens: *"Advise on tidy-first splits for high unrelated fan-in nodes."*
  It flags split candidates via graph queries (`specs/refactor.md`);
  the split itself is yours.
- **`spk model-check` is bounded, and honest about it.** Exhaustive runs
  report `exploration_only`, never a fabricated clean
  (`spk model-check --help`, backend description); a backend that cannot
  execute Rust fragments reports `exploration_only`, never a fabricated
  pass (`specs/model_check.md`, `executable_invariants_execute`). TLC
  cannot execute Rust fragments at all and never claims clean.
- **`spk verify` never re-compiles.** It consumes the artifacts a prior
  `spk compile` wrote to the same out-dir (`spk verify --help`); a stale
  compile is a rerun, never an acceptance.
- **`spk compile` gates on lint.** A file with lint findings fails at
  `precondition_satisfied` with the fired rule ids named (`spk compile`
  on any unlinted file; lint is corpus-wide, so a file referencing other
  specs only compiles in whole-corpus context).
- **`spk lint`, `spk graph`, `spk rename`, `spk merge`, `spk orchestrate`**
  run as specced: lint is the corpus-wide join over the eight gating
  checkers, rename is atomic and verified against the linters
  (`spk rename --help`), merge is a pre-merge detector, orchestrate gates
  each pipeline stage in Checker Ownership order (`spk orchestrate --help`).

When in doubt about a verb's exact contract, the corpus file of the same
name under [`specs/`](specs/specodelic.md) — also served in this book
under *Pipeline* — is the source of truth, and `spk explain` works
offline.
