# Design: add-skill-pack

## Decisions

### D1 — Tracked markdown + `include_str!`, not content-in-Rust

Whisper embeds skill content as Rust string literals (`skill_pack.rs`
`generate_whisper()`). Correct for its terse verb protocols, wrong here:
the conversation protocol is real prose that must be editable with the
same tooling as every other doc in a doc-first repo. The protocol lives
in tracked `skill-packs/specodelic-design/SKILL.md` and is compiled in
via `include_str!`. Whisper's rationale — skill and binary evolve in the
same commit — is preserved by the membership and lint tests (the
requirements), not by content location.

Rejected alternative: generating from the book page at runtime — a
released binary cannot read repo files, and the book page would become a
second authoring surface for executable content.

### D2 — The example is `include_str!` from the book, never copied

`references/example.md` embeds `docs/src/examples/reservation-order.md`
verbatim. That file is already the single authoring surface for the
worked example (the `doc-examples` gate re-runs the quickstart against
its captured output; the grounding session verified it lints 0 findings /
0 warnings at HEAD). A second copy in the pack source would recreate the
exact drift the grounding session found (the draft's example lagged the
book's `observes` column). The managed marker is prepended at generate
time — the repo source stays marker-free (requirement: marker survives
verbatim embedding).

### D3 — Explain-derived lists are rendered, not restated

The skill's "topics you can consult" and rule-name lists are rendered at
generate time from the same tables `spk explain` serves, replacing
rendered placeholder markers in the tracked source. This kills the drift
class that motivated the change: the grounding session found the draft
listing 8 of 9 topics and 24 of 34 rules, including a rule cited in its
own failure table but absent from its own list. In-binary membership
tests back the renderer: any mention in generated text must resolve
against the topic/rule tables.

Placeholder syntax: HTML-comment markers (`<!-- spk:render:explain-topics -->`)
— invisible in rendered markdown, unambiguous, and immune to mdbook's own
`{{ }}` templating collision in book context.

### D4 — Pack layout follows the pi/Claude skill convention, not whisper's router

Whisper uses router + `subs/` because its surface is a command family.
The design skill is **one procedure** (Listen → Intent → Constraints →
Model → Properties → Assemble) with two references. The drafted layout —
`SKILL.md` + `references/format.md` + `references/example.md` — matches
the convention the user's draft already used and the skill loaders
(pi `installed-from` metadata, Claude `SKILL.md`) expect. No routing
logic is needed; no sub-skills are created.

### D5 — Staleness without stored state

Whisper compares generate-time hash vs on-disk hash — no state file, no
manifest. specodelic adopts the same: `pack_content_hash` is a pure
function of the generated pack, so `spk doctor` recomputes both sides.
`missing = pass` (whisper's precedent: the pack is optional, absence is a
hint not a failure); `stale` is reportable but non-fatal for doctor's
exit code — the regenerate command is the remediation. The hash algorithm
(SHA-256 over sorted `path` + NUL + `content`) is copied from whisper so
the pattern stays consistent across the tool family.

### D6 — Overwrite refusal, not silent replacement

`spk skill install` overwrites only marker-bearing files (its own prior
installs). A non-marker file at a target path is a refusal with exit 2
and a hint naming the path — same fails-closed discipline as the hooks
verb (never claims `core.hooksPath`, never overwrites foreign files) and
the archive-companion (fails closed on a delta lacking the layer).

### D7 — Distribution is the verb; no zip in git

The user-side zip draft (`~/Downloads/specodelic-design-skill.zip`) is
superseded by `spk skill install`. No zip artifact is committed or
release-staged; the release tarball needs no new files. Users on hosts
without the binary keep the book page as the reading copy.

### D8 — Doctor envelope compatibility

The check is additive within the existing doctor report schema (a new
optional check entry keyed `spk.managed-skill-pack`, mirroring whisper's
`turu.managed-skills`). No doctor requirement text changes; the
capability delta for doctor is not needed — this is within its existing
"workspace diagnosis with remediation hints" contract. Noted in the
proposal as a modified-in-place boundary so archivists don't hunt for a
doctor delta that doesn't exist.
