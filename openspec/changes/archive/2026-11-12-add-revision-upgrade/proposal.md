# Add: `spk migrate --to-revision <N>` — upgrade a corpus across the known format revisions

## Why

The format corpus has nineteen revision headings (Revision 2 … Revision 20)
and every one of them is a *compatibility* statement: a corpus authored
against Revision 12 must still be readable — and upgradable — by the
Revision 20 tool. Today the tool can only migrate two shapes:

- the plain-delta → `layer.md` sidecar wrap (Revision 19), and
- `--rekey`, the one-file `id: spec` → real-id rewrite (Revision 18).

Everything else is a per-file sed one-liner the adopter writes by hand.
The audit that produced this ticket (specodelic-75m) recorded exactly
that finding: **`spk migrate` has no path across the revision history** —
the one revision that carries a corpus-level mechanical transform
(Revision 18's id law) has a single-file tool, and the revisions that
changed the corpus's *shape* in other ways have nothing at all.

The missing piece is not a pile of per-revision sed scripts. It is a
**thin, core-owned registry** that says, for every known format revision:
what it was, when it landed, and — only where a real corpus rewrite
exists — the mechanical step that carries a file across it. A revision
whose change was additive (a new rule, a new field, a reworded
constraint) has no corpus rewrite at all: its step is a byte-stable
no-op, and that is the *honest* entry, not a missing one.

## What Changes

- **`spk migrate --to-revision <N> [--from-revision <R>] [--write]
  <paths...>`** — a new mode on the existing `migrate` verb:
  - The known revisions resolve from a **core-owned registry** (a
    `const` manifest in `src/migrate.rs`): each entry carries the
    revision number, its date (where the format corpus records one), a
    one-line change note, and an **optional** migration step. Revision
    18 is the only entry with a step today (the id law); every other
    revision upgrades as a byte-stable no-op.
  - The upgrade applies the registered steps for the revisions *between*
    the corpus's source revision and the target revision, in ascending
    revision order. A step is **shape-guarded**: a file that does not
    carry the step's pre-shape is not rewritten by it.
  - The source revision is either declared (`--from-revision`) or
    **detected per file**: a corpus file that still declares `id: spec`
    satisfies Revision 17 and no later; a file that already carries a
    real id satisfies every registered step. The corpus's source
    revision is the oldest revision any file in the set still satisfies.
  - **Dry-run by default**: the upgrade previews (the would-be content
    rides in the envelope) and writes nothing; `--write` (alias
    `--force`) applies it. `--write` and `--dry-run` together are
    refused.
  - **Refusals (exit 2), never rewrites**: an unknown source or target
    revision (the message names the known range and a remediation hint);
    a target below the corpus's current revision (the message names the
    current revision and the file that still carries the older shape);
    a file whose id cannot be derived.
  - **Byte stability**: a file that no step changes is never written —
    its bytes, line endings (CRLF included) and final terminator are
    preserved exactly.
  - **Scope confinement**: only the files under the supplied paths are
    read and written; the upgrade creates no file, so it can never
    write a sibling `layer.md` the way the wrap path does.
  - **Atomicity**: the upgrade is all-or-nothing — when any file in the
    set cannot be upgraded, every file stays unchanged.

- **The migrate capability's intent statement** gains a clause naming
  the revision-upgrade path (the deployed `migrate` spec's frontmatter
  statement is the capability's one-line EARS intent).

This is an **addition** to the `migrate` capability. The wrap path and
`--rekey` keep every existing behavior byte-for-byte: `--rekey`'s
refusal set, exit codes, byte-preservation rules and envelope fields are
untouched, and the Revision 19 plain-delta contract (a plain delta stays
byte-plain, a sidecar rides beside it) is not re-implemented here — the
upgrade path never touches a delta.

## Impact

- Affected specs: `migrate` — ADDED requirements (the upgrade contract,
  the registry, the refusals, the atomic/scope laws) plus the matching
  constraint rows, the `upgrading` / `upgraded` states, the upgrade
  transitions and the upgrade properties in the layer sidecar.
- Affected code: `src/migrate.rs` (the revision registry, the revision
  detection, the step transform and the pure upgrade planner),
  `src/main.rs` (the migrate verb's flags, the path list, the upgrade
  arm and its envelope).
- Affected tests: `tests/cli/migrate_upgrade.rs` — the RED-first
  integration tests (old-revision fixture corpus, transformed file,
  unchanged file, lint-clean upgrade, byte identity, dry-run
  non-mutation, unknown source/target, target below the current
  revision, no writes outside the supplied paths).
- Affected contracts: `.espectacular/migrate/*.toml` — one contract per
  new scenario, bound to the new test flags.
- Release: ships as the sanctioned corpus-upgrade path (Revision 12 →
  Revision 20 corpora included) alongside `spk migrate` and
  `spk migrate --rekey`.
