//! `spk archive-companion <CHANGE_ID>` — archive an openspec change
//! while re-attaching its dual-format layer.
//!
//! Purpose: make the dual-format corpus survive the openspec archive
//! round-trip at the tool level (beads specodelic-fzo, GH#7; reworked
//! by add-delta-layer-sidecar / Revision 19). Responsibilities: refuse
//! any delta whose `layer.md` sidecar is missing or unlintable BEFORE
//! the destructive archive, invoke `openspec archive <id>` with DEFAULT
//! spec application (plain deltas are openspec's native merge — the
//! companion does not re-implement it, `openspec_not_forked` holds),
//! then re-attach the layer: prepend each archived sidecar's
//! frontmatter + Constraints/Model/Properties tables (identity
//! re-keyed to the deployed spec id) onto openspec's applied deployed
//! body, writing `openspec/specs/<cap>/spec.md`. The archived delta
//! stays byte-plain and the sidecar rides along in the archive — the
//! companion never rewrites either.
//!
//! Design decisions (openspec/changes/add-delta-layer-sidecar/design.md,
//! Decision 2): openspec applies, the companion re-attaches — steps 1–4
//! (pre-invocation sidecar refusal; openspec archive with application;
//! per-cap layer merge; fail-closed post-attach shape assertion with
//! revert). Decision 4: deltas may be partial — the dual-era
//! self-containedness and existing-target coverage guards are retired
//! with their subject (openspec's apply defines the merge). Decision 5:
//! sidecar identity recurs across archive history; rerun re-resolves
//! from the NEWEST archive dir and re-attaches to identical deployed
//! bytes. `--dry-run` resolves and plans, never invokes, never writes.
//!
//! Fail-closed guards (the NEW failure shapes only):
//! - sidecar-less / unlintable sidecar refusal fires BEFORE the
//!   openspec invocation — the archive is destructive, so nothing may
//!   move until every affected capability has a restorable layer;
//! - the post-attach shape assertion (no delta headers, frontmatter +
//!   all three tables present) runs on the assembled text before the
//!   write, and the deployed-corpus lint runs after it; on violation
//!   the write is reverted and — when this run invoked openspec — the
//!   change dir is moved back out of `changes/archive/` (best-effort
//!   pre-archive state), and the command fails labeled with a
//!   remediation hint;
//! - a sidecar-less ARCHIVED delta whose deployed target already
//!   carries the dual-format layer is recognized as converged (the
//!   Stage-1-era archives stay idempotent under rerun); anything else
//!   is refused.

use std::fs;
use std::path::{Path, PathBuf};

use thiserror::Error;

use crate::lint;
use crate::parse::parse_file;
use crate::spec::{self, Spec};

/// Errors from the companion flow. Every variant renders as a labeled
/// message with a remediation hint (output discipline).
#[derive(Debug, Error)]
pub enum CompanionError {
    /// No active change and no archive directory match the id.
    #[error(
        "no openspec change or archive matches `{change_id}` under {root}/openspec/changes — list candidates with: openspec list"
    )]
    ChangeNotFound {
        /// The requested change id.
        change_id: String,
        /// The repo root searched.
        root: PathBuf,
    },
    /// The `openspec` binary could not be invoked (missing or failed to
    /// spawn). Never a silent fallback — archive semantics are openspec's
    /// (design D2).
    #[error(
        "could not invoke `openspec archive {change_id}`: {spawn_detail} — ensure the openspec CLI is installed and on PATH"
    )]
    OpenSpecMissing {
        /// The requested change id.
        change_id: String,
        /// Spawn failure detail.
        spawn_detail: String,
    },
    /// `openspec archive` ran and failed. Its stderr is surfaced for
    /// diagnosis; the tree is left as openspec left it.
    #[error(
        "`openspec archive {change_id}` failed: {failure_detail} — inspect the openspec output, fix, and re-run: spk archive-companion {change_id}"
    )]
    OpenSpecFailed {
        /// The requested change id.
        change_id: String,
        /// openspec's own failure output (trimmed).
        failure_detail: String,
    },
    /// One or more deltas cannot have their dual-format layer restored:
    /// no sibling `layer.md` sidecar, or a sidecar that fails parse or
    /// lacks one of the three layer tables. The command fails BEFORE the
    /// destructive openspec archive (nothing moved, nothing written).
    #[error(
        "refusing to archive {count} delta(s) whose dual-format layer is unrestorable: {refused:?} — wrap each plain delta first (the wrap path scaffolds the sibling layer.md): spk migrate <delta>"
    )]
    SidecarRefusal {
        /// (delta or sidecar path, reason) pairs, one per refusal.
        refused: Vec<(PathBuf, String)>,
        /// How many (rendered in the message).
        count: usize,
    },
    /// Reading a delta or sidecar failed — cannot verify what cannot be
    /// read.
    #[error(
        "unreadable file {path}: {read_detail} — check file permissions, then re-run: spk archive-companion"
    )]
    UnreadableFile {
        /// The file path.
        path: PathBuf,
        /// IO error detail.
        read_detail: String,
    },
    /// The post-attach assertion failed: the assembled deployed spec
    /// carries a shape 1.x strict rejects (a delta header, or missing
    /// frontmatter / one of the three tables), or the deployed corpus
    /// lint reports findings. The write was REVERTED — and when this run
    /// invoked openspec, the change dir was moved back out of the
    /// archive — before this error is raised.
    #[error(
        "layer re-attachment for capability `{cap}` violated the post-attach assertion: {detail} — the write was reverted; fix the sidecar (`spk migrate` refreshes the layer tables), then re-run: spk archive-companion"
    )]
    AttachViolation {
        /// The capability whose assembly failed.
        cap: String,
        /// The shape problems or lint findings (rendered).
        detail: String,
    },
}

/// Result of a companion run (real or dry-run).
#[derive(Debug, Clone, PartialEq)]
pub struct RestoreOutcome {
    /// The change id this run targeted.
    pub change_id: String,
    /// Whether the `openspec archive` invocation actually ran (false
    /// for already-archived changes and for every dry-run).
    pub openspec_ran: bool,
    /// The archive directory the layer was attached from (or, for a
    /// dry-run over an active change, the still-active change dir).
    pub archive_dir: PathBuf,
    /// Deployed spec paths written — or, for a dry-run, the paths that
    /// WOULD be written.
    pub restored: Vec<PathBuf>,
    /// Sidecar paths consumed — archived sidecars after a real run; for
    /// a dry-run over an active change, the still-active sidecar paths.
    pub sidecars: Vec<PathBuf>,
    /// Whether this run mutated anything (always false when set).
    pub dry_run: bool,
}

/// The openspec invocation seam (design D2): the real runner shells out
/// to `openspec archive <id> --yes` in the repo root; tests inject a
/// closure so no test ever spawns a process.
pub type ArchiveRunner<'a> = dyn FnMut(&Path, &str) -> Result<(), String> + 'a;

/// Locate the newest archive directory matching `*-<change_id>` under
/// `<root>/openspec/changes/archive/` (design D6: lexicographically
/// greatest name wins, mirroring the recipe's `sort | tail -1`).
pub fn resolve_archive_dir(root: &Path, change_id: &str) -> Result<PathBuf, CompanionError> {
    let archive_root = root.join("openspec/changes/archive");
    let mut matches: Vec<PathBuf> = fs::read_dir(&archive_root)
        .map_err(|_| CompanionError::ChangeNotFound {
            change_id: change_id.to_string(),
            root: root.to_path_buf(),
        })?
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|p| {
            p.is_dir()
                && p.file_name()
                    .and_then(|n| n.to_str())
                    .map(|n| n.ends_with(&format!("-{change_id}")))
                    .unwrap_or(false)
        })
        .collect();
    matches.sort();
    matches.pop().ok_or(CompanionError::ChangeNotFound {
        change_id: change_id.to_string(),
        root: root.to_path_buf(),
    })
}

/// Code-fence tracker shared by the structural scanners: a line inside
/// a fence is content (a `## ` header there is prose, not structure).
/// `feed` returns whether the line is OUTSIDE any fence after the
/// update (call it once per line, in order).
struct FenceTracker {
    marker: Option<String>,
}

impl FenceTracker {
    fn new() -> Self {
        Self { marker: None }
    }

    fn feed(&mut self, line: &str) -> bool {
        let trimmed = line.trim_start();
        if self.marker.is_none() && (trimmed.starts_with("```") || trimmed.starts_with("~~~")) {
            self.marker = Some(trimmed.chars().take(3).collect());
            return false;
        }
        if let Some(marker) = &self.marker {
            if trimmed.starts_with(marker.as_str()) {
                self.marker = None;
            }
            return false;
        }
        true
    }
}

/// True when the line is a `## <title>` header (outside any fence — the
/// caller owns fence state) whose lowercased title matches one of
/// `titles`.
fn is_header(line: &str, titles: &[&str]) -> bool {
    let trimmed = line.trim_end();
    if let Some(rest) = trimmed.strip_prefix("## ") {
        let title = rest.trim().to_ascii_lowercase();
        titles.iter().any(|t| title == *t)
    } else {
        false
    }
}

/// Byte span of a `## <heading>` section: from the heading line's start
/// to the next `## ` header (exclusive) or EOF. Indices are relative to
/// `text`. `None` when the section is absent. Fence state is tracked
/// from the section start — a fenced header inside the section is
/// content, not a boundary.
fn section_span(text: &str, heading: &str) -> Option<(usize, usize)> {
    let start = if text.starts_with(&format!("{heading}\n")) {
        0
    } else {
        text.find(&format!("\n{heading}\n"))? + 1
    };
    let mut fence = FenceTracker::new();
    let mut offset = start;
    for line in text[start..].split_inclusive('\n') {
        if fence.feed(line) && offset > start && line.trim_start().starts_with("## ") {
            return Some((start, offset));
        }
        offset += line.len();
    }
    Some((start, text.len()))
}

/// Remove every `## <name>` section whose heading is in `names`
/// (fence-aware: a header inside a code fence is content). Used to
/// strip a previous assembly's layer tables off a rerun's deployed
/// body, so re-assembly always starts from openspec's applied shape.
fn strip_sections(text: &str, names: &[&str]) -> String {
    let mut out = String::with_capacity(text.len());
    let mut in_removed = false;
    let mut fence = FenceTracker::new();
    for line in text.split_inclusive('\n') {
        let structural = fence.feed(line);
        if structural && is_header(line, names) {
            in_removed = true;
            continue;
        }
        if structural && in_removed && line.trim_start().starts_with("## ") {
            in_removed = false;
        }
        if !in_removed {
            out.push_str(line);
        }
    }
    out
}

/// Remove a leading frontmatter block (`---` … `---`) from `text`,
/// returning the body. `None` when there is no well-formed block
/// (left untouched — malformed frontmatter is not silently consumed).
fn strip_frontmatter_block(text: &str) -> Option<String> {
    let mut offset = 0usize;
    let mut in_frontmatter = false;
    for line in text.split_inclusive('\n') {
        let trimmed = line.trim_end();
        if !in_frontmatter {
            if trimmed.is_empty() {
                offset += line.len();
                continue;
            }
            if trimmed == "---" {
                in_frontmatter = true;
                offset += line.len();
                continue;
            }
            return None;
        }
        if trimmed == "---" {
            return Some(text[offset + line.len()..].to_string());
        }
        offset += line.len();
    }
    None
}

/// The frontmatter block of `text` (from the leading `---` through the
/// closing `---` line, inclusive), or `None` when absent/malformed.
fn frontmatter_block(text: &str) -> Option<String> {
    if !text.starts_with("---\n") && !text.starts_with("---\r\n") {
        return None;
    }
    let mut offset = 0usize;
    for line in text.split_inclusive('\n') {
        let trimmed = line.trim_end();
        if offset > 0 && trimmed == "---" {
            return Some(text[..offset + line.len()].to_string());
        }
        offset += line.len();
    }
    None
}

/// Does `text` carry the dual-format layer (frontmatter + the three
/// table headers)? The rerun-recognition probe for a sidecar-less
/// archived delta whose deployed target was already attached.
fn carries_dual_format(text: &str) -> bool {
    let has_fm = text.starts_with("---\n") || text.starts_with("---\r\n");
    let tables = strip_frontmatter_block(text)
        .map(|b| {
            ["## Constraints", "## Model", "## Properties"]
                .iter()
                .all(|h| section_span(&b, h).is_some())
        })
        .unwrap_or(false);
    has_fm && tables
}

/// Re-key a sidecar's identity for deployment: the frontmatter id line
/// becomes the deployed spec id, and every `[[<sidecar_id>]]` /
/// `[[<sidecar_id>.` reference follows (the `migrate --rekey` semantics
/// applied at attach time — the sidecar's rows must point at the
/// deployed file's own intent).
fn rekey_sidecar(sidecar_text: &str, sidecar_id: &str, spec_id: &str) -> String {
    let mut out = String::with_capacity(sidecar_text.len());
    let mut in_frontmatter = false;
    let mut id_line_done = false;
    for line in sidecar_text.split_inclusive('\n') {
        let trimmed = line.trim_end();
        if trimmed == "---" {
            in_frontmatter = !in_frontmatter && out.is_empty();
            out.push_str(line);
            continue;
        }
        if in_frontmatter && !id_line_done && trimmed.starts_with("id:") {
            out.push_str(&format!("id: {spec_id}\n"));
            id_line_done = true;
            continue;
        }
        let line = line.replace(&format!("[[{sidecar_id}."), &format!("[[{spec_id}."));
        let line = line.replace(&format!("[[{sidecar_id}]]"), &format!("[[{spec_id}]]"));
        out.push_str(&line);
    }
    out
}

/// Assemble the deployed dual-format spec: the sidecar's frontmatter
/// (id re-keyed to `spec_id`) and three layer tables, merged onto
/// openspec's applied deployed body — preserved verbatim (it IS the
/// native merge output) with the tables inserted before the
/// `## Requirements` section, or appended when there is none. Interior
/// bytes of every part are preserved; separators are normalized to
/// blank lines so re-assembly is deterministic (rerun convergence to
/// identical deployed bytes).
fn assemble(
    spec_id: &str,
    sidecar_text: &str,
    sidecar_id: &str,
    deployed_text: Option<&str>,
) -> String {
    let rekeyed = rekey_sidecar(sidecar_text, sidecar_id, spec_id);
    let fm = frontmatter_block(&rekeyed).unwrap_or_default();
    let sidecar_body = strip_frontmatter_block(&rekeyed).unwrap_or_default();

    // Sidecar tables in canonical order.
    let mut tables: Vec<String> = Vec::new();
    for heading in ["## Constraints", "## Model", "## Properties"] {
        if let Some((s, e)) = section_span(&sidecar_body, heading) {
            tables.push(sidecar_body[s..e].trim_end().to_string());
        }
    }

    let mut parts: Vec<String> = vec![fm.trim_end().to_string()];
    if let Some(deployed) = deployed_text {
        let body = strip_frontmatter_block(deployed).unwrap_or_else(|| deployed.to_string());
        let body = strip_sections(&body, &["constraints", "model", "properties"]);
        if let Some((s, _e)) = section_span(&body, "## Requirements") {
            let head = body[..s].trim().to_string();
            if !head.is_empty() {
                parts.push(head);
            }
            parts.extend(tables);
            parts.push(body[s..].trim().to_string());
        } else {
            if !body.trim().is_empty() {
                parts.push(body.trim().to_string());
            }
            parts.extend(tables);
        }
    } else {
        parts.extend(tables);
    }

    let mut out = parts.join("\n\n");
    out.push('\n');
    out
}

/// Pre-attach verification of one sidecar (design Decision 2 step 1):
/// it must PARSE (frontmatter + tables well-formed — the shapes `spk
/// lint` reports as failures) and carry all three layer tables. The
/// full corpus lint runs post-attach, where cross-file references
/// resolve against the deployed corpus.
fn verify_sidecar(path: &Path) -> Result<Spec, CompanionError> {
    let text = fs::read_to_string(path).map_err(|source| CompanionError::UnreadableFile {
        path: path.to_path_buf(),
        read_detail: source.to_string(),
    })?;
    let parsed = match spec::parse_str(&text) {
        Ok(parsed) => parsed,
        Err(error) => {
            return Err(CompanionError::SidecarRefusal {
                refused: vec![(
                    path.to_path_buf(),
                    format!("sidecar fails parse/lint: {error}"),
                )],
                count: 1,
            });
        }
    };
    let body = strip_frontmatter_block(&text).unwrap_or_default();
    let mut missing: Vec<&str> = Vec::new();
    for heading in ["## Constraints", "## Model", "## Properties"] {
        if section_span(&body, heading).is_none() {
            missing.push(&heading[3..]);
        }
    }
    if !missing.is_empty() {
        return Err(CompanionError::SidecarRefusal {
            refused: vec![(
                path.to_path_buf(),
                format!("sidecar lacks the layer table(s): {}", missing.join(", ")),
            )],
            count: 1,
        });
    }
    Ok(parsed)
}

/// One capability's archive pair: the archived plain delta and its
/// sibling sidecar (parsed).
struct SidecarPair {
    /// The archived plain delta (`specs/<cap>/spec.md`).
    delta: PathBuf,
    /// The sidecar (`specs/<cap>/layer.md`).
    sidecar: PathBuf,
    /// The sidecar's parsed spec (its id drives the re-key).
    parsed: Spec,
}

/// Pre-attach guard (design Decision 2 step 1, run BEFORE the openspec
/// invocation when the change is still active — the archive is
/// destructive, so nothing may move until every affected capability has
/// a restorable layer). Every delta in `source_dir` must have a sibling
/// `layer.md` sidecar that parses and carries the three layer tables.
///
/// Recognition arm (already-archived reruns only): a delta without a
/// sidecar whose deployed target already carries the dual-format layer
/// is converged — skipped, not refused (Stage-1-era archives stay
/// idempotent). Anything else is refused as a whole (every refusal is
/// listed, nothing is written).
fn check_sidecars(
    root: &Path,
    source_dir: &Path,
    pending_archive: bool,
) -> Result<Vec<SidecarPair>, CompanionError> {
    let deltas = list_deltas(source_dir);
    if deltas.is_empty() {
        return Ok(Vec::new()); // zero deltas is a success (design D8)
    }
    let mut refused: Vec<(PathBuf, String)> = Vec::new();
    let mut pairs: Vec<SidecarPair> = Vec::new();
    for delta in &deltas {
        let cap = delta
            .parent()
            .and_then(|p| p.file_name())
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();
        let sidecar = delta.with_file_name("layer.md");
        if !sidecar.is_file() {
            if pending_archive {
                refused.push((
                    delta.clone(),
                    "no layer.md sidecar beside the delta — the dual-format layer is unrestorable"
                        .to_string(),
                ));
                continue;
            }
            // Already archived: is the deployed target already attached?
            let target = root.join("openspec/specs").join(&cap).join("spec.md");
            let converged = fs::read_to_string(&target)
                .map(|t| carries_dual_format(&t))
                .unwrap_or(false);
            if converged {
                continue;
            }
            refused.push((
                delta.clone(),
                "no layer.md sidecar in the archive and the deployed target carries no dual-format layer".to_string(),
            ));
            continue;
        }
        match verify_sidecar(&sidecar) {
            Ok(parsed) => pairs.push(SidecarPair {
                delta: delta.clone(),
                sidecar,
                parsed,
            }),
            Err(CompanionError::SidecarRefusal {
                refused: mut more, ..
            }) => refused.append(&mut more),
            Err(other) => return Err(other),
        }
    }
    if !refused.is_empty() {
        let count = refused.len();
        return Err(CompanionError::SidecarRefusal { refused, count });
    }
    Ok(pairs)
}

/// Attach the layer: for every pair, merge the archived sidecar's
/// frontmatter + tables with the applied deployed spec into
/// `openspec/specs/<cap>/spec.md` (design Decision 2 step 3), assert
/// the assembled shape pre-write, write, then lint the deployed corpus
/// — reverting every write on any violation (step 4, fail-closed).
pub fn restore(root: &Path, archive_dir: &Path) -> Result<Vec<PathBuf>, CompanionError> {
    let pairs = check_sidecars(root, archive_dir, false)?;
    if pairs.is_empty() {
        return Ok(Vec::new());
    }

    // Assemble + assert shape pre-write; save prior bytes for revert.
    let mut saved: Vec<(PathBuf, Option<String>)> = Vec::new();
    let mut assembled: Vec<(PathBuf, String, String)> = Vec::new(); // (target, text, cap)
    for pair in &pairs {
        let cap = pair
            .sidecar
            .parent()
            .and_then(|p| p.file_name())
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();
        let spec_id = cap.replace('-', ".");
        let target = root.join("openspec/specs").join(&cap).join("spec.md");
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent).map_err(|source| CompanionError::UnreadableFile {
                path: target.clone(),
                read_detail: source.to_string(),
            })?;
        }
        let sidecar_text =
            fs::read_to_string(&pair.sidecar).map_err(|source| CompanionError::UnreadableFile {
                path: pair.sidecar.clone(),
                read_detail: source.to_string(),
            })?;
        let deployed_text = fs::read_to_string(&target).ok();
        let text = assemble(
            &spec_id,
            &sidecar_text,
            &pair.parsed.intent.id,
            deployed_text.as_deref(),
        );
        if let Err(detail) = assert_attached_shape(&cap, &text) {
            return Err(CompanionError::AttachViolation { cap, detail });
        }
        let prior = fs::read_to_string(&target).ok();
        saved.push((target.clone(), prior));
        assembled.push((target, text, cap));
    }

    // Write, then lint the deployed corpus (fail-closed, design step 4).
    let mut written: Vec<PathBuf> = Vec::new();
    for (target, text, _) in &assembled {
        fs::write(target, text).map_err(|source| CompanionError::UnreadableFile {
            path: target.clone(),
            read_detail: source.to_string(),
        })?;
        written.push(target.clone());
    }
    if let Err(detail) = lint_deployed_corpus(root) {
        revert(&saved);
        return Err(CompanionError::AttachViolation {
            cap: assembled[0].2.clone(),
            detail,
        });
    }
    Ok(written)
}

/// Revert a set of writes: restore prior bytes, or remove files that
/// did not exist before (best-effort pre-run state).
fn revert(saved: &[(PathBuf, Option<String>)]) {
    for (path, prior) in saved {
        match prior {
            Some(text) => {
                let _ = fs::write(path, text);
            }
            None => {
                let _ = fs::remove_file(path);
            }
        }
    }
}

/// Post-attach shape assertion (design Decision 2 step 4, pre-write):
/// the assembled deployed spec must carry NO delta headers
/// (`## ADDED/MODIFIED/REMOVED/RENAMED Requirements`), and must carry
/// frontmatter + all three layer tables — a shape openspec 1.x strict
/// rejects is forbidden before anything lands.
fn assert_attached_shape(cap: &str, text: &str) -> Result<(), String> {
    let mut problems: Vec<String> = Vec::new();
    let delta_titles = [
        "added requirements",
        "modified requirements",
        "removed requirements",
        "renamed requirements",
    ];
    let mut fence = FenceTracker::new();
    for line in text.split_inclusive('\n') {
        if !fence.feed(line) {
            continue;
        }
        if is_header(line, &delta_titles) {
            problems.push(format!(
                "delta header survived the assembly: {}",
                line.trim_end()
            ));
        }
    }
    if !text.starts_with("---\n") && !text.starts_with("---\r\n") {
        problems.push("no frontmatter block".to_string());
    }
    for heading in ["## Constraints", "## Model", "## Properties"] {
        if section_span(text, heading).is_none() {
            problems.push(format!("missing {heading} table"));
        }
    }
    if problems.is_empty() {
        Ok(())
    } else {
        Err(format!("capability {cap}: {}", problems.join("; ")))
    }
}

/// Lint the deployed corpus (`root/openspec/specs`) with the real
/// linter. `Ok` when zero findings; `Err` carries the rendered findings.
/// Cross-file references resolve corpus-wide — this is why the layer
/// tables are only linted here, never in the pre-invocation guard.
fn lint_deployed_corpus(root: &Path) -> Result<(), String> {
    let specs_dir = root.join("openspec/specs");
    let mut findings: Vec<String> = Vec::new();
    let mut specs: Vec<Spec> = Vec::new();
    for file in collect_md_files(&specs_dir) {
        let Ok(text) = fs::read_to_string(&file) else {
            findings.push(format!("{}: unreadable", file.display()));
            continue;
        };
        if !text.starts_with("---\n") && !text.starts_with("---\r\n") {
            continue; // not a specodelic file (plain files parse-skip)
        }
        match parse_file(&file) {
            Ok(parsed) => specs.push(parsed),
            Err(error) => findings.push(format!("{}: parse error: {error}", file.display())),
        }
    }
    if !specs.is_empty() {
        let report = lint::lint_corpus(&specs);
        for issue in &report.issues {
            findings.push(format!(
                "{} [{}] {}",
                issue.file, issue.rule_id, issue.message
            ));
        }
    }
    if findings.is_empty() {
        Ok(())
    } else {
        Err(findings.join("; "))
    }
}

/// Every `*.md` file under `dir` (recursive, sorted; hidden and build
/// dirs skipped — the same walk discipline as the CLI's collection).
fn collect_md_files(dir: &Path) -> Vec<PathBuf> {
    fn skipped(p: &Path) -> bool {
        p.file_name()
            .and_then(|n| n.to_str())
            .map(|n| n.starts_with('.') || n == "target" || n == "node_modules")
            .unwrap_or(true)
    }
    fn walk(dir: &Path, files: &mut Vec<PathBuf>) {
        let mut entries: Vec<PathBuf> = fs::read_dir(dir)
            .into_iter()
            .flatten()
            .flatten()
            .map(|e| e.path())
            .collect();
        entries.sort();
        for p in entries {
            if p.is_dir() {
                if !skipped(&p) {
                    walk(&p, files);
                }
            } else if p.extension().is_some_and(|e| e == "md") {
                files.push(p);
            }
        }
    }
    let mut files = Vec::new();
    if dir.is_dir() {
        walk(dir, &mut files);
    }
    files
}

/// Run the full companion flow. `dry_run` resolves and verifies without
/// invoking openspec or writing anything (design D7).
pub fn run(
    root: &Path,
    change_id: &str,
    dry_run: bool,
    runner: &mut ArchiveRunner<'_>,
) -> Result<RestoreOutcome, CompanionError> {
    let active = root.join("openspec/changes").join(change_id);
    let archived_dir = resolve_archive_dir(root, change_id).ok();

    if !active.is_dir() && archived_dir.is_none() {
        return Err(CompanionError::ChangeNotFound {
            change_id: change_id.to_string(),
            root: root.to_path_buf(),
        });
    }

    let openspec_ran;
    // Delta source for the pre-invocation guard: the archive dir once it
    // exists, otherwise (pending archive) the still-active change dir.
    let (source_dir, pending_archive) = if archived_dir.is_some() {
        (archived_dir.clone().unwrap(), false)
    } else {
        (active.clone(), true)
    };

    // Pre-invocation guard (design Decision 2 step 1): every affected
    // delta must have a restorable sidecar — BEFORE the destructive
    // archive. Dry-runs run the same guard (D7: same resolution).
    let pairs = check_sidecars(root, &source_dir, pending_archive)?;

    if dry_run {
        openspec_ran = false;
        let restored = pairs
            .iter()
            .map(|p| deployed_target(root, &p.delta))
            .collect();
        let sidecars = pairs.iter().map(|p| p.sidecar.clone()).collect();
        return Ok(RestoreOutcome {
            change_id: change_id.to_string(),
            openspec_ran,
            archive_dir: source_dir,
            restored,
            sidecars,
            dry_run: true,
        });
    }

    let archive_dir;
    if pending_archive {
        runner(root, change_id).map_err(|detail| {
            if detail.starts_with("spawn") || detail.contains("No such file") {
                CompanionError::OpenSpecMissing {
                    change_id: change_id.to_string(),
                    spawn_detail: detail,
                }
            } else {
                CompanionError::OpenSpecFailed {
                    change_id: change_id.to_string(),
                    failure_detail: detail,
                }
            }
        })?;
        openspec_ran = true;
        // Re-resolve: openspec just created `<date>-<change_id>`.
        archive_dir = resolve_archive_dir(root, change_id)?;
    } else {
        openspec_ran = false;
        archive_dir = archived_dir.unwrap();
    }

    match restore(root, &archive_dir) {
        Ok(restored) => {
            // After a real run the sidecars live in the archive (the
            // change dir moved); report those paths.
            let sidecars = pairs
                .iter()
                .map(|p| {
                    let cap = p
                        .sidecar
                        .parent()
                        .and_then(|q| q.file_name())
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_default();
                    archive_dir.join("specs").join(cap).join("layer.md")
                })
                .collect();
            Ok(RestoreOutcome {
                change_id: change_id.to_string(),
                openspec_ran,
                archive_dir,
                restored,
                sidecars,
                dry_run: false,
            })
        }
        Err(err) => {
            // Best-effort restore of the pre-archive state: when this
            // run invoked openspec, move the change dir back out of the
            // archive (the attach itself already reverted its writes).
            if openspec_ran {
                let _ = fs::rename(&archive_dir, &active);
            }
            Err(err)
        }
    }
}

/// The real openspec invocation (the seam's production
/// implementation): `openspec archive <id> --yes` in the repo root —
/// DEFAULT spec application (Revision 19: plain deltas are openspec's
/// native merge; the companion re-attaches the layer afterwards),
/// `--yes` because the companion already owns the decision to archive;
/// stdout/stderr are captured for diagnosis.
pub fn openspec_archive_runner(root: &Path, change_id: &str) -> Result<(), String> {
    let output = std::process::Command::new("openspec")
        .args(["archive", change_id, "--yes"])
        .current_dir(root)
        .output()
        .map_err(|e| format!("spawn failed: {e}"))?;
    if output.status.success() {
        Ok(())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).trim().to_string())
    }
}

/// List the `specs/*/spec.md` deltas under a directory (sorted).
fn list_deltas(dir: &Path) -> Vec<PathBuf> {
    let specs_dir = dir.join("specs");
    let mut deltas: Vec<PathBuf> = match fs::read_dir(&specs_dir) {
        Ok(rd) => rd
            .filter_map(|entry| entry.ok())
            .map(|entry| entry.path().join("spec.md"))
            .filter(|p| p.is_file())
            .collect(),
        Err(_) => Vec::new(),
    };
    deltas.sort();
    deltas
}

/// The deployed spec path a delta would be attached to.
fn deployed_target(root: &Path, delta: &Path) -> PathBuf {
    delta
        .parent()
        .and_then(|p| p.file_name())
        .map(|cap| {
            // Forward slashes in the reported path: the envelope crosses
            // the fleet boundary (espectacular consumes the paths), and
            // a windows-only `\specs\c\spec.md` string is unusable there
            // (windows CI: tests/cli.rs archive_companion_dry_run).
            root.join("openspec/specs")
                .join(cap)
                .join("spec.md")
                .to_string_lossy()
                .replace('\\', "/")
                .into()
        })
        .unwrap_or_else(|| root.join("openspec/specs/spec.md"))
}

#[cfg(test)]
mod tests;
