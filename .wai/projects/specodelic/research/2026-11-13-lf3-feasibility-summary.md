---
tags: [pipeline-step:research, pipeline-run:epic-orchestrator-2026-11-13-specodelic-lf3-scoped-verify-migration]
---

# LF3 feasibility — summary, corpus sweep, runner-readiness prerequisite

**Ticket:** specodelic-lf3 (scoped-first) · **run:** 2026-11-13 ·
**Outcome:** **no file qualifies** — 0 of 537 corpus property rows are
honestly expressible under the runner as it exists at HEAD. Deliverable B
(the `verify-specs` gate wiring) is therefore **not** wired, and no corpus
cell was migrated. The blocker is filed as a prerequisite ticket.

## 1. Question

Which property rows can become executable `**rust:**` fragments today,
honestly — a real predicate over real generated values, not a `Just(name)`
scaffold or a `true` placeholder?

## 2. Measurements (all reproduced this run, `TMPDIR=/var/tmp/specodelic-xch1`)

| # | fact | evidence |
|---|------|----------|
| M1 | **The scratch crate has no dependency on the specodelic lib.** `src/verify.rs` `SCRATCH_MANIFEST` = `[dev-dependencies] proptest = "1"` and nothing else — a fragment cannot call the linter, the compiler, the graph builder or the pack scanner. | `src/verify.rs:1228` |
| M2 | **A named generator compiles to a constant.** `spec_gen::<name>()` is `Just("<name>".into())` — every property gets exactly ONE input, and it is the generator's own name. | `src/compile.rs` (`spec_gen` emission); artifact `specodelic/kinds_props.rs`; the failing block's shrunk input `minimal failing input: v0 = "word"` |
| M3 | **Fragments DO execute.** A `**rust:**` predicate compiles into the proptest artifact and the properties gate really runs it: `properties.state = "pass"` for a true fragment, `status: "properties_failed"` with the shrunk counterexample for a false one. The runner is not the blocker for *execution* — only for *reach*. | `spk compile` + `spk verify` on a scratch spec; blocked entries `properties.state` = `pass` / `failed` |
| M4 | **A law-kind row can never carry a fragment.** `fragment_law_rejected` refuses it, and a cell may hold at most one marker (`more than one executable-fragment marker`). Each named case needs its own body — a format Revision, not a runner change. | `spk compile` on a law row: `law-kind properties cannot carry **rust:** fragments`; two markers: `at most one executable fragment per cell` |
| M5 | **Fragments may not read files or spawn processes.** `FRAGMENT_BANNED` bans `std::fs`, `std::process`, `std::net`, `std::env`, `unsafe`, `extern`, `include!`, `include_str!`, `include_bytes!`, `asm!`, `Command` — in table cells AND in `## Fragments` helper bodies. A fragment therefore cannot read the corpus, and cannot shell out to `spk`. | `src/compile.rs` `FRAGMENT_BANNED` |
| M6 | **A fragment CAN run its own randomized strategy.** `proptest::test_runner::TestRunner::new(Config::default()).run(&strategy, …)` compiles and runs inside a fragment. So "pure Rust over generated values" is technically reachable — but the strategy then lives in the fragment, not in the generator cell, and the row's claim (about the tool) is still untested: the fragment would be a re-implementation of the checker, i.e. a tautology with a divergence risk. | probe crate: `TestRunner::new(Config::default())` (note: `new()` takes a `Config`) |

**Consequence of M1 + M2 + M5:** an honest fragment can only assert a
fact about data it constructs itself. Every corpus property row is a claim
about *the tool's* behavior (or about corpus content), so an honest
executable form must CALL the tool — which needs M1's seam — and must
receive *varied* inputs — which needs M2's generator vocabulary.

## 3. Per-file verdicts (deliverable A)

| file | property rows | expressible now | needs runner harness |
|------|---------------|-----------------|----------------------|
| `openspec/specs/specodelic/spec.md` | 30 | 0 | 30 |
| `openspec/specs/kinds/spec.md` | 17 | 0 | 17 |
| `openspec/specs/packs/spec.md` | 16 | 0 | 16 |
| `openspec/specs/compile/spec.md` | 35 | 0 | 35 |
| **total (owned scope)** | **98** | **0** | **98** |

Row-by-row tables: [specodelic](./2026-11-13-lf3-feasibility-specodelic.md) ·
[kinds](./2026-11-13-lf3-feasibility-kinds.md) ·
[packs](./2026-11-13-lf3-feasibility-packs.md) ·
[compile](./2026-11-13-lf3-feasibility-compile.md).

Blocker codes: **T** tool dependency · **F** corpus/fixture access ·
**G** generator vocabulary · **L** law-case fragments.

## 4. Fallback sweep — no corpus file qualifies either

The brief's fallback is "the smallest fully-expressible corpus file".
Swept all 48 corpus files:

| corpus file | property rows | law rows | rows naming a tool operation | fully expressible today? |
|-------------|---------------|----------|--------------------------------|--------------------------|
| `openspec/specs/acset-core/spec.md` | 27 | 0 | 11 | no |
| `openspec/specs/acset-writer/spec.md` | 13 | 1 | 12 | no |
| `openspec/specs/archive-companion/spec.md` | 11 | 0 | 7 | no |
| `openspec/specs/audit/spec.md` | 7 | 0 | 2 | no |
| `openspec/specs/bioimage-data-pack/spec.md` | 12 | 0 | 11 | no |
| `openspec/specs/compile/spec.md` | 35 | 0 | 29 | no |
| `openspec/specs/conform/spec.md` | 14 | 0 | 7 | no |
| `openspec/specs/data-lineage-pack/spec.md` | 9 | 0 | 9 | no |
| `openspec/specs/docs-site/spec.md` | 5 | 0 | 1 | no |
| `openspec/specs/doctor/spec.md` | 3 | 0 | 2 | no |
| `openspec/specs/embedded-guide/spec.md` | 5 | 0 | 2 | no |
| `openspec/specs/empirical-registry-pack/spec.md` | 11 | 0 | 10 | no |
| `openspec/specs/error-contract/spec.md` | 15 | 0 | 15 | no |
| `openspec/specs/errors/spec.md` | 14 | 0 | 14 | no |
| `openspec/specs/graph-views/spec.md` | 11 | 0 | 4 | no |
| `openspec/specs/graph/spec.md` | 10 | 0 | 5 | no |
| `openspec/specs/hooks/spec.md` | 6 | 0 | 4 | no |
| `openspec/specs/kernel/spec.md` | 6 | 0 | 1 | no |
| `openspec/specs/kinds/spec.md` | 17 | 1 | 17 | no |
| `openspec/specs/lint-findings/spec.md` | 2 | 0 | 2 | no |
| `openspec/specs/linter-coverage/spec.md` | 10 | 1 | 10 | no |
| `openspec/specs/linter-ears_syntax/spec.md` | 7 | 0 | 7 | no |
| `openspec/specs/linter-external_completeness/spec.md` | 11 | 1 | 11 | no |
| `openspec/specs/linter-failure_shape/spec.md` | 7 | 0 | 7 | no |
| `openspec/specs/linter-frontmatter/spec.md` | 6 | 0 | 6 | no |
| `openspec/specs/linter-graph_shape/spec.md` | 9 | 1 | 9 | no |
| `openspec/specs/linter-model_shape/spec.md` | 8 | 0 | 8 | no |
| `openspec/specs/linter-observability/spec.md` | 7 | 0 | 3 | no |
| `openspec/specs/linter-referential_integrity/spec.md` | 16 | 1 | 14 | no |
| `openspec/specs/linter-schema_shape/spec.md` | 15 | 0 | 14 | no |
| `openspec/specs/merge/spec.md` | 12 | 0 | 10 | no |
| `openspec/specs/migrate/spec.md` | 17 | 0 | 9 | no |
| `openspec/specs/model-check/spec.md` | 7 | 0 | 5 | no |
| `openspec/specs/model_check/spec.md` | 22 | 0 | 17 | no |
| `openspec/specs/numeric-predicates-pack/spec.md` | 11 | 0 | 10 | no |
| `openspec/specs/observability/spec.md` | 10 | 0 | 9 | no |
| `openspec/specs/orchestrate/spec.md` | 11 | 0 | 11 | no |
| `openspec/specs/packs/spec.md` | 16 | 0 | 12 | no |
| `openspec/specs/parse/spec.md` | 6 | 0 | 4 | no |
| `openspec/specs/quant-finance-pack/spec.md` | 6 | 0 | 4 | no |
| `openspec/specs/refactor/spec.md` | 6 | 0 | 4 | no |
| `openspec/specs/rename/spec.md` | 11 | 1 | 9 | no |
| `openspec/specs/skill-pack/spec.md` | 9 | 0 | 6 | no |
| `openspec/specs/spec-integration/spec.md` | 5 | 0 | 5 | no |
| `openspec/specs/specodelic/spec.md` | 30 | 1 | 29 | no |
| `openspec/specs/svc-contracts-pack/spec.md` | 7 | 0 | 4 | no |
| `openspec/specs/verification-claims/spec.md` | 5 | 0 | 2 | no |
| `openspec/specs/verify/spec.md` | 17 | 0 | 14 | no |

**Corpus total: 537 property rows across 48 files.**


**Why every file fails.** A file qualifies only if ALL of its property
rows are honestly expressible. Every file in the corpus carries at least
one row that asserts the tool's own behavior; the smallest candidates
fail on their first row:

| candidate | rows | blocking row | blocker |
|-----------|------|---------------|---------|
| `lint-findings` | 2 | `process_lifecycle_checked` — `check(file) == passed` | T |
| `doctor` | 3 | `p_mode` — `mode == expected(ws) ∧ mode ∈ envelope_data` | T |
| `quant-finance-pack` | 6 | `no_pack_no_change` — `lint findings byte-identical …` | T |
| `spec-integration` | 5 | `both_parsers_green` — `openspec validate … and spk lint both exit 0` | T |
| `verification-claims` | 5 | `freshness_checked` — `verify rejects stale or incomplete evidence` | T |

The `process_lifecycle_checked` row (`check(file) == passed`) appears in
21 of the 48 files; in the remaining 27 the tool-behavior rows are
equally explicit — the sweep column "rows naming a tool operation"
counts at least one such row in EVERY file, and the smallest candidates
above are listed with their specific blocking row.

## 5. Runner-readiness prerequisite (deliverable C)

What the harness must provide before any corpus property cell can be
migrated honestly. P1–P3 are the runner seams; P4 is a format Revision;
P5 is a scope-verdict gap.

### P1 — tool-dependency seam (the missing piece; nobody owns it)

`src/verify.rs` must let the scratch crate depend on the `specodelic`
lib, so a fragment can call the real checker instead of re-implementing
it. The lib already exposes everything needed (`pub mod lint`,
`pub mod compile`, `pub mod graph`, `pub mod packs`, `pub mod verify`,
`pub mod spec` in `src/lib.rs`), and `spec::parse_str(text) -> Spec` plus
`lint::lint_corpus(&[Spec]) -> Report` are exactly the surface a fixture
predicate needs.

Sketch: `SCRATCH_MANIFEST` gains a path dependency on the crate root
(the runner knows it via `env!("CARGO_MANIFEST_DIR")`); the fragment then
reads e.g.

```rust
**rust:** specodelic::lint::lint_corpus(&[specodelic::spec::parse_str(&@fixture).unwrap()])
        .issues.iter().any(|i| i.rule_id == "linter.model_shape")
```

Caveats to settle in the design: the scratch crate would rebuild the lib
on every verify run (cache the target dir — the runner already reuses
one); the lib's `deny.toml`/MSRV constraints apply to the scratch crate;
the corpus-wide nature of some rules (lint is corpus-wide) means a
fixture must be self-contained or the corpus must be supplied.

### P2 — generator vocabulary (owned by specodelic-jwjj)

A named generator must yield a REAL strategy, not `Just(name)`.
specodelic-jwjj already owns this ("extract rust generator-strategy half
into its own demand-gated change proposal", decision
`openspec/decisions/2026-11-12-rust-strategy-sequencing.md`), and its
demand gate — "the corpus property-cell migration (specodelic-lf3) is
authorized and real generators are needed" — is met by this run.

Natural fit with the existing machinery: `## Fragments` already carries
Rust **items** (a helper body may define `fn`s), so a generator cell
could name a helper fn that returns a typed
`impl Strategy<Value = String>`; `compile.rs` would emit that helper as
the block's strategy instead of `Just(name)`.

### P3 — fixture API

The corpus's generators name FIXTURES, not values:
`spec_file_with(transition.to_not_in_states: true)`,
`row_belonging_to_a_sixth_section("Actions")`,
`pack_pinned_to_older_revision()`, `mixed_claim_corpus()`. Nothing in
the tree turns such a name into a concrete spec/row today. Two shapes:

- **in-corpus fixtures** — a `## Fixtures` section (or a `tests/fixtures/`
  corpus) holding the offending spec text, referenced by name from the
  generator cell; the fragment parses it with `spec::parse_str` and calls
  the linter/compiler. Keeps the fixture next to the claim it serves.
- **builder API in the lib** — `specodelic::fixtures::spec_with(...)`
  returning a `Spec`; the fragment calls the builder, then the checker.
  Avoids duplicating fixture text in the corpus, at the cost of a lib
  API surface (a public fixture builder is a supported-API decision).

### P4 — law-case fragments (format Revision)

Law rows cannot carry a fragment at all (M4). `specodelic/spec.md`
(`rename_naturality`) and `kinds/spec.md` (`kind_shape_naturality`) each
carry one, so both files stay unmigratable even after P1–P3. A Revision
is needed that lets a law row carry one body per named case (e.g. a
per-case `**rust:**` cell), plus the compile-side emission for it. Until
then, those two files can only be migrated by dropping the law row —
which the append-only discipline forbids.

### P5 — properties-only scope verdict

`spk verify` reports the FIRST blocking stage per file, so a file with
properties passing but no model-check run reports `missing_model_run`,
not `properties_failed`. A scoped CI gate therefore needs either a
`--properties-only` mode (report the properties gate alone) or a script
over the JSON envelope that fails on any `properties.state == "failed"`.
Not a blocker for the migration itself; a blocker for the recipe.

## 6. Recommendation / sequencing

1. **specodelic-jwjj** (open, P3) — real generator strategies. Demand
   gate now met.
2. **New prerequisite ticket (filed this run)** — the tool-dependency
   seam + fixture API (P1 + P3), plus the properties-only scope verdict
   (P5).
3. **Format Revision** — law-case fragment bodies (P4), needed by the
   `specodelic` and `kinds` files specifically.
4. **Then** specodelic-lf3 resumes: migrate `compile` (35 rows, no law
   row) first, then `packs` (16), `kinds` (17), `specodelic` (30), wire
   `verify-specs` over that explicit list, and extend the list per run.

## 7. What this run deliberately did NOT do

- **No corpus cell migrated.** Every candidate cell would have been a
  `Just(name)` scaffold, a re-implementation of the checker (a
  tautology), or a `true` placeholder — all three are the anti-goals.
- **No `verify-specs` recipe wired.** With zero honestly executable
  cells, the recipe would fail on `properties_failed` for every migrated
  row, or pass only over vacuous cells — a false green.
- **No `src/**` change.** P1–P3 are runner changes; the brief scopes
  them out of this run.
- **No hygiene bypass.** `FRAGMENT_BANNED` is a substring ban and could
  be evaded (`use std::{fs::File}` does not literally contain
  `std::fs`); that evasion was deliberately NOT used — `fragment_hygiene`
  is defense-in-depth with a stated intent, and quietly routing around
  it would make the artifacts dishonest.
