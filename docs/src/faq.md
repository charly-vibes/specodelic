# FAQ — the questions a cold read asks

These are the questions three independent cold reads of this project
(two external AI models in 2026-10, plus a third whose export is still
pending) actually asked while onboarding. Every answer either states an
editorial judgment or **links** to the authoritative page — answers never
restate machine-generated truth (`spk explain` topics, lint-rule
semantics, corpus-derived views), because a hand-copied restatement
becomes an unconsulted claim the first time the format revises.

For the design-oriented questions ("why do the layers impose these
constraints at all?"), see [specs as the design
substrate](design-canvas.md). For "which command when?", see the
[decision guide](decision-guide.md).

## Basics

### What is specodelic in one sentence?

A specification format and CLI toolchain for AI agents: specs are plain
markdown with a closed four-layer schema, checked by a linter that never
reads your prose. [The introduction](index.md) and the
[glossary](glossary.md) expand every term in that sentence.

### Is this a documentation format or a testing tool?

Neither, exactly — it is a **format with a checking pipeline**. The
structured tables are linted, compiled, model-checked, and verified; the
prose stays free for humans. What a green check attests — and what it
does *not* — is the subject of [verification
boundaries](verification-boundaries.md).

### Should I use this for my hand-maintained codebase?

Probably not. The format presumes **spec-first, agent-authored,
regenerable code** — its value comes from something mechanical consuming
the structured rows (compiled properties, the TLA+ model, the contract
gate). Under hand-maintained code the coverage/anchoring discipline is
pure tax: every third-party evaluation of the format reached this verdict
independently. The [positioning section of
USAGE.md](specs/USAGE.md) states the assumption, the honest one-line test
("will anything mechanical consume the structured rows?"), and the cases
where an ordinary design doc wins.

### Why can't I write prose and have the linter check it?

Deliberately not. The linter never parses prose, so prose is free to
argue, motivate, and explain. Anything the toolchain reasons about must
live in a structured row — a prose-only invariant stays explicitly
unchecked and is never counted as verified
([verification boundaries](verification-boundaries.md), rung 4), and
`spk conform` reports prose-covered traces as `unknown` rather than
silently passing them ([conform](conform.md)).

### Why is the schema closed? Why can't I add a fifth table?

A closed schema is what makes every other guarantee stateable: row kinds
are closed sets, every `[[wiki-link]]` is typed, and format evolution is
additive-only through numbered revisions — the theory behind this is in
[theory.md](specs/theory.md), the mechanism in
[specodelic.md](specs/specodelic/spec.md). Domain-specific structure goes
through [domain packs](specs/packs/spec.md), which extend the format without
a format revision.

## Checks and guarantees

### What does a green check actually mean?

It depends on which rung of the [guarantee ladder](verification-boundaries.md)
you are standing on: lint-clean, well-modeled, checked-within-bounds,
matches-oracle, better-design. Each rung presupposes the ones below it,
and each is honest about what it does not assure.

### Is my spec "done"?

A file's honest status is the highest rung it has *evidence* for, never
a single undifferentiated PASS. Lint-clean means format-valid; `verified`
means the compiled property blocks executed and every opted-in invariant
claim checked — and still not application correctness. Walk the ladder
in [verification boundaries](verification-boundaries.md).

### `model-check` returned a counterexample — is the tool broken?

Usually the opposite: a refuted claim names the violated invariant, and
the counterexample is often the design lesson rather than a tool
failure. The [worked example](examples/worked-example.md) records the
precedent — the checker found a two-step trace to a state the author
swore was unreachable, and the over-claiming row turned out to be the
bug.

### What can the checkers NOT find?

Known limitations are stated up front, each backed by a runnable
command — for example, coverage cannot detect a *missing* constraint.
See [verification boundaries — known
limitations](verification-boundaries.md#known-limitations).

## Workflows

### Which command should I run when?

The [decision guide](decision-guide.md) routes "what do I have, what do
I want" to the right verb — including the honest maturity notes on
advisory verbs like `spk refactor`.

### I have an existing system or oracle but no specodelic implementation — how do I check specs against it?

`spk conform` is the one verb that brings evidence from *outside* the
spec: it classifies traces recorded from a legacy system, a curated case
set, or a reference implementation against what the spec declares. The
verdict taxonomy (`permitted` / `forbidden` / `underspecified` /
`unknown` / `unsupported`) and the open- vs closed-world choice are in
[conform](conform.md).

### How do I get large numbers of scenarios out of a spec?

Two complementary verbs: `spk model-check` explores the compiled model's
reachable state space within a bound
([model_check](specs/model_check/spec.md)), and `spk verify` executes the
compiled property blocks whose generators produce concrete cases
([verify](specs/verify/spec.md)). For scenarios coming from *outside* the
spec, use [conform](conform.md) instead.

### Can one spec be implemented in different languages or paradigms?

The layers are implementation-neutral by construction — the pattern of
"several interchangeable implementations of one contract" is documented
in [USAGE.md §2.3](specs/USAGE.md). One honesty note: `**rust:**`
predicate fragments are compiled and executed natively by
`model-check`/`verify`; `**py:**` and `**ts:**` fragments are
illustrative and are *not* executable
([known limitations](verification-boundaries.md#known-limitations)).

## Format mechanics

### A lint finding says `linter.<something>` — what does it mean?

Every finding carries its own rule id and one-line semantics string.
`spk explain lint-rules` prints the catalog from the same table the
linter emits from, and each rule has a page under *Lint Rules* in this
book — start at [the linter overview](specs/specodelic/spec.md).

### How do I safely rename an id, or merge two diverged branches?

`spk rename` rewrites the defining row, every referencing `[[link]]`,
and checklist `mapped_ids` as one in-memory-verified transaction
([rename](specs/rename/spec.md)); `spk merge --branch` checks id collisions,
dangling renames, and intersecting blast radii *before* you join the
trees ([merge](specs/merge/spec.md)).

### Where are the docs agents read without repo access?

`spk explain` serves the distilled format guide from inside the binary —
nine topics (`format`, `ears`, `kinds`, `references`, `lifecycle`,
`lint-rules`, `dual-format`, `packs`, `graph-views`) — and `llms.txt` at
the site root is the machine summary ([For agents](index.md#for-agents)).

## Ecosystem

### How does specodelic relate to OpenSpec?

OpenSpec changes are the engineering-truth workflow; specodelic specs
share the same grammar via the dual-format protocol, and `ah`/
espectacular enforces spec↔test correspondence read-only. The full
division of ownership is in [tool relationships](tool-relationships.md).

### Can I extend the format for my domain?

Through a [domain pack](specs/packs/spec.md): a `kind: profile` manifest
declaring typed sections, fiber kinds, and reference fields — discovered
by corpus scan, no config file. Six packs ship in-repo — five standard
packs (`data.lineage`, `numeric.predicates`, `empirical.registry`,
`quant.finance`, `svc.contracts`) plus the first cross-pack `## Requires`
pilot (`bioimage.data`).
