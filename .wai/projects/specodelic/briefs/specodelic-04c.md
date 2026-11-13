# Subagent brief: specodelic-04c — kernel vocabulary pack (owner-decided)

You are a subagent in repo `/var/home/sasha/para/areas/dev/gh/charly/specodelic`
(Rust CLI `specodelic`, alias `spk`). An orchestrator has claimed ticket
`specodelic-04c` and is hands-off: you own the implementation end-to-end.
The orchestrator verifies with real gates — never claim work the diff
doesn't contain.

## Decision of record (owner-countersigned)

Ship a **kernel vocabulary pack** — a `kind: profile` spec file in the
corpus, namespace `kernel` — so `orphan_vocabulary` treats `kernel.binding`
as enabled vocabulary. NOT a linter carve-out. Rationale: packs are the
declared extension path (packs spec: enablement by vocabulary use or a
declared `uses` edge, never by config, never by narrowing any base closed
set); bf5's fixture already treats a minimal kernel pack as the rule's
sanctioned remediation.

## Orientation (do this first)

- `export WAI_PROJECT=specodelic`
- Read: `bd show specodelic-04c`; `openspec/specs/packs/spec.md` (pack
  shape: six facet tables, namespace rules, `pack_shape`,
  `orphan_vocabulary` semantics); `src/packs.rs` — HOW pack vocabulary is
  collected (which facet tables/columns feed discovery; the namespace
  derivation; the column-header scan that flags `kernel.binding`); the
  exact minimal pack fixture in `tests/cli/parse_misc.rs`
  (`write_pack_file`, ~line 681 — six facet tables with two-column rows);
  `tests/cli/parse_misc.rs` ~1405 (bf5's kernel.binding compile-path test —
  keep it green); a deployed pack for style: `openspec/specs/numeric-predicates-pack/spec.md`
  or `quant-finance-pack/spec.md`; and the naming law (parent-dir id).
- `wai search "pack"`, `wai search "orphan_vocabulary"` — accumulated
  patterns.
- Start and execute the executor pipeline literally:
  `wai pipeline start tdd-ro5 --topic="specodelic-04c: kernel vocabulary pack"`
  Advance ONLY via `wai pipeline next` — never hand-edit run YAMLs.
- `export TMPDIR=/var/tmp/specodelic-xch1` (mkdir -p it) — MANDATORY.

## What to build

1. **RED first**: a test proving the enablement — e.g. a corpus/workspace
   fixture that carries a `kernel.binding` column and currently fails
   `orphan_vocabulary` (labeled remediation naming a kernel pack) now lints
   clean once the pack is discovered by scan alone, no config. Also pin
   discovery (`spk doctor`/packs listing names the new pack) and that a
   file WITHOUT the column still lints byte-identically.
2. **GREEN**: author the pack at `openspec/specs/kernel-pack/spec.md`
   (`id: kernel.pack`, `kind: profile`, EARS statement), five-layer clean
   (Constraints/Model/Properties with coverage + guards + a deriving
   property per constraint) and the SIX facet tables per `pack_shape`:
   Sections / Kinds / References / Checkers / Floors / Requires. Declare
   the `kernel.binding` vocabulary in the facet the discovery rule
   actually scans (verify in `src/packs.rs` — do not guess); Kinds rows
   must stay pack-qualified and must not reuse a base closed-set name;
   Floors rows use legal case labels; Requires pins the current base
   revision (20). If the corpus architecture requires the addition to ride
   a change delta, scaffold `openspec/changes/add-kernel-vocabulary-pack/`
   with the delta + `specs/kernel-pack/layer.md` sidecar and deploy via
   `just archive-change add-kernel-vocabulary-pack`; direct new-file
   authoring is acceptable only if the recipe path cannot express a new
   capability (record the reason). Never hand-edit EXISTING deployed
   specs.
3. Refresh the AGENTS.md managed block via `spk init --force` if the
   catalog changes (verify the diff is block/catalog-only; revert any
   `.genesis/tools.toml` no-op reorder rather than committing it).
4. `cargo fmt` before committing; `pretender check` green; targeted tests
   during red/green only (orchestrator owns full gates).

## Scope guard

- Allowed files: `openspec/specs/kernel-pack/spec.md` (new),
  `openspec/changes/add-kernel-vocabulary-pack/**` (if used),
  `openspec/specs/**` ONLY via `just archive-change`, `tests/**`,
  `.espectacular/` ONLY new contract TOMLs for YOUR change's scenarios,
  `AGENTS.md` ONLY via `spk init --force`.
- Never edit: other corpus files, `src/**` (this ticket is data/pack only —
  if the rule needs a code change to accept the pack, STOP and report the
  gap rather than widening scope), `specs/`, `pretender.toml`,
  `.wai/resources/**`, run YAMLs, `.beads/**`.
- Anti-goal: no linter carve-out, no base-set narrowing, no py/ts pull-in.

## Repo facts (boilerplate — applies to every ticket)

- **Commit hygiene**: `git status --short` before every commit; stage only
  your files (`git add <paths>`); blame only what the diff contains.
- **beads**: do NOT claim/close/update — the orchestrator owns bd state.
- **Do NOT push**; **do NOT run `wai close`**.
- **New source files** (if any) need Purpose/Responsibilities/Rationale
  headers. **Non-interactive shells**: `-f`/`-rf`/`-y`.
- **TMPDIR=/var/tmp/specodelic-xch1** for every cargo call.

## Report format (end with this — the orchestrator verifies against it)

## Report

**Commits**
- `<hash>` <message> — <what it does, one line>

**Enablement evidence**
- the before/after finding (rule id + remediation) and the test names

**Deviations** (or "none")

**Next**
- <exact next action for the orchestrator, or "ticket complete">