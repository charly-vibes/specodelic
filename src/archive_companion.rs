//! `spk archive-companion <CHANGE_ID>` — archive an openspec change
//! while preserving its dual-format layer.
//!
//! Purpose: make the dual-format corpus survive the openspec archive
//! round-trip at the tool level (beads specodelic-fzo, GH#7 from
//! espectacular). Responsibilities: invoke `openspec archive <id>
//! --skip-specs --yes` (skipped when the change is already archived —
//! idempotent re-run), locate the newest archive directory matching the
//! change id, verify each archived delta still carries the specodelic
//! layer (frontmatter + `## Constraints`), and deploy it verbatim to
//! `openspec/specs/<cap>/spec.md`. Fail-closed: a delta lacking the
//! layer is refused before any copy — the tool must never be the thing
//! that deploys a stripped spec (the exact failure this module exists
//! to prevent).
//!
//! Design decisions (openspec/changes/add-archive-companion/design.md):
//! companion flow over post-hoc restore (D1); openspec stays external,
//! injected behind a runner seam so tests never shell out (D2);
//! fail-closed dual-format guard (D3); verbatim copy, no normalization
//! (D4); idempotent re-run (D5); newest archive dir wins, named in the
//! envelope (D6); dry-run resolves, never mutates (D7); zero deltas is
//! a success, not an error (D8).

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

/// Deploy every verified delta from `archive_dir` verbatim to its
/// deployed spec path. Empty delta sets are a success with an empty
/// list (design D8) — nothing to restore is a stated outcome.
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

    // Fail closed BEFORE any copy (design D3): verify everything first,
    // then deploy. A refusal never leaves a half-restored tree.
    for delta in &deltas {
        verify_dual_format(delta).map_err(|_| CompanionError::UnverifiableDeltas {
            deltas: deltas.clone(),
            count: deltas.len(),
        })?;
    }

    let mut restored = Vec::new();
    for delta in &deltas {
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
        fs::copy(delta, &target).map_err(|source| CompanionError::UnreadableDelta {
            path: target.clone(),
            read_detail: source.to_string(),
        })?;
        restored.push(target);
    }
    Ok(restored)
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
        // D7: resolve + verify, never invoke, never write.
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
        return Ok(RestoreOutcome {
            change_id: change_id.to_string(),
            openspec_ran,
            archive_dir,
            restored,
            dry_run: false,
        });
    }

    // Already archived (design D5): skip the invocation, re-run restore.
    let archive_dir = archived_dir.unwrap();
    let restored = restore(root, &archive_dir)?;
    Ok(RestoreOutcome {
        change_id: change_id.to_string(),
        openspec_ran: false,
        archive_dir,
        restored,
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
        .map(|cap| root.join("openspec/specs").join(cap).join("spec.md"))
        .unwrap_or_else(|| root.join("openspec/specs/spec.md"))
}

#[cfg(test)]
mod tests;
