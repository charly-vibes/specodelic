# Change: Add skill-pack — `spk skill install` ships the first-party specodelic-design skill

## Why

The repo's docs answer **what the format is** (`spk explain`, the book,
the FAQ) but nothing ships a **procedure** for the highest-friction
moment of onboarding: getting from a design conversation to a lint-clean
spec file. The external understandability review (2026-10-09, `~/Downloads/cv`
bundle) scored the docs 2/5 on onboarding flow and recommended
agent-facing guidance; the 2026-10-13 grounding session found the
user-drafted remedy (`specodelic-design` skill + chat primer) had already
drifted from the corpus in four ways while maintained outside the repo's
checking loop — 8-vs-9 explain topics (the documented 7-vs-9 drift class),
a 24-vs-34 lint-rule list that contradicted its own failure table, a
false fixture-provenance claim, and a missing `observes` column that made
its own example trip `linter.observability`.

The drift is structural, not accidental: the skill references machine
truth (explain topics, rule catalog, worked example) that changes with
every format revision, so any hand-maintained copy rots each cycle.
The fix follows the **whisper (`turu`) skill-pack precedent**
(`charly-vibes/whisper`, `src/skill_pack.rs`): the skill ships *from this
repo* so the skill and the binary evolve together — generated content,
content-hash staleness detection, managed marker, in-binary tests.

**specodelic-specific upgrades** (strictly stronger than whisper's gates):

- The tracked skill source embeds the worked example **verbatim from
  `docs/src/examples/reservation-order.md`** via `include_str!` — zero
  duplication; the file is already grounded by `just doc-examples`, and
  the in-binary test re-lints it (0 findings, 0 warnings).
- The topics list and rule names in the skill are **rendered from the
  same tables `spk explain` serves** at generate time — the 8-vs-9 drift
  class becomes structurally impossible.
- The self-check's rule names are **membership-tested** against the
  linter rule catalog — the `id_matches_file` self-inconsistency class
  is test-enforced away.

## What Changes

- **Tool — `spk skill install [--dir <dir>]`:** a new verb that writes the
  first-party managed `specodelic-design` skill pack (one procedure skill
  `SKILL.md` + `references/format.md` + `references/example.md`) to the
  given directory (default `~/.claude/skills`; alternate targets accepted),
  and reports a deterministic `pack_content_hash` (SHA-256 over sorted
  `path\0content`) in the envelope.
- **Pack content is embedded, not copied:** the protocol prose lives in a
  tracked markdown file compiled into the binary via `include_str!`;
  the example reference embeds the book's worked example verbatim; the
  format reference's EARS/kinds/reference-typing content is rendered by
  the same code that serves `spk explain`.
- **Managed marker:** every installed file starts with a
  `<!-- MANAGED BY specodelic — DO NOT EDIT MANUALLY. Regenerate: spk skill install -->`
  block; manual edits are detectable by construction.
- **Doctor staleness check:** `spk doctor` gains a managed-skill-pack
  check comparing the freshly generated pack hash against the on-disk
  installation — `pass` / `stale` / `missing`, where `missing = pass`
  (the pack is optional) and `stale`/`missing` carry a regenerate hint.
- **No zip artifact in git:** distribution is the install command (the
  user-side zip draft is superseded); releases need no new files.
- **Docs:** a thin-pointer book page (`docs/src/design-conversations.md`)
  linking the procedure to its source (never duplicating it); README
  command surface, `docs/src/SUMMARY.md`, and `llms.txt` gain the new
  verb/pack entries.
- **Spec — `skill-pack` capability:** dual-format capability spec under
  `openspec/specs/skill-pack/` on archive (this change's delta drafts the
  requirements).

## Capabilities

### New

- `skill-pack`: first-party managed skill packs — embedded single-source
  content, explain-derived tables, lint-gated example, deterministic
  hashing, install verb, doctor staleness check.

### Modified

- None. `spk doctor` gains a check inside its existing report schema
  (same envelope, new optional check entry — additive, within the
  existing doctor contract). `spk explain` serves no new topic in this
  change.

## Impact

- Code: new `src/skill_pack.rs` module; one new CLI verb; one doctor
  check; tracked source dir `skill-packs/specodelic-design/`.
- Gates: new in-binary tests (determinism, marker completeness, explain
  membership, example lint 0/0); no existing gate changes its scope.
- Docs: one new thin-pointer book page; README/SUMMARY/llms.txt entries.
- Out of scope: sub-skill routing (whisper's `router + subs` layout — the
  design skill is one procedure with two references, not a verb surface);
  third-party packs; editing installed packs; zip bundling.
