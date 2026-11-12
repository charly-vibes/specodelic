# Design: `spk migrate --to-revision <N>` — corpus revision upgrade

Context: the format corpus carries nineteen revision headings
(Revision 2 … Revision 20). The tool can migrate two shapes today (the
Revision 19 plain-delta wrap, the Revision 18 `--rekey` rewrite) and has
no path across the history. This change adds one.

## D1 — The revision registry is a thin, core-owned `const` manifest

`src/migrate.rs` grows a `pub const REVISIONS: &[RevisionEntry]` — the
single source of truth for "which format revisions exist". Each entry
carries:

| field  | meaning                                                          |
|--------|------------------------------------------------------------------|
| `number` | the revision number (2 … 20)                                   |
| `date`   | the date the format corpus records for it (`None` when the corpus records none — revisions 2 … 15 carry no date) |
| `note`   | a one-line change note                                           |
| `step`   | an **optional** migration step                                   |

The registry is *thin by design*: only Revision 18 carries a step (the
id law — `id: spec` → the parent-directory-derived id, with every
`[[spec…]]` ref re-keyed). Every other revision was additive, a
reworded rule, or a change to the tool's own semantics — none of them
requires a corpus rewrite, so their step is absent and the upgrade
across them is a byte-stable no-op. Manufacturing a transform per
revision would be a lie about the history; the registry records what
the history actually says.

The registry range (2 … 20) is the corpus's own revision history —
`openspec/specs/specodelic/spec.md` carries the headings, and the
embedded `guide::FORMAT_REVISION` names the current one. The tool
refuses a revision outside the range rather than guessing.

## D2 — The source revision is declared or detected per file

A corpus file's revision is a property of its **shape**:

- a file whose frontmatter still declares `id: spec` predates
  Revision 18 — it satisfies Revision 17 and no later;
- a file that already carries a real id satisfies every registered
  step (no step after Revision 18 changes a corpus spec file's shape).

`--from-revision <R>` declares the source revision explicitly (the
adopter knows their corpus's revision); without it the source is
**detected per file** and the corpus's source revision is the oldest
revision any file in the set still satisfies. A declared source wins
over detection — the adopter's word beats the tool's inference.

Why not "the corpus's revision is a corpus-wide constant"? Because a
corpus is edited file by file: one file can lag a decade behind the
rest. A per-file source revision is what makes "apply the steps from
the source revision to the target" mean something precise.

**The detection is only as sharp as the registered step boundaries.** A
file's revision is inferred from the steps it still needs, so the
detector can only tell "before Revision 18" from "Revision 18 or
later". A corpus that is older than Revision 18 reads as Revision 17:
the registry's steps are the only boundaries the corpus's own history
records, and inventing finer ones would be manufacturing history. A
file older than the registry's oldest step is therefore upgraded by
the same steps as a Revision-17 file — which is correct for every
revision that needs no corpus rewrite.

## D3 — Steps are shape-guarded

A registered step is applied to a file only when the file still carries
that step's pre-shape. Consequences:

- an up-to-date file is never rewritten by a step → it stays
  byte-identical;
- a mixed corpus (one pre-Revision-18 file, one modern file) upgrades
  cleanly: the modern file is left alone;
- the upgrade is idempotent — running it twice changes nothing the
  second time.

The Revision 19 step (delta → sidecar) is deliberately **absent** from
the registry: it rewrites a *plain delta* and creates a *sibling file*,
which is the wrap path's job (`spk migrate <delta>`). The upgrade path
never rewrites a delta and never creates a file — see D7.

## D4 — Dry-run by default; `--write` applies

The upgrade is a bulk rewriter over a whole corpus, so it previews by
default: the would-be content rides in the envelope and the disk is
untouched. `--write` (alias `--force`) applies the plan. `--write` and
`--dry-run` together are refused (exit 2) rather than silently picking
one.

The asymmetry with the wrap path (which writes by default and previews
under `--dry-run`) is deliberate and is stated in the CLI help: the
wrap path touches one file the adopter named, the upgrade path touches
every file under the supplied roots. The wrap path's behavior is not
changed.

## D5 — Byte stability is a write decision, not a formatting decision

The planner computes each file's new content in memory and compares it
to the original bytes. Only a file whose bytes actually differ is
written; a file that needs no step is never opened for writing at all.
Line endings and the final terminator are preserved because the
Revision 18 step only rewrites the id line and the `[[spec…]]` refs —
it never touches the rest of the file, and it detects the frontmatter
boundary under both LF and CRLF.

`--rekey` keeps its current (LF-only) behavior: the upgrade path is a
new path, and changing `--rekey`'s refusal set is out of scope.

## D6 — The upgrade is atomic

The planner runs over the whole file set before anything is written. If
any file cannot be upgraded (an id that cannot be derived, an unreadable
file), the run fails with exit 2 and **no** file is written — the
rename/refactor convention (all-or-nothing), reused for a bulk
rewriter.

The atomicity covers the **planning** phase. The write phase writes
the changed files one by one, so an I/O failure part-way through (a
file that vanished between the plan and the write, a read-only
filesystem) leaves the already-written files upgraded while the command
reports exit 2. A re-run is idempotent, so the residue heals itself;
the alternative — staging every file under a temporary name and
renaming them into place — buys a stronger guarantee at the cost of a
second copy of the corpus on disk, which D7 forbids. The spec's laws
are planning-scoped ("when any file in the supplied set cannot be
upgraded, every file in the set stays unchanged") and hold as written.

## D7 — Scope confinement

The upgrade reads and writes only the files under the supplied paths
(directories expand to their `*.md` files, exactly as `spk lint`
collects them). It creates no file: no sibling `layer.md`, no backup,
no report file. A file outside the supplied set is never read and never
written — the corpus outside the supplied roots is the adopter's
business.

## D8 — The migrate verb's positional becomes a path list

The upgrade takes several roots (a corpus is a tree), so the verb's
positional becomes `paths: Vec<String>`. The wrap and `--rekey` paths
still take exactly one file: more than one is refused (exit 2) with a
labeled message, which preserves their single-file contract while the
upgrade gets its bulk surface. Clap's "required argument" behavior for
`spk migrate` with no path is unchanged.

## D9 — Failure taxonomy and output

All refusals are invocation errors (exit 2), not tool failures — the
`specodelic-7rr` precedent. Every refusal carries a labeled message and
a remediation hint, and is emitted through `genesis::guide::Output`
(`Output::failure(...).with_next_step(...)`) exactly like the existing
migrate refusals:

| refusal                | message names                                       |
|------------------------|-----------------------------------------------------|
| unknown target         | the known revision range (2 … 20) + a hint          |
| unknown source         | the known revision range (2 … 20) + a hint          |
| target below current   | the corpus's current revision + the file that lags  |
| conflicting flags      | `--write` vs `--dry-run`, and `--rekey` vs `--to-revision` |
| unupgradable file      | the file and why (no derivable id)                  |

The success envelope carries the resolved source and target revisions,
the scanned/changed/skipped counts, and — in dry-run — the would-be
content per changed file.

## D10 — Out of scope

- **No format change.** This is a Revision 20 tool; the format gains
  no field, no section and no rule. The capability's *intent
  statement* gains a clause naming the upgrade path, because the
  statement is the capability's one-line summary, not a format rule.
- **No delta-sidecar migration.** Creating a `layer.md` beside a plain
  delta stays the wrap path's job (D3).
- **No `--rekey` change.** Its refusal set, exit codes and byte
  behavior are untouched.
- **No new spec capability.** The upgrade belongs to `migrate`: it is a
  migration verb with a second mode, and the layer sidecar stays the
  capability's single source of truth.
