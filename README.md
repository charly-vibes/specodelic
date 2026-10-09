# Specodelic

> A markdown specification format — one file gives you Intent,
> Constraints, a state Model, and Properties — and `specodelic` (alias `spk`), the CLI that
> lints, compiles, verifies, and refactors it.

> **Why:** specs written for LLM implementation drift from prose-shaped wish lists
> that can't be checked — specodelic gives agent-facing specs checkable structure
> (Constraints, state Model, Properties) plus a linter and verify pipeline, so a
> spec can fail CI instead of silently underdetermining the build.
> **Status:** [experimental](docs/src/status.md) · v0.7.0, self-hosting round in progress · [Motivation & design](docs/src/index.md) · [charly-vibes Tool Ecosystem](https://charly-vibes.github.io/dulce-de-leche/ecosystem-map.html)
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

Domain packs: a workspace extends the format by adding `kind: profile`
spec files (see the [packs spec](specs/packs.md)) — no config, no
registry; vocabulary use activates a pack's checks advisory-first. The
first three standard packs and the bioimage D6 pilot ship in-repo:
[`packs/data-lineage.md`](packs/data-lineage.md) (a typed `## Data`
table with lineage edges; external data standards are bridged through
an opaque `binding` column, never absorbed),
[`packs/numeric-predicates.md`](packs/numeric-predicates.md) (a typed
`## Quantities` table with tolerance case-label floors; unit systems
are bridged through opaque `unit`/`domain` columns, never absorbed), and
[`packs/empirical-registry.md`](packs/empirical-registry.md) (a typed
`## StatTests` table with the `empirical.statistic` kind and per-kind
case-label floors over `alpha`/`window`; statistical tests stay
authoritative outside the format, never absorbed), and
[`packs/bioimage-data.md`](packs/bioimage-data.md) — the D6 pilot: a
typed `## Axes` table with the `bioimage.transform` kind, the R2
data-shaped predicate grammar as named checkers, and the mechanism's
first cross-pack `## Requires` consuming the three standard packs;
OME/NGFF stays authoritative outside the format, never absorbed.

Every specced pipeline verb ships (`compile`, `model-check`, `verify`,
`rename`, `refactor`, `merge`, `orchestrate`, `migrate`, `parse`); each
verb's contract lives in its own spec ([`specs/compile.md`](specs/compile.md),
[`specs/verify.md`](specs/verify.md),
[`specs/orchestrate.md`](specs/orchestrate.md), etc.) — see `spk --help`
for the full list.

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

## Install

```bash
V=$(basename "$(curl -fsSLI -o /dev/null -w '%{url_effective}' \
  https://github.com/charly-vibes/specodelic/releases/latest)" | sed 's/^v//')
TGT="$(uname -s | tr '[:upper:]' '[:lower:]')_$(uname -m | sed 's/^x86_64$/amd64/; s/^aarch64$/arm64/')"
curl -fsSL "https://github.com/charly-vibes/specodelic/releases/download/v${V}/specodelic_${V}_${TGT}.tar.gz" | tar xz
chmod +x specodelic spk && sudo mv specodelic spk /usr/local/bin/
```

Or via Cargo: `cargo install specodelic` (installs `specodelic` and `spk`).

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
- ✅ `graph` — derived typed reference graph, dangling detection; text
  projections (`--format edges|dot|mermaid`) and the file-level
  `--view wiring` producer→consumer projection (see
  [graph views](docs/src/graph-views.md))
- ✅ `new`, `doctor`, `completions`
- ✅ `explain` — embedded AIX guide (nine topics: format/ears/kinds/
  references/lifecycle/lint-rules/dual-format/packs/graph-views) plus
  `format_revision` in `--version --json`
- ✅ `compile` — Constraints → TOML, Model → TLA+ module, Properties →
  proptest! scaffolding (backend-neutral ModelIR), byte-stable artifacts
- ✅ `model-check` — native stateright backend over the compiled model,
  exhaustive-within-bound runs, persisted `.check.json` reports with
  artifact provenance; executable `**rust:**` predicate fragments
  (specodelic.md Revision 15) are compiled and executed natively, so
  `no_counterexample` can be honestly earned — prose-guard-only models
  still report `invariants_checked: []` honestly
- ✅ `hooks` — `spk hooks install`/`uninstall` wire the dual-format gate
  (`spk lint openspec`) into the repo's pre-commit chain as a
  marker-guarded lefthook managed block — never claiming
  `core.hooksPath`, never writing foreign hook files; install reports a
  gate dry-run over the envelope (failing gate = warning + escape hint)
- ✅ `archive-companion` — archives an openspec change with the
  dual-format layer preserved (`openspec archive --skip-specs` +
  verbatim deploy of the archived deltas); fails closed on any delta
  lacking the layer, `--dry-run` previews the plan (GH#7)
- ✅ `verify` — combined gate: compiled `proptest!` blocks executed and
  the current model run's outcome checked (`no_counterexample` only —
  `exploration_only` is explicitly not clean); every opted-in invariant
  claim must be verified before a file can verify — prose-only
  invariants stay explicitly unchecked, and the JSON, human, and
  persisted report views name the same blockers — fails closed on every
  deviation
- ✅ `verification claims` — opted-in invariant claims are classified,
  evaluated with labeled unknown reasons, and aggregated by the
  required-claim gate; claim reports are versioned and bound to the
  structured corpus scope (stale or foreign evidence is a rerun, never
  an acceptance)
- ⏳ nothing — every specced command ships; see `spk --help` for the full
  verb list

**Verification is not application testing.** Specodelic's verification
is bounded model checking of the invariant claims a spec explicitly
opts into, plus execution of the compiled property blocks — it is not
a substitute for the application's own test suite, and no proof of
application correctness is inferred from a `verified` verdict.

Known corpus gaps (found by dogfooding `specodelic lint specs`) are tracked in
beads: `bd list`.