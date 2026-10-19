# Add: `spk migrate --rekey <file>` — migrate a 0.6.0-era `id: spec` dual-format file to Revision 18 real ids

## Why

Revision 18 (`update-id-derivation-spec-md`) retired `id: spec`, but
corpora written against the 0.6.0 dual-format protocol still carry it:
frontmatter `id: spec` and `[[spec.*]]` / `[[spec]]` refs throughout.
Under the new naming law those files fail `id_matches_file` (the wall
test: exactly that one finding), and the mechanical fix is uniform —
derive the real id from the parent directory, re-key every
`[[spec…]]` ref to it. Hand-editing that transform per file is the
exact onboarding cost `spk migrate` was built to remove (gh#6 item 1);
releasing 0.7.0 without a one-command path would strand every existing
dual-format corpus on a sed one-liner.

This is a new capability surface of the existing `migrate` command, not
a change to the wrap path: `--rekey` targets files the wrap path
REFUSES (already dual-format), and the wrap path's laws
(`delta_text_preserved`, `mirror_byte_identical`,
`refuse_already_migrated`) are untouched.

## What Changes

- **`spk migrate <file> --rekey`** (new flag on the existing command):
  rewrites an `id: spec` dual-format file in place —
  1. Derive the real id via the naming law: the parent directory name
     for a `spec.md` file (`-` ⇔ `.`, `_` literal); the stem for any
     other filename (bare `spec.md` with no parent directory is a
     refusal — there is no id to derive and guessing would corrupt the
     corpus).
  2. Replace the frontmatter `id: spec` line with the derived id.
  3. Re-key every `[[spec]]` → `[[<derived-id>]]` and every
     `[[spec.` → `[[<derived-id>.` in the body (frontmatter statement
     untouched unless it carries a link — statements never do; they are
     EARS prose).
  4. NOT rewritten: bare `[[c1]]`-style dotless refs (metasyntactic in
     every file since Revision 18 — the file may carry them as format
     prose; re-keying only the `spec.`-qualified spellings is the
     complete mechanical transform).
- **Refusals (exit 2), never rewrites**: file without frontmatter (a
  plain delta — use the wrap path); frontmatter id present but not
  `spec` (nothing to re-key — a no-op is reported as ok-with-warning,
  the file untouched); no derivable id (bare `spec.md`).
- **Idempotence**: a re-keyed file (id ≠ `spec`) is a warning no-op, so
  a directory-wide `--rekey` sweep is safe to run twice.
- **Statement EARS check**: if the re-keyed file's statement no longer
  makes sense (it never references the id), the statement is left
  verbatim — rekey changes ids and refs only.

## Impact

- Affected specs: `migrate` (ADDED requirement + constraint rows + a
  `rekeying` state in the Model).
- Affected code: `src/migrate.rs` (`rekey` function + unit tests),
  `src/main.rs` (flag), `src/commands/manage.rs` (`cmd_migrate`
  dispatch + envelope fields).
- Docs: `docs/src/commands.md` migrate section; `spk explain migrate`
  (embedded guide topic, if present).
- Release: ships in **v0.7.0** as the sanctioned 0.6.0 → 0.7.0 corpus
  migration path (release-notes recipe: `spk lint <tree>` to find the
  files, `spk migrate <file> --rekey` per file).
