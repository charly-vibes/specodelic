# Tool relationships — specodelic, OpenSpec, and the `ah` companion

specodelic does not replace the tools a repo already uses for change
management — it plugs into them. Two relationships matter, and this
page names both honestly: the **dual-format protocol** that binds
specodelic files to the [OpenSpec](https://openspec.sh) change
workflow, and the **`ah` companion CLI**
([espectacular](https://github.com/charly-vibes/espectacular)) that
checks spec↔test correspondence *around* those files without ever
editing them.

| Tool | What it does | Who owns what |
|------|--------------|---------------|
| OpenSpec | proposal → tasks → archive change workflow | owns the `openspec/` directory structure |
| specodelic (`spk`) | the format + linter; every change delta is also a lint-clean specodelic file | owns the *content* of the dual-format layer |
| espectacular (`ah`) | spec↔test correspondence checking | **read-only over `openspec/`** — never edits a spec file |

## The OpenSpec dual-format relationship

This repo manages its engineering changes in openspec: a change lives
in `openspec/changes/<id>/` as a proposal plus task list, and when it
ships, `openspec archive` moves its deltas into the capability specs
under `openspec/specs/`. The dual-format protocol makes every one of
those files a **specodelic file at the same time**: one markdown file,
two parsers, requirement text authored once.

A dual-format file carries both grammars:

- the **specodelic half** — YAML frontmatter (a real `id` from the
  [naming law](specs/specodelic.md), `kind: intent`, one EARS
  `statement`) plus the `## Constraints`, `## Model`, and
  `## Properties` tables;
- the **openspec half** — `## Purpose`, a delta section such as
  `## ADDED Requirements`, and a sibling `## Requirements` section
  with identical text.

Two rules keep the halves honest:

- **Naming law**: openspec hard-requires the filename `spec.md`, so
  the id law derives a `spec.md` file's id from its *parent directory*
  name — `openspec/specs/ge-cli/spec.md` declares `id: ge.cli`.
- **No drift**: `linter.requirement_drift` fails when the delta
  requirements and the mirrored `## Requirements` section drift apart,
  and `linter.dual_format_valid` requires a capability spec under
  `openspec/specs/` to be dual-format at all.

The full protocol — the migration recipe, the delta-mirror rules, and
the exact gate commands — is served from inside the binary:

```sh
spk explain dual-format
```

That topic is the single source of truth for the protocol; this page
only orients you. When the protocol text and `spk explain dual-format`
disagree, trust the command — it ships with the linter that enforces
it.

**Why not plain `openspec archive`?** The stock archive command
regenerates spec files and destroys the specodelic layer (frontmatter
and the three tables). specodelic ships `spk archive-companion` — it
runs `openspec archive --skip-specs` and deploys the archived deltas
verbatim, fail-closed on any delta lacking the layer. `spk hooks
install` wires the dual-format gate (`spk lint openspec`) into the
repo's pre-commit chain, so an unmirrored or unconverted delta is
caught before commit, not after archive.

## The `ah` companion (espectacular)

[espectacular](https://github.com/charly-vibes/espectacular) is a
sibling tool in the charly-vibes ecosystem — a separate crate whose
binary is `ah` (see its [upstream
README](https://github.com/charly-vibes/espectacular) for install and
the full command table). It solves a problem specodelic deliberately
does not: **spec↔test correspondence**. A spec scenario is only as
good as the test that actually runs it, and `ah` keeps those two
honestly wired together.

How it works, in this repo:

- Each deployed scenario in `openspec/specs/<capability>/spec.md` is
  backed by a **contract TOML** at
  `.espectacular/<capability>/<scenario-id>.toml`, which declares the
  tests that verify that scenario.
- `ah check` validates every deployed scenario against its contract —
  and with `--run-tests` executes the declared tests.
- `ah doctor` diagnoses the setup; `ah explain <topic>` serves playbook
  guidance for a finding or workflow question.

The boundary is one sentence and it is a hard constraint, recorded in
this repo's [AGENTS.md](../../AGENTS.md) sibling-tool rules:

> espectacular is **read-only over `openspec/specs/` and
> `openspec/changes/`**: it reads the `#### Scenario:` blocks, checks
> correspondence, and reports — it never edits, regenerates, or
> archives spec files.

That is why the dual-format archive recipe above exists as the *only*
sanctioned write path for capability specs: the two tools split the
work cleanly, with specodelic owning content and espectacular owning
the correspondence check. Nothing in `ah`'s enforcement is
specodelic-owned, and no specodelic command changes `ah`'s behavior —
if a docs page or tool output ever claims otherwise, it is wrong; the
guards in this repo's CI (`scripts/guards/sibling-blockers.sh`) treat
boundary violations as bugs, not preferences.

## Reading order for an OpenSpec user

1. Run `spk explain dual-format` — the protocol, from the binary that
   enforces it.
2. Skim the [core format](specs/specodelic.md) — the four tables the
   dual-format layer adds to an openspec delta.
3. Read [verification boundaries](verification-boundaries.md) — what a
   lint-clean or verified file does *not* assure; the same honesty
   discipline `ah check` applies to spec↔test claims.
