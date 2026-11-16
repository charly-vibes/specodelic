# Specodelic

> A markdown specification format — one file gives you Intent,
> Constraints, a state Model, and Properties — and `specodelic` (alias `spk`), the CLI that
> lints, compiles, verifies, and refactors it.

> **Why:** specs written for LLM implementation drift from prose-shaped wish lists
> that can't be checked — specodelic gives agent-facing specs checkable structure
> (Constraints, state Model, Properties) plus a linter and verify pipeline, so a
> spec can fail CI instead of silently underdetermining the build.
>
> **Status:** [experimental](docs/src/status.md) · v0.14.0, self-hosting round in progress ·
> [Motivation & design](docs/src/index.md) ·
> [FAQ](docs/src/faq.md) ·
> [When it fits — and when it doesn't](openspec/USAGE.md#positioning-what-the-format-assumes--and-when-not-to-use-it) ·
> [charly-vibes Tool Ecosystem](https://charly-vibes.github.io/dulce-de-leche/ecosystem-map.html)

Specodelic (formerly `spec-format`) is a self-hosting specification
format: every file in [`openspec/specs/`](openspec/specs/) is a markdown
spec describing either the format or one check its linter performs,
written in the format itself. Start at
[`openspec/specs/specodelic/spec.md`](openspec/specs/specodelic/spec.md),
then [`openspec/STATUS.md`](openspec/STATUS.md) §1 for the primer.

## When to use

Use `specodelic` when **something mechanical will consume the structured
rows**:

- **an agent implements from the spec** and the code is regenerable — the
  linter catches shape drift, and `compile` emits the TOML / proptest / TLA+
  artifacts the build consumes;
- **several implementations or agents honor one contract** — a dangling
  `[[link]]`, an unguarded transition, or an uncovered constraint fails CI
  instead of living in review;
- **you want counterexamples before code exists** — `model-check` explores the
  state machine within a stated bound, and a refuted claim usually names the
  design lesson rather than a tool bug;
- **a change already lives in OpenSpec** — the dual-format layer keeps every
  delta lint-clean and `archive-companion` archives it without destroying the
  structured layer.

**Don't use it when** the spec will never be mechanically consumed: under
hand-maintained code the four layers are tax, not leverage — write an ordinary
design doc. Reach for a sibling instead:

- **spec↔test correspondence only** → use [`ah`](https://github.com/charly-vibes/espectacular)
  (espectacular): it checks deployed scenarios against the tests that run them,
  read-only, and owns nothing specodelic owns;
- **change management** (proposal → tasks → archive) → use [OpenSpec](https://openspec.sh):
  it owns the change workflow, and specodelic adds the checkable layer to its
  deltas rather than replacing them;
- **a runnable conformance harness** (each property bound to an executable
  contract test, a red contract un-pushable) → use `wanna`, which consumes a
  specodelic corpus; specodelic itself classifies recorded traces with
  `spk conform` and never executes your system.

The assumption it makes: **the spec is the design artifact and the
implementation is agent-authored, regenerable, machine-checked code.** The
honest one-line test — *will anything mechanical consume the structured rows?* —
and the cases where a prose design doc wins are in the
[positioning statement](openspec/USAGE.md#positioning-what-the-format-assumes--and-when-not-to-use-it).

## Install

```bash
V=$(basename "$(curl -fsSLI -o /dev/null -w '%{url_effective}' \
  https://github.com/charly-vibes/specodelic/releases/latest)" | sed 's/^v//')
TGT="$(uname -s | tr '[:upper:]' '[:lower:]')_$(uname -m | sed 's/^x86_64$/amd64/; s/^aarch64$/arm64/')"
curl -fsSL "https://github.com/charly-vibes/specodelic/releases/download/v${V}/specodelic_${V}_${TGT}.tar.gz" | tar xz
chmod +x specodelic spk && sudo mv specodelic spk /usr/local/bin/
```

Or via Cargo: `cargo install specodelic` (installs `specodelic` and the `spk` alias).
Homebrew, Scoop, and the per-platform archive list are in
[installation](docs/src/installation.md).

## Quick start

```console
$ spk new order.cancel --file order.cancel.md
created order.cancel.md
→ Run: specodelic lint order.cancel.md

$ spk lint order.cancel.md
lint: 1 file(s) linted, 0 finding(s), 3 advisory warning(s)
  advisory order.cancel.md [linter.placeholder_content] intent statement is a whole-cell placeholder: THE system SHALL …
  … (constraint and property placeholder advisories)
→ Run: specodelic graph
```

Exit code `0`: the scaffold teaches its own four-layer tables and lints clean
with three advisories naming the cells still to fill. Replace the placeholders,
rerun `spk lint`, and every finding names its `linter.<rule>` id and a one-line
semantics string. The full captured flow — scaffold → lint → graph → compile →
model-check — is in [installation](docs/src/installation.md).

## Common workflows

1. **Gate specs in CI** — `spk lint openspec/specs` → one summary line, exit `0`
   clean / `1` on findings. `spk orchestrate <paths>` → parse → lint → compile →
   model-check → verify, stage-gated: it stops at the first failing stage and
   names it.
2. **Explore a design before writing code** — `spk model-check order.cancel.md` →
   a `specodelic/<id>.check.json` report and an explicit outcome
   (`no_counterexample`, `exploration_only`, or a counterexample trace with the
   violated claim); a refuted claim exits `1`.
3. **Keep an OpenSpec change dual-format** — `spk archive-companion <change>` →
   stock `openspec archive` plus the archived `layer.md` sidecar re-attached, so
   the capability spec keeps its frontmatter and tables; a delta without a
   restorable sidecar is refused before the archive runs.

## Commands

| Command | What it does |
|---|---|
| `lint` | check the invariants; findings carry their `linter.<rule>` id and semantics |
| `graph` | derive the typed reference graph — edges, DOT, Mermaid, wiring views |
| `compile` | Constraints → TOML, Model → TLA+ module, Properties → `proptest!` blocks |
| `model-check` | explore the compiled state machine (stateright; TLC opt-in), write `.check.json` |
| `verify` | execute compiled property blocks and gate on the model-check outcome |
| `orchestrate` | parse → lint → compile → model-check → verify, gating each stage |
| `conform` | classify an external oracle's recorded traces against a spec (read-only) |
| `audit` | re-validate a recorded oracle corpus; `--before` diffs a prior report |
| `new` | scaffold a spec file from the four-layer template |
| `rename` | rename a row id and every `[[link]]` atomically, verified before writing |
| `merge` | detect id collisions, dangling renames, intersecting blast radii |
| `refactor` | advise tidy-first splits from fan-in evidence |
| `migrate` | wrap a plain OpenSpec delta in a `layer.md` sidecar; rekey or revision upgrades |
| `archive-companion` | archive an OpenSpec change with the dual-format layer preserved |
| `explain` | embedded format guide, offline (thirteen topics) |
| `guide` | closed value sets and the acset schema export as JSON |
| `parse` | export the parsed Spec IR for one file (syntax-only) |
| `doctor` | diagnose the workspace setup and block currency |
| `init` | write/refresh the `SPECODELIC` managed block in `AGENTS.md` |
| `hooks` | wire the dual-format gate into the repo hook chain (marker-guarded) |
| `skill` | install the managed `specodelic-design` skill pack |
| `completions` | generate shell completions |
| `feedback` | file an issue against upstream via `gh` |

Output follows the [genesis-vibes](https://github.com/charly-vibes/genesis)
envelope convention: JSON envelopes by default for agents/pipes,
`--human` for terminals (real report text, never a Rust Debug dump).
Every verb's full contract is in the [command reference](docs/src/commands.md);
`spk --help` and `<command> --help` are authoritative.

## FAQ

Cold-read questions are answered in [`docs/src/faq.md`](docs/src/faq.md) —
the page to open next:

- **Should I use this at all?** → the
  [positioning statement](openspec/USAGE.md#positioning-what-the-format-assumes--and-when-not-to-use-it)
  (assumption, honest test, when a prose design doc wins).
- **What does a green check actually mean?** →
  [verification boundaries](docs/src/verification-boundaries.md) — the guarantee
  ladder, rung by rung.
- **Which command do I run when?** →
  [the decision guide](docs/src/decision-guide.md).
- **How does this relate to OpenSpec and `ah`?** →
  [tool relationships](docs/src/tool-relationships.md).
- **Where do agents read docs without repo access?** → `spk explain` and the
  site's [`llms.txt`](https://charly-vibes.github.io/specodelic/llms.txt).

## Documentation

- Rendered book: <https://charly-vibes.github.io/specodelic/>
- Command status and dogfooding: [docs/src/status.md](docs/src/status.md)
- License: [LICENSE](LICENSE)
- Known corpus gaps (found by dogfooding `specodelic lint specs`) are tracked in
  beads: `bd list`.

Rendered docs are generated artifacts; the repository sources are
authoritative — edit the repo sources, not the rendered output.

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

## Domain packs

A workspace extends the format by adding `kind: profile` spec files (see the
[packs spec](openspec/specs/packs/spec.md)) — no config, no registry;
vocabulary use activates a pack's checks advisory-first. The six standard packs
and the bioimage D6 pilot ship in-repo:
[`packs/data-lineage.md`](packs/data-lineage.md) (a typed `## Data`
table with lineage edges; external data standards are bridged through
an opaque `binding` column, never absorbed),
[`packs/numeric-predicates.md`](packs/numeric-predicates.md) (a typed
`## Quantities` table with tolerance case-label floors; unit systems
are bridged through opaque `unit`/`domain` columns, never absorbed),
[`packs/empirical-registry.md`](packs/empirical-registry.md) (a typed
`## StatTests` table with the `empirical.statistic` kind and per-kind
case-label floors over `alpha`/`window`; statistical tests stay
authoritative outside the format, never absorbed),
[`packs/quant-finance.md`](packs/quant-finance.md) (a namespaced
`## Limits` table with the `quant.risk`/`quant.pricing` kinds and a
`capped_by` reference field; market-data conventions stay authoritative
outside the format, never absorbed),
[`packs/svc-contracts.md`](packs/svc-contracts.md) (a typed
`## Interfaces` table with the `svc.compat` kind and a `published_as`
reference field for cross-repo service contracts; OpenAPI/AsyncAPI/protobuf/
JSON-Schema artifacts stay authoritative outside the format, never
absorbed; still `draft` — activates advisory-first),
[`packs/kernel-pack.md`](packs/kernel-pack.md) (the `kernel.binding`
reference field and the `kernel.predicate` kind for the kernel's opaque
claim carrier and pack-side predicates; checker languages stay
authoritative outside the format, never absorbed), and
[`packs/bioimage-data.md`](packs/bioimage-data.md) — the D6 pilot: a
typed `## Axes` table with the `bioimage.transform` kind, the R2
data-shaped predicate grammar as named checkers, and the mechanism's
first cross-pack `## Requires` consuming `data.lineage`,
`numeric.predicates`, and `empirical.registry`;
OME/NGFF stays authoritative outside the format, never absorbed.

## Exit codes

Consumers distinguish "your spec is bad" from "you typoed the path":

- `0` — success (lint with zero findings counts)
- `1` — the stage produced findings, or a tool-level failure
- `2` — invocation error: nothing was processed (path not found, no spec
  files matched, unreadable input); argument-parse failures exit 2 too.
  The JSON envelope's `envelope_kind` agrees (`"error"`).

## Verification is not application testing

Specodelic's verification is bounded model checking of the invariant claims a
spec explicitly opts into, plus execution of the compiled property blocks — it
is not a substitute for the application's own test suite, and no proof of
application correctness is inferred from a `verified` verdict.

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
- ✅ `explain` — embedded AIX guide (thirteen topics:
  format/ears/kinds/references/lifecycle/lint-rules/dual-format/packs/
  graph-views/conform/oracle-records/verdict-taxonomy/claim-executable)
  plus `format_revision` in `--version --json`
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
- ✅ `skill` — `spk skill install` writes the first-party managed
  `specodelic-design` skill pack (protocol + format reference + the
  book's worked example) under `~/.claude/skills` or `--dir`; refuses
  to overwrite files without the managed marker, and the envelope
  carries `pack_content_hash` so `spk doctor` can flag staleness
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

How specodelic relates to OpenSpec (dual-format delta files) and to the
read-only `ah`/espectacular spec↔test correspondence companion:
[docs/src/tool-relationships.md](docs/src/tool-relationships.md).