//! `spk archive-companion <CHANGE_ID>` — archive an openspec change
//! while preserving its dual-format layer.
//!
//! Purpose: make the dual-format corpus survive the openspec archive
//! round-trip at the tool level (beads specodelic-fzo, GH#7 from
//! espectacular). Responsibilities: invoke `openspec archive <id>
//! --skip-specs --yes` (skipped when the change is already archived —
//! idempotent re-run), locate the newest archive directory matching the
//! change id, verify each archived delta still carries the specodelic
//! layer (frontmatter + `## Constraints`), and deploy it to
//! `openspec/specs/<cap>/spec.md`. Fail-closed: a delta lacking the
//! layer is refused before any write — the tool must never be the thing
//! that deploys a stripped spec (the exact failure this module exists
//! to prevent).
//!
//! Stage-1 deployed-copy shape (specodelic-s8fg, decision record
//! openspec/decisions/2026-10-19-delta-section-strip.md): the DEPLOYED
//! copy is the delta minus its delta sections (the `## ADDED/MODIFIED/
//! REMOVED/RENAMED Requirements` headers) — the `## Requirements`
//! mirror stays as the deployed Requirements section, so the deployed
//! spec is a clean openspec 1.x main spec. The ARCHIVED delta keeps the
//! full dual-format layer (D4 lives on for the archived copy); the
//! deployed copy is the layer-preserving re-assembly of the same
//! content, no longer byte-identical to it.
//!
//! Design decisions (openspec/changes/add-archive-companion/design.md):
//! companion flow over post-hoc restore (D1); openspec stays external,
//! injected behind a runner seam so tests never shell out (D2);
//! fail-closed dual-format guard (D3); verbatim copy, no normalization
//! (D4); idempotent re-run (D5); newest archive dir wins, named in the
//! envelope (D6); dry-run resolves, never mutates (D7); zero deltas is
//! a success, not an error (D8).
//!
//! Single-tree redefinition (specodelic-bhp, D9): after a successful
//! deploy the companion strips the frontmatter block from each ARCHIVED
//! delta copy, returning it to the openspec-archival shape — the
//! frontmatter-less form the corpus linter expects (a frontmattered
//! archived `spec.md` collides with the deployed spec's id; the
//! manual-strip step that every archive since the quant pack had to
//! redo by hand). The deployed spec keeps its dual-format layer
//! verbatim (D4 unchanged). Re-run recognition replaces blind re-copy:
//! a layer-less archived delta whose deployed target already carries
//! the dual-format layer is recognized as already deployed (skip,
//! success); anything else stays fail-closed (D3).

use std::fs;
use std::path::{Path, PathBuf};

use thiserror::Error;

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
        "could not invoke `openspec archive {change_id} --skip-specs --yes`: {spawn_detail} — ensure the openspec CLI is installed and on PATH"
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
    /// One or more archived deltas lack the dual-format layer. Nothing
    /// was deployed (fail closed, design D3).
    #[error(
        "refusing to deploy {count} delta(s) lacking the dual-format layer (frontmatter or Constraints section missing): {deltas:?} — migrate them first per the recipe: spk explain dual-format"
    )]
    UnverifiableDeltas {
        /// Deltas that failed verification (absolute within repo root).
        deltas: Vec<PathBuf>,
        /// How many (rendered in the message).
        count: usize,
    },
    /// Reading a delta failed — cannot verify what cannot be read.
    #[error(
        "unreadable delta {path}: {read_detail} — check file permissions, then re-run: spk archive-companion"
    )]
    UnreadableDelta {
        /// The delta path.
        path: PathBuf,
        /// IO error detail.
        read_detail: String,
    },
    /// Post-deploy normalization failed: the frontmatter block could
    /// not be stripped from the archived delta copy (design D9). The
    /// deployed spec is already intact; fix the delta file, then re-run.
    #[error(
        "could not strip frontmatter from archived delta {path}: {detail} — the deployed spec is already written; fix the delta file's permissions, then re-run: spk archive-companion"
    )]
    StripFailed {
        /// The delta path.
        path: PathBuf,
        /// IO error detail.
        detail: String,
    },
    /// The delta's ADDED/MODIFIED requirements are not all restated in
    /// its own `## Requirements` mirror — the corpus's self-contained
    /// convention (constraint `delta_self_contained`) is violated.
    /// Deploying would narrow the spec; nothing was written.
    #[error(
        "delta {path} is not self-contained: ADDED/MODIFIED requirement(s) {missing:?} missing from its own ## Requirements mirror — restate every requirement in the mirror (the corpus convention: every delta restates the capability's full requirement set), then re-run: spk archive-companion"
    )]
    DeltaNotSelfContained {
        /// The delta path.
        path: PathBuf,
        /// Requirement names the mirror does not carry.
        missing: Vec<String>,
    },
    /// The existing deployed spec carries requirement(s) the delta's
    /// full requirement set (mirror ∪ REMOVED ∪ RENAMED) does not
    /// restate — a deploy would silently narrow the spec. Nothing was
    /// written (fail-closed coverage guard, specodelic-s8fg Stage 1).
    #[error(
        "refusing to deploy {path}: the existing deployed spec carries requirement(s) {missing:?} that the delta does not restate (the corpus convention: every delta restates the capability's full requirement set — restate them in the delta's mirror or REMOVED section), then re-run: spk archive-companion"
    )]
    UncoveredDeployedRequirements {
        /// The delta path.
        path: PathBuf,
        /// Deployed requirement names the delta does not carry.
        missing: Vec<String>,
    },
    /// The stripped deployed copy would still carry a delta header or a
    /// requirement header outside `## Requirements` — the two shapes
    /// openspec 1.x rejects in main specs. Nothing was written.
    #[error(
        "stripped deployed copy of {path} would still be structurally invalid: {detail} — fix the delta (every requirement must live in its ## Requirements mirror or a delta section), then re-run: spk archive-companion"
    )]
    DeployedShapeViolation {
        /// The delta path.
        path: PathBuf,
        /// What survived the strip and where.
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
    /// The archive directory the deltas were restored from (or, for a
    /// dry-run over an active change, the still-active change dir).
    pub archive_dir: PathBuf,
    /// Deployed spec paths written — or, for a dry-run, the paths that
    /// WOULD be written.
    pub restored: Vec<PathBuf>,
    /// Archived delta copies whose frontmatter block was stripped after
    /// deployment (design D9; empty for dry-runs and no-op re-runs).
    pub stripped: Vec<PathBuf>,
    /// Whether this run mutated anything (always false when set).
    pub dry_run: bool,
}

/// The openspec invocation seam (design D2): the real runner shells out
/// to `openspec archive <id> --skip-specs --yes` in the repo root;
/// tests inject a closure so no test ever spawns a process.
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

/// Verify a delta still carries the dual-format layer (design D3):
/// the first non-empty line opens frontmatter (`---`) AND the body has
/// a `## Constraints` section header.
pub fn verify_dual_format(delta: &Path) -> Result<(), CompanionError> {
    let text = fs::read_to_string(delta).map_err(|source| CompanionError::UnreadableDelta {
        path: delta.to_path_buf(),
        read_detail: source.to_string(),
    })?;
    let opens_frontmatter = text
        .lines()
        .map(str::trim_start)
        .find(|l| !l.is_empty())
        .map(|l| l == "---")
        .unwrap_or(false);
    let has_constraints = text
        .lines()
        .any(|l| l.trim_start().starts_with("## Constraints"));
    if opens_frontmatter && has_constraints {
        Ok(())
    } else {
        Err(CompanionError::UnverifiableDeltas {
            deltas: vec![delta.to_path_buf()],
            count: 1,
        })
    }
}

/// Deploy every verified delta to its deployed spec path. Empty delta
/// sets are a success with an empty list (design D8) — nothing to
/// restore is a stated outcome.
///
/// Re-run recognition (D9): a delta lacking the dual-format layer is
/// skipped (not copied, not reported) when its deployed target already
/// exists and carries the layer — the post-normalization state a prior
/// run left behind. Any other unverifiable delta is still refused
/// before any copy (D3).
///
/// Stage-1 deployed-copy shape (specodelic-s8fg): the deployed copy is
/// the delta minus its delta sections (`## ADDED/MODIFIED/REMOVED/
/// RENAMED Requirements`) — the mirror stays as the deployed
/// Requirements section, so the deployed spec is a clean openspec 1.x
/// main spec (no delta headers, no requirement headers outside
/// `## Requirements`). Guards run fail-closed before any write:
/// self-containedness, existing-target coverage, and a post-strip
/// shape assertion.
pub fn restore(root: &Path, archive_dir: &Path) -> Result<Vec<PathBuf>, CompanionError> {
    let specs_dir = archive_dir.join("specs");
    let mut deltas: Vec<PathBuf> = match fs::read_dir(&specs_dir) {
        Ok(rd) => rd
            .filter_map(|entry| entry.ok())
            .map(|entry| entry.path().join("spec.md"))
            .filter(|p| p.is_file())
            .collect(),
        Err(_) => Vec::new(),
    };
    deltas.sort();

    // Fail closed BEFORE any write (design D3, now covering the shape
    // guards too): verify everything first, then deploy. A refusal never
    // leaves a half-restored tree. A layer-less delta is recognized as
    // already deployed (D9) only when its deployed target carries the
    // dual-format layer.
    let mut to_deploy: Vec<(PathBuf, String)> = Vec::new();
    for delta in &deltas {
        if verify_dual_format(delta).is_ok() {
            let text =
                fs::read_to_string(delta).map_err(|source| CompanionError::UnreadableDelta {
                    path: delta.clone(),
                    read_detail: source.to_string(),
                })?;
            to_deploy.push((delta.clone(), text));
            continue;
        }
        let target = deployed_target(root, delta);
        let recognized = target.is_file() && verify_dual_format(&target).is_ok();
        if !recognized {
            return Err(CompanionError::UnverifiableDeltas {
                deltas: deltas.clone(),
                count: deltas.len(),
            });
        }
    }

    // Coverage guards before any write: a refusal never leaves a
    // half-restored tree.
    for (delta, text) in &to_deploy {
        // Self-containedness (corpus constraint delta_self_contained):
        // every ADDED/MODIFIED requirement must be restated in the
        // delta's own mirror.
        let added = requirement_names_in_delta_sections(text);
        let mirror = requirement_names_under_requirements(text);
        let missing_self: Vec<String> = added
            .iter()
            .filter(|n| !mirror.contains(n))
            .cloned()
            .collect();
        if !missing_self.is_empty() {
            return Err(CompanionError::DeltaNotSelfContained {
                path: delta.clone(),
                missing: missing_self,
            });
        }
        // Existing-target coverage: a deploy replaces the target, so
        // every requirement it already carries must be restated by the
        // delta's full requirement set (mirror ∪ REMOVED ∪ RENAMED).
        // `### Requirement:` headers anywhere in the delta cover all
        // four sections plus the mirror in one scan.
        let target = deployed_target(root, delta);
        if target.is_file() {
            let old =
                fs::read_to_string(&target).map_err(|source| CompanionError::UnreadableDelta {
                    path: target.clone(),
                    read_detail: source.to_string(),
                })?;
            let covered = requirement_names(text);
            let missing: Vec<String> = requirement_names(&old)
                .into_iter()
                .filter(|n| !covered.contains(n))
                .collect();
            if !missing.is_empty() {
                return Err(CompanionError::UncoveredDeployedRequirements {
                    path: delta.clone(),
                    missing,
                });
            }
        }
    }

    let mut restored = Vec::new();
    for (delta, text) in &to_deploy {
        let cap = delta.parent().and_then(|p| p.file_name()).ok_or_else(|| {
            CompanionError::UnreadableDelta {
                path: delta.clone(),
                read_detail: "delta has no capability directory".to_string(),
            }
        })?;
        let target = root.join("openspec/specs").join(cap).join("spec.md");
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent).map_err(|source| CompanionError::UnreadableDelta {
                path: target.clone(),
                read_detail: source.to_string(),
            })?;
        }
        let deployed_text = strip_delta_sections(text);
        assert_deployed_shape(delta, &deployed_text)?;
        fs::write(&target, &deployed_text).map_err(|source| CompanionError::UnreadableDelta {
            path: target.clone(),
            read_detail: source.to_string(),
        })?;
        restored.push(target);
    }
    Ok(restored)
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

/// Strip the delta sections (`## ADDED/MODIFIED/REMOVED/RENAMED
/// Requirements`) from `text`, leaving everything else byte-identical
/// (specodelic-s8fg Stage 1). Fence-aware: a `## ` header inside a code
/// fence is content, not structure. Case-insensitive on the title,
/// permissive on trailing whitespace.
pub fn strip_delta_sections(text: &str) -> String {
    let is_delta_header = |line: &str| {
        let trimmed = line.trim_end();
        if let Some(rest) = trimmed.strip_prefix("## ") {
            let title = rest.trim().to_ascii_lowercase();
            matches!(
                title.as_str(),
                "added requirements"
                    | "modified requirements"
                    | "removed requirements"
                    | "renamed requirements"
            )
        } else {
            false
        }
    };
    let mut out = String::with_capacity(text.len());
    let mut in_delta_section = false;
    let mut fence = FenceTracker::new();
    for line in text.split_inclusive('\n') {
        let trimmed = line.trim_start();
        let structural = fence.feed(line);
        if structural && is_delta_header(line) {
            in_delta_section = true;
            continue;
        }
        if structural && in_delta_section && trimmed.starts_with("## ") {
            in_delta_section = false;
        }
        if !in_delta_section {
            out.push_str(line);
        }
    }
    out
}

/// The `### Requirement:` header names in `text` (fence-aware, order
/// preserved, duplicates kept — callers dedupe by membership).
fn requirement_names(text: &str) -> Vec<String> {
    let mut names = Vec::new();
    let mut fence = FenceTracker::new();
    for line in text.split_inclusive('\n') {
        if !fence.feed(line) {
            continue;
        }
        if let Some(rest) = line.trim_start().strip_prefix("### Requirement:") {
            names.push(rest.trim().to_string());
        }
    }
    names
}

/// Requirement names inside the delta sections only (ADDED/MODIFIED —
/// the sections a mirror must restate for self-containedness).
fn requirement_names_in_delta_sections(text: &str) -> Vec<String> {
    let mut names = Vec::new();
    let mut in_delta = false;
    let mut fence = FenceTracker::new();
    for line in text.split_inclusive('\n') {
        if !fence.feed(line) {
            continue;
        }
        let trimmed = line.trim_start();
        if trimmed.starts_with("## ") {
            let title = trimmed[2..].trim().to_ascii_lowercase();
            in_delta = matches!(
                title.as_str(),
                "added requirements" | "modified requirements"
            );
        }
        if let (true, Some(rest)) = (in_delta, trimmed.strip_prefix("### Requirement:")) {
            names.push(rest.trim().to_string());
        }
    }
    names
}

/// Requirement names under the `## Requirements` section (the mirror).
fn requirement_names_under_requirements(text: &str) -> Vec<String> {
    let mut names = Vec::new();
    let mut in_requirements = false;
    let mut fence = FenceTracker::new();
    for line in text.split_inclusive('\n') {
        if !fence.feed(line) {
            continue;
        }
        let trimmed = line.trim_start();
        if trimmed.starts_with("## ") {
            in_requirements = trimmed[2..].trim().eq_ignore_ascii_case("requirements");
        }
        if let (true, Some(rest)) = (in_requirements, trimmed.strip_prefix("### Requirement:")) {
            names.push(rest.trim().to_string());
        }
    }
    names
}

/// Post-strip shape assertion (specodelic-s8fg Stage 1, EDGE-001): the
/// deployed copy must carry no delta headers and no requirement headers
/// outside `## Requirements` — the two shapes openspec 1.x rejects in
/// main specs (spec-structure.js's only file-level ERROR classes).
/// Fail-closed before the write.
fn assert_deployed_shape(delta: &Path, text: &str) -> Result<(), CompanionError> {
    let mut problems: Vec<String> = Vec::new();
    let mut in_requirements = false;
    let mut fence = FenceTracker::new();
    for (i, line) in text.split_inclusive('\n').enumerate() {
        if !fence.feed(line) {
            continue;
        }
        let trimmed = line.trim_start();
        if trimmed.starts_with("## ") {
            let title = trimmed[2..].trim().to_ascii_lowercase();
            if matches!(
                title.as_str(),
                "added requirements"
                    | "modified requirements"
                    | "removed requirements"
                    | "renamed requirements"
            ) {
                problems.push(format!(
                    "line {}: delta header `## {}` survived the strip",
                    i + 1,
                    title
                ));
                continue;
            }
            in_requirements = title == "requirements";
        }
        if trimmed.starts_with("### Requirement:") && !in_requirements {
            let name = trimmed["### Requirement:".len()..].trim();
            problems.push(format!(
                "line {}: requirement `{name}` outside ## Requirements",
                i + 1
            ));
        }
    }
    if problems.is_empty() {
        Ok(())
    } else {
        Err(CompanionError::DeployedShapeViolation {
            path: delta.to_path_buf(),
            detail: problems.join("; "),
        })
    }
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
    // Delta source for verification: the archive dir once it exists,
    // otherwise (dry-run over an active change) the change's own deltas.
    let (source_dir, pending_archive) = if archived_dir.is_some() {
        (archived_dir.clone().unwrap(), false)
    } else {
        (active.clone(), true)
    };

    if dry_run {
        openspec_ran = false;
        // D7: resolve + verify, never invoke, never write (and never
        // strip — the archived copy stays untouched).
        let deltas = list_deltas(&source_dir);
        for delta in &deltas {
            verify_dual_format(delta)?;
        }
        let restored = deltas.iter().map(|d| deployed_target(root, d)).collect();
        return Ok(RestoreOutcome {
            change_id: change_id.to_string(),
            openspec_ran,
            archive_dir: source_dir,
            restored,
            stripped: Vec::new(),
            dry_run: true,
        });
    }

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
        let archive_dir = resolve_archive_dir(root, change_id)?;
        let restored = restore(root, &archive_dir)?;
        let stripped = strip_archived_deltas(&archive_dir)?;
        return Ok(RestoreOutcome {
            change_id: change_id.to_string(),
            openspec_ran,
            archive_dir,
            restored,
            stripped,
            dry_run: false,
        });
    }

    // Already archived (design D5): skip the invocation, re-run
    // restore — which recognizes the post-strip state as a no-op
    // success (D9) instead of re-copying a stripped delta.
    let archive_dir = archived_dir.unwrap();
    let restored = restore(root, &archive_dir)?;
    let stripped = strip_archived_deltas(&archive_dir)?;
    Ok(RestoreOutcome {
        change_id: change_id.to_string(),
        openspec_ran: false,
        archive_dir,
        restored,
        stripped,
        dry_run: false,
    })
}

/// The real openspec invocation (the seam's production
/// implementation): `openspec archive <id> --skip-specs --yes` in the
/// repo root, `--yes` because the companion already owns the decision
/// to archive; stdout/stderr are captured for diagnosis.
pub fn openspec_archive_runner(root: &Path, change_id: &str) -> Result<(), String> {
    let output = std::process::Command::new("openspec")
        .args(["archive", change_id, "--skip-specs", "--yes"])
        .current_dir(root)
        .output()
        .map_err(|e| format!("spawn failed: {e}"))?;
    if output.status.success() {
        Ok(())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).trim().to_string())
    }
}

/// Strip the frontmatter block from every archived delta copy under
/// `archive_dir` (design D9). Idempotent: an already-frontmatter-less
/// delta is left untouched and unreported. Returns the paths actually
/// stripped (sorted, mirroring `list_deltas`).
pub fn strip_archived_deltas(archive_dir: &Path) -> Result<Vec<PathBuf>, CompanionError> {
    let mut stripped = Vec::new();
    for delta in list_deltas(archive_dir) {
        let text = fs::read_to_string(&delta).map_err(|source| CompanionError::StripFailed {
            path: delta.clone(),
            detail: source.to_string(),
        })?;
        if let Some(rest) = strip_frontmatter_block(&text) {
            fs::write(&delta, &rest).map_err(|source| CompanionError::StripFailed {
                path: delta.clone(),
                detail: source.to_string(),
            })?;
            stripped.push(delta);
        }
    }
    Ok(stripped)
}

/// Remove a leading frontmatter block (`---` … `---`) from `text`,
/// returning the body. `None` when there is no well-formed block:
/// a first non-empty line that is not `---`, or an unclosed fence
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

/// The deployed spec path a delta would be copied to.
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
