//! `spk migrate` — wrap an existing openspec delta file in place into the
//! dual-format four-layer skeleton (gh#6 item 1, beads specodelic-c32).
//!
//! The pure seam: [`migrate`] transforms in-memory text, the CLI arm
//! (src/main.rs) does file IO and envelope emission. Design decisions live
//! in openspec/changes/add-spk-migrate/design.md — notably D2 (the mirror
//! is the migration marker: refuse anything that already has it), D3
//! (merge adds only missing pieces), D4 (the scaffold is wired so it
//! lints clean as written), D5 (no backup files — `--dry-run` is the
//! preview path), D6 (byte fidelity: the mirror is a byte-exact slice,
//! CRLF preserved) — extended for specodelic-54v: a delta may carry both
//! ADDED and MODIFIED requirements, and the mirror must aggregate every
//! requirement from every carried delta section or
//! `linter.requirement_drift` (per-requirement comparison across all
//! carried sections) fails the freshly migrated file.

use std::fmt;

/// Why a file cannot be migrated (specs/migrate, refuse_already_migrated).
#[derive(Debug, PartialEq, Clone)]
pub enum MigrateError {
    /// Frontmatter + `## Requirements` both present — already dual-format.
    AlreadyMigrated,
    /// `## Requirements` without `## ADDED Requirements` — a plain spec.
    NotADelta,
    /// Neither section present — nothing to wrap.
    NoDeltaSection,
    /// The same requirement heading appears in both the ADDED and the
    /// MODIFIED sections (specodelic-54v): the mirror can hold only one
    /// text per requirement, so aggregating both sections faithfully is
    /// impossible — migrating would guarantee a
    /// `linter.requirement_drift` failure.
    ConflictingDelta { heading: String },
}

impl fmt::Display for MigrateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MigrateError::AlreadyMigrated => write!(
                f,
                "file already carries frontmatter and a ## Requirements mirror — already dual-format"
            ),
            MigrateError::NotADelta => write!(
                f,
                "file has ## Requirements but no ## ADDED Requirements — a plain spec, not an openspec delta"
            ),
            MigrateError::NoDeltaSection => {
                write!(
                    f,
                    "no ## ADDED Requirements section found — nothing to wrap"
                )
            }
            MigrateError::ConflictingDelta { heading } => write!(
                f,
                "requirement `{heading}` appears in both ## ADDED Requirements and ## MODIFIED Requirements — the Requirements mirror cannot hold two different texts for one requirement, so migrating would guarantee a linter.requirement_drift failure"
            ),
        }
    }
}

/// Outcome of a migration: the new content plus what was inserted.
#[derive(Debug, PartialEq, Clone)]
pub struct MigrateOutcome {
    /// The full rewritten file content.
    pub content: String,
    /// True when frontmatter was generated (vs kept verbatim).
    pub inserted_frontmatter: bool,
    /// Names of the layer sections inserted (empty when all present).
    pub inserted_layers: Vec<&'static str>,
    /// True when the `## Requirements` mirror was appended.
    pub inserted_mirror: bool,
}

/// Inserted scaffold rows carry this prefix — the author's hand-finish
/// checklist is "replace every scaffold_* id with the real thing".
pub const SCAFFOLD_PREFIX: &str = "scaffold_";

/// Wrap an openspec delta into the dual-format skeleton.
///
/// Merge semantics (D3): existing frontmatter and existing layer sections
/// pass through verbatim; only missing pieces are inserted. The mirror
/// aggregates every carried delta section's requirements per-requirement
/// (specodelic-54v): the `## ADDED Requirements` body byte-exact (D6),
/// plus the `## MODIFIED Requirements` body in its modified form — the
/// same per-requirement comparison `linter.requirement_drift` performs.
pub fn migrate(text: &str) -> Result<MigrateOutcome, MigrateError> {
    let has_added = section_span(text, "## ADDED Requirements").is_some();
    let has_mirror = section_span(text, "## Requirements").is_some();
    let has_frontmatter = text.starts_with("---\n") || text.starts_with("---\r\n");
    match (has_frontmatter, has_added, has_mirror) {
        // Already migrated (D2): frontmatter + mirror ⇒ refuse.
        (true, _, true) => return Err(MigrateError::AlreadyMigrated),
        // Plain spec: mirror without the delta heading ⇒ refuse.
        (_, false, true) => return Err(MigrateError::NotADelta),
        // Nothing to wrap.
        (_, false, false) => return Err(MigrateError::NoDeltaSection),
        _ => {}
    }

    let eol = if text.contains("\r\n") { "\r\n" } else { "\n" };
    let mut inserted_layers: Vec<&'static str> = Vec::new();
    let mut out = String::new();

    // Frontmatter (D3: keep existing verbatim; generate only when absent).
    let body_start = if has_frontmatter {
        let close = if eol == "\r\n" {
            "\r\n---\r\n"
        } else {
            "\n---\n"
        };
        let end = text
            .find(close)
            .map(|i| i + close.len())
            .expect("frontmatter opening checked above");
        out.push_str(&text[..end]);
        end
    } else {
        out.push_str(&generated_frontmatter(eol));
        0
    };
    let body = &text[body_start..];

    // Layer sections in canonical order; insert the scaffold for any
    // missing one (D4 wired scaffold). Each present section is copied
    // verbatim up to the next `## ` heading (or EOF).
    let mut cursor = 0usize;
    for (layer, name) in [
        ("## Constraints", "Constraints"),
        ("## Model", "Model"),
        ("## Properties", "Properties"),
    ] {
        match section_span_from(body, layer, cursor) {
            Some((_, end)) => {
                out.push_str(&body[cursor..end]);
                cursor = end;
            }
            None => {
                out.push_str(&scaffold_layer(layer, eol));
                inserted_layers.push(name);
            }
        }
    }

    // The delta sections, verbatim through their bodies. MODIFIED is
    // located over the whole body (it may sit before the layers, between
    // them and ADDED, or in the trailing region — all copied verbatim
    // below, never destroyed).
    let (added_start, added_end) =
        section_span_from(body, "## ADDED Requirements", cursor).expect("has_added checked above");
    let added_body = section_body(body, added_start, added_end).to_string();
    out.push_str(&body[cursor..added_end]);
    cursor = added_end;
    // Trailing content after the delta section (rare, but never destroy).
    out.push_str(&body[cursor..]);

    // Mirror (D6 byte-exact slice, extended specodelic-54v): every
    // requirement from every carried delta section, under the Requirements
    // heading — the ADDED body byte-exact, the MODIFIED body in its
    // modified form, so the fresh file satisfies linter.requirement_drift's
    // per-requirement comparison. Trimmed to exactly one trailing newline;
    // sections joined by one blank line.
    let mut mirror_parts = vec![added_body.trim_end_matches(['\r', '\n'])];
    if let Some((modified_start, modified_end)) =
        section_span_from(body, "## MODIFIED Requirements", 0)
    {
        let modified_body = section_body(body, modified_start, modified_end);
        let modified = modified_body.trim_matches(['\r', '\n']);
        let modified_headings = requirement_headings(modified_body);
        for heading in requirement_headings(&added_body) {
            if modified_headings.contains(&heading) {
                return Err(MigrateError::ConflictingDelta { heading });
            }
        }
        mirror_parts.push(modified);
    }
    let separator = format!("{eol}{eol}");
    out.push_str(eol);
    out.push_str("## Requirements");
    out.push_str(eol);
    out.push_str(&mirror_parts.join(&separator));
    out.push_str(eol);

    Ok(MigrateOutcome {
        content: out,
        inserted_frontmatter: !has_frontmatter,
        inserted_layers,
        inserted_mirror: true,
    })
}

/// The body text of a section span: everything after the heading line
/// (works for both LF and CRLF files).
fn section_body(text: &str, start: usize, end: usize) -> &str {
    let heading_len = text[start..end].find('\n').map(|i| i + 1).unwrap_or(0);
    &text[start + heading_len..end]
}

/// The requirement headings (`### Requirement: X`) carried by a delta
/// section body, in order.
fn requirement_headings(body: &str) -> Vec<String> {
    body.lines()
        .filter_map(|l| l.trim().strip_prefix("### Requirement:"))
        .map(|h| h.trim().to_string())
        .collect()
}

/// Byte span of a `## <name>` section: from the heading line's start to
/// the next `## ` heading (exclusive) or EOF. Indices are relative to
/// `text`.
fn section_span(text: &str, heading: &str) -> Option<(usize, usize)> {
    section_span_from(text, heading, 0)
}

fn section_span_from(text: &str, heading: &str, from: usize) -> Option<(usize, usize)> {
    let hay = &text[from..];
    // Heading at a line start: either the very beginning or after \n.
    let start = if hay.starts_with(heading) {
        from
    } else {
        let needle = format!("\n{heading}\n");
        from + hay.find(&needle)? + 1
    };
    let body_after = start + heading.len() + 1; // past heading + its newline
    let end = text[body_after..]
        .find("\n## ")
        .map(|i| body_after + i + 1)
        .unwrap_or(text.len());
    Some((start, end))
}

/// Generated frontmatter: `id: spec` per the openspec naming law, EARS
/// scaffold statement (D4) — the author replaces it.
fn generated_frontmatter(eol: &str) -> String {
    format!(
        "---{eol}\
         id: spec{eol}\
         kind: intent{eol}\
         statement: \"WHEN the migrated delta is elaborated, THE author SHALL replace this scaffold statement with the real requirement.\"{eol}\
         ---{eol}{eol}"
    )
}

/// Wired scaffold for one missing layer (D4): rows reference each other
/// so the file lints clean before the author writes anything real.
fn scaffold_layer(layer: &str, eol: &str) -> String {
    let p = SCAFFOLD_PREFIX;
    let self_ref = format!("[[spec.{p}constraint]]");
    match layer {
        "## Constraints" => format!(
            "## Constraints{eol}{eol}\
             | id | kind | expr | traces_to |{eol}\
             |----|------|------|-----------|{eol}\
             | {p}constraint | invariant | `true` | [[spec]] |{eol}{eol}"
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

    // --- D2 refusals ---

    #[test]
    fn already_migrated_is_refused() {
        let dual = "---\nid: spec\nkind: intent\nstatement: \"THE x SHALL x.\"\n---\n\n## ADDED Requirements\n\n### Requirement: W\nThe system SHALL w.\n\n## Requirements\n\n### Requirement: W\nThe system SHALL w.\n";
        assert_eq!(migrate(dual), Err(MigrateError::AlreadyMigrated));
    }

    #[test]
    fn plain_spec_is_refused() {
        let plain = "# Title\n\n## Requirements\n\n### Requirement: R\nThe system SHALL r.\n";
        assert_eq!(migrate(plain), Err(MigrateError::NotADelta));
    }

    #[test]
    fn no_delta_section_is_refused() {
        assert_eq!(
            migrate("# Just prose\n\nSome text.\n"),
            Err(MigrateError::NoDeltaSection)
        );
    }

    // --- D1/D4 wrap ---

    #[test]
    fn wraps_plain_delta_with_frontmatter_and_mirror() {
        let delta = plain_delta();
        let out = migrate(&delta).expect("migrates");
        assert!(out.inserted_frontmatter);
        assert_eq!(out.inserted_layers.len(), 3, "all three layers inserted");
        assert!(out.inserted_mirror);
        assert!(out.content.starts_with("---\nid: spec\nkind: intent\n"));
        // Delta text preserved byte-for-byte.
        assert!(
            out.content.contains(
                "## ADDED Requirements\n\n### Requirement: Widget\nThe system SHALL wiget.\n"
            ),
            "ADDED body preserved: {}",
            out.content
        );
        // Mirror is the byte-exact slice under the Requirements heading.
        let body = "### Requirement: Widget\nThe system SHALL wiget.\n";
        assert!(
            out.content.contains(&format!("## Requirements\n\n{body}")),
            "mirror present: {}",
            out.content
        );
        assert_eq!(
            out.content.matches("## Requirements").count(),
            1,
            "exactly one mirror"
        );
    }

    // --- D3 merge ---

    #[test]
    fn merge_keeps_existing_frontmatter_verbatim() {
        let delta = "---\nid: spec\nkind: intent\nstatement: \"THE real SHALL hold.\"\n---\n\n## ADDED Requirements\n\n### Requirement: W\nThe system SHALL w.\n";
        let out = migrate(delta).expect("migrates");
        assert!(!out.inserted_frontmatter, "frontmatter kept, not generated");
        assert!(
            out.content.starts_with(
                "---\nid: spec\nkind: intent\nstatement: \"THE real SHALL hold.\"\n---\n"
            ),
            "existing frontmatter verbatim: {}",
            out.content
        );
        assert!(
            out.content
                .contains("## Requirements\n\n### Requirement: W")
        );
    }

    #[test]
    fn merge_keeps_existing_constraints_layer() {
        let constraints = "## Constraints\n\n| id | kind | expr | traces_to |\n|----|------|------|-----------|\n| real | invariant | `x > 0` | |\n\n";
        let delta = format!(
            "{constraints}## ADDED Requirements\n\n### Requirement: W\nThe system SHALL w.\n"
        );
        let out = migrate(&delta).expect("migrates");
        assert_eq!(out.inserted_layers, vec!["Model", "Properties"]);
        assert!(
            out.content.contains("| real | invariant | `x > 0` | |"),
            "existing constraints verbatim: {}",
            out.content
        );
        assert!(
            !out.content.contains("| scaffold_constraint |"),
            "no row injected into existing layer"
        );
        assert!(out.content.contains("scaffold_transition"));
        assert!(out.content.contains("scaffold_property"));
    }

    // --- D6 byte fidelity ---

    #[test]
    fn mirror_is_byte_exact_when_delta_is_last_section() {
        let delta = "## ADDED Requirements\n\n### Requirement: W\nThe system SHALL w.\n";
        let out = migrate(delta).expect("migrates");
        assert!(
            out.content
                .ends_with("## Requirements\n\n### Requirement: W\nThe system SHALL w.\n")
        );
    }

    #[test]
    fn mirror_is_byte_exact_when_trailing_section_follows() {
        let delta = "## ADDED Requirements\n\n### Requirement: W\nThe system SHALL w.\n\n## Notes\n\nSome notes.\n";
        let out = migrate(delta).expect("migrates");
        let added = out
            .content
            .split("## ADDED Requirements\n")
            .nth(1)
            .and_then(|s| s.split("\n## Notes").next())
            .expect("split");
        let mirror = out
            .content
            .split("## Requirements\n")
            .nth(1)
            .and_then(|s| s.split("\n## Notes").next())
            .expect("split");
        assert_eq!(mirror, added, "mirror == ADDED body");
    }

    #[test]
    fn crlf_is_preserved() {
        let delta = "## ADDED Requirements\r\n\r\n### Requirement: W\r\nThe system SHALL w.\r\n";
        let out = migrate(delta).expect("migrates");
        assert!(out.content.contains("## Requirements\r\n"));
        assert!(
            out.content.contains("\r\n- `draft`\r\n"),
            "scaffold uses the file's CRLF: {}",
            out.content
        );
        assert!(
            !out.content.contains("\n- `draft`\n"),
            "no LF scaffold leaked in"
        );
    }

    #[test]
    fn missing_trailing_newline_is_normalized() {
        let delta = "## ADDED Requirements\n\n### Requirement: W\nThe system SHALL w.";
        let out = migrate(delta).expect("migrates");
        assert!(out.content.ends_with('\n'), "single trailing newline");
        assert!(!out.content.ends_with("\n\n"));
    }

    // --- mixed delta (specodelic-54v): the mirror aggregates ADDED +
    // MODIFIED per-requirement, matching linter.requirement_drift ---

    #[test]
    fn mixed_delta_mirror_holds_both_sections_requirements() {
        let delta = "## ADDED Requirements\n\n### Requirement: One\none holds\n\n## MODIFIED Requirements\n\n### Requirement: Two\ntwo holds\n";
        let out = migrate(delta).expect("migrates");
        let mirror = out
            .content
            .split("## Requirements\n")
            .nth(1)
            .expect("mirror present");
        assert!(
            mirror.contains("### Requirement: One\none holds"),
            "ADDED requirement in mirror: {mirror}"
        );
        assert!(
            mirror.contains("### Requirement: Two\ntwo holds"),
            "MODIFIED requirement in mirror: {mirror}"
        );
    }

    #[test]
    fn mixed_delta_lints_clean_through_the_real_linter() {
        let delta = "## ADDED Requirements\n\n### Requirement: One\none holds\n\n## MODIFIED Requirements\n\n### Requirement: Two\ntwo holds\n";
        let out = migrate(delta).expect("migrates");
        let spec = crate::spec::parse_str(&out.content).expect("migrated file parses");
        let report = crate::lint::lint_corpus(&[spec]);
        assert!(
            report.issues.is_empty(),
            "migrated mixed delta must lint clean: {:?}",
            report.issues.iter().map(|i| &i.message).collect::<Vec<_>>()
        );
    }

    #[test]
    fn mixed_delta_crlf_uses_file_eol_in_mirror() {
        let delta = "## ADDED Requirements\r\n\r\n### Requirement: One\r\none holds\r\n\r\n## MODIFIED Requirements\r\n\r\n### Requirement: Two\r\ntwo holds\r\n";
        let out = migrate(delta).expect("migrates");
        assert!(out.content.contains("## Requirements\r\n"));
        assert!(
            out.content
                .contains("### Requirement: Two\r\ntwo holds\r\n")
        );
        assert!(!out.content.contains("### Requirement: Two\ntwo holds"));
    }

    #[test]
    fn modified_section_between_layers_and_added_is_mirrored() {
        let delta = "## MODIFIED Requirements\n\n### Requirement: Two\ntwo holds\n\n## ADDED Requirements\n\n### Requirement: One\none holds\n";
        let out = migrate(delta).expect("migrates");
        let mirror = out
            .content
            .split("## Requirements\n")
            .nth(1)
            .expect("mirror present");
        assert!(mirror.contains("### Requirement: One"));
        assert!(mirror.contains("### Requirement: Two"));
    }

    #[test]
    fn duplicate_requirement_heading_across_sections_is_refused() {
        let delta = "## ADDED Requirements\n\n### Requirement: One\none holds\n\n## MODIFIED Requirements\n\n### Requirement: One\nmodified holds\n";
        match migrate(delta) {
            Err(MigrateError::ConflictingDelta { heading }) => {
                assert_eq!(heading, "One");
            }
            other => panic!("expected ConflictingDelta, got {other:?}"),
        }
    }

    // --- D4 scaffold lints clean (through the real linter, in-process) ---

    #[test]
    fn scaffold_lints_clean_through_the_real_linter() {
        let delta = plain_delta();
        let out = migrate(&delta).expect("migrates");
        let spec = crate::spec::parse_str(&out.content).expect("scaffold parses as a spec");
        let report = crate::lint::lint_corpus(&[spec]);
        assert!(
            report.issues.is_empty(),
            "scaffold must lint clean: {:?}",
            report.issues.iter().map(|i| &i.message).collect::<Vec<_>>()
        );
    }
}
