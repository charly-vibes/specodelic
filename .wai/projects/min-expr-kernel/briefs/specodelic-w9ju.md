# Subagent brief: specodelic-w9ju — compile rustfmt-clean *_props.rs emission (gh-15)

You are a pi subagent in repo `/var/home/sasha/para/areas/dev/gh/charly/specodelic`
(Rust CLI `specodelic`, alias `spk`). An orchestrator has claimed ticket
`specodelic-w9ju` and is hands-off: you own the implementation end-to-end.
The orchestrator will verify with real gates — never claim work the diff
doesn't contain.

## Orientation (do this first)

- `export WAI_PROJECT=min-expr-kernel`
- `bd show specodelic-w9ju` — ticket carries a grounded Meter note
  (2026-10-15 @ a1bda72): `rustfmt --check --edition 2021
  specodelic/compile_props.rs` flags the one-lined `macro_rules!
  todo_predicate` (reproduced: the body churns every recompile); green is
  `spk compile openspec/specs/<cap>` && `rustfmt --check --edition 2021
  specodelic/*.rs` both exit 0. Test home: `tests/cli/compile.rs`
  (byte-stable emission contract).
- `gh issue view 15 --repo charly-vibes/specodelic` — upstream report with
  three observed churn classes (one-lined macro body, over-wide/extra-indented
  `todo_predicate!` calls, stray blank lines) and three candidate fix shapes.
- `wai search "compile emission"` — accumulated patterns.
- Start your tdd-ro5 run and execute each step literally:
  `wai pipeline start tdd-ro5 --topic="specodelic-w9ju: compile rustfmt-clean *_props.rs emission (gh-15)"`
  Advance ONLY via `wai pipeline next` — never hand-edit run YAMLs.

## What to build

`spk compile` emits generated predicate artifacts (`*_props.rs`) that are not
rustfmt-clean; every spec amendment leaves ~14 files of pure formatting churn
that fails downstream `cargo fmt --check` gates, forcing a manual
`rustfmt --edition 2021 specodelic/*.rs` post-pass.

Fix direction (upstream's "least surprising" option, chosen): make the
compile write path emit rustfmt-clean `*.rs` output.

Grounded code facts (verified live by the orchestrator):
- The `.rs` artifact string is built by `properties_to_proptest` in
  `src/compile.rs` (~line 853). The one-lined macro is at ~line 896
  (`macro_rules! todo_predicate {\n    ($reason:expr) => { todo!("{}", $reason) };\n}\n\n`).
- The write choke point is `write_artifacts` at `src/compile.rs:1341` —
  shared by `commands/corpus.rs:496` (the `spk compile` command path) and
  `orchestrate.rs:308`. Formatting here covers every emitter of `*.rs`
  artifacts; do NOT fork a second formatting path per caller.
- The `.toml`/`.check.json` companions are not rustfmt's business — only
  format the `*.rs` files.

Design guidance (decide the final shape yourself, but honor these):
- Prefer invoking `rustfmt` (edition 2021) on the generated source inside
  `write_artifacts` before the `std::fs::write`, with a graceful labeled
  fallback when the `rustfmt` binary is not found: write the unformatted
  output and keep the command succeeding — a missing rustfmt must never
  turn compile into a hard failure. Carry the fallback fact in the
  emitted path's remediation hint or a warning channel per the repo's
  output discipline (`genesis::guide::Output::emit`, errors carry
  remediation hints).
- Formatting by construction (hand-fixing the macro line, indentation,
  blank lines) alone is fragile across rustfmt versions — do not rely on
  it exclusively. If you ALSO fix the obvious construction defects (the
  one-lined macro), that is fine as long as the invariant is enforced by
  the actual rustfmt pass, not by hand-maintained strings.
- Byte-stable emission contract: `tests/cli/compile.rs` has byte-stable
  fixture expectations for the emitted artifacts. If formatting changes
  the bytes, update the fixture through the test-red step — never delete
  the byte-stability contract; the contract is "output is byte-stable
  across a RE-run", not "output never changes".
- TDD: red test first in `tests/cli/compile.rs` — run `spk compile` on a
  fixture spec (reuse an existing test fixture corpus dir if one exists;
  otherwise a minimal inline spec with a Properties table), then assert
  `rustfmt --check --edition 2021` over the emitted `*_props.rs` exits 0.
  Red because the current emission churns. Then implement, then tidy.
  Honor the ticket meter as the live verification: after green, run
  `spk compile openspec/specs/<a capability dir>` and
  `rustfmt --check --edition 2021 specodelic/*.rs` — both must exit 0.
  IMPORTANT: if you run the meter against the repo's own `specodelic/`
  artifacts, that regenerates corpus-committed artifacts — check
  `git status --short` after and restore/revert any churn to `specodelic/`
  that your commit should not carry (the committed artifacts must only
  change if the formatted bytes differ, which is expected exactly once
  with this fix — decide and state explicitly in your report whether the
  formatted `specodelic/*.rs` regen is included in your diff or left out;
  the orchestrator prefers INCLUDED, as one clean reformat commit).
- Do not chase upstream's clippy-knock-on item (#4) — that is downstream
  fmt-gate territory, out of scope.

## Hard scope guard

- Allowed files: `src/compile.rs`, `tests/cli/compile.rs`,
  `specodelic/*.rs` (only the rustfmt-induced regen churn, per above)
- Never edit: `openspec/specs/`, `.espectacular/`, `pretender.toml`,
  `specs/` (the corpus), `.wai/resources/**`, this template.
- Respect shrink-only ratchets: run `pretender check` before committing; if a
  pinned source file breaches, move new tests to `tests/` rather than raising
  entries. `src/model_check.rs` is pinned (see `pretender.toml`) — new
  model-check tests go in `tests/` integration files.

## Repo facts (boilerplate — applies to every ticket)

- **Commit hygiene** (standing, every commit): before ANY `git commit`, run
  `git status --short` and stage only files YOU authored
  (`git add <paths>`, never bare `git add`); unstage foreign files.
  Attribute the message only to what the diff contains.
- **Output discipline**: every command emits through
  `genesis::guide::Output::emit`; errors carry a remediation hint.
- **beads**: issues live in `.beads/issues.jsonl`; commit the export after
  `bd close` (`bd export` gotcha: run it or the close is invisible to git).
- **Do NOT push** — the orchestrator verifies and pushes.
- **Do NOT run `wai close`** — the orchestrator owns session close.
- **ah/espectacular**: read-only over `openspec/`; contract TOMLs for
  change-phase scenarios are orphans until archive — author them in the
  archive commit, never mid-change.
- **New source files** need Purpose/Responsibilities/Rationale headers
  (file-headers skill convention).
- **Non-interactive shells**: use `-f`/`-rf`/`-y` flags; commands may be
  aliased with `-i` and will hang on prompts.

## Report format (end with this — the orchestrator verifies against it)

## Report

**Commits**
- `<hash>` <message> — <what it does, one line>

**Gates run**
- `just test` → <result>
- `ah check` → <result>
- `openspec validate --all --strict` → <result>

**Deviations** (or "none")
- <any scope/plan deviation and why>

**Next**
- <exact next action for the orchestrator, or "ticket complete">
