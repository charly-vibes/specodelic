# Specs as the design substrate — a canvas for thinking about the system

The four layers are not documentation sections that later get
implemented. They are the place where the system is designed: the
contract you reason over while inventing, refining, and validating a
design. An implementation — in Rust, Python, whatever paradigm — is a
later, optional rendering of a model that already lives in the spec, and
it is the rendering that must keep proving itself, never the other way
around.

> **Provenance.** This page is distilled from three independent
> cold-reads of the project by external AI models (2026-10, against
> v0.7.0); the questions their humans asked seeded the
> [FAQ](faq.md), and the answers were re-grounded against the current
> corpus before being written down here. Where this page and the format
> disagree, the format wins — `spk explain format` is the machine
> truth these paragraphs compress, and [architecture](architecture.md)
> holds for the lint pipeline's pass structure.

## The shift: the spec is the artifact

Most design docs describe a system that lives somewhere else — in a
codebase, in someone's head, in a slide deck. A specodelic corpus makes
the design itself the load-bearing artifact:

- It is **lintable** — the closed schema means every design decision is
  in a place the toolchain can see.
- It is **connected** — the `[[wiki-link]]`s are typed foreign keys, so
  claims trace upward to the promise they serve and downward to the
  behavior that verifies them ([architecture](architecture.md)).
- It is **checkable** — the pipeline can challenge the design, not just
  its spelling ([the design loop](#the-design-loop)).

## Each layer is a question you must answer

The format's constraints are not arbitrary gatekeeping — each layer
forces one design question to have an explicit, wired answer. If you
skip the question, lint tells you where.

**Intent — what do we promise?** One EARS-patterned sentence in the
frontmatter. If a feature cannot be stated in one sentence, the feature
is not understood yet; the EARS patterns ([kinds and
statement](specs/kinds/spec.md), [EARS](specs/linter-ears_syntax/spec.md)) force
the promise to be about a system response with defined conditions, not a
vague ambition.

**Constraints — what must always hold, and what may be observed?**
Invariants and effects, each a table row tracing to the Intent it serves.
This is where you discover whether your invariants are actually
invariant, and which observable outcomes are *contractual* rather than
incidental ([specodelic.md](specs/specodelic/spec.md)).

**Model — what can happen?** A state machine: states, guarded
transitions. The guards must cite the Constraints — the checker demands
the wiring — so "the model says so" always decomposes into "these named
claims say so". Writing the Model is where you find out that your
constraints were silent about the failure paths
([linter.model_shape](specs/linter-model_shape/spec.md)).

**Properties — what would convince us?** Every constraint needs a
deriving property with a generator and a predicate ([coverage,
linter.coverage](specs/linter-coverage/spec.md)). A claim with no way to
challenge it is a wish, not a design decision.

## The design loop

`spk new` → `spk lint` → `spk compile` → `spk model-check` → `spk
verify` is usually presented as a CI pipeline. It works equally well as
a *thinking loop*, run while designing:

1. Draft the layers — the scaffold from `spk new` teaches its own
   tables ([quick start](installation.md)).
2. Lint early and often; every finding is self-describing, so each
   failed pass is one concrete design question ("this transition has no
   guard — what claim says when it may fire?").
3. Compile and model-check to let the machine explore what you wrote;
   a `counterexample_found` verdict is the highest-value design moment,
   not a failure — see the [better-design rung](verification-boundaries.md)
   and the [worked example](examples/worked-example.md), where the
   checker caught the author over-claiming a rejection state.
4. Verify what can execute; record what cannot, honestly
   ([verification boundaries](verification-boundaries.md)).

The loop converges on a corpus where every claim is wired, guarded, and
either checked or explicitly unchecked — a design that can be handed to
an implementer, human or agent, with an honest account of what has and
has not been challenged.

## Where the canvas ends

The format can force *accountable* design, not *good* design. A wrong
but internally coherent model lints clean (that is rung 2 of the
[guarantee ladder](verification-boundaries.md), not a defect); the
checkers can only challenge what you wrote, and a missing constraint is
the blind spot they cannot see past
([known limitations](verification-boundaries.md#known-limitations)).
The honest status of any corpus is the highest rung it has evidence for
— this page is the canvas, the ladder is the audit.
