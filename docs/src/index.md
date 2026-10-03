# specodelic

**specodelic** (`spk`) is a specification format and toolchain for AI
agents: specs are plain markdown with YAML frontmatter and four fixed
tables — **Constraints**, **Model**, **Properties** — under a closed
schema. The linter never reads prose, so prose is free; everything the
toolchain reasons about lives in the structured rows.

The repo dogfoods itself: every file in [`specs/`](specs/specodelic.md)
is a spec written in the format it describes.

## The four layers

| Layer        | Carries                                                        | Checked by                    |
|--------------|----------------------------------------------------------------|-------------------------------|
| Frontmatter  | `id`, `kind`, `statement` (one EARS sentence)                   | `linter.frontmatter`          |
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
corpus scan). Four packs ship in-repo — the three standard packs
(`data.lineage`, `numeric.predicates`, `empirical.registry`) plus the
bioimage D6 pilot (`bioimage.data`, the first cross-pack `## Requires`);
see the [packs spec](specs/packs.md).

## For agents

No repo access needed: `spk explain` serves the distilled format guide
from inside the binary — eight topics (`format`, `ears`, `kinds`,
`references`, `lifecycle`, `lint-rules`, `dual-format`, `packs`), and every lint
finding carries its own `rule_id` and one-line semantics. See `llms.txt`
at the site root for a machine summary.

## Ecosystem

specodelic is part of the [charly-vibes tool
ecosystem](https://charly-vibes.github.io/dulce-de-leche/ecosystem-map.html)
— the ecosystem map shows how `spk` relates to the sibling tools
(`wai`, beads, openspec, genesis-vibes, and the checkers).

## Install

```sh
cargo install specodelic
```

Then `spk --version --json` (reports the embedded `format_revision`),
`spk new my.first_spec`, and `spk lint my.first_spec.md`.
