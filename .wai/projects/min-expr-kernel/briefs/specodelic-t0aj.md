# Subagent brief: specodelic-t0aj — spk explain conform topic (gh-18)

You are a pi subagent in repo `/var/home/sasha/para/areas/dev/gh/charly/specodelic`
(Rust CLI `specodelic`, alias `spk`). An orchestrator has claimed ticket
`specodelic-t0aj` and is hands-off: you own the implementation end-to-end.
The orchestrator will verify with real gates — never claim work the diff
doesn't contain.

## Orientation (do this first)

- `export WAI_PROJECT=min-expr-kernel`
- `bd show specodelic-t0aj` — the ticket description IS the content
  specification: five numbered sections (oracle JSONL schema, verdict
  taxonomy with decision rules, permitted-requires-executable-claims, the
  no-emits sentence, setup.state anchoring) plus the kernel-grammar
  fold-or-separate decision and the append-only/additive-only constraints.
  Meter: `spk explain conform` exits 0 and prints the embedded primer.
- `gh issue view 18 --repo charly-vibes/specodelic` — upstream asks (the
  reporter's 7-item reverse-engineering list; item 6's stderr-merge
  question was already resolved in the ticket grounding — stdout is clean
  JSON in -j mode).
- `wai search "explain"` — accumulated patterns.
- Read `src/guide.md` (topic table + one existing topic for the house
  style) and `src/conform.rs` (especially lines ~227-231: the
  deliberately-named-`observations` rationale you must surface), plus the
  `--structural-only` mode shipped by specodelic-1v30 (its `--help` text
  and the `structural_agree_prose_guard_unchecked` reason are the doc
  hooks — the topic must document BOTH the default honest `unknown` and
  the opt-in mode).

## What to build

1. **The `conform` explain topic** (`spk explain conform`) covering the
   ticket's five numbered sections, grounded in the ACTUAL current
   behavior (verify every claim against the live CLI before writing it —
   a doc that lies is worse than none):
   - Oracle JSONL schema: `{id, setup?, trace}` per line,
     TraceStep `{action, observations}`, observations is a MAP of
     free-var → declared state/effect id (string or array),
     `deny_unknown_fields`, and the deliberate `observations`-not-
     `observes` naming rationale (currently only in a source comment —
     surface it in the doc).
   - Verdict taxonomy WITH decision rules: not explainable by the Model
     transition relation → `forbidden`; undeclared action/observation
     vocabulary → `underspecified` (open world, never forbidden);
     covering claim expr is prose → `unknown` (never implies a
     judgment); covering claim `py:`/`ts:` → `unsupported` (no emitter
     this run); all covering claims executable AND the walk explains the
     trace → `permitted`.
   - The permitted-requires-executable-claims consequence: a fully
     prose spec can never yield `permitted`; the path is
     `rust:`/`kernel:`/citation-expr evaluators (cross-feature
     interaction conform × kernel markers × citation algebra).
   - One sentence: no `emits` column in the Model means every
     non-target observation is `underspecified`.
   - `setup.state` keys are free-variable names anchoring the trace to
     the States list; no setup → `underspecified` with the no-anchor
     reason.
   - The `--structural-only` opt-in (from 1v30): what it flips
     (`unknown` → `agree` for relation-conforming prose-guarded
     traces), what the reason string still names, and that the default
     mode is unchanged.
2. **Kernel grammar decision** (fold vs separate): prefer folding a
   short kernel-grammar subsection into the conform topic ONLY if it
   stays coherent; if it deserves its own topic, file a bd ticket
   (`specodelic-explain-kernel`) and note it in your report — do not
   silently drop the ask.
3. **Grounding discipline**: the primer is embedded text in
   `src/guide.md` — every verdict-name, field-name, reason-string, and
   flag-name in the doc must match the code (grep-verify). Topics are
   append-only (never renumber); shared guide code edits strictly
   additive (gre.9 scope guard). The meter: `spk explain conform` exits
   0 and prints the primer.
4. **TDD**: red test first in the explain-topic test file (find where
   existing topics are tested — likely `tests/cli/` or an inline test
   module): `spk explain conform` exits 0, names the topic, and carries
   the required content markers (e.g. `observations`, `underspecified`,
   `permitted-requires-executable`, `--structural-only`). Then write the
   topic, then tidy.

## Hard scope guard

- Allowed files: `src/guide.md`, the existing explain-topic test file
  (find it; likely `tests/cli/parse_misc.rs` or a dedicated one),
  `src/conform.rs` ONLY if a comment move/wording fix is needed to
  surface the `observations` rationale (keep minimal)
- Never edit: `openspec/specs/`, `.espectacular/`, `pretender.toml`,
  `specs/` (the corpus), `.wai/resources/**`, this template.
- Respect shrink-only ratchets: run `pretender check` before committing; if a
  pinned source file breaches, move new tests to `tests/` rather than raising
  entries.

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
