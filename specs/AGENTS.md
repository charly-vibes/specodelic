# AGENTS.md

Standing instructions for any agent (human or AI) working in this repo.
Read this once per session, before touching any file. It doesn't change
often — if you find yourself wanting to note something here that's really
about *today's* state or *today's* plan, that belongs in `STATUS.md`
instead; if it's a record of something that already happened, it belongs
in `CHANGELOG.md`.

## What this repo is

A self-hosting specification format (`specodelic`) written in itself:
every file here is a markdown spec describing either the format, or one
check the format's own linter performs on other spec files. Start with
`specodelic.md`, then `STATUS.md` §1 for the full primer if you weren't
already familiar with this project.

**If you're about to write a spec for a piece of software** (a feature, a
service, a library) rather than for this repo's own linter, read
`USAGE.md` first. It has the four-layer quick-start and a pattern catalog
(closed enumerations, Moore-machine output, multi-implementation
conformance, staged/lazy evaluation, algebraic laws, consumer-extended
contracts, event-sourced/append-only logs, empirical runtime bounds)
worked out from checking this format against a real domain — most things
that look like a missing feature are one of those patterns aimed at the
wrong section, not an actual gap.

## File naming — non-negotiable

A file's declared `id` (in its YAML frontmatter) must equal its filename
with the following exact mapping:

- `-` (hyphen) in the filename ⇔ `.` (dot, namespace separator) in the id
- `_` (underscore) is preserved literally in both

Example: `linter-graph_shape.md` ⇔ `id: linter.graph_shape`.

Do not use a hyphen to represent an underscore, or vice versa — this was
the exact bug fixed in Revision 3 of `specodelic.md` (see `CHANGELOG.md`).
If you're about to name a new file and the id has more than one word per
namespace segment, use `_` inside the segment, `-` only between segments.

## Workflow for adding or editing a spec file

1. Every new file follows the four-layer shape: YAML frontmatter (Intent),
   a `## Constraints` table, a `## Model` section (states + transitions),
   a `## Properties` table. Don't skip a layer even if it feels thin for
   a small feature — an empty-but-present section is more honest than an
   absent one.
2. Every `traces_to`/`derives_from`/`guard`/`from`/`to` must resolve to a
   real row, and must respect `specodelic.md`'s Reference Typing table —
   look it up there rather than assuming the mapping; it's the single
   source of truth and this file won't be kept in sync if it changes.
3. Before considering a new checker file finished, deliberately check it
   against `specodelic.md`'s existing constraint list. Four of the first
   seven checker files surfaced a gap this way — treat that as the base
   rate, not an exception. If you find one, fold it back into
   `specodelic.md` as a new Revision **in the same session**, not as a
   follow-up. Gaps that sit unaddressed across multiple files are how
   documentation drift happened in Revision 2→3 (see `CHANGELOG.md`).
3a. Before writing a new rule, check whether it's the same rule as one
   that already exists stated slightly differently. `append_only_variants`,
   `kind_field_extensible`, and `reference_field_extensible` were the
   identical "grows only, only via Revision" rule discovered three times
   before being merged into one in Revision 6 — by then their wordings had
   already drifted out of sync with each other. Prefer widening an
   existing constraint's scope over adding a same-shaped new one; it's
   also usually the fix for a domain-spec pattern that looks unsupported —
   see `USAGE.md` §2 before concluding the format itself needs to change.
   The same check applies to a *file's own* constraints against another
   file's: before adding an invariant asserting "X can never gate Y,"
   check whether `specodelic.md`'s existing Reference Typing already
   guarantees it structurally (e.g. `advisory_cannot_gate` — any
   `advisory`- or `effect`-kind Constraint is already excluded from every
   `guard` field, everywhere, by typing). If it does, cite that constraint
   in Notes rather than restating it as a new one; restating it needs a
   new unit test for something already unconditionally true. Only add a
   fresh "never gates" invariant when the claim depends on something the
   type system doesn't already forbid — e.g. a wiring fact ("no current
   transition happens to cite this checker's output") rather than a
   typing fact, which the type system can't enforce and so still needs
   asserting and testing.
3b. Record the result of check #3 in frontmatter, not as restated prose:
   `checked_against_core: clear` when no gap surfaced. Only write Notes
   prose about the check when a gap *was* found and folded into a
   Revision (say what and where), or when the check involved a genuine
   judgment call worth a reader seeing (e.g. "this looks like a duplicate
   of rule X but isn't, because ___" — see `merge.md`'s Notes on
   `rename_replayed_onto_foreign_edits` for the shape). A file with
   `checked_against_core: clear` and nothing else to say about it needs no
   sentence restating that in every Notes section — the field is the
   record.
4. Every `kind = "law"` property needs its full case set as defined by
   `specodelic.md`'s `law_requires_cases` constraint — check that file
   for the current requirement rather than assuming it from memory.
5. Never edit prose inside a `rationale`/description field into something
   the linter would need to parse. Prose stays prose; only frontmatter
   fields and table cells are structured.
6. Before filing a new naming-confusion item in `STATUS.md` §4 (an id, a
   `kind` value, or a state name that reads as if it asserts something a
   similarly-named neighbor doesn't actually guarantee), check whether
   it's an instance of the same class of problem as `CLAR-001`/`CLAR-002`/
   `CLAR-003` rather than a new, unrelated one. All three share one shape:
   two names sharing a token differ in what they actually claim, without a
   disambiguating rename. File it as another instance of that guideline —
   "no two names sharing a token may differ in what they guarantee without
   a disambiguating rename" — rather than opening an unrelated backlog
   item; a batch of same-shaped renames is one Revision, not three.

## After any change, before ending the session

- Add an entry to `CHANGELOG.md` (top of file — newest first, past entries
  never edited).
- Update `STATUS.md`'s inventory table and, if the change affects
  priorities, §4's backlog.
- If you renamed or restructured anything, grep the whole repo for the old
  name before considering the change done — Revision 3 needed a full
  cross-file grep to find every stale reference; don't rely on memory of
  which files mention which other files.

## What doesn't exist yet — don't assume it does

There is no working parser, linter, or model-checker generator yet — only
the specs describing what they must do. `STATUS.md` §4 has the prioritized
list of what's unbuilt (kind definitions, the compile/model-check/verify
pipeline, the rename tool, external completeness checking, orchestration).
Don't write code that assumes any of this exists without checking §4 first.
