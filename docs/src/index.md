# specodelic

**specodelic** (`spk`) is a specification format and toolchain for AI
agents: specs are plain markdown with YAML frontmatter (the
**Intent** layer) and three fixed tables — **Constraints**, **Model**,
**Properties** — under a closed schema. The linter never reads prose, so prose is free; everything the
toolchain reasons about lives in the structured rows.

The repo dogfoods itself: every corpus file in
[`openspec/specs/`](specs/specodelic/spec.md) is a spec written in the
format it describes.

Hit jargon? The [glossary](glossary.md) defines every term a cold read
needs, from EARS to the guarantee ladder.

**Is this for you?** The format presumes a spec-first workflow — the spec
is the design artifact and the implementation is agent-authored,
regenerable, machine-checked code. If humans hand-maintain the code and
nothing consumes the structured rows, the layers are pure tax, not
leverage. The full when-not-to-use statement (assumption, honest one-line
test, and the cases where an ordinary design doc wins) is in
[USAGE.md](specs/USAGE.md).

New to the format? Read the [worked example](examples/worked-example.md) —
one reservation/order spec built layer by layer, with every lint finding
and counterexample the author hit on the way.

Before trusting a green checkmark, read
[verification boundaries](verification-boundaries.md) — the guarantee
ladder from lint-clean to matches-oracle, and what each rung does *not*
assure.

Thinking about a design rather than code? [Specs as the design
substrate](design-canvas.md) is the canvas for that — and
[which command when?](decision-guide.md) routes any task to the right
verb. Questions a cold read asks are answered in the
[FAQ](faq.md).

## The four layers

| Layer        | Carries                                                        | Checked by                    |
|--------------|----------------------------------------------------------------|-------------------------------|
| Intent (frontmatter) | `id`, `kind`, `statement` (one EARS sentence)  | `linter.frontmatter`          |
| Constraints  | invariants and effects, `[[wiki-linked]]` traces                | `linter.schema_shape`, graph  |
| Model        | states + guarded transitions (a state machine)                  | `linter.model_shape`          |
| Properties   | deriving properties with generator + predicate                  | `linter.coverage`             |

## The pipeline

`spk` walks a spec file through stages, each with its own command:

1. **`spk lint`** — the gate: every rule, every file.
2. **`spk compile`** — Constraints → TOML, Properties → proptest
   scaffolding, Model → TLA+ module (all three artifacts, byte-stable).
3. **`spk model-check`** — the compiled model interpreted as a
   pc-automaton by the embedded stateright backend.
4. **`spk verify`** — executes the emitted proptest scaffolding.

Plus tooling around the corpus: `spk graph` (reference graph with
fan-in/out and dangling detection), `spk rename` / `spk merge` /
`spk refactor` / `spk orchestrate`, `spk doctor` (workspace diagnosis),
`spk explain` (the embedded format guide for agents), `spk new` (scaffold),
`spk init` and `spk feedback`.

**Domain packs** extend the format without a format revision: a
`kind: profile` manifest declares typed sections, fiber kinds, and
reference fields for a domain (no config file — lint discovers packs by
corpus scan). Six packs ship in-repo — the five standard packs
(`data.lineage`, `numeric.predicates`, `empirical.registry`,
`quant.finance`, `svc.contracts` — still `draft`) plus the bioimage
D6 pilot (`bioimage.data`, the first cross-pack `## Requires`); see
the [packs spec](specs/packs/spec.md) and the
[using packs guide](using-packs.md).

## For agents

No repo access needed: `spk explain` serves the distilled format guide
from inside the binary — ten topics (`format`, `ears`, `kinds`,
`references`, `lifecycle`, `lint-rules`, `dual-format`, `packs`,
`graph-views`, `conform`), and every lint
finding carries its own `rule_id` and one-line semantics. See `llms.txt`
at the site root for a machine summary.

## Ecosystem

specodelic is part of the [charly-vibes tool
ecosystem](https://charly-vibes.github.io/dulce-de-leche/ecosystem-map.html)
— the ecosystem map shows how `spk` relates to the sibling tools
(`wai`, beads, openspec, genesis-vibes, and the checkers). For the two
relationships that touch this repo daily — the OpenSpec dual-format
protocol and the `ah`/espectacular correspondence companion — see
[tool relationships](tool-relationships.md).

## Install

```sh
cargo install specodelic
```

Then `spk --version --json` (reports the embedded `format_revision`),
`spk new my.first_spec`, and `spk lint my.first_spec.md`.

## Documentation provenance

Rendered docs are generated artifacts; the repository sources are
authoritative — edit the repo sources, not the rendered output.
