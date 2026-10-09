# Subagent brief: specodelic-lf4b.5 — Add one evolving worked example to the docs book

You are a pi subagent in repo `/var/home/sasha/para/areas/dev/gh/charly/specodelic`
(Rust CLI `specodelic`, alias `spk`). An orchestrator has claimed ticket
`specodelic-lf4b.5` and is hands-off: you own the implementation end-to-end.
The orchestrator will verify with real gates — never claim work the diff
doesn't contain.

## Orientation (do this first)

- `export WAI_PROJECT=domain-specific-extensions`
- Read: `bd show specodelic-lf4b.5` (acceptance criteria + anti-goals), the
  epic `bd show specodelic-lf4b` (docs-only scope + staleness gate),
  `docs/src/examples/batch-resume.md` (the one existing example — your
  voice/format precedent), `docs/src/SUMMARY.md`, and `docs/src/index.md`.
- **Source canvas:** `~/Downloads/cv/grok-specodelic-design-documentation-canvas.md`
  contains a near-usable human-facing draft of an evolving worked example
  (order/reservation domain). **It was written WITHOUT running the binary —
  treat every claim in it as unverified.** Adapt prose only after per-claim
  verification against current main: run the actual `spk` commands it
  describes and keep only what the binary actually demonstrates.
- **Staleness gate (mandatory, epic rule):** re-verify all format claims
  against current main before writing prose. Authoritative sources:
  `specs/specodelic.md` (format core), `specs/STATUS.md` §1 (primer),
  `spk explain` topic list. The four-layer shape is Intent → Constraints →
  Model (States + Transitions) → Properties — verify against the corpus,
  not against the canvas or ticket text.
- `wai search "worked example"` / `wai search "docs"` — accumulated patterns.

## What to build

**Primary deliverable (the ticket's Must): one evolving worked example**
(order/reservation domain) in the docs book, spanning the full four-layer
journey: Intent → Constraints → Model → Properties → **unresolved
behaviors**, and explicitly INCLUDING failure paths — real `spk lint`
findings on a deliberately broken intermediate version, at least one
counterexample, and the fixes — not just the happy path.

Suggested shape (adapt if you find a better one that satisfies the meter):

- A new hand-authored page, e.g. `docs/src/examples/worked-example.md`
  (mdBook page; `docs/src/examples/` is hand-authored — NOT generated,
  unlike `docs/src/specs/` and `docs/src/openspec/`).
- The example spec itself as a **standalone spec file** in
  `docs/src/examples/` (e.g. `reservation-order.md`) so the meter
  "`spk lint` at 0 findings" is mechanically checkable on a real file.
  Naming rule: frontmatter `id` equals the filename stem with `-` ⇔ `.`.
  The page then walks through it layer by layer, quoting fragments and
  interleaving captured CLI output.
- Build the example incrementally IN the page: start from Intent prose,
  add Constraints, show a lint failure at an intermediate broken state
  (capture the real `spk lint` output at that state — do not fabricate
  findings), fix it, continue to Model and Properties, end with the
  unresolved-behaviors section (what the spec deliberately does NOT decide).

**Secondary (ticket description, NOT the Must): 2–3 complete annotated
specs in `docs/src/examples/`** per the Z.ai review. Only add these if the
evolving example does not already leave the examples dir feeling sparse,
and only as complete lint-clean spec files with short annotation pages or
header commentary. Do not let this bloat the change — the Must is the one
evolving example. If you skip or trim the secondary, say so explicitly in
your report under Deviations.

**Meter (all must pass):**
- the example spec file passes `spk lint` at **0 findings**
- every CLI output shown in the page is **captured at HEAD** (run the
  command, paste real output; re-run before commit and diff — must be
  diff-clean)
- `docs/src/SUMMARY.md` links the new page
- `just docs-build` green

**Anti-goals (from the ticket):**
- no happy-path-only example
- no claims the binary does not demonstrate
- do not lead with a command catalog — the example teaches the format, the
  commands appear only where they earn their place

## Hard scope guard

- Allowed files: new pages/files under `docs/src/examples/`,
  `docs/src/SUMMARY.md`, and optionally small cross-link edits in
  `docs/src/index.md`.
- Never edit: `openspec/specs/`, `openspec/changes/`, `.espectacular/`,
  `pretender.toml`, `specs/` (the corpus), `book.toml`, `src/**`,
  `tests/**`, `.wai/resources/**`, `docs/src/specs/**`,
  `docs/src/openspec/**` (generated), this template.
- Respect shrink-only ratchets: run `pretender check` before committing;
  if a pinned source file breaches, move new tests to `tests/` rather than
  raising entries. (Docs-only ticket — you should not touch pinned files
  at all.)

## Repo facts (boilerplate — applies to every ticket)

- **Commit hygiene** (standing, every commit): before ANY `git commit`, run
  `git status --short` and stage only files YOU authored
  (`git add <paths>`, never bare `git add`); unstage foreign files.
  Attribute the message only to what the diff contains.
- **Output discipline**: every command emits through
  `genesis::guide::Output::emit`; errors carry a remediation hint.
- **beads**: issues live in `.beads/issues.jsonl`; commit the export after
  `bd close` (`bd export` gotcha: run it or the close is invisible to git).
- **Do NOT push** — the orchestrator verifies and pushes.
- **Do NOT run `wai close`** — the orchestrator owns session close.
- **ah/espectacular**: read-only over `openspec/`; contract TOMLs for
  change-phase scenarios are orphans until archive — author them in the
  archive commit, never mid-change.
- **New source files** need Purpose/Responsibilities/Rationale headers
  (file-headers skill convention).
- **Non-interactive shells**: use `-f`/`-rf`/`-y` flags; commands may be
  aliased with `-i` and will hang on prompts.
- **Scratch work outside /tmp**: /tmp quota exhausts and makes cargo fail
  with "Disk quota exceeded" — use `/var/tmp/<slug>` if you need scratch
  space.

## Mechanical finish checklist (execute in order, do not skip)

1. `spk lint <example-spec-file>` → 0 findings (paste in report)
2. Re-capture every CLI output quoted in the page at final HEAD; diff-clean
3. `just docs-build` → green
4. `grep -F "worked-example" docs/src/SUMMARY.md` → link present
5. `git status --short` → only your files; stage explicitly
6. `git commit` with honest attribution
7. `bd export -o .beads/issues.jsonl` and commit the export
8. `bd close specodelic-lf4b.5`
9. Do NOT push; do NOT run `wai close`
10. End with the Report block below

## Report format (end with this — the orchestrator verifies against it)

## Report

**Commits**
- `<hash>` <message> — <what it does, one line>

**Gates run**
- `just test` → <result>
- `ah check` → <result>
- `openspec validate --all --strict` → <result>
- `just docs-build` → <result>
- `spk lint <example-spec>` → <result>

**Canvas claims verified/discarded**
- <which Grok-canvas claims you kept after binary verification, which you
  discarded and why — one line each>

**Deviations** (or "none")
- <any scope/plan deviation and why>

**Next**
- <exact next action for the orchestrator, or "ticket complete">