//! The openspec dual-format view — derived from the raw source text.
//!
//! Purpose: keep the openspec integration's metadata (which delta and
//! capability headings a file carries, and their verbatim bodies) out of
//! the core [`crate::spec::Spec`] IR. Responsibilities: scan the source
//! text for `## ADDED Requirements` / `## Requirements` / `## MODIFIED
//! Requirements` headings, record their presence, and capture each
//! section's body verbatim; insert that view into the parse envelope
//! under the historical key names. Rationale: the Spec IR models the
//! four specodelic layers only; the openspec delta view is an adapter
//! projection over the same bytes (`dual_format_view_is_derived_from_source`),
//! so a consumer that wants it asks for it and the IR stays single-purpose.

use serde_json::Value;

/// The openspec dual-format view of one spec source file.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DualFormat {
    /// The source carries an `## ADDED Requirements` heading (the openspec
    /// delta half of a dual-format file).
    pub has_added_requirements: bool,
    /// The source carries a `## Requirements` heading (the specodelic
    /// capability half of a dual-format file).
    pub has_requirements_section: bool,
    /// Body of `## ADDED Requirements` (lines rstripped, joined).
    pub added_requirements_body: String,
    /// The source carries a `## MODIFIED Requirements` heading (the
    /// repo's first MODIFIED delta — the mirror rules treat it exactly
    /// like the ADDED one).
    pub has_modified_requirements: bool,
    /// Body of `## MODIFIED Requirements` (lines rstripped, joined).
    pub modified_requirements_body: String,
    /// Body of `## Requirements` (lines rstripped, joined).
    pub requirements_body: String,
}

/// Which requirement section the scan is currently inside.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
enum Section {
    #[default]
    None,
    Added,
    Specodelic,
    Modified,
}

impl DualFormat {
    /// Derive the view from a spec file's raw text: the same heading
    /// recognition and verbatim capture the parser used to perform
    /// inline — every line except the `## ` headings themselves, with
    /// `### Requirement:` lines and blank lines included.
    pub fn from_source(source: &str) -> DualFormat {
        let mut view = DualFormat::default();
        let mut section = Section::None;
        for line in source.lines().skip(crate::spec::frontmatter_lines(source)) {
            let t = line.trim();
            if let Some(heading) = t.strip_prefix("## ") {
                section = match heading.trim() {
                    "ADDED Requirements" => Section::Added,
                    "Requirements" => Section::Specodelic,
                    "MODIFIED Requirements" => Section::Modified,
                    _ => Section::None,
                };
                match section {
                    Section::Added => view.has_added_requirements = true,
                    Section::Specodelic => view.has_requirements_section = true,
                    Section::Modified => view.has_modified_requirements = true,
                    Section::None => {}
                }
                continue;
            }
            let body = match section {
                Section::Added => &mut view.added_requirements_body,
                Section::Specodelic => &mut view.requirements_body,
                Section::Modified => &mut view.modified_requirements_body,
                Section::None => continue,
            };
            body.push_str(line.trim_end());
            body.push('\n');
        }
        view
    }

    /// Insert the six historical `spk parse` keys into a JSON payload
    /// (the envelope `data` object). Non-object payloads are left alone.
    pub fn insert_into(&self, payload: &mut Value) {
        let Some(map) = payload.as_object_mut() else {
            return;
        };
        map.insert(
            "has_added_requirements".into(),
            self.has_added_requirements.into(),
        );
        map.insert(
            "has_requirements_section".into(),
            self.has_requirements_section.into(),
        );
        map.insert(
            "added_requirements_body".into(),
            self.added_requirements_body.clone().into(),
        );
        map.insert(
            "has_modified_requirements".into(),
            self.has_modified_requirements.into(),
        );
        map.insert(
            "modified_requirements_body".into(),
            self.modified_requirements_body.clone().into(),
        );
        map.insert(
            "requirements_body".into(),
            self.requirements_body.clone().into(),
        );
    }
}
