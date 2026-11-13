# Design: add-verify-runner-seams

## Context

`spk verify` executes the compiled `proptest!` blocks of a spec through
the runner seam in `src/verify.rs` (`CargoRunner`): the compiled
`*_props.rs` artifact is staged into a scratch cargo crate and run with
`cargo test`. The scratch crate's manifest (`SCRATCH_MANIFEST`) declares
`proptest` and nothing else — so a `**rust:**` fragment compiled into
that artifact cannot name the tool it is supposed to be checking.

The lf3 scoped run measured the consequence: 0/537 corpus property rows
are honestly expressible. Three seams are the blocker; the design below
settles each one.

## Goals / Non-Goals

- Goals: (P1) let a compiled fragment call the real tool; (P2) let a
  generator name a fixture and get a concrete input; (P3) let a scoped
  CI gate read the properties verdict without the model stage hiding it.
- Non-Goals: real generator strategies (specodelic-jwjj), law-case
  fragment bodies (a format Revision), the generator-adequacy gate
  (specodelic-l8l.10), the bulk corpus migration (specodelic-lf3),
  in-corpus fixture sections (a format Revision), and CLI flag wiring in
  `src/main.rs` / `src/commands/**` (outside this ticket's allowed file
  set).

## Decisions

### D1 (P1) — the scratch crate depends on the tool, but only when the artifact asks for it

The scratch manifest gains a path dependency on the specodelic lib:

```toml
[dependencies]
specodelic = { path = "/abs/path/to/specodelic" }
```

- **The dependency is CONDITIONAL on the artifact referencing
  `specodelic::`.** The runner scans the staged artifact source; when the
  token `specodelic::` appears, the tool dependency is added, otherwise
  the manifest is today's manifest byte-for-byte.
  - *Alternatives considered*: (a) always depend on the tool — rejected:
    every verify run would then compile the whole lib (clap, miette,
    stateright, genesis-vibes) into the scratch target dir, and a
    distributed binary (whose crate root does not exist on the user's
    machine) would turn EVERY verify into `properties_uncompilable`;
    (b) depend on the tool only when a fragment mentions it — chosen:
    the dependency exists exactly when it is needed, so the blast radius
    of a missing tool root is confined to the artifacts that call the
    tool.
- **The tool root is `env!("CARGO_MANIFEST_DIR")`** — the crate root of
  the specodelic package that built this binary — overridable with
  `SPECODELIC_TOOL_PATH` for a binary built elsewhere (a release
  tarball's crate root is not on the user's disk).
- **A missing tool root is a LABELED runner error, never a silent pass.**
  When the artifact needs the tool and the root is absent, the runner
  returns `RunnerError::Compile` carrying the reason (the artifact calls
  the tool, but the tool's crate root is not available at `<path>`) —
  the properties gate then reports `properties_uncompilable`, which is
  the honest shape for "the artifact could not be built here".
- **Offline behavior.** The scratch crate's `Cargo.lock` is seeded from
  the repository's own `Cargo.lock` when the tool dependency is needed,
  so cargo resolves the tool's transitive dependencies from the pinned
  versions instead of re-resolving against the registry index. Measured:
  a scratch crate with the tool dependency builds and runs
  `specodelic::lint::lint_corpus` with `--offline` in ~16 s.
- **Build cost.** The lib compiles once per machine: the scratch
  `CARGO_TARGET_DIR` is already shared across verify invocations, so the
  cost is a one-time build, not a per-run cost. The shared target dir
  keeps the incremental-GC race that `specodelic-68m.5` fixed for
  concurrent verifies (incremental stays off).
- **deny.toml / MSRV.** The scratch crate lives outside the repository,
  so `cargo deny` (advisories/licenses/bans) and the repository's MSRV
  gate are unaffected: the tool is a path dependency of a throwaway
  crate, not a new dependency of the shipped crate. The tool's own MSRV
  (1.88) governs the tool's own compilation.
- **The runner contract is unchanged.** The `PropertiesRunner` trait, the
  `RunReport`/gate shapes and the per-block results are exactly as
  before; only the manifest the production runner writes differs.

### D2 (P2) — the fixture API is a public module in the lib, and the emitted strategy materializes through it

The corpus's generators name fixtures, not values. Two shapes were on the
table:

- **(a) in-corpus fixtures** (a `## Fixtures` section or a
  `tests/fixtures/` corpus referenced by name from the generator cell).
  Rejected for this ticket: a `## Fixtures` section is a FORMAT Revision
  (the corpus grammar changes, the linter must learn the section, the
  format revision bumps) — explicitly out of scope here.
- **(b) a builder/registry API in the lib** (`specodelic::fixtures`).
  Chosen.

The decisive constraint is *where the materialization happens*: the
generator cell becomes a strategy inside the compiled artifact, and that
artifact runs in the scratch crate. Only code the scratch crate can
reach can materialize a fixture — and after D1, that is exactly the
tool's lib. A runner-side registry (the third candidate) could not
materialize a strategy that runs inside the scratch crate.

The API (each symbol justified):

| symbol | why it exists |
|---|---|
| `fixtures::registry() -> &'static [Fixture]` | the registry itself — the one list of named fixtures, inspectable by tooling and tests |
| `fixtures::lookup(name) -> Option<&'static Fixture>` | resolve one fixture by name without materializing it |
| `fixtures::materialize(name) -> Option<String>` | the concrete spec TEXT the emitted strategy yields — the one call the emitted artifact makes |
| `fixtures::names() -> Vec<&'static str>` | the fixture names, so a caller can check a name before using it |
| `fixtures::is_known(name) -> bool` | the compile-time question "is this generator name a fixture?" — the emission's branch condition |

`Fixture` carries `name`, `summary` (what the fixture is for) and `text`
(the concrete spec text). The text is a real four-layer spec file: the
real parser (`specodelic::spec::parse_str`) consumes it and the real
checkers (`specodelic::lint::lint_corpus`,
`specodelic::compile::properties_to_proptest`) judge it. **No fixture is
a test double and no fixture re-implements a checker.**

The emission branch: a generator whose name is a fixture name yields
`specodelic::fixtures::materialize("<name>")` — a real concrete input —
while every other generator keeps today's `Just("<name>".into())`
byte-for-byte. The corpus's current generator names collide with no
fixture name, so no existing artifact changes.

*Declared extension point*: a corpus-valued fixture (lf3's
`mixed_claim_corpus()`) needs a strategy that yields several specs; the
one-value-per-cell binding of the current property grammar cannot carry
it. That is a follow-up for lf3, declared here rather than faked with a
one-file fixture that pretends to be a corpus.

### D3 (P3) — the scoped verdict is a library-level verdict; the shipped CLI surface is the envelope's `properties.state`

`verify::verdict(properties, model)` reports the FIRST blocking stage,
with properties blockers ahead of model blockers. That is right for the
`verified` transition and wrong for a scoped CI gate: a file whose
properties pass but whose model run is missing reports
`missing_model_run`, and the recipe cannot tell "properties passed" from
"properties were never reached".

- **The scoped verdict is a separate function, not a flag on the full
  one.** `verify::properties_only_verdict(&PropsGateState) -> Verdict`
  maps the properties gate state alone:
  `Failed → properties_failed` (the same status, message and hint the
  full verdict uses), `Pass → properties_passed` — **never `verified`** —
  and every other state to its own status (`stale_properties_artifact`,
  `missing_properties_artifact`, `properties_uncompilable`,
  `runner_unavailable`, `properties_timed_out`).
- **`both_gates_required` is untouched.** The full verdict path is
  byte-identical: `verdict()` keeps its precedence, and the scoped
  verdict can never report `verified`, so no caller can mistake a
  properties-only pass for a verified file. The model gate's states
  (`Missing`, `Stale`, `NotClean`) never produce `properties_failed` —
  the scoped verdict does not read them at all.
- **The CLI surface.** The ticket's allowed file set excludes
  `src/main.rs` and `src/commands/**`, so no `--properties-only` flag is
  added here. The envelope already carries exactly the signal a scoped
  gate needs: each file's `properties.state` is `"failed"` if and only if
  a compiled block failed (`"pass"`, `"stale"`, `"missing_artifact"`,
  `"uncompilable"`, `"runner_unavailable"`, `"timed_out"` otherwise),
  and the model gate's state is a separate field. The `just
  verify-properties` recipe gates on that field: it fails iff some file's
  `properties.state` is `"failed"` — never on a skipped or stale model
  stage. The CLI flag remains a declared follow-up for whoever owns
  `src/commands/**`.
- **Where the scoped verdict lives.** The scope verdict and its selector
  live in the crate-root module `specodelic::scope_verdict`
  (`VerifyScope`, `verdict_in_scope`, `properties_only_verdict`), and
  `src/verify.rs` re-exports them, so the API is reachable both as
  `specodelic::scope_verdict::*` and `specodelic::verify::*`. The
  reason is a hard ratchet, not taste: `pretender.toml` caps every file
  at `file_lines_max = 2300` and says explicitly "do NOT raise
  thresholds to add new headroom" — and `pretender.toml` is outside the
  ticket's allowed file set, so the threshold cannot move. The P1 seam
  plus the P3 verdict would push `src/verify.rs` (already the corpus's
  largest file at 2205 lines) past the cap, and the only two ways out
  are moving the API out or deleting the rationale prose that makes the
  seam reviewable. Moving the API out keeps both. The public symbols
  added to the library root are exactly three: `VerifyScope` (the scope
  selector), `verdict_in_scope` (the scope-aware verdict), and
  `properties_only_verdict` (the scoped verdict itself) — the same
  three the design settled on, now reachable under the library root.

### D4 — the corpus demonstration is ONE row of `compile.md`, migrated honestly

`openspec/specs/compile/spec.md`'s `proptest_block_per_property` row
(`generator: arbitrary_property_row()`, `predicate:
count(compile(row).blocks) ≥ 1`) becomes:

- generator: `property_row_with_one_property()` — a fixture name, so the
  strategy yields the concrete fixture rather than the generator's name;
- predicate: a `**rust:**` fragment that calls the REAL compiler and the
  REAL block-metadata parser on the materialized fixture:
  `specodelic::verify::blocks_from_source(&specodelic::compile::properties_to_proptest(&specodelic::spec::parse_str(&v0).expect(...))).len() >= 1`.

The row's meaning stays the property it always was ("a property row
compiles to at least one proptest! block") — it is now checked against a
real input by the real compiler instead of asserted in prose. The
generator cell changes from the prose `arbitrary_property_row()` to the
fixture name (the honest label for what the strategy now yields), and the
mirrored scenario in the openspec face is updated in the same delta.

The other 34 rows of the file keep their `todo_predicate!` placeholders:
they fail honestly at execution, so the FILE's properties gate is
`properties_failed` — the migrated row's own block reports `passed` in
the per-block results. That is the honest state of the corpus today; the
bulk migration is lf3's next run, and this ticket deliberately does not
pre-empt it.

### D5 — the contract change is dual-format, deployed only through the archive companion

- `verify` gains one Constraint (`properties_scope_verdict`), one
  deriving Property, and one Requirement with its scenarios; `compile`'s
  migrated row and its mirrored scenario are carried by the change's
  layer sidecars.
- The corpus is edited ONLY through `just archive-change id=...` (the
  archive companion re-attaches the layer sidecars); the sidecars carry
  the full post-change layer tables.
- The new scenarios get contract TOMLs under `.espectacular/` bound to
  the tests that prove them.

## Risks / Trade-offs

- **Risk: the tool dependency makes the scratch build heavy.** Mitigated
  by the conditional manifest (only tool-calling artifacts pay) and the
  shared scratch target dir (the lib compiles once per machine).
- **Risk: a distributed binary cannot resolve the tool root.** Mitigated
  by `SPECODELIC_TOOL_PATH` and by the labeled `properties_uncompilable`
  failure — never a silent pass.
- **Risk: the fixtures module is a supported public API.** Accepted
  deliberately: the fixture text must be reachable from the emitted
  artifact, which runs in the scratch crate; the module's surface is
  five functions and one struct, each justified in D2's table.
- **Risk: the demonstration row's file-level verdict stays blocked.**
  Accepted and disclosed: 34 rows remain untranslated, and the migrated
  row's per-block result is the evidence.

## Open Questions

- Should the CLI grow a `--properties-only` flag? Deferred — it belongs
  to whoever owns `src/commands/**`; the envelope field plus the recipe
  is the shipped surface.
- Should the fixture registry grow corpus-valued fixtures? Deferred to
  lf3 (D2's declared extension point).
- Should real generator strategies replace the fixture-yielding
  strategy? That is specodelic-jwjj's demand-gated change; the fixture
  seam does not pre-empt it.
