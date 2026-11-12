# Subagent brief: specodelic-ez9a.2 — migrate layer-scaffold namespace bug

You are a subagent in repo `/var/home/sasha/para/areas/dev/gh/charly/specodelic`
(Rust CLI `specodelic`, alias `spk`). An orchestrator has claimed ticket
`specodelic-ez9a.2` and is hands-off: you own the diagnosis + fix end-to-end.
The orchestrator verifies with real gates — never claim work the diff
doesn't contain.

## The report (bug of record)

`spk migrate`'s layer scaffold and the archive companion's deploy jointly
produce a **post-archive corpus lint failure**: the layer scaffold derives
the error-label namespace from the parent spec id instead of the sidecar's
own file id (repro_hash `f29f4cf1f143d662`; discovered during the d8c
archive). Error labels are file-id-namespaced (`error-contract`: each file's
labels are `<that-file-id>.<variant>(detail)`), and the companion's
`assemble` → `rekey_sidecar` path only rewrites the `id:` frontmatter line
and `[[sidecar_id.*]]` / `[[sidecar_id]]` wikilinks — bare non-link label
occurrences are NOT rekeyed.

## Measured facts at HEAD (orchestrator, reproducible)

- `spk migrate <plain-delta>` on a delta at `.../specs/widget-cap/spec.md`
  writes the sibling `layer.md` with `id: widget.cap.layer`, rows
  `scaffold_constraint` etc., links `[[widget.cap.layer]]` /
  `[[widget.cap.layer.scaffold_constraint]]`; the sidecar lints CLEAN as a
  standalone file (verified: `/var/tmp/ez9a2/specs/widget-cap/layer.md`).
- `src/archive_companion.rs` `rekey_sidecar(sidecar_text, sidecar_id,
  spec_id)` rewrites only the `id:` line and the two wikilink forms above.
- Therefore an error constraint added to a scaffolded sidecar (labeled by
  its own file id, `<cap>.layer.<variant>(detail)` — the natural reading of
  the error contract) is deployed UNREKEYED into `specs/<cap>/spec.md`,
  whose id is `<cap>` → post-archive lint failure on the error label
  namespace (and/or `id:`-related mismatch depending on the path taken).

## Your job

1. **Reproduce END-TO-END first (RED)**, in a scratch copy under
   `/var/tmp/ez9a2-run/` (clone/copy the repo or use a git worktree so you
   can run `just archive-change` for real — do NOT run archive on the real
   repo): create a change dir with a plain delta + a scaffolded sidecar
   (`spk migrate`), add ONE error constraint + failure state/transition
   labeled by the sidecar's own id (e.g. `widget.cap.layer.boom_failure`),
   archive via the companion, then `spk lint openspec/specs` (or the
   scratch corpus) and confirm the failure the ticket names. Capture the
   exact finding id/message in the test.
2. **Fix the namespace derivation** so the post-archive deployed spec lints
   clean:
   - the scaffold (`src/migrate.rs`) must emit labels/ids consistently with
     the error contract (its own file id while it IS the sidecar), and
   - the companion (`src/archive_companion.rs` `rekey_sidecar`) must rekey
     every identity namespace — including bare error-label occurrences of
     `sidecar_id.` (not just wikilinks), carefully so it never mangles
     unrelated text (e.g. a label already carrying `spec_id.`; a
     `sidecar_id` that is a prefix of another id).
   Choose the minimal coherent fix; record the direction in the commit
   message. Do not change the byte-plain delta contract, the fail-closed
   layer validation, or the two-arg `assemble` contract.
3. **Regression tests** (RED first): a unit test around `rekey_sidecar`
   (or `assemble`) proving an error-labeled sidecar deploys with the
   deployed id namespace and no `.layer.` remnant; plus a scaffold test
   pinning the emitted label namespace. Place tests in `tests/**` if a
   ratchet pins the source file.
4. `cargo fmt` before committing; `pretender check` green.

## Scope guard

- Allowed files: `src/migrate.rs` (scaffold), `src/archive_companion.rs`
  (rekey/assemble), `tests/**` (regression tests).
- Never edit: other `src/**` modules, `openspec/**`/`specs/`,
  `.espectacular/**`, `pretender.toml`, `.wai/resources/**`, run YAMLs,
  `.beads/**`.
- Iteration testing: TARGETED tests only (`TMPDIR=/var/tmp/specodelic-xch1
  cargo test --lib archive -- migrate`, etc.). Do NOT run full `just ci` /
  `ah check` / `openspec validate` — the orchestrator owns full gates.

## Repo facts (boilerplate — applies to every ticket)

- **Commit hygiene**: before ANY `git commit`, `git status --short`; stage
  only files YOU authored (`git add <paths>`, never bare `git add`).
- **beads**: do NOT claim/close/update — the orchestrator owns bd state.
- **Do NOT push**; **do NOT run `wai close`**.
- **New source files** need Purpose/Responsibilities/Rationale headers.
- **Non-interactive shells**: `-f`/`-rf`/`-y` flags.
- **TMPDIR=/var/tmp/specodelic-xch1** (mkdir -p) for every cargo call.

## Report format (end with this — the orchestrator verifies against it)

## Report

**Commits**
- `<hash>` <message> — <what it does, one line>

**Repro + fix evidence**
- exact failing finding before, exact clean result after (test names)

**Deviations** (or "none")

**Next**
- <exact next action for the orchestrator, or "ticket complete">