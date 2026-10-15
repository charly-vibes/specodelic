# Subagent brief: specodelic-1v30 — conform gate corpus scope + unknown flooding (gh-17)

You are a pi subagent in repo `/var/home/sasha/para/areas/dev/gh/charly/specodelic`
(Rust CLI `specodelic`, alias `spk`). An orchestrator has claimed ticket
`specodelic-1v30` and is hands-off: you own the implementation end-to-end.
The orchestrator will verify with real gates — never claim work the diff
doesn't contain.

## Orientation (do this first)

- `export WAI_PROJECT=min-expr-kernel`
- `bd show specodelic-1v30` — ticket carries a grounded Meter note
  (2026-10-15 @ a1bda72): friction 1 verified — `src/conform.rs:345` gate
  calls `orchestrate::run_lint_stage(std::slice::from_ref(spec))` = a
  single-spec slice, while `linter.total_refs` resolves corpus-wide, so a
  `[[ref]]` whose target lives in a sibling spec dangles in conform's gate
  but passes `spk lint openspec`. Green: corpus where a `[[ref]]` target
  lives in a sibling spec + `spk conform --oracle o.jsonl
  openspec/specs/<cap>/spec.md` exits 0. Test home:
  `tests/cli/conform.rs`. SCOPE NOTE from the ticket: friction 3 (trace
  schema docs) overlaps `specodelic-t0aj` — this ticket fixes gate scope +
  unknown flooding; t0aj owns the documentation.
- `gh issue view 17 --repo charly-vibes/specodelic` — full upstream report
  (frictions 1-3, and the "what worked well" section — do not regress the
  praised error-message behavior).
- `wai search "conform"` — accumulated patterns.

## Grounded code facts (verified live by the orchestrator)

- `conform::gate` (`src/conform.rs:~345`): the input gate (design D5)
  calls `crate::orchestrate::run_lint_stage(std::slice::from_ref(spec), &[])`
  — one spec, no corpus. Refusal shape: `ConformError { stage:
  "artifact_gate", message: "the file has lint findings — conform
  requires a lint-clean spec, verdicts are never derived from invalid
  artifacts (…; run: spk lint)" }`.
- `orchestrate::run_lint_stage` (`src/orchestrate.rs:143`) takes
  `specs: &[Spec]` — it is corpus-ready by signature; the bug is the
  caller slicing one spec.
- `cmd_lint` (`src/commands/corpus.rs:8`) is the reference for how the
  CLI discovers a corpus from a path: `parse_batch(paths, …)` walks
  directories recursively (hidden/build dirs skipped) and
  `lint::lint_all(&specs, &checklists)` lints the whole discovered batch
  — `total_refs` resolves across that batch. `openspec_tree_present()`
  exists there as the corpus-presence probe.

## What to build

### Friction 1 — gate scope (the meter's green criterion)

Conform's input gate must lint the file in the same corpus context `spk
lint` uses, not in isolation:

- On `gate(spec, out_dir)`, discover the corpus the file belongs to
  (walk up from the file's path to the corpus root — the same discovery
  discipline `spk lint` applies; deterministic, no env dependence). If
  no corpus root can be discovered, fall back to today's
  single-spec slice — that fallback is the labeled, documented behavior
  for an isolated file, not an error.
- Lint the discovered corpus (`run_lint_stage` over the parsed corpus
  specs). The gate verdict keys on THIS file's findings within that
  corpus lint: the gate passes when the target file is clean, fails with
  today's artifact_gate refusal when it is not. A finding in a SIBLING
  file must not fail this file's gate — but see invariant (3).
- Invariants:
  1. the meter's green: `[[ref]]` target in a sibling spec →
     `spk conform --oracle o.jsonl <file>` exits 0 (with the sibling
     present in the corpus root);
  2. no false green: the target file's own lint findings still refuse
     with the same stage/message/remediation shape;
  3. no silent degradation: if corpus siblings fail to PARSE, the ref
     resolution the corpus lint performs is against a partial corpus —
     surface that fact as a labeled warning (the existing warning
     channel), never a silent partial-corpus verdict;
  4. deterministic: same corpus root from the same file every run.
- Parse cost is acceptable (conform is a batch-style gate command), but
  keep the discovery bounded (reuse `parse_batch`'s machinery — do not
  fork a second walker).

### Friction 2 — unknown flooding (the ticket's "pick one")

Pick option (a) from upstream: an explicit structural-only mode reporting
`agree` for traces that conform to the transition relation even when the
guard cites a prose invariant (today: `unknown` /
`prose_claim_not_interpretable`). Design requirements:

- The DEFAULT verdict space is unchanged — closed-world, honest
  `unknown` for prose guards. The new mode is an explicit CLI flag (e.g.
  `--structural-only`; name it well) that the caller opts into; its
  semantics: prose-guarded traces whose step sequence conforms to the
  transition relation report `agree` WITH a reason string that still
  names the uninterpretable guard (the honesty survives — the verdict
  record must not claim the prose invariant was checked).
- The flag is declared in `--help`, and its effect is stated in the
  verdict reason. Doc prose beyond `--help` belongs to t0aj — do not
  write doc pages here.
- TDD both halves: red tests first in `tests/cli/conform.rs` —
  (1) sibling-ref corpus gate green (friction 1 meter), (2) target-file
  own-findings still refused, (3) structural-only mode flips the
  prose-guard `unknown` to `agree` with the guard-naming reason, (4)
  default mode unchanged (still `unknown`), (5) isolated-file fallback
  (no corpus root discovered) behaves as today.

## Hard scope guard

- Allowed files: `src/conform.rs`, `src/commands/corpus.rs` (only if the
  corpus-discovery machinery needs a small extraction for reuse — keep
  it minimal), `tests/cli/conform.rs`
- Never edit: `openspec/specs/`, `.espectacular/`, `pretender.toml`,
  `specs/` (the corpus), `.wai/resources/**`, this template.
- Respect shrink-only ratchets: run `pretender check` before committing; if a
  pinned source file breaches, move new tests to `tests/` rather than raising
  entries. `src/model_check.rs` is pinned (see `pretender.toml`).

## Repo facts (boilerplate — applies to every ticket)

- **Commit hygiene** (standing, every commit): before ANY `git commit`, run
  `git status --short` and stage only files YOU authored
  (`git add <paths>`, never bare `git add`); unstage foreign files.
  Attribute the message only to what the diff contains.
- **Output discipline**: every command emits through
  `genesis::guide::Output::emit`; errors carry a remediation hint.
- **beads**: issues live in `.beads/issues.jsonl`; commit the export after
  `bd close` (`bd export -o .beads/issues.jsonl` — stdout is the default,
  you must pass the repo path).
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
