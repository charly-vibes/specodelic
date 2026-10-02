# Specodelic

> A markdown specification format — one file gives you Intent,
> Constraints, a state Model, and Properties — and `specodelic` (alias `spk`), the CLI that
> lints, compiles, verifies, and refactors it.

> **Why:** specs written for LLM implementation drift from prose-shaped wish lists
> that can't be checked — specodelic gives agent-facing specs checkable structure
> (Constraints, state Model, Properties) plus a linter and verify pipeline, so a
> spec can fail CI instead of silently underdetermining the build.
> **Status:** [experimental](docs/src/status.md) · v0.4.0, self-hosting round in progress · [Motivation & design](docs/src/index.md) · [charly-vibes Tool Ecosystem](https://charly-vibes.github.io/dulce-de-leche/ecosystem-map.html)
Specodelic (formerly `spec-format`) is a self-hosting specification
format: every file in [`specs/`](specs/) is a markdown spec describing
either the format or one check its linter performs, written in the
format itself. Start at [`specs/specodelic.md`](specs/specodelic.md),
then [`specs/STATUS.md`](specs/STATUS.md) §1 for the primer.

## The four layers

Every spec file is the same four sections:

| Layer | Where | Purpose |
|---|---|---|
| **Intent** | YAML frontmatter (`id`, `kind`, `statement`) + prose | Human-readable purpose; EARS-pattern statement |
| **Constraints** | `## Constraints` table | Invariants, each tracing to the Intent |
| **Model** | `## Model` — States + guarded Transitions | A finite state machine |
| **Properties** | `## Properties` table | PBT-style checks deriving from Constraints |

Humans read markdown; machines parse frontmatter and fixed-schema tables
only — prose is never inspected.

## The `specodelic` (alias `spk`) CLI

```console
$ specodelic lint specs      # check the invariants (linter-*.md)
$ specodelic graph specs     # derive the typed reference graph
$ specodelic new order.cancel --file order-cancel.md   # scaffold a spec
$ specodelic explain lint-rules   # embedded format guide + lint rule catalog
$ specodelic init            # write the SPECODELIC rules block into AGENTS.md
$ specodelic doctor          # diagnose the workspace + block currency
$ specodelic feedback bug --dry-run   # file an issue against upstream
$ specodelic hooks install   # wire the dual-format gate into the pre-commit chain
$ specodelic archive-companion my-change   # archive with the dual-format layer preserved
```

Pipeline commands (`compile`, `model-check`, `verify`, `rename`,
`refactor`, `merge`, `orchestrate`) are specced in
[`specs/compile.md`](specs/compile.md),
[`specs/rename.md`](specs/rename.md),
[`specs/orchestrate.md`](specs/orchestrate.md), etc. — not implemented
yet; track progress with `bd ready`.

Output follows the [genesis-vibes](https://github.com/charly-vibes/genesis)
envelope convention: JSON envelopes by default for agents/pipes,
`--human` for terminals (real report text, never a Rust Debug dump).

**Exit codes** — consumers distinguish "your spec is bad" from "you
typoed the path":

- `0` — success (lint with zero findings counts)
- `1` — the stage produced findings, or a tool-level failure
- `2` — invocation error: nothing was processed (path not found, no spec
  files matched, unreadable input); argument-parse failures exit 2 too.
  The JSON envelope's `envelope_kind` agrees (`"error"`).

## Development

```console
$ just ci        # fmt-check + clippy + tests + release build
$ just test      # unit + integration tests
$ just lint-specs  # dogfood: lint the repo's own spec corpus
```

Requires Rust 2024 edition. Depends on
[`genesis-vibes`](https://crates.io/crates/genesis-vibes) for shared
CLI/envelope/self-healing infrastructure.

## Status

- ✅ `lint` — frontmatter, filename↔id mapping, id uniqueness, guards,
  EARS grammar, coverage, total reference resolution; every finding is
  self-describing: it carries its `linter.<name>` rule id and a one-line
  semantics string (rendered by `specodelic explain lint-rules` from the
  same table the linter emits from)
- ✅ `graph` — derived typed reference graph, dangling detection
- ✅ `new`, `doctor`, `completions`
- ✅ `explain` — embedded AIX guide (format/ears/kinds/references/
  lifecycle/lint-rules topics) plus `format_revision` in `--version --json`
- ✅ `compile` — Constraints → TOML, Model → TLA+ module, Properties →
  proptest! scaffolding (backend-neutral ModelIR), byte-stable artifacts
- ✅ `model-check` — native stateright backend over the compiled model,
  exhaustive-within-bound runs, persisted `.check.json` reports with
  artifact provenance (no executable predicate language yet — reports
  `invariants_checked: []` honestly)
- ✅ `hooks` — `spk hooks install`/`uninstall` wire the dual-format gate
  (`spk lint openspec`) into the repo's pre-commit chain as a
  marker-guarded lefthook managed block — never claiming
  `core.hooksPath`, never writing foreign hook files; install reports a
  gate dry-run over the envelope (failing gate = warning + escape hint)
- ✅ `archive-companion` — archives an openspec change with the
  dual-format layer preserved (`openspec archive --skip-specs` +
  verbatim deploy of the archived deltas); fails closed on any delta
  lacking the layer, `--dry-run` previews the plan (GH#7)
- ⏳ nothing — every specced command ships; see `spk --help` for the full
  verb list

Known corpus gaps (found by dogfooding `specodelic lint specs`) are tracked in
beads: `bd list`.