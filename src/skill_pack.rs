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
use std::path::{Path, PathBuf};

/// Relative path → file content, sorted by path (BTreeMap ordering).
pub type PackFiles = BTreeMap<String, String>;

/// Prepended to every generated file. Names the regenerate command so a
/// manual edit is detectable and remediable by construction (proposal,
/// design D6).
pub const MANAGED_MARKER: &str =
    "<!-- MANAGED BY specodelic — DO NOT EDIT MANUALLY. Regenerate: spk skill install -->\n\n";

/// The one managed pack this binary ships (design D4: one procedure, two
/// references — no sub-skills).
pub const PACK_NAME: &str = "specodelic-design";

/// The default install target: the Claude/pai skill roots convention
/// (tasks.md §4.1). The pack lands under `<target>/<PACK_NAME>/` so
/// skill loaders that expect `skills/<name>/SKILL.md` find it.
pub fn default_target() -> Result<PathBuf, String> {
    let home = match std::env::var("HOME") {
        Ok(h) if !h.is_empty() => h,
        _ => {
            return Err(
                "cannot determine the home directory (HOME unset) — pass --dir <dir>".to_string(),
            );
        }
    };
    Ok(PathBuf::from(home).join(".claude").join("skills"))
}

/// The recorded-targets state file: one absolute target directory per
/// line, written by [`install`] so the doctor staleness check (§5) can
/// re-check non-default `--dir` targets. This is a target record, NOT a
/// staleness state — D5's "no state file" rule is about hashes, which
/// are recomputed from the generated pack on both sides.
pub fn targets_record_path() -> Result<PathBuf, String> {
    let base = match std::env::var("XDG_STATE_HOME") {
        Ok(x) if !x.is_empty() => PathBuf::from(x),
        _ => {
            let home = std::env::var("HOME")
                .map_err(|_| "cannot determine the home directory (HOME unset)".to_string())?;
            PathBuf::from(home).join(".local").join("state")
        }
    };
    Ok(base.join("specodelic").join("skill-targets"))
}

/// Append a successfully installed target to the record file (deduped).
fn record_target(target: &Path) {
    let Ok(record) = targets_record_path() else {
        return;
    };
    if let Some(parent) = record.parent()
        && let Err(_) = std::fs::create_dir_all(parent)
    {
        return;
    }
    let line = target.display().to_string();
    let existing = std::fs::read_to_string(&record).unwrap_or_default();
    if existing.lines().any(|l| l == line) {
        return;
    }
    let mut out = existing;
    if !out.is_empty() && !out.ends_with('\n') {
        out.push('\n');
    }
    out.push_str(&line);
    out.push('\n');
    // Best-effort: a failed record write must not fail an install that
    // fully succeeded (the envelope already names the target).
    let _ = std::fs::write(&record, out);
}

/// The targets the doctor staleness check evaluates: the default target
/// first, then every recorded `--dir` target (deduped, insertion order).
pub fn doctor_targets() -> Vec<PathBuf> {
    let mut targets = vec![];
    if let Ok(default) = default_target() {
        targets.push(default);
    }
    if let Ok(record) = targets_record_path()
        && let Ok(existing) = std::fs::read_to_string(&record)
    {
        for line in existing.lines() {
            let p = PathBuf::from(line);
            if !targets.contains(&p) {
                targets.push(p);
            }
        }
    }
    targets
}

/// What [`install`] did: the resolved target, the written relative
/// paths, the per-file SHA-256s, and the pack hash (design D5 — the
/// hash is emitted at install so the doctor can recompute both sides).
#[derive(Debug, Clone, serde::Serialize)]
pub struct InstallReport {
    pub target: String,
    pub paths: Vec<String>,
    pub files: Vec<FileHash>,
    pub pack_content_hash: String,
}

/// One written file and its SHA-256.
#[derive(Debug, Clone, serde::Serialize)]
pub struct FileHash {
    pub path: String,
    pub sha256: String,
}

/// Why an install did not happen.
#[derive(Debug)]
pub enum InstallError {
    /// A target path holds a file without the managed marker — refuse,
    /// never silently replace (design D6). Exit 2.
    Refusal(PathBuf),
    /// The filesystem refused the write. Exit 1.
    Io(PathBuf, std::io::Error),
    /// No home directory — an invalid invocation. Exit 2.
    NoHome(String),
}

impl std::fmt::Display for InstallError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InstallError::Refusal(p) => write!(
                f,
                "refusing to overwrite a file without the managed marker: {}",
                p.display()
            ),
            InstallError::Io(p, e) => write!(f, "could not write {}: {e}", p.display()),
            InstallError::NoHome(msg) => write!(f, "{msg}"),
        }
    }
}

impl InstallError {
    /// The remediation hint the envelope carries (output discipline:
    /// every error names its next step).
    pub fn hint(&self) -> String {
        match self {
            InstallError::Refusal(p) => format!(
                "move or delete {} first (it is not specodelic-managed), then re-run: spk skill install",
                p.display()
            ),
            InstallError::Io(p, _) => format!(
                "check permissions on {} (or pick another target with --dir), then re-run: spk skill install",
                p.display()
            ),
            InstallError::NoHome(_) => {
                "pass --dir <dir> to install into an explicit directory".to_string()
            }
        }
    }
}

/// Write the generated pack under `<target>/<PACK_NAME>/`. Refuses
/// (exit-2 class) BEFORE writing anything when any target path holds a
/// non-marker file — the pre-check is atomic across the whole pack, so
/// a refusal leaves the target byte-identical (design D6).
pub fn install(target: &Path) -> Result<InstallReport, InstallError> {
    let files = generate_pack(PACK_NAME).expect("known pack generates");
    let pack_dir = target.join(PACK_NAME);

    // Fail-closed pre-check: every target path must be either absent or
    // marker-bearing. Nothing is written until all paths pass.
    for rel in files.keys() {
        let path = pack_dir.join(rel);
        if path.is_file()
            && !std::fs::read_to_string(&path)
                .map(|c| c.starts_with(MANAGED_MARKER))
                .unwrap_or(false)
        {
            return Err(InstallError::Refusal(path));
        }
    }

    std::fs::create_dir_all(&pack_dir).map_err(|e| InstallError::Io(pack_dir.clone(), e))?;
    let mut hashes = Vec::with_capacity(files.len());
    for (rel, content) in &files {
        let path = pack_dir.join(rel);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| InstallError::Io(path.clone(), e))?;
        }
        std::fs::write(&path, content).map_err(|e| InstallError::Io(path.clone(), e))?;
        hashes.push(FileHash {
            path: rel.clone(),
            sha256: content_sha256(content),
        });
    }
    let hash = pack_content_hash(&files);
    record_target(target);
    Ok(InstallReport {
        target: target.display().to_string(),
        paths: files.keys().cloned().collect(),
        files: hashes,
        pack_content_hash: hash,
    })
}

/// SHA-256 over one file's bytes (hex) — the per-file rows of the
/// install envelope.
fn content_sha256(content: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    hasher
        .finalize()
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

/// Hash the on-disk pack: read ONLY the pack's own relative paths (a
/// mutated non-pack file in the target dir must not flip pass to stale,
/// tasks.md §5.1) and hash them the same way [`pack_content_hash`] does.
/// `Err` when a pack file is missing or unreadable — the caller reports
/// stale (or missing when the whole directory is absent).
pub fn disk_content_hash(pack_dir: &Path) -> Result<String, std::io::Error> {
    let files = generate_pack(PACK_NAME).expect("known pack generates");
    let mut disk = PackFiles::new();
    for rel in files.keys() {
        let content = std::fs::read_to_string(pack_dir.join(rel))?;
        disk.insert(rel.clone(), content);
    }
    Ok(pack_content_hash(&disk))
}

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
    // design D2: the example is `include_str!` straight from the book —
    // `docs/src/examples/reservation-order.md` is the only authoring
    // surface for the worked example (the doc-examples gate replays the
    // quickstart against it), so no copied second source exists here to
    // drift. The marker is prepended at generate time; the book file
    // stays marker-free.
    files.insert(
        "references/example.md".to_string(),
        marked(include_str!("../docs/src/examples/reservation-order.md")),
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
