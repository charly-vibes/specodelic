//! `spk archive-companion` end-to-end (CLI) tests: the Stage-2
//! envelope surface over the tempdir fixtures — dry-run planning,
//! fail-closed sidecar refusal, and the unknown-change label. The
//! test names are bound by the espectacular contract TOMLs under
//! `.espectacular/archive-companion/` — do not rename them casually.

use crate::common::UnwrapPanic;
use assert_cmd::Command;

fn spk() -> Command {
    Command::cargo_bin("specodelic").unwrap_p()
}

/// Fixture: a fake repo root (a `.git` dir is all repo_root() needs to
/// anchor) with an already-archived change; the sibling `layer.md`
/// sidecar rides along (the Stage-2 shape, Revision 19). `sidecar`
/// controls whether the sidecar exists at all.
fn fixture(id: &str, delta: &str, sidecar: Option<&str>) -> tempfile::TempDir {
    let root = tempfile::tempdir().unwrap_p();
    std::fs::create_dir_all(root.path().join(".git")).unwrap_p();
    let delta_path = root.path().join(format!(
        "openspec/changes/archive/2026-10-01-{id}/specs/c/spec.md"
    ));
    std::fs::create_dir_all(delta_path.parent().unwrap_p()).unwrap_p();
    std::fs::write(&delta_path, delta).unwrap_p();
    if let Some(sidecar) = sidecar {
        std::fs::write(delta_path.with_file_name("layer.md"), sidecar).unwrap_p();
    }
    root
}

/// Fixture: a fake repo root with an ACTIVE (not yet archived) change —
/// the shape that sends `archive-companion` through the openspec
/// invocation seam instead of the already-archived restore path.
#[cfg(unix)]
fn active_fixture(id: &str, delta: &str, sidecar: &str) -> tempfile::TempDir {
    let root = tempfile::tempdir().unwrap_p();
    std::fs::create_dir_all(root.path().join(".git")).unwrap_p();
    let delta_path = root
        .path()
        .join(format!("openspec/changes/{id}/specs/c/spec.md"));
    std::fs::create_dir_all(delta_path.parent().unwrap_p()).unwrap_p();
    std::fs::write(&delta_path, delta).unwrap_p();
    std::fs::write(delta_path.with_file_name("layer.md"), sidecar).unwrap_p();
    root
}

/// A plain openspec delta (Revision 19): no frontmatter, no tables.
const PLAIN_DELTA: &str =
    "## ADDED Requirements\n\n### Requirement: One holds\nThe system SHALL hold one.\n";

/// A lint-clean `layer.md` sidecar: frontmatter (`id: c.layer`) + the
/// three specodelic tables.
const SIDECAR: &str = concat!(
    "---\nid: c.layer\nkind: intent\n",
    "statement: \"WHEN the c layer is elaborated, THE author SHALL write the real requirement.\"\n",
    "---\n\n",
    "## Constraints\n\n| id | kind | expr | traces_to |\n|----|------|------|-----------|\n",
    "| k1 | invariant | `true` | [[c.layer]] |\n\n",
    "## Model\n\n### States\n\n- `draft`\n\n### Transitions\n\n",
    "| id | from | to | guard |\n|----|------|----|-------|\n",
    "| t1 | draft | draft | [[c.layer.k1]] |\n\n",
    "## Properties\n\n",
    "| id | kind | derives_from | generator | predicate |\n",
    "|----|------|--------------|-----------|-----------|\n",
    "| p1 | unit | [[c.layer.k1]] | `todo()` | `true` |\n"
);

/// archive-companion dry-run over an already-archived change: exit 0,
/// envelope lists the planned restore AND the consumed sidecar, and
/// NOTHING is written (specodelic-fzo, dry_run_never_mutates).
#[test]
fn archive_companion_dry_run_reports_without_writing() {
    let root = fixture("dryx", PLAIN_DELTA, Some(SIDECAR));
    let out = spk()
        .args(["archive-companion", "dryx", "--dry-run", "--json"])
        .current_dir(root.path())
        .output()
        .unwrap_p();
    let stdout = String::from_utf8(out.stdout).unwrap_p();
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap_p();
    assert_eq!(out.status.code(), Some(0), "{stdout}");
    assert_eq!(json["data"]["dry_run"], serde_json::json!(true));
    assert_eq!(json["data"]["openspec_ran"], serde_json::json!(false));
    assert_eq!(
        json["data"]["restored"],
        serde_json::json!(["openspec/specs/c/spec.md"])
    );
    assert_eq!(
        json["data"]["sidecars"],
        serde_json::json!(["openspec/changes/archive/2026-10-01-dryx/specs/c/layer.md"])
    );
    assert!(
        !root.path().join("openspec/specs").exists(),
        "dry-run must not create deployed spec paths"
    );
}

/// archive-companion fails closed on a sidecar-less archived delta
/// (specodelic-s8fg, fail-closed on sidecar-less deltas — the
/// successor of the Stage-1 frontmatterless-delta refusal): exit 1,
/// the delta is named with the wrap-path hint, and no deployed spec
/// is written.
#[test]
fn archive_companion_refuses_frontmatterless_delta() {
    let root = fixture("plain", PLAIN_DELTA, None);
    let out = spk()
        .args(["archive-companion", "plain", "--json"])
        .current_dir(root.path())
        .output()
        .unwrap_p();
    assert_eq!(out.status.code(), Some(1));
    let stderr = String::from_utf8(out.stderr).unwrap_p();
    assert!(
        stderr.contains("unrestorable"),
        "refusal must name the failure: {stderr}"
    );
    assert!(
        stderr.contains("spk migrate"),
        "refusal must carry the wrap-path hint: {stderr}"
    );
    assert!(
        !root.path().join("openspec/specs").exists(),
        "a refusal must not write any deployed spec"
    );
}

/// archive-companion over an unknown change id: labeled error naming
/// the id and the `openspec list` hint.
#[test]
fn archive_companion_unknown_change_is_labeled_error() {
    let root = fixture("other", PLAIN_DELTA, Some(SIDECAR));
    let out = spk()
        .args(["archive-companion", "ghost-change", "--json"])
        .current_dir(root.path())
        .output()
        .unwrap_p();
    assert_eq!(out.status.code(), Some(1));
    let stderr = String::from_utf8(out.stderr).unwrap_p();
    assert!(stderr.contains("ghost-change"), "{stderr}");
    assert!(stderr.contains("openspec list"), "{stderr}");
}

// ---- specodelic-54c6: the invocation bound and the reported rollback ----

/// A hung `openspec archive` must not block `spk archive-companion`
/// forever: the invocation is bounded by wall clock, and expiry is a
/// LABELED failure that names the bound command and the bound itself
/// (specodelic-54c6).
///
/// Seam this test pins (none exists today — `openspec_archive_runner`
/// waits on `Command::output()` with no bound): the production runner
/// reads its bound from `SPK_ARCHIVE_TIMEOUT_SECS` (seconds; 1 here so
/// the test stays fast). Green contract for the failure detail: it must
/// name the command as `openspec archive <id> --yes` and the bound as
/// `timed out after 1s`.
#[cfg(unix)]
#[test]
fn archive_companion_bounds_hung_openspec_as_labeled_timeout() {
    use std::os::unix::fs::PermissionsExt;

    let root = active_fixture("slow", PLAIN_DELTA, SIDECAR);
    let bin = tempfile::tempdir().unwrap_p();
    let openspec = bin.path().join("openspec");
    let invoked = bin.path().join("invoked");
    // Marks the invocation, then never returns within the bound (and
    // outlives the red-stage hang).
    std::fs::write(
        &openspec,
        format!("#!/bin/sh\n: > {}\nsleep 30\n", invoked.display()),
    )
    .unwrap_p();
    std::fs::set_permissions(&openspec, std::fs::Permissions::from_mode(0o755)).unwrap_p();
    let path = format!(
        "{}:{}",
        bin.path().display(),
        std::env::var("PATH").unwrap_or_else(|_| "/usr/bin:/bin".to_string())
    );

    let out = spk()
        .args(["archive-companion", "slow", "--json"])
        .current_dir(root.path())
        .env("PATH", path)
        // The green stage's bound override: 1s keeps this test fast.
        .env("SPK_ARCHIVE_TIMEOUT_SECS", "1")
        // Test watchdog: the red run hangs here and is killed at 5s.
        .timeout(std::time::Duration::from_secs(5))
        .output()
        .unwrap_p();
    let stderr = String::from_utf8_lossy(&out.stderr).to_string();

    assert!(
        invoked.exists(),
        "the fixture `openspec` was never invoked — the PATH seam did not reach \
         the production runner (test setup problem, not the bug under test)"
    );
    assert!(
        out.status.code().is_some(),
        "`spk archive-companion` did not return within the test watchdog: \
         the openspec invocation is unbounded (SPK_ARCHIVE_TIMEOUT_SECS=1 was ignored); stderr: {stderr}"
    );
    assert_eq!(
        out.status.code(),
        Some(1),
        "a timed-out archiver is a labeled failure, never a hang and never a success: {stderr}"
    );
    assert!(
        stderr.contains("openspec archive slow --yes"),
        "the timeout must name the bound command: {stderr}"
    );
    assert!(
        stderr.contains("timed out after 1s"),
        "the timeout must name the bound it expired at (SPK_ARCHIVE_TIMEOUT_SECS=1): {stderr}"
    );
    assert!(
        !root.path().join("openspec/specs").exists(),
        "a timed-out archive must not proceed to the attach stage"
    );
}

/// A revert that cannot restore its saved pre-run state must be REPORTED
/// — error detail (or a warnings channel), never the silent `let _ =`
/// discard in `revert()` (specodelic-54c6).
///
/// Scenario: the deployed target exists but is unreadable (mode 0200)
/// inside a read-only capability dir (0555). The companion therefore
/// cannot read the prior bytes (its rollback action becomes "remove
/// what I wrote"), it CAN truncate-rewrite the file, the post-attach
/// corpus lint then fails on the unreadable file, and the rollback's
/// `remove_file` is denied by the read-only directory.
///
/// Green contract: the AttachViolation detail carries
/// `rollback failed: <path>: <io error>` for every unrestorable entry.
#[cfg(unix)]
#[test]
fn archive_companion_reports_failed_rollback() {
    use std::os::unix::fs::PermissionsExt;

    let root = fixture("revx", PLAIN_DELTA, Some(SIDECAR));
    let deployed_dir = root.path().join("openspec/specs/c");
    let target = deployed_dir.join("spec.md");
    std::fs::create_dir_all(&deployed_dir).unwrap_p();
    std::fs::write(
        &target,
        "# c Specification\n\n## Requirements\n\n### Requirement: Old\nOld.\n",
    )
    .unwrap_p();
    // Write-only file in a read-only dir: the forward write succeeds, the
    // rollback's unlink is denied. Precondition: a non-root uid (the
    // DAC-denial this test rides on — `just ci` and CI both run non-root).
    std::fs::set_permissions(&target, std::fs::Permissions::from_mode(0o200)).unwrap_p();
    std::fs::set_permissions(&deployed_dir, std::fs::Permissions::from_mode(0o555)).unwrap_p();

    let out = spk()
        .args(["archive-companion", "revx", "--json"])
        .current_dir(root.path())
        .timeout(std::time::Duration::from_secs(10))
        .output()
        .unwrap_p();
    // Restore the fixture modes first so the tempdir can be removed.
    std::fs::set_permissions(&deployed_dir, std::fs::Permissions::from_mode(0o755)).unwrap_p();

    let stderr = String::from_utf8_lossy(&out.stderr).to_string();
    assert_eq!(
        out.status.code(),
        Some(1),
        "the attach must still fail closed: {stderr}"
    );
    assert!(
        stderr.contains("rollback failed"),
        "a revert that cannot restore its saved state must be reported, not silently discarded: {stderr}"
    );
    assert!(
        stderr.contains("openspec/specs/c/spec.md"),
        "the failed rollback must name the unrestorable path: {stderr}"
    );
}
