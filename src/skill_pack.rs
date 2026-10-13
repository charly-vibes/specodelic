//! First-party managed skill packs — the whisper `turu skill_pack`
//! precedent (add-skill-pack, design D1/D5).
//!
//! Purpose: ship the `specodelic-design` skill from this repo so the
//! skill and the binary evolve in the same commit. Responsibilities:
//! generate the pack file map (`generate_pack`), hash a pack
//! deterministically for staleness detection (`pack_content_hash`),
//! render `<!-- spk:render:* -->` placeholders through the same tables
//! that serve `spk explain` (design D3 — explain-derived lists are
//! rendered, never restated), and enforce the membership gate
//! (`check_membership`) over the hand-authored protocol's mentions.
//! The pack core is pure: no filesystem access here; the install verb
//! (phase 4) and the doctor check (phase 5) consume it.
//!
//! Rationale: content lives in tracked markdown compiled in via
//! `include_str!` (editable with the repo's doc tooling), while the
//! membership and lint tests back the generated output — the drift
//! classes that motivated this change (stale topic/rule lists, a rule
//! cited in a failure table but absent from the served list) become
//! structurally impossible rather than review-caught.

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

/// Placeholder open/close delimiters: `<!-- spk:render:` … `-->`
/// (design D3 — HTML-comment markers are invisible in rendered markdown
/// and immune to mdbook's `{{ }}` templating collision).
const PLACEHOLDER_OPEN: &str = "<!-- spk:render:";
const PLACEHOLDER_CLOSE: &str = "-->";

/// Render every `<!-- spk:render:* -->` placeholder in a tracked source
/// body through the render table. A placeholder with no renderer (or a
/// `topic:<id>` naming no served topic) is a labeled generation failure
/// naming the identifier — never a silent passthrough (tasks.md §1.3,
/// design D3). The render table resolves through the same code paths
/// that serve `spk explain`: the topic bodies come from
/// [`crate::guide::topic_body`], the topics line from
/// [`crate::guide::TOPICS`] — the closed tables cannot be restated here
/// without rendering from them.
pub fn render_body(body: &str) -> Result<String, String> {
    let mut out = String::with_capacity(body.len());
    let mut rest = body;
    while let Some(start) = rest.find(PLACEHOLDER_OPEN) {
        out.push_str(&rest[..start]);
        let after = &rest[start + PLACEHOLDER_OPEN.len()..];
        let Some(end) = after.find(PLACEHOLDER_CLOSE) else {
            return Err(format!(
                "unterminated render placeholder: spk:render:{} (missing `-->`)",
                after.trim()
            ));
        };
        out.push_str(&render_placeholder(after[..end].trim())?);
        rest = &after[end + PLACEHOLDER_CLOSE.len()..];
    }
    out.push_str(rest);
    Ok(out)
}

/// The render table: placeholder id → rendered text.
fn render_placeholder(id: &str) -> Result<String, String> {
    if let Some(topic) = id.strip_prefix("topic:") {
        let topic = topic.trim();
        return crate::guide::topic_body(topic).ok_or_else(|| {
            format!(
                "unknown render placeholder: spk:render:{id} (no served explain topic: {topic:?})"
            )
        });
    }
    match id {
        "explain-topics" => Ok(explain_topics_line()),
        other => Err(format!(
            "unknown render placeholder: spk:render:{other} (no renderer registered)"
        )),
    }
}

/// The topics line content: the served topic ids, in serving order —
/// the same list `spk explain` (no argument) serves.
fn explain_topics_line() -> String {
    crate::guide::TOPICS
        .iter()
        .map(|(id, _)| *id)
        .collect::<Vec<_>>()
        .join(", ")
}

// ---- membership gate (tasks.md §2.1, design D3) ----
//
// Scope note: the gate runs over the hand-authored protocol text (and
// any fixture) — the rendered `references/format.md` bodies are
// table-derived by construction (they are the served `spk explain`
// text, set-equality-tested in `tests/skill_pack.rs`), so their rule
// mentions cannot drift from the tables they are rendered from.

/// Membership gate over generated pack text: every topic name, lint
/// rule id, and EARS pattern name *mentioned* in the text must resolve
/// against the in-binary tables that serve `spk explain`. A
/// hand-maintained list may be stale; a generated one cannot — this is
/// the test-enforced counterpart of the render pipeline. Fails naming
/// the unresolvable identifier.
pub fn check_membership(text: &str) -> Result<(), String> {
    check_topic_mentions(text)?;
    check_explain_command_mentions(text)?;
    check_rule_id_mentions(text)?;
    check_ears_pattern_mentions(text)?;
    Ok(())
}

/// Every `topics:` comma list must name served topic ids.
fn check_topic_mentions(text: &str) -> Result<(), String> {
    for line in text.lines() {
        if !line.contains("topics:") {
            continue;
        }
        for id in topics_mentioned(line) {
            if !crate::guide::TOPICS.iter().any(|(t, _)| *t == id) {
                return Err(format!(
                    "unresolvable topic mention: `{id}` is not a served spk explain topic (served: {})",
                    explain_topics_line()
                ));
            }
        }
    }
    Ok(())
}

/// The topic ids mentioned in a `topics:` line, in mention order.
pub fn topics_mentioned(line: &str) -> Vec<String> {
    let Some(idx) = line.find("topics:") else {
        return Vec::new();
    };
    let list = &line[idx + "topics:".len()..];
    let list = list.split(')').next().unwrap_or(list);
    list.split(',')
        .map(|e| e.trim().to_string())
        .filter(|e| !e.is_empty())
        .collect()
}

/// Every `spk explain <token>` mention must name a served topic
/// (`<topic>` metasyntax and bare `spk explain` are not mentions).
fn check_explain_command_mentions(text: &str) -> Result<(), String> {
    for mention in explain_command_mentions(text) {
        if !crate::guide::TOPICS.iter().any(|(t, _)| *t == mention) {
            return Err(format!(
                "unresolvable topic mention: `spk explain {mention}` is not a served spk explain topic (served: {})",
                explain_topics_line()
            ));
        }
    }
    Ok(())
}

/// The tokens directly following a `spk explain ` mention, where they
/// look like topic ids (lowercase word chars and hyphens).
fn explain_command_mentions(text: &str) -> Vec<String> {
    let mut found = Vec::new();
    let mut rest = text;
    while let Some(idx) = rest.find("spk explain ") {
        let after = &rest[idx + "spk explain ".len()..];
        let token: String = after
            .chars()
            .take_while(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || *c == '-')
            .collect();
        if !token.is_empty() {
            found.push(token);
        }
        rest = after;
    }
    found
}

/// Every canonical `` `linter.<rule>` `` backticked token must resolve
/// against [`crate::lint::RULE_TABLE`] (the catalog's stable ids are
/// `linter.<name>` via [`crate::lint::rule_id`]). Bare `linter.`-prefixed
/// ids with further dots are corpus row ids, not rule mentions.
fn check_rule_id_mentions(text: &str) -> Result<(), String> {
    for token in backticked_spans(text) {
        let Some(rule) = token.strip_prefix("linter.") else {
            continue;
        };
        if rule.contains('.') {
            continue; // file-qualified row id, not a rule mention
        }
        if !crate::lint::RULE_TABLE
            .iter()
            .any(|(name, _)| *name == rule)
        {
            return Err(format!(
                "unresolvable rule mention: `linter.{rule}` is not in the lint rule catalog"
            ));
        }
    }
    Ok(())
}

/// The spans between single backticks (in-code segments), scanning
/// pairwise left to right.
fn backticked_spans(text: &str) -> Vec<&str> {
    let mut spans = Vec::new();
    let mut rest = text;
    while let Some(i) = rest.find('`') {
        let after = &rest[i + 1..];
        match after.find('`') {
            Some(j) => {
                spans.push(&after[..j]);
                rest = &after[j + 1..];
            }
            None => break,
        }
    }
    spans
}

/// EARS pattern mentions: a markdown table row whose first cell matches
/// the pattern-name shape (capitalized, hyphen-separated) and carries a
/// `SHALL` in the row (the guide table's shape column does; other
/// tables' capitalized first cells do not) must be one of the five
/// grammar patterns ([`crate::ears::PATTERNS`]).
fn check_ears_pattern_mentions(text: &str) -> Result<(), String> {
    for name in pattern_name_mentions(text) {
        if !crate::ears::PATTERNS
            .iter()
            .any(|p| p.display_name() == name)
        {
            return Err(format!(
                "unresolvable EARS pattern mention: `{name}` is not one of the five EARS grammar patterns"
            ));
        }
    }
    Ok(())
}

/// The pattern names mentioned in markdown table rows (first cell
/// matches the pattern-name shape, row carries `SHALL`), in row order.
pub fn pattern_name_mentions(text: &str) -> Vec<String> {
    let mut found = Vec::new();
    for line in text.lines() {
        let Some(row) = line.trim().strip_prefix("| ") else {
            continue;
        };
        if !row.contains("SHALL") {
            continue;
        }
        let Some(end) = row.find(" |") else {
            continue;
        };
        let first = row[..end].trim();
        if is_pattern_name_shape(first) {
            found.push(first.to_string());
        }
    }
    found
}

/// Capitalized single words or hyphen-joined capitalized pairs — the
/// EARS display-name shape (`Ubiquitous`, `Event-Driven`).
fn is_pattern_name_shape(s: &str) -> bool {
    !s.is_empty()
        && s.split('-').all(|part| {
            let mut chars = part.chars();
            matches!(chars.next(), Some(c) if c.is_ascii_uppercase())
                && chars.all(|c| c.is_ascii_alphabetic())
        })
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
