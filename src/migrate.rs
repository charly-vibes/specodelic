//! `spk migrate` — wrap an existing plain openspec delta into the
//! Revision 19 pair: the delta stays byte-plain, the specodelic layer is
//! scaffolded into a sibling `layer.md` sidecar (gh#6 item 1, beads
//! specodelic-c32; reworked by add-delta-layer-sidecar / Revision 19).
//!
//! The pure seam: [`migrate`] returns the sidecar content, the CLI arm
//! (src/commands/manage.rs) does file IO and envelope emission. The old
//! dual-format wrap — frontmatter + layers + `## Requirements` mirror
//! inserted into the delta itself — retired with the mirror: plain
//! deltas cannot carry the mirror without failing openspec validation.
//! Design decisions now live in openspec/changes/add-delta-layer-sidecar/
//! design.md; the carried-over decisions keep their force: D3 (merge
//! adds only missing pieces — an existing sidecar passes through
//! untouched, enforced at the CLI layer), D4 (the scaffold is wired so
//! it lints clean as written), D5 (no backup files — `--dry-run` is the
//! preview path). `--rekey` (add-migrate-rekey) keeps the Revision 18
//! semantics unchanged: an `id: spec` dual-format file re-keyed in
//! place, every `[[spec.*]]` ref re-keyed, every other byte untouched.

use std::fmt;

/// Why a file (or pair) cannot be migrated (specs/migrate,
/// refuse_already_migrated).
#[derive(Debug, PartialEq, Clone)]
pub enum MigrateError {
    /// The file already carries frontmatter — it is a specodelic file,
    /// not a plain delta (the dual-format wrap retired with the mirror).
    AlreadyMigrated,
    /// `## Requirements` without a delta section — a plain spec.
    NotADelta,
    /// Neither `## ADDED Requirements` nor `## MODIFIED Requirements`
    /// present — nothing to wrap.
    NoDeltaSection,
    /// A `## ADDED Requirements` heading with no trailing newline and no
    /// body — the degenerate one-line delta (specodelic-7i7): a bare
    /// heading is not a section, so there is nothing to wrap.
    BareDeltaHeading,
    /// `--rekey` on a bare `spec.md` with no parent directory: the
    /// naming law has nothing to derive the real id from (Revision 18).
    NoDerivableId,
}

impl fmt::Display for MigrateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MigrateError::AlreadyMigrated => write!(
                f,
                "file already carries frontmatter — a specodelic file, not a plain delta"
            ),
            MigrateError::NotADelta => write!(
                f,
                "file has ## Requirements but no ## ADDED Requirements or ## MODIFIED Requirements — a plain spec, not an openspec delta"
            ),
            MigrateError::NoDeltaSection => {
                write!(
                    f,
                    "no ## ADDED Requirements or ## MODIFIED Requirements section found — nothing to wrap"
                )
            }
            MigrateError::NoDerivableId => write!(
                f,
                "the real id cannot be derived (bare spec.md, or parent dir named `spec`) — move the file under its capability directory"
            ),
            MigrateError::BareDeltaHeading => write!(
                f,
                "the ## ADDED Requirements heading has no trailing newline or body — a bare heading is not a section"
            ),
        }
    }
}

/// Outcome of a wrap migration: the sibling `layer.md` sidecar content
/// to create beside the delta — which the transform never touches.
#[derive(Debug, PartialEq, Clone)]
pub struct WrapOutcome {
    /// The sidecar's frontmatter id (parent-directory id + `.layer`).
    pub sidecar_id: String,
    /// The full `layer.md` content.
    pub content: String,
    /// Names of the layer sections generated (the scaffold for a fresh
    /// sidecar carries all three).
    pub inserted_layers: Vec<&'static str>,
}

/// Outcome of a `--rekey` migration: the new content plus what was done.
#[derive(Debug, PartialEq, Clone)]
pub struct MigrateOutcome {
    /// The full rewritten file content.
    pub content: String,
    /// True when the id/refs were rewritten (false = idempotent no-op).
    pub rekeyed: Option<bool>,
    /// Always false for a rekey (kept for envelope continuity).
    pub inserted_frontmatter: bool,
    /// Always empty for a rekey (kept for envelope continuity).
    pub inserted_layers: Vec<&'static str>,
    /// Always false for a rekey (kept for envelope continuity).
    pub inserted_mirror: bool,
}

/// Inserted scaffold rows carry this prefix — the author's hand-finish
/// checklist is "replace every scaffold_* id with the real thing".
pub const SCAFFOLD_PREFIX: &str = "scaffold_";

/// `--rekey` transform (add-migrate-rekey): rewrite an `id: spec`
/// dual-format file to its Revision 18 real id. The naming law derives
/// the id from the parent directory (spec.md) or the stem (anything
/// else); `[[spec]]`/`[[spec.` refs re-key to the derived id; every
/// other byte stays untouched. A file already carrying its real id is
/// a no-op (idempotent sweeps); a plain delta or a bare spec.md is
/// refused, never rewritten.
pub fn rekey(text: &str, path: &std::path::Path) -> Result<MigrateOutcome, MigrateError> {
    // Shape refusals first — mirror the wrap path's triage.
    if !text.starts_with("---\n") && !text.starts_with("---\r\n") {
        return Err(MigrateError::NotADelta);
    }
    let derived = crate::lint::expected_id_from_path(path);
    if derived == "spec" || derived.is_empty() {
        // Nothing to re-key TO: a bare spec.md (no parent dir) or a
        // spec.md under a directory named `spec` both derive `spec` back
        // — refusing beats a self-referential rewrite.
        return Err(MigrateError::NoDerivableId);
    }
    let fm_close = text[4..]
        .find("\n---\n")
        .map(|i| i + 4)
        .map(|i| i + 5)
        .ok_or(MigrateError::NotADelta)?;
    let frontmatter = &text[..fm_close];
    let id_line = frontmatter
        .lines()
        .find(|l| l.trim_start().starts_with("id:"))
        .ok_or(MigrateError::NotADelta)?;
    let current_id = id_line.trim_start()[3..].trim();
    if current_id != "spec" {
        // Idempotence: already re-keyed (or never was id:spec) — a
        // no-op, never rewritten, so directory-wide sweeps are safe.
        return Ok(MigrateOutcome {
            rekeyed: Some(false),
            content: text.to_string(),
            inserted_frontmatter: false,
            inserted_layers: vec![],
            inserted_mirror: false,
        });
    }
    // A no-op when the derived id equals the current one is impossible
    // here (current == "spec", derived != "spec"), so a real rewrite
    // always follows: rekey the id line, then the body refs.
    let new_fm = frontmatter.replacen(&format!("id: {current_id}"), &format!("id: {derived}"), 1);
    let body = &text[fm_close..];
    let body = body.replace("[[spec]]", &format!("[[{derived}]]"));
    let body = body.replace("[[spec.", &format!("[[{derived}."));
    let content = format!("{new_fm}{body}");
    Ok(MigrateOutcome {
        rekeyed: Some(true),
        content,
        inserted_frontmatter: false,
        inserted_layers: vec![],
        inserted_mirror: false,
    })
}

/// Wrap a plain openspec delta into the Revision 19 pair: the delta text
/// is returned untouched in spirit (the transform never rewrites it —
/// the CLI writes nothing back to the delta file) and the specodelic
/// layer is scaffolded into the sibling `layer.md` sidecar content.
///
/// Refusal triage (D3's missing-half form): a file already carrying
/// frontmatter is a specodelic file, not a plain delta
/// ([`MigrateError::AlreadyMigrated`]); a `## Requirements` section
/// without a delta heading is a plain spec, not a delta
/// ([`MigrateError::NotADelta`]); neither delta section present is
/// nothing to wrap ([`MigrateError::NoDeltaSection`]); a bare
/// `## ADDED Requirements` heading at EOF (specodelic-7i7) is not a
/// section ([`MigrateError::BareDeltaHeading`]).
pub fn migrate(text: &str, path: &std::path::Path) -> Result<WrapOutcome, MigrateError> {
    // specodelic-7i7: a bare `## ADDED Requirements` heading at EOF (no
    // trailing newline) is not a section — refuse labeled.
    if has_bare_heading(text, "## ADDED Requirements")
        || has_bare_heading(text, "## MODIFIED Requirements")
    {
        return Err(MigrateError::BareDeltaHeading);
    }
    let has_added = section_span(text, "## ADDED Requirements").is_some();
    // specodelic-nvs9 (gh-16): a MODIFIED section alone is a wrappable
    // delta — the triage accepts ADDED, MODIFIED, or both.
    let has_modified = section_span(text, "## MODIFIED Requirements").is_some();
    let has_requirements = section_span(text, "## Requirements").is_some();
    let has_frontmatter = text.starts_with("---\n") || text.starts_with("---\r\n");
    match (has_frontmatter, has_added || has_modified, has_requirements) {
        // Already a specodelic file: frontmatter present ⇒ refuse.
        (true, _, _) => return Err(MigrateError::AlreadyMigrated),
        // Plain spec: Requirements without a delta heading ⇒ refuse.
        (_, false, true) => return Err(MigrateError::NotADelta),
        // Nothing to wrap (neither ADDED nor MODIFIED present).
        (_, false, false) => return Err(MigrateError::NoDeltaSection),
        _ => {}
    }

    // The sidecar id: the parent-directory id (the capability dir the
    // delta sits in) plus a `.layer` suffix — the naming law's sidecar
    // arm (Revision 19). A bare delta file (no parent dir) falls back to
    // the stem law, matching expected_id_from_path.
    let parent_id = path
        .parent()
        .and_then(|p| p.file_name())
        .and_then(|s| s.to_str())
        .map(|d| d.replace('-', "."))
        .unwrap_or_else(|| {
            path.file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("spec")
                .replace('-', ".")
        });
    let sidecar_id = format!("{parent_id}.layer");

    let eol = if text.contains("\r\n") { "\r\n" } else { "\n" };
    let mut inserted_layers: Vec<&'static str> = Vec::new();
    let mut content = generated_frontmatter(eol, &sidecar_id);
    for layer in ["## Constraints", "## Model", "## Properties"] {
        content.push_str(&scaffold_layer(layer, eol, &sidecar_id));
        inserted_layers.push(match layer {
            "## Constraints" => "Constraints",
            "## Model" => "Model",
            _ => "Properties",
        });
    }

    Ok(WrapOutcome {
        sidecar_id,
        content,
        inserted_layers,
    })
}

/// True when a `## <heading>` line occurs at a line start but is not
/// terminated by a newline (LF or CRLF) — a bare heading at EOF, the
/// degenerate one-line delta (specodelic-7i7). Such a heading opens no
/// section; `migrate` refuses labeled instead of misreading it.
fn has_bare_heading(text: &str, heading: &str) -> bool {
    let mut from = 0;
    while let Some(i) = text[from..].find(heading) {
        let at = from + i;
        let at_line_start = at == 0 || text.as_bytes()[at - 1] == b'\n';
        if at_line_start && !text[at + heading.len()..].starts_with(['\n', '\r']) {
            return true;
        }
        from = at + heading.len();
    }
    false
}

/// Byte span of a `## <name>` section: from the heading line's start to
/// the next `## ` heading (exclusive) or EOF. Indices are relative to
/// `text`.
fn section_span(text: &str, heading: &str) -> Option<(usize, usize)> {
    let hay = text;
    // Heading at a line start: either the very beginning or after \n.
    // A heading must be a complete line (terminated by \n or \r\n) to
    // open a section — a bare heading not terminated by a newline (the
    // degenerate one-line delta, specodelic-7i7) falls through to the
    // needle search, which can never match it: the caller refuses
    // labeled instead of misreading it.
    let start = if hay.starts_with(heading)
        && (hay[heading.len()..].starts_with('\n') || hay[heading.len()..].starts_with("\r\n"))
    {
        0
    } else {
        let needle = format!("\n{heading}\n");
        hay.find(&needle)? + 1
    };
    let body_after = start + heading.len() + 1; // past heading + its newline
    let end = text[body_after..]
        .find("\n## ")
        .map(|i| body_after + i + 1)
        .unwrap_or(text.len());
    Some((start, end))
}

/// Generated sidecar frontmatter: the parent-dir + `.layer` id (the
/// naming law's sidecar arm, Revision 19), EARS scaffold statement (D4)
/// — the author replaces it.
fn generated_frontmatter(eol: &str, id: &str) -> String {
    format!(
        "---{eol}\
         id: {id}{eol}\
         kind: intent{eol}\
         statement: \"WHEN the migrated delta is elaborated, THE author SHALL replace this scaffold statement with the real requirement.\"{eol}\
         ---{eol}{eol}"
    )
}

/// Wired scaffold for one layer (D4): rows reference each other and the
/// sidecar's own intent so the file lints clean before the author writes
/// anything real.
fn scaffold_layer(layer: &str, eol: &str, id: &str) -> String {
    let p = SCAFFOLD_PREFIX;
    let self_ref = format!("[[{id}.{p}constraint]]");
    match layer {
        "## Constraints" => format!(
            "## Constraints{eol}{eol}\
             | id | kind | expr | traces_to |{eol}\
             |----|------|------|-----------|{eol}\
             | {p}constraint | invariant | `true` | [[{id}]] |{eol}{eol}"
        ),
        "## Model" => format!(
            "## Model{eol}{eol}\
             ### States{eol}{eol}\
             - `draft`{eol}{eol}\
             ### Transitions{eol}{eol}\
             | id | from | to | guard |{eol}\
             |----|------|----|-------|{eol}\
             | {p}transition | draft | draft | {self_ref} |{eol}{eol}"
        ),
        _ => format!(
            "## Properties{eol}{eol}\
             | id | kind | derives_from | generator | predicate |{eol}\
             |----|------|--------------|-----------|-----------|{eol}\
             | {p}property | unit | {self_ref} | `todo()` | `true` |{eol}{eol}"
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn plain_delta() -> String {
        "## ADDED Requirements\n\n### Requirement: Widget\nThe system SHALL wiget.\n".to_string()
    }

    /// Fixture path: a spec.md under a capability directory — the
    /// sidecar id derives from the parent dir plus `.layer` (Revision 19).
    fn spec_path() -> std::path::PathBuf {
        std::path::PathBuf::from("demo-cap/spec.md")
    }

    // --- refusals ---

    #[test]
    fn frontmatter_carrying_file_is_refused() {
        let dual = "---\nid: spec\nkind: intent\nstatement: \"THE x SHALL x.\"\n---\n\n## ADDED Requirements\n\n### Requirement: W\nThe system SHALL w.\n";
        assert_eq!(
            migrate(dual, &spec_path()),
            Err(MigrateError::AlreadyMigrated)
        );
    }

    #[test]
    fn plain_spec_is_refused() {
        let plain = "# Title\n\n## Requirements\n\n### Requirement: R\nThe system SHALL r.\n";
        assert_eq!(
            migrate(plain, &spec_path()),
            Err(MigrateError::NotADelta)
        );
    }

    #[test]
    fn no_delta_section_is_refused() {
        let prose = "# Title\n\nSome prose, no sections at all.\n";
        assert_eq!(
            migrate(prose, &spec_path()),
            Err(MigrateError::NoDeltaSection)
        );
    }

    #[test]
    fn degenerate_one_line_delta_is_refused_labeled() {
        let bare = "## ADDED Requirements";
        assert_eq!(
            migrate(bare, &spec_path()),
            Err(MigrateError::BareDeltaHeading)
        );
        let bare_modified = "## MODIFIED Requirements\n\n### Requirement: W\nThe system SHALL w.\n\n## ADDED Requirements";
        assert_eq!(
            migrate(bare_modified, &spec_path()),
            Err(MigrateError::BareDeltaHeading)
        );
    }

    // --- the wrap ---

    #[test]
    fn wraps_plain_delta_into_a_sidecar_leaving_the_delta_alone() {
        let delta = plain_delta();
        let out = migrate(&delta, &spec_path()).expect("migrates");
        // The sidecar id is the parent-dir id + .layer (Revision 19).
        assert_eq!(out.sidecar_id, "demo.cap.layer");
        assert!(out.content.contains("id: demo.cap.layer\n"), "{}", out.content);
        // The sidecar carries the layer skeleton — and no mirror, no
        // delta grammar.
        assert!(out.content.contains("## Constraints"));
        assert!(out.content.contains("## Model"));
        assert!(out.content.contains("## Properties"));
        assert!(!out.content.contains("## ADDED Requirements"));
        assert!(!out.content.contains("## Requirements"));
        // The transform never touches the delta text.
        assert_eq!(delta, plain_delta(), "delta bytes unchanged");
        assert_eq!(out.inserted_layers, vec!["Constraints", "Model", "Properties"]);
    }

    #[test]
    fn sidecar_id_ignores_the_delta_filename() {
        // A non-spec.md delta still sits in the capability directory —
        // the sidecar id derives from the PARENT DIRECTORY, not the stem.
        let out = migrate(&plain_delta(), std::path::Path::new("specs/cap-a/delta.md"))
            .expect("migrates");
        assert_eq!(out.sidecar_id, "cap.a.layer");
    }

    #[test]
    fn crlf_delta_gets_crlf_sidecar() {
        let delta = "## ADDED Requirements\r\n\r\n### Requirement: Widget\r\nThe system SHALL wiget.\r\n";
        let out = migrate(delta, &spec_path()).expect("migrates");
        assert!(out.content.contains("\r\n"), "CRLF preserved in sidecar");
        assert!(!out.content.contains("\n\n"), "no LF blank lines leak");
    }

    #[test]
    fn no_delta_section_message_names_both_accepted_sections() {
        let err = migrate("# nothing\n", &spec_path()).unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("## ADDED Requirements") && msg.contains("## MODIFIED Requirements"));
    }

    #[test]
    fn scaffold_lints_clean_through_the_real_linter() {
        let delta = plain_delta();
        let out = migrate(&delta, &spec_path()).expect("migrates");
        let spec = crate::spec::parse_str(&out.content).expect("sidecar parses as a spec");
        assert_eq!(spec.intent.id, "demo.cap.layer");
        let report = crate::lint::lint_corpus(&[spec]);
        assert!(
            report.issues.is_empty(),
            "sidecar scaffold must lint clean: {:?}",
            report.issues.iter().map(|i| &i.message).collect::<Vec<_>>()
        );
    }
}

#[cfg(test)]
mod rekey_tests {
    use super::*;

    fn spec_path() -> std::path::PathBuf {
        std::path::PathBuf::from("openspec/specs/ge-cli/spec.md")
    }

    /// A 0.6.0-era dual-format file: `id: spec` + `[[spec.*]]` refs.
    fn legacy_file() -> String {
        "---\nid: spec\nkind: intent\nstatement: \"THE ge cli SHALL hold\"\n---\n\n\
         ## Constraints\n\n\
         | id | kind | expr | traces_to |\n\
         |----|------|------|-----------|\n\
         | c1 | invariant | `x` | [[spec]] |\n\
         | c2 | invariant | `y` | [[spec.c1]] |\n\n\
         ## Model\n\n\
         ### States\n\n- `s1`\n\n\
         ### Transitions\n\n\
         | id | from | to | guard |\n\
         |----|------|----|-------|\n\
         | t | s1 | s1 | [[spec.c1]] |\n\n\
         ## Properties\n\n\
         | id | kind | derives_from | generator | predicate |\n\
         |----|------|--------------|-----------|------------|\n\
         | p | unit | [[spec.c1]] | `g()` | `x` |\n\n\
         ## Requirements\n\n\
         ### Requirement: Ge cli\nThe ge cli SHALL hold.\n"
            .to_string()
    }

    #[test]
    fn rekey_rewrites_id_and_refs() {
        let out = rekey(&legacy_file(), &spec_path()).expect("rekeys");
        assert!(out.content.contains("id: ge.cli\n"), "{}", out.content);
        assert!(out.content.contains("[[ge.cli]]"));
        assert!(out.content.contains("[[ge.cli.c1]]"));
        assert!(
            !out.content.contains("[[spec"),
            "no spec-qualified ref survives"
        );
        // Everything else byte-identical: strip the id line and refs, the
        // skeleton is untouched.
        assert!(out.content.contains("| c1 | invariant | `x` |"));
        assert!(
            out.content
                .contains("### Requirement: Ge cli\nThe ge cli SHALL hold.")
        );
    }

    #[test]
    fn rekey_refuses_plain_delta() {
        let delta = "## ADDED Requirements\n\n### Requirement: W\nThe system SHALL w.\n";
        let err = rekey(delta, &spec_path()).unwrap_err();
        assert_eq!(err, MigrateError::NotADelta);
    }

    #[test]
    fn rekey_refuses_bare_spec_md() {
        let err = rekey(&legacy_file(), std::path::Path::new("spec.md")).unwrap_err();
        assert!(matches!(err, MigrateError::NoDerivableId));
    }

    #[test]
    fn rekey_idempotent_second_run_is_noop() {
        let out = rekey(&legacy_file(), &spec_path()).expect("rekeys");
        // Second run: id is already real — a no-op.
        let second = rekey(&out.content, &spec_path()).expect("second run ok");
        assert_eq!(second.content, out.content, "no-op never rewrites");
        assert!(second.rekeyed.is_none() || !second.rekeyed.unwrap());
    }

    #[test]
    fn rekey_preserves_prose_mentions() {
        let file = legacy_file().replace(
            "### Requirement: Ge cli\n",
            "### Requirement: Ge cli\nSee spec.c1 prose mention — not a wiki-link.\n",
        );
        let out = rekey(&file, &spec_path()).expect("rekeys");
        assert!(
            out.content.contains("See spec.c1 prose mention"),
            "prose (non-bracketed) mentions stay untouched: {}",
            out.content
        );
    }

    #[test]
    fn rekey_dry_run_semantics_via_outcome() {
        // The command layer owns --dry-run; the library transform is pure.
        let out = rekey(&legacy_file(), &spec_path()).expect("rekeys");
        assert!(out.rekeyed.unwrap());
        assert!(!out.inserted_frontmatter);
        assert!(out.inserted_layers.is_empty());
        assert!(!out.inserted_mirror);
    }
}
