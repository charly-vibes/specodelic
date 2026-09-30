# Tasks: add-spk-migrate

## 1. RED — pure seam tests (src/migrate.rs unit tests)

- [ ] 1.1 Wrap a frontmatter-less delta: result has generated frontmatter
      (`id: spec`, kind intent, EARS scaffold statement), scaffold layers,
      byte-identical `## Requirements` mirror; `## ADDED Requirements`
      text preserved byte-for-byte.
- [ ] 1.2 Merge: existing frontmatter kept verbatim (including a non-`spec`
      id, with a warning surfaced by the caller); existing Constraints
      layer kept untouched (no row injection); missing layers + mirror
      added.
- [ ] 1.3 Refusals: already dual-format (frontmatter + `## Requirements`);
      plain spec (`## Requirements`, no ADDED); not a delta (neither
      section). Each with a distinct message.
- [ ] 1.4 Mirror byte identity: mirror body slice == ADDED body slice,
      including when the ADDED section is the last section (EOF) and when
      it is followed by another `## ` heading.
- [ ] 1.5 CRLF file in → CRLF inserted content; missing trailing newline
      normalized to exactly one.

## 2. GREEN — src/migrate.rs

- [ ] 2.1 Implement `migrate()` + `MigrateError` + scaffold constants;
      section extraction helpers shared with the mirror.
- [ ] 2.2 All 1.x tests green.

## 3. CLI wiring (src/main.rs)

- [ ] 3.1 `Migrate { file: String, #[arg(long)] dry_run: bool }` arm →
      envelope data (file, inserted pieces, hand-finish steps), warnings
      for each scaffold piece, refusal = failure envelope + exit 2,
      hint `spk lint <file>` on success.
- [ ] 3.2 `--dry-run` writes nothing, returns content in data.

## 4. Integration tests (tests/cli.rs)

- [ ] 4.1 End-to-end: write a plain delta to a temp file, run
      `spk migrate`, assert exit 0 + file rewritten + second run refuses
      (exit 2) with the already-migrated message.
- [ ] 4.2 Migrated file `spk lint`s clean (exit 0, zero findings).
- [ ] 4.3 Dry-run: file unchanged on disk, data carries the content.

## 5. Docs + gates

- [ ] 5.1 docs/src/commands.md migrate section; CHANGELOG entry.
- [ ] 5.2 just ci + lint-specs + graph 0 dangling + openspec strict green.
- [ ] 5.3 Archive the change via the dual-format recipe.
