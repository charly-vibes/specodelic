# Tasks: add-skill-pack

Each phase is one red→green→refactor cycle; the failing test is written
and observed to fail before the implementation that makes it pass.
Tidying commits are separate from feature commits. (Phases map 1:1 to
beads tickets `specodelic-*` carrying the `add-skill-pack` label.)

## 1. Pack core — tracked sources, `include_str!`, deterministic render

- [x] 1.1 **RED**: tests in `tests/skill_pack.rs`: `generate_pack` output
      is deterministic (two calls byte-identical); every file starts with
      the managed marker naming the regenerate command; the pack contains
      exactly `SKILL.md`, `references/format.md`, `references/example.md`;
      `pack_content_hash` is SHA-256 over sorted `path\0content` (known
      fixture vector). Run `just test-smart` — all new tests must fail
      (module does not exist).
- [x] 1.2 **GREEN**: create tracked sources (`skill-packs/specodelic-design/SKILL.md`
      — protocol prose from the grounded 2026-10-13 draft, corrected
      per the grounding session); `src/skill_pack.rs` with
      `generate_pack`/`pack_content_hash`; marker prepended at generate
      time (repo sources stay marker-free).
- [x] 1.3 **RED→GREEN**: render pipeline replaces
      `<!-- spk:render:* -->` placeholders in the tracked source with
      explain-derived lists (phase 2); unknown placeholder is a labeled
      generation failure, never a silent passthrough.
- [x] 1.4 **TIDY**: hash + collect helpers into one testable unit;
      dead-code and clippy sweep.

## 2. Explain-derived tables + membership gate

- [x] 2.1 **RED**: membership tests — every topic name, lint rule id, and
      EARS pattern name appearing in generated text resolves against the
      served `spk explain` tables; a deliberately dangling mention in a
      test fixture fails with the identifier named.
- [x] 2.2 **GREEN**: render `references/format.md` (EARS patterns, kind
      sets, typed-reference table, naming law) through the same code
      paths that serve `spk explain ears`/`kinds`/`references`; the
      topics line renders from the topic list table.
- [x] 2.3 **TIDY**: single render entry point shared by explain and pack
      generation (no table restatement in either direction).

## 3. Example embedding + lint gate

- [x] 3.1 **RED**: test — generated `references/example.md` equals
      `docs/src/examples/reservation-order.md` content plus the managed
      marker prefix; running the shipped linter over the generated
      example reports zero findings and zero warnings; a corpus change
      that dirties the example fails this test.
- [x] 3.2 **GREEN**: `include_str!` embed of the book example into
      `references/example.md`; wire the lint test through the same
      linter API `just lint-specs` uses.
- [x] 3.3 **TIDY**: none expected; verify the doc-examples gate and the
      new test cannot disagree (both parse the same file).

## 4. CLI verb — `spk skill install`

- [x] 4.1 **RED**: CLI tests — default target `~/.claude/skills`
      (parent dirs created), `--dir` override; overwrite refusal for a
      non-marker target file (exit 2, hint names the path, file
      untouched); marker-bearing overwrite succeeds; exit 0/1/2 per the
      error contract; envelope lists written paths + `pack_content_hash`.
- [x] 4.2 **GREEN**: implement the verb on the genesis CLI surface
      (envelope output, `--human` for TTYs); register in the command
      list, completions.
- [x] 4.3 **TIDY**: verb docs (`spk skill --help`), README command
      surface entry.

## 5. Doctor staleness check

- [x] 5.1 **RED**: tests — doctor over a fresh install → `pass`; over an
      older-binary install (mutated installed file) → `stale` with both
      hashes + regenerate command; over no install → `missing` with the
      install hint, doctor exit code unchanged; mutated non-pack files in
      the target dir do not flip `pass` to `stale`.
- [x] 5.2 **GREEN**: implement the check (key `spk.managed-skill-pack`)
      in the existing doctor report schema; check the default and
      `--dir`-recorded targets (installed envelope carries the target so
      doctor re-checks the right directory).
- [x] 5.3 **TIDY**: none expected.

## 6. Docs wiring + change completion

- [x] 6.1 Book thin-pointer page `docs/src/design-conversations.md`
      (links procedure source, install command, FAQ/design-canvas; quotes
      only a one-line summary — never duplicates protocol prose or the
      example); SUMMARY.md entry under Thinking in Specs; README + llms
      verb entries; `just docs-build` green (unresolved-include gate).
- [x] 6.2 `openspec validate add-skill-pack --strict`; `just ci` green;
      tasks in this file checked; change ready for the dual-format
      archive recipe (separate rollout ticket).
