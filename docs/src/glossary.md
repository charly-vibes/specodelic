# Glossary

The jargon this book (and the `specs/` corpus) uses, defined for a cold
read. Definitions are one to three sentences; where a deeper explanation
exists, the entry links to it instead of restating.

## The format and its files

- **spec / specodelic file** — one markdown file: YAML frontmatter plus
  fixed tables under a closed schema. Every file in [`specs/`](specs/specodelic.md)
  is a spec written in the format it describes — see
  [the core format](specs/specodelic.md).
- **frontmatter** — the YAML block at the top of a spec file carrying the
  file-level identity: `id`, `kind`, and `statement`. Checked shape-first
  by `linter.frontmatter`.
- **Intent** — the file-level layer: a real `id`, a `kind`, and one EARS
  `statement` saying what the spec promises.
- **four layers** — the fixed structure of every spec: frontmatter
  (Intent), **Constraints**, **Model**, **Properties**. The linter never
  reads prose — everything the toolchain reasons about lives in these
  structured rows. See [architecture](architecture.md).
- **EARS** — Easy Approach to Requirements Syntax: five fixed sentence
  patterns (Ubiquitous, Event-Driven, State-Driven, Unwanted-Behavior,
  Optional-Feature) with an imperative `SHALL`. A spec's `statement`
  must match exactly one pattern; conformance is shape-only grammar
  checking. `spk explain ears` has the pattern table.
- **kind** — the type tag on a spec file (`kind: intent`, `kind: profile`)
  or a table row (`invariant`, `effect`, `advisory`, `extension_point`,
  `deriving`, …), drawn from closed sets: a value outside the set is a
  labeled finding, never silently accepted. See [kinds](specs/kinds.md).
- **Constraints** — the table of the spec's claims (invariants, effects,
  advisories, extension points), with `[[wiki-linked]]` traces. Compiles
  to TOML.
- **Model** — the behavior table: states plus guarded transitions — a
  state machine. Compiles to a TLA+ module and is explored by
  `spk model-check`.
- **Properties** — the table of deriving properties, each with a
  generator + predicate. Compiles to proptest scaffolding and is
  executed by `spk verify`.
- **corpus** — the set of spec files a command operates on, usually a
  whole tree (`specs/`, or an `openspec/` tree in consumer repos). Lint
  discovers packs by scanning it; `spk graph` derives edges over it.
- **lifecycle** — the ordered stage gates a spec moves through —
  `parse → lint → compile → model_check → verify` — each stage gated on
  the previous one's clean run. `spk explain lifecycle` states the
  gates; `spk orchestrate` drives the whole run.
- **dogfooding** — the repo eating its own dog food: every file in
  `specs/` is a spec in the format specodelic defines, so the tool
  validates itself with every lint run (see self-hosting round, below).
- **naming law** — a spec file's frontmatter `id` equals its filename
  stem with `-` ⇔ `.`: `linter-graph_shape.md` ⇔ `id: linter.graph_shape`.
- **format_revision / Revision N** — the format's version stamp. Each
  corpus file states the current `Revision N` in its revision history,
  and the binary embeds `format_revision` (reported by
  `spk --version --json`); `spk doctor` warns when a corpus lags the
  binary, never fails.
- **wiki-linked reference** — a `[[id]]` link in a table cell pointing at
  another row or file. Typed reference fields are checked by
  `spk graph` and `linter.referential_integrity`.
- **link kinds** — the typed reference fields that connect rows:
  `traces_to`/`derives_from` trace provenance (`derives_from` is the
  coverage edge — a Property derives from its Constraint), `emits`/
  `observes` are a typed effect protocol between files (an observation
  is not a dependency), `satisfies` consumes a published
  `extension_point` contract, `supersedes` replaces an older row. The
  authority is specodelic.md's Reference Typing table.
- **pack** (domain pack) — a `kind: profile` spec file whose manifest
  tables declare typed sections, fiber kinds, and reference fields for a
  domain. Discovered by corpus scan (no config file); a consuming file
  opts in explicitly via a `uses` reference to the pack's id. See
  [packs](specs/packs.md).
- **fiber kind** — a kind declared by a pack. While the pack is active,
  the base closed kind set widens to base ∪ active-pack-fiber; the same
  token without an active pack fires a labeled finding.
- **external checklist** — a `*.checklist.md` file declaring items that
  must each be explicitly `covered` (mapped to real constraint/property
  rows) or `waived` with a rationale; checked by
  `linter.external_completeness`, which never gates a lifecycle stage.
- **dual-format** — one markdown file that parses as both a specodelic
  spec and an OpenSpec delta: the specodelic half (frontmatter +
  Constraints/Model/Properties) plus the openspec half (`## ADDED
  Requirements` mirrored verbatim in `## Requirements`). One file, two
  parsers, requirement text authored once. `spk explain dual-format`
  carries the protocol and the migration recipe.

## The toolchain and its output

- **`spk`** — the CLI binary; the crate is named `specodelic`.
- **finding** — one labeled rule violation a checker reports, carrying a
  `rule_id` (`linter.<name>`) and one-line semantics. Findings fail a
  stage (exit 1) and never ride a success-shaped envelope.
- **envelope** — the output wrapper every command emits through:
  JSON (`ok`, `envelope_kind`, `data`) for pipes, human-readable for
  TTYs. Failures carry a remediation hint. See [errors](specs/errors.md).
- **artifact** — a compiled output on disk: `<stem>.toml` (Constraints),
  `<stem>_props.rs` (proptest scaffolding), `<stem>.tla` (TLA+ module),
  and `<stem>.check.json` (the model-check run report).
- **scaffold** — placeholder rows or sections that a template or
  `spk migrate` inserts so a file lints clean immediately, to be replaced
  with real content while staying green.
- **staleness** — compiled artifacts carry digests or fingerprints of the
  spec they were compiled from. `spk verify` recomputes them and rejects
  stale, foreign, or malformed reports with a rerun hint — a stored
  report is never rewritten to manufacture evidence.
- **`scope_sha256`** — the digest in the `<stem>.check.json` run report
  binding the run's structured content and consumed compiled artifacts.
  Prose edits and input reordering preserve it; any invariant-content
  change breaks it — so a report can never be reused as evidence for
  different inputs. See [model_check](specs/model_check.md).

## Verification and checking

- **lint-clean** — the file passes the join of the gating linter
  checkers (exit 0 from `spk lint`). Rung 1 of the ladder; format-valid,
  not verified.
- **guarantee ladder** — the five rungs of what a green checkmark
  actually assures, from lint-clean to matches-oracle (and the
  better-design rung a counterexample puts you on). A file's honest
  status is the highest rung it has evidence for. See
  [verification boundaries](verification-boundaries.md).
- **coverage** — every Constraint row must have at least one Property
  deriving from it (`p.derives_from == c`), checked by
  `linter.coverage`. Uncovered constraints are invisible to the rest of
  the pipeline.
- **proptest** — Rust's property-based-testing library: a generator
  produces many random inputs and a predicate asserts a property over
  each. The Properties table compiles to proptest scaffolding;
  `spk verify` executes it and reports the shrunk minimal failing input.
- **TLA+** — a formal specification language for concurrent and
  stateful systems. The Model table compiles to a TLA+ module (`.tla`).
- **stateright** — the embedded Rust model-checking backend (the
  default): breadth-first exhaustive exploration of the compiled model
  within stated bounds (`--max-depth`, `--max-states`,
  `--timeout-secs`).
- **TLC** — the TLA+ reference model checker, opt-in as a JVM subprocess
  (`--backend tlc --tlc-jar <tla2tools.jar>`); a missing binary or jar
  is a `missing_checker` error, never a verdict.
- **pc-automaton** — how the compiled Model is interpreted for checking:
  a program-counter automaton whose state is an index into the Model's
  states and whose actions are the transitions, explored by stateright
  with exactly the semantics the committed `.tla` emission commits to.
- **kernel** — the opt-in `**kernel:**` marker placing an expression cell
  in a closed atomic grammar (equality, comparisons, bounded ∀/∃,
  reference atomics like `resolves`/`unique`/`acyclic`) that evaluates
  three-valued (verified / counterexample / unknown) during model
  checks. A cell outside the grammar is a labeled failure, never prose
  silently ignored. See [compile](specs/compile.md).
- **`kernel.binding`** — an opaque string (test node id, filter, or shell
  payload) on an invariant row, surfaced verbatim in compiled artifacts
  so *external* checkers (like `ah`) can bind their tests to spec
  claims; the toolchain never parses or interprets it.
- **claim partition** — the split a model-check run makes of each file's
  invariant-kind Constraints into a **required** set (those opted into
  evaluation: a Rust fragment, a kernel claim, or a citation) and an
  explicitly **unchecked** set (prose-only). The run's outcome is
  governed by the required set only — prose wording never implies
  evaluation. See [model_check](specs/model_check.md).
- **model-check outcomes** — the four terminal states of
  `spk model-check`: `no_counterexample` (every required claim verified
  over a completed bounded exploration), `counterexample_found` (a
  required claim was refuted), `timed_out` (the bound expired), and
  `exploration_only` (completed exploration over a corpus with no
  executable claims). The last two are honest non-failures — `spk
  verify`'s model gate is what rejects them as not clean.
- **counterexample** — a concrete input sequence a model checker found
  that violates a claim. The ladder's better-design rung treats it as
  the design lesson, not a tool failure; the
  [worked example](examples/worked-example.md) shows the precedent.
- **oracle** — the executable ground truth a spec's claims are measured
  against. Here that is the compiled property blocks and required
  invariant claims that `spk verify` executes; rung 4 of the ladder,
  **matches-oracle**, means exactly that they ran and passed.
- **SUT** — System Under Test: in classic property-based testing, the
  implementation the properties run against. In specodelic the checked
  "system" is the spec's own compiled model and claims, so a `verified`
  verdict attests about the specification, never about a deployed
  implementation — see [verification boundaries](verification-boundaries.md).
- **advisory** — a Constraint kind that records intent without enforcing:
  advisory rows can never gate a transition (by typing, not convention)
  and are never required in a model-check run — but their findings are
  still labeled, never silent.
- **fan-in / fan-out** — per-row counts of incoming and outgoing
  references in the derived graph. High fan-in flags a row too many
  others depend on — a `spk refactor` candidate.
- **dangling reference** — a `[[id]]` link pointing at nothing that
  exists. Detected by `spk graph` and `linter.referential_integrity`;
  a post-merge corpus with zero dangling references is the relint gate.

## Ecosystem and workflow

- **self-hosting round** — the `specs/` corpus is specodelic's own
  specification, written in the format it defines, so the tool lints and
  verifies its own documentation the way it would a user's spec. The
  round keeps that corpus lint-clean as the format grows, making the
  tool's own spec the first consumer of every new rule. See
  [status](status.md).
- **`ah` / espectacular** — the companion CLI enforcing spec↔test
  correspondence via scenario-contract TOMLs; read-only over
  `openspec/` — it never enforces against the `specs/` corpus.
