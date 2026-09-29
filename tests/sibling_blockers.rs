//! Tests for scripts/guards/sibling-blockers.sh — the hook-enforced guard for
//! the AGENTS.md sibling-tool constraints (AGENTS.md: "Sibling-tool
//! constraints (hard blockers)").
//!
//! The script takes an optional repo directory argument (defaults to `.`) so
//! tests can exercise it against throwaway fixture repos. Exit 0 = compliant,
//! exit 1 = violation (stderr carries a remediation hint).

use std::fs;
use std::path::Path;
use std::process::Command;

fn script_path() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("scripts/guards/sibling-blockers.sh")
}

fn git(dir: &Path, args: &[&str]) {
    let out = Command::new("git")
        .args(["-C", dir.to_str().unwrap()])
        .args(args)
        .output()
        .expect("git should be available");
    assert!(
        out.status.success(),
        "git {:?} failed: {}",
        args,
        String::from_utf8_lossy(&out.stderr)
    );
}

fn run_guard(dir: &Path) -> (bool, String) {
    // (passed, output)
    let out = Command::new("bash")
        .arg(script_path())
        .arg(dir)
        .output()
        .expect("guard script should be runnable");
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    (out.status.success(), combined)
}

fn fixture_repo(name: &str) -> (tempfile::TempDir, std::path::PathBuf) {
    let dir = tempfile::tempdir_in(std::env::temp_dir()).expect("tempdir");
    let repo = dir.path().join(name);
    fs::create_dir_all(&repo).unwrap();
    let repo = repo.canonicalize().unwrap();
    git(&repo, &["init", "-q"]);
    git(&repo, &["config", "user.email", "test@example.com"]);
    git(&repo, &["config", "user.name", "test"]);
    // Keep the TempDir alive alongside the repo path so the fixture survives.
    (dir, repo)
}

/// A fully compliant repo: beads owns core.hooksPath, .git/hooks is pristine
/// (only .sample files), workflows have no espectacular/vampiro references.
fn make_compliant(repo: &Path) {
    git(repo, &["config", "core.hooksPath", ".beads/hooks"]);
    fs::create_dir_all(repo.join(".git/hooks")).unwrap();
    fs::write(repo.join(".git/hooks/pre-commit.sample"), "# sample").unwrap();
    fs::create_dir_all(repo.join(".github/workflows")).unwrap();
    fs::write(
        repo.join(".github/workflows/ci.yml"),
        "name: CI\njobs:\n  ci:\n",
    )
    .unwrap();
}

#[test]
fn compliant_repo_passes() {
    let (_dir, repo) = fixture_repo("compliant");
    make_compliant(&repo);
    let (ok, out) = run_guard(&repo);
    assert!(ok, "compliant repo should pass, got: {out}");
}

#[test]
fn claimed_hookspath_fails() {
    let (_dir, repo) = fixture_repo("claimed");
    make_compliant(&repo);
    // pretender claiming core.hooksPath — the hard blocker
    git(&repo, &["config", "core.hooksPath", ".githooks"]);
    let (ok, out) = run_guard(&repo);
    assert!(!ok, "claimed core.hooksPath must fail");
    assert!(
        out.contains("core.hooksPath"),
        "remediation hint should name core.hooksPath, got: {out}"
    );
}

#[test]
fn unguarded_git_hooks_write_fails() {
    let (_dir, repo) = fixture_repo("unguarded");
    make_compliant(&repo);
    fs::write(repo.join(".git/hooks/pre-commit"), "#!/bin/sh\n").unwrap();
    let (ok, out) = run_guard(&repo);
    assert!(!ok, "non-.sample file in .git/hooks must fail");
    assert!(
        out.contains(".git/hooks"),
        "hint should name .git/hooks, got: {out}"
    );
}

#[test]
fn espectacular_in_ci_fails() {
    let (_dir, repo) = fixture_repo("espectacular_ci");
    make_compliant(&repo);
    fs::write(
        repo.join(".github/workflows/ci.yml"),
        "jobs:\n  scenario-conformance:\n    run: espectacular check\n",
    )
    .unwrap();
    let (ok, out) = run_guard(&repo);
    assert!(!ok, "espectacular wired into CI must fail");
    assert!(
        out.contains("espectacular"),
        "hint should name the tool, got: {out}"
    );
}

#[test]
fn vampiro_in_ci_fails() {
    let (_dir, repo) = fixture_repo("vampiro_ci");
    make_compliant(&repo);
    fs::write(
        repo.join(".github/workflows/ci.yml"),
        "jobs:\n  seam-checks:\n    run: vampiro check src\n",
    )
    .unwrap();
    let (ok, out) = run_guard(&repo);
    assert!(!ok, "vampiro wired into CI must fail");
    assert!(
        out.contains("vampiro"),
        "hint should name the tool, got: {out}"
    );
}

#[test]
fn comment_mention_in_ci_passes() {
    // Prose references in YAML comments are not wiring — must NOT fail
    // (regression: publish.yml mentions the tools in a comment).
    let (_dir, repo) = fixture_repo("ci_comment");
    make_compliant(&repo);
    fs::write(
        repo.join(".github/workflows/publish.yml"),
        "# follow-up — see the pretender/espectacular notes\njobs:\n  publish:\n    run: cargo publish\n",
    )
    .unwrap();
    let (ok, out) = run_guard(&repo);
    assert!(ok, "comment-only mention must pass, got: {out}");
}

#[test]
fn missing_hookspath_fails() {
    let (_dir, repo) = fixture_repo("no_hookspath");
    make_compliant(&repo);
    git(&repo, &["config", "--unset", "core.hooksPath"]);
    let (ok, out) = run_guard(&repo);
    assert!(!ok, "unset core.hooksPath must fail (beads must own it)");
    assert!(out.contains("core.hooksPath"), "got: {out}");
}
