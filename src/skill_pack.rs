//! First-party managed skill packs — the whisper `turu skill_pack`
//! precedent (add-skill-pack, design D1/D5).
//!
//! Purpose: ship the `specodelic-design` skill from this repo so the
//! skill and the binary evolve in the same commit. Responsibilities:
//! generate the pack file map (`generate_pack`), hash a pack
//! deterministically for staleness detection (`pack_content_hash`), and
//! render the managed marker at generate time — tracked sources stay
//! marker-free. The pack core is pure: no filesystem access here; the
//! install verb (phase 4) and the doctor check (phase 5) consume it.
//!
//! Rationale: content lives in tracked markdown compiled in via
//! `include_str!` (editable with the repo's doc tooling), while the
//! membership/lint tests (later phases) back the generated output —
//! drift that motivated this change is made structurally impossible
//! rather than caught by review.

use sha2::{Digest, Sha256};
use std::collections::BTreeMap;

/// Relative path → file content, sorted by path (BTreeMap ordering).
pub type PackFiles = BTreeMap<String, String>;

/// Prepended to every generated file. Names the regenerate command so a
/// manual edit is detectable and remediable by construction (proposal,
/// design D6).
pub const MANAGED_MARKER: &str =
    "<!-- MANAGED BY specodelic — DO NOT EDIT MANUALLY. Regenerate: spk skill install -->\n\n";

/// Generate all files for a named first-party managed skill pack.
pub fn generate_pack(pack_name: &str) -> Result<PackFiles, String> {
    match pack_name {
        "specodelic-design" => Ok(generate_design_pack()),
        other => Err(format!(
            "unknown managed skill pack: {other:?} (known packs: specodelic-design)"
        )),
    }
}

/// SHA-256 over the pack files sorted by relative path, concatenated as
/// `path + NUL + content`. Pure function of the generated pack — the
/// doctor staleness check (phase 5) recomputes both sides from it.
pub fn pack_content_hash(files: &PackFiles) -> String {
    let mut hasher = Sha256::new();
    for (path, content) in files {
        hasher.update(path.as_bytes());
        hasher.update(b"\0");
        hasher.update(content.as_bytes());
    }
    let digest = hasher.finalize();
    digest.iter().map(|b| format!("{b:02x}")).collect()
}

/// Placeholder open delimiter: `<!-- spk:render:` ... `-->` (design D3 —
/// HTML-comment markers are invisible in rendered markdown and immune to
/// mdbook's `{{ }}` templating collision).
const PLACEHOLDER_OPEN: &str = "<!-- spk:render:";

/// Render `<!-- spk:render:* -->` placeholders in a tracked source body.
/// A placeholder with no registered renderer is a labeled generation
/// failure naming the identifier — never a silent passthrough
/// (tasks.md §1.3). Phase 1 registers no renderers, so any placeholder
/// is unknown; phase 2 (specodelic-ifm7) extends this with the
/// explain-derived render table.
pub fn render_body(body: &str) -> Result<String, String> {
    let Some(start) = body.find(PLACEHOLDER_OPEN) else {
        return Ok(body.to_string());
    };
    let after = &body[start + PLACEHOLDER_OPEN.len()..];
    let Some(end) = after.find("-->") else {
        return Err(format!(
            "unterminated render placeholder: spk:render:{} (missing `-->`)",
            after.trim()
        ));
    };
    Err(format!(
        "unknown render placeholder: spk:render:{} (no renderer registered)",
        after[..end].trim()
    ))
}

fn generate_design_pack() -> PackFiles {
    let mut files = PackFiles::new();
    files.insert(
        "SKILL.md".to_string(),
        marked(include_str!("../skill-packs/specodelic-design/SKILL.md")),
    );
    files.insert(
        "references/format.md".to_string(),
        marked(include_str!(
            "../skill-packs/specodelic-design/references/format.md"
        )),
    );
    files.insert(
        "references/example.md".to_string(),
        marked(include_str!(
            "../skill-packs/specodelic-design/references/example.md"
        )),
    );
    files
}

/// Render placeholders in the body, then prepend the marker. Tracked
/// sources stay marker-free and placeholder-free-at-REST; the render and
/// the marker are applied exactly once, at generate time.
fn marked(body: &str) -> String {
    // `render_body` failures are pack-generation failures; the Ok/Err
    // split keeps the render pipeline testable independently.
    let rendered =
        render_body(body).expect("tracked source must render cleanly (see tests/skill_pack.rs)");
    format!("{MANAGED_MARKER}{rendered}")
}
